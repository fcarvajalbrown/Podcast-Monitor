use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::chunk::{self, Chunk};
use crate::names::Glossary;
use crate::run::{read_json, write_json};

#[cfg(windows)]
fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console(_command: &mut Command) {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Claude,
    Codex,
}

const LIMIT_MARKERS: [&str; 7] = ["hit your", "usage limit", "spend limit", "rate limit", "rate_limit", "429", "credit balance is too low"];

#[derive(Debug)]
pub struct LimitReached(pub String);

impl std::fmt::Display for LimitReached {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "assistant limit reached: {}", self.0)
    }
}

impl std::error::Error for LimitReached {}

fn failure(message: String) -> anyhow::Error {
    let lowered = message.to_lowercase().replace('\u{2019}', "'");
    if LIMIT_MARKERS.iter().any(|marker| lowered.contains(marker)) {
        LimitReached(message).into()
    } else {
        anyhow!(message)
    }
}

const CODEX_SELF_CONTAINED: &str = "This task is self-contained: everything you need is in this message. Do not run any command and do not read any file, including instruction or skill files; answer from this message alone.

";
pub const FALLBACK_MODEL: &str = "claude-sonnet-5";

pub trait Assistant {
    fn ask(&self, prompt: &str, schema: &Value) -> Result<Value>;

    fn model(&self) -> Option<&str> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicSpan {
    pub name: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteRef {
    pub text: String,
    pub minute: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkReply {
    pub summary: String,
    pub topics: Vec<TopicSpan>,
    pub people: Vec<String>,
    pub quotes: Vec<QuoteRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub provider: Provider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub chunk_count: usize,
    pub chunks: Vec<ChunkReply>,
}

impl Summary {
    pub fn merged(&self) -> ChunkReply {
        let mut people: Vec<String> = Vec::new();
        for name in self.chunks.iter().flat_map(|part| &part.people) {
            if !people.contains(name) {
                people.push(name.clone());
            }
        }
        ChunkReply {
            summary: self.chunks.iter().map(|part| part.summary.trim()).filter(|text| !text.is_empty()).collect::<Vec<_>>().join("\n\n"),
            topics: self.chunks.iter().flat_map(|part| part.topics.iter().cloned()).collect(),
            people,
            quotes: self.chunks.iter().flat_map(|part| part.quotes.iter().cloned()).collect(),
        }
    }
}

pub fn schema() -> Value {
    let text = json!({ "type": "string" });
    let object = |properties: Value, required: &[&str]| json!({ "type": "object", "properties": properties, "required": required, "additionalProperties": false });
    object(
        json!({
            "summary": text,
            "topics": { "type": "array", "items": object(json!({ "name": text, "start": text, "end": text }), &["name", "start", "end"]) },
            "people": { "type": "array", "items": text },
            "quotes": { "type": "array", "items": object(json!({ "text": text, "minute": text }), &["text", "minute"]) },
        }),
        &["summary", "topics", "people", "quotes"],
    )
}

pub fn prompt(part: &Chunk, index: usize, count: usize, glossary: Option<&Glossary>) -> String {
    let names = glossary.map(Glossary::prompt).filter(|names| !names.is_empty()).unwrap_or_else(|| "(none)".into());
    format!(
        "You are reading part {part} of {count} of a Spanish-language podcast transcript, from {start} to {end}. Every line starts with its time as [mm:ss].\n\
         \n\
         Return:\n\
         - summary: one paragraph in Spanish on what this part covers. Describe what was said and who said it, without judging it.\n\
         - topics: every topic discussed, with a short Spanish name and the [mm:ss] start and end of each stretch where it is discussed. A topic that comes back later gets one entry per stretch, with the same name.\n\
         - people: every person named in this part, spelled as in the names list below when they appear in it.\n\
         - quotes: up to five short passages worth quoting, each copied word for word from the transcript lines, 8 to 40 words, with the mm:ss of the line where it starts. Do not correct the transcript's spelling inside a quote.\n\
         \n\
         The transcript comes from speech recognition and misspells names. The correct spellings are: {names}. Use them in the summary, topics and people even where the transcript has them wrong.\n\
         \n\
         Transcript:\n\
         {text}",
        part = index + 1,
        start = chunk::clock(part.start_s),
        end = chunk::clock(part.end_s),
        text = part.text,
    )
}

pub fn summarize(chunks: &[Chunk], glossary: Option<&Glossary>, provider: Provider, assistant: &dyn Assistant, path: &Path) -> Result<Summary> {
    let mut summary = read_json::<Summary>(path)?
        .filter(|done| done.provider == provider && done.chunk_count == chunks.len())
        .unwrap_or(Summary { provider, model: assistant.model().map(String::from), chunk_count: chunks.len(), chunks: Vec::new() });
    let schema = schema();
    for (index, part) in chunks.iter().enumerate().skip(summary.chunks.len()) {
        log::info!("  summarizing part {} of {} ({} to {})", index + 1, chunks.len(), chunk::clock(part.start_s), chunk::clock(part.end_s));
        let reply = assistant.ask(&prompt(part, index, chunks.len(), glossary), &schema)?;
        summary.chunks.push(serde_json::from_value(reply).context("reading the assistant's reply")?);
        write_json(path, &summary)?;
    }
    Ok(summary)
}

pub struct Cli {
    pub provider: Provider,
    pub model: Option<String>,
}

impl Cli {
    fn program(&self) -> &'static str {
        match (self.provider, cfg!(windows)) {
            (Provider::Claude, true) => "claude.cmd",
            (Provider::Claude, false) => "claude",
            (Provider::Codex, true) => "codex.cmd",
            (Provider::Codex, false) => "codex",
        }
    }

    fn scratch(&self) -> Result<PathBuf> {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-{}", std::process::id()));
        fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        Ok(dir)
    }

    fn call(&self, args: &[String], dir: &Path, prompt: &str) -> Result<String> {
        log::debug!("calling {} with {args:?}, prompt {} chars", self.program(), prompt.chars().count());
        let started = std::time::Instant::now();
        let mut command = Command::new(self.program());
        hide_console(&mut command);
        let mut child = command
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("starting {}", self.program()))?;
        child.stdin.take().ok_or_else(|| anyhow!("no stdin"))?.write_all(prompt.as_bytes())?;
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        log::debug!(
            "{} exited with {} after {:.1} s
stdout: {stdout}
stderr: {}",
            self.program(),
            output.status,
            started.elapsed().as_secs_f64(),
            String::from_utf8_lossy(&output.stderr)
        );
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(failure(format!("{} exited with {}: {}", self.program(), output.status, tail(&format!("{stdout}\n{stderr}"), 800))));
        }
        Ok(stdout)
    }
}

impl Cli {
    pub fn new(provider: Provider) -> Self {
        Self { provider, model: None }
    }

    pub fn fallback_for(provider: Provider) -> Option<Self> {
        (provider == Provider::Codex).then(|| Self { provider: Provider::Claude, model: Some(FALLBACK_MODEL.into()) })
    }
}

impl Assistant for Cli {
    fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn ask(&self, prompt: &str, schema: &Value) -> Result<Value> {
        let dir = self.scratch()?;
        let reply = match self.provider {
            Provider::Claude => {
                let mut args = ["-p", "--output-format", "json", "--json-schema", &schema.to_string(), "--no-session-persistence", "--tools", ""].map(String::from).to_vec();
                args.extend(self.model.iter().flat_map(|model| ["--model".to_string(), model.clone()]));
                claude_reply(&self.call(&args, &dir, prompt)?)
            }
            Provider::Codex => {
                let schema_path = dir.join("schema.json");
                let reply_path = dir.join("reply.json");
                fs::write(&schema_path, schema.to_string())?;
                let _ = fs::remove_file(&reply_path);
                let args = ["exec", "--ephemeral", "--skip-git-repo-check", "-s", "read-only", "--output-schema"].map(String::from).into_iter()
                    .chain([schema_path.display().to_string(), "-o".into(), reply_path.display().to_string()])
                    .chain(self.model.iter().flat_map(|model| ["-m".to_string(), model.clone()]))
                    .chain(["-".into()])
                    .collect::<Vec<_>>();
                self.call(&args, &dir, &format!("{CODEX_SELF_CONTAINED}{prompt}"))?;
                let text = fs::read_to_string(&reply_path).with_context(|| format!("reading {}", reply_path.display()))?;
                log::debug!("codex reply: {text}");
                serde_json::from_str(&text).context("parsing codex reply")
            }
        };
        let _ = fs::remove_dir_all(&dir);
        reply
    }
}

fn claude_reply(stdout: &str) -> Result<Value> {
    let envelope: Value = serde_json::from_str(stdout).context("parsing claude output")?;
    if envelope["is_error"].as_bool().unwrap_or(false) {
        return Err(failure(format!("claude returned an error: {}", envelope["result"].as_str().unwrap_or("no message"))));
    }
    envelope.get("structured_output").cloned().ok_or_else(|| anyhow!("claude output has no structured_output"))
}

fn tail(text: &str, chars: usize) -> String {
    let count = text.chars().count();
    text.chars().skip(count.saturating_sub(chars)).collect::<String>().trim().to_string()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    struct Fake {
        prompts: RefCell<Vec<String>>,
    }

    impl Assistant for Fake {
        fn ask(&self, prompt: &str, _schema: &Value) -> Result<Value> {
            self.prompts.borrow_mut().push(prompt.into());
            Ok(json!({
                "summary": "Resumen.",
                "topics": [{ "name": "Tema", "start": "00:00", "end": "01:00" }],
                "people": ["Ana Pérez"],
                "quotes": [{ "text": "hola a todos", "minute": "00:00" }],
            }))
        }
    }

    fn chunk(start_s: f64, text: &str) -> Chunk {
        Chunk { start_s, end_s: start_s + 60.0, text: text.into() }
    }

    #[test]
    fn prompt_carries_glossary_and_transcript() {
        let glossary = Glossary::parse("Ana Pérez\nLuis Soto\n");
        let text = prompt(&chunk(60.0, "[01:00] hola\n"), 1, 3, Some(&glossary));
        assert!(text.contains("part 2 of 3"));
        assert!(text.contains("from 01:00 to 02:00"));
        assert!(text.contains("Ana Pérez, Luis Soto"));
        assert!(text.ends_with("[01:00] hola\n"));
    }

    #[test]
    fn summary_saves_each_part_and_resumes() {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-summary-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("summary.json");
        let chunks = [chunk(0.0, "[00:00] hola a todos\n"), chunk(60.0, "[01:00] chao\n")];
        let fake = Fake { prompts: RefCell::new(Vec::new()) };
        let first = summarize(&chunks[..], None, Provider::Claude, &fake, &path).unwrap();
        assert_eq!(first.chunks.len(), 2);
        assert_eq!(first.chunks[0].people, vec!["Ana Pérez"]);
        assert!(fake.prompts.borrow()[0].contains("(none)"));
        summarize(&chunks[..], None, Provider::Claude, &fake, &path).unwrap();
        assert_eq!(fake.prompts.borrow().len(), 2);
        summarize(&chunks[..], None, Provider::Codex, &fake, &path).unwrap();
        assert_eq!(fake.prompts.borrow().len(), 4);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parts_merge_into_one_episode() {
        let part = |summary: &str, topic: &str, people: &[&str], quote: &str| ChunkReply {
            summary: summary.into(),
            topics: vec![TopicSpan { name: topic.into(), start: "00:00".into(), end: "01:00".into() }],
            people: people.iter().map(|&name| name.into()).collect(),
            quotes: vec![QuoteRef { text: quote.into(), minute: "00:00".into() }],
        };
        let summary = Summary {
            provider: Provider::Claude,
            model: None,
            chunk_count: 2,
            chunks: vec![part(" Primera parte. ", "Tema", &["Ana Pérez", "Luis Soto"], "uno"), part("Segunda parte.", "Tema", &["Luis Soto", "Eva Ríos"], "dos")],
        };
        let merged = summary.merged();
        assert_eq!(merged.summary, "Primera parte.\n\nSegunda parte.");
        assert_eq!(merged.topics.len(), 2);
        assert_eq!(merged.people, vec!["Ana Pérez", "Luis Soto", "Eva Ríos"]);
        assert_eq!(merged.quotes.iter().map(|quote| quote.text.as_str()).collect::<Vec<_>>(), vec!["uno", "dos"]);
    }

    #[test]
    fn claude_envelope_errors_and_replies() {
        assert_eq!(claude_reply(r#"{"is_error":false,"structured_output":{"a":1}}"#).unwrap(), json!({ "a": 1 }));
        assert!(claude_reply(r#"{"is_error":true,"result":"limit"}"#).unwrap_err().to_string().contains("limit"));
        assert!(claude_reply(r#"{"is_error":false,"result":"x"}"#).is_err());
    }

    #[test]
    fn limit_messages_are_told_apart() {
        let limited = |error: anyhow::Error| error.downcast_ref::<LimitReached>().is_some();
        assert!(limited(claude_reply(r#"{"is_error":true,"result":"You’ve hit your session limit · resets 5pm"}"#).unwrap_err()));
        assert!(limited(failure("codex exited with 1: You've hit your usage limit. Try again in 2 days".into())));
        assert!(limited(failure("exceeded retry limit, last status: 429 Too Many Requests".into())));
        assert!(!limited(claude_reply(r#"{"is_error":true,"result":"API Error: 500 Internal server error"}"#).unwrap_err()));
    }
}
