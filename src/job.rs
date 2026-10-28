use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::assemble;
use crate::chunk;
use crate::feed::{self, Episode};
use crate::names::Glossary;
use crate::run::{self, ShowInfo, Transcription, read_json, write_json};
use crate::shows::SavedShow;
use crate::summarize::{self, Assistant, LimitReached, Provider};
use crate::transcribe::Progress;

pub const MODEL_FILE: &str = "ggml-large-v3-turbo.bin";
pub const REPORT_FILE: &str = "informe.pdf";
const MODELS_FOLDER: &str = "models";
const RUNS_FOLDER: &str = "runs";
const UNFINISHED_FILE: &str = "current-run.json";
const SETTINGS_FILE: &str = "settings.json";
const GLOSSARY_FILE: &str = "glossary.txt";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub show: SavedShow,
    pub episode: Episode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stage {
    Preparing,
    Transcribing(i32),
    Summarizing,
    Report,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Failure {
    MissingModel(PathBuf),
    Limit,
    MissingCli,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub provider: Provider,
}

impl Default for Settings {
    fn default() -> Self {
        Self { provider: Provider::Claude }
    }
}

pub type StageSink = Arc<dyn Fn(Stage) + Send + Sync>;

pub fn model_path(data: &Path) -> PathBuf {
    data.join(MODELS_FOLDER).join(MODEL_FILE)
}

pub fn episode_folder(data: &Path, request: &Request) -> PathBuf {
    data.join(RUNS_FOLDER).join(feed::slug(&request.show.title, 60)).join(request.episode.id())
}

pub fn unfinished(data: &Path) -> Result<Option<Request>> {
    read_json(&data.join(UNFINISHED_FILE))
}

pub fn mark_unfinished(data: &Path, request: &Request) -> Result<()> {
    fs::create_dir_all(data)?;
    write_json(&data.join(UNFINISHED_FILE), request)
}

pub fn clear_unfinished(data: &Path) -> Result<()> {
    match fs::remove_file(data.join(UNFINISHED_FILE)) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}

pub fn settings(data: &Path) -> Settings {
    read_json(&data.join(SETTINGS_FILE)).unwrap_or_else(|error| {
        eprintln!("reading settings: {error:#}");
        None
    }).unwrap_or_default()
}

pub fn save_settings(data: &Path, settings: &Settings) -> Result<()> {
    fs::create_dir_all(data)?;
    write_json(&data.join(SETTINGS_FILE), settings)
}

pub fn run(data: &Path, request: &Request, provider: Provider, assistant: &dyn Assistant, stage: StageSink) -> Result<PathBuf, Failure> {
    let folder = episode_folder(data, request);
    let model = model_path(data);
    if !folder.join("transcript.json").is_file() && !model.is_file() {
        return Err(Failure::MissingModel(model));
    }
    steps(data, &folder, model, request, provider, assistant, stage).map_err(|error| classify(&error))
}

fn steps(data: &Path, folder: &Path, model: PathBuf, request: &Request, provider: Provider, assistant: &dyn Assistant, stage: StageSink) -> Result<PathBuf> {
    fs::create_dir_all(folder).with_context(|| format!("creating {}", folder.display()))?;
    if let Some(show_dir) = folder.parent() {
        write_json(&show_dir.join("show.json"), &ShowInfo { title: request.show.title.clone() })?;
    }
    let glossary_path = data.join(GLOSSARY_FILE);
    let glossary = glossary_path.is_file().then(|| Glossary::load(&glossary_path)).transpose()?;

    stage(Stage::Preparing);
    let transcribing = stage.clone();
    let progress: Progress = Arc::new(move |percent| transcribing(Stage::Transcribing(percent)));
    run::process(&request.episode, folder, &Transcription::with_defaults(model), glossary.as_ref(), &mut None, &progress)?;

    stage(Stage::Summarizing);
    let fixed = folder.join("transcript.fixed.json");
    let transcript = if glossary.is_some() && fixed.is_file() { fixed } else { folder.join("transcript.json") };
    let chunks = chunk::split(&chunk::load_segments(&transcript)?, chunk::DEFAULT_MAX_CHARS);
    summarize::summarize(&chunks, glossary.as_ref(), provider, assistant, &folder.join("summary.json"))?;

    stage(Stage::Report);
    let pdf = folder.join(REPORT_FILE);
    assemble::build(folder, &pdf)?;
    Ok(pdf)
}

fn classify(error: &anyhow::Error) -> Failure {
    eprintln!("run failed: {error:#}");
    if error.downcast_ref::<LimitReached>().is_some() {
        return Failure::Limit;
    }
    let cli_missing = error.chain().any(|cause| cause.downcast_ref::<std::io::Error>().is_some_and(|io| io.kind() == ErrorKind::NotFound))
        && error.chain().any(|cause| cause.to_string().starts_with("starting "));
    if cli_missing { Failure::MissingCli } else { Failure::Other }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use anyhow::anyhow;
    use serde_json::{Value, json};

    use super::*;
    use crate::run::Transcript;
    use crate::transcribe::Segment;

    struct Reply(bool);

    impl Assistant for Reply {
        fn ask(&self, _prompt: &str, _schema: &Value) -> Result<Value> {
            if !self.0 {
                return Err(LimitReached("You've hit your usage limit".into()).into());
            }
            Ok(json!({
                "summary": "Resumen del episodio.",
                "topics": [{ "name": "Tema", "start": "00:00", "end": "00:20" }],
                "people": ["Ana Pérez"],
                "quotes": [{ "text": "hola a todos", "minute": "00:00" }],
            }))
        }
    }

    fn request() -> Request {
        let episode = Episode { guid: "ep-1".into(), title: "Ep. 1".into(), published: None, duration_s: 20, audio_url: "https://example.com/ep1.mp3".into(), description: String::new() };
        Request { show: SavedShow { title: "Programa de prueba".into(), publisher: "Radio".into(), feed: "https://example.com/feed".into() }, episode }
    }

    fn data_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-job-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn place_transcript(data: &Path, request: &Request) {
        let folder = episode_folder(data, request);
        fs::create_dir_all(&folder).unwrap();
        let segments = vec![
            Segment { start_s: 0.0, end_s: 10.0, text: "Hola a todos, soy Ana Pérez.".into() },
            Segment { start_s: 10.0, end_s: 20.0, text: "Hoy hablamos de un tema.".into() },
        ];
        let transcript = Transcript { model: MODEL_FILE.into(), beam_size: 5, language: "es".into(), audio_s: 20.0, compute_s: 1.0, segments };
        write_json(&folder.join("transcript.json"), &transcript).unwrap();
    }

    fn recorder() -> (StageSink, Arc<Mutex<Vec<Stage>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        (Arc::new(move |stage| sink.lock().unwrap().push(stage)), seen)
    }

    #[test]
    fn finished_transcript_runs_on_to_the_summary_and_the_pdf() {
        let data = data_dir("done");
        let request = request();
        place_transcript(&data, &request);
        let (sink, seen) = recorder();
        let pdf = run(&data, &request, Provider::Claude, &Reply(true), sink).unwrap();
        assert_eq!(pdf, episode_folder(&data, &request).join(REPORT_FILE));
        assert!(fs::read(&pdf).unwrap().starts_with(b"%PDF"));
        assert_eq!(*seen.lock().unwrap(), [Stage::Preparing, Stage::Summarizing, Stage::Report]);
        let show: ShowInfo = read_json(&data.join(RUNS_FOLDER).join("programa-de-prueba").join("show.json")).unwrap().unwrap();
        assert_eq!(show.title, "Programa de prueba");
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn missing_model_stops_before_any_download() {
        let data = data_dir("model");
        let (sink, seen) = recorder();
        assert_eq!(run(&data, &request(), Provider::Claude, &Reply(true), sink), Err(Failure::MissingModel(model_path(&data))));
        assert!(seen.lock().unwrap().is_empty());
        assert!(!episode_folder(&data, &request()).exists());
    }

    #[test]
    fn limit_keeps_the_transcript_and_the_unfinished_marker() {
        let data = data_dir("limit");
        let request = request();
        place_transcript(&data, &request);
        mark_unfinished(&data, &request).unwrap();
        let (sink, _) = recorder();
        assert_eq!(run(&data, &request, Provider::Codex, &Reply(false), sink), Err(Failure::Limit));
        assert_eq!(unfinished(&data).unwrap(), Some(request.clone()));
        assert!(episode_folder(&data, &request).join("transcript.json").is_file());
        let (sink, _) = recorder();
        assert!(run(&data, &request, Provider::Codex, &Reply(true), sink).is_ok());
        clear_unfinished(&data).unwrap();
        clear_unfinished(&data).unwrap();
        assert_eq!(unfinished(&data).unwrap(), None);
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn a_missing_cli_is_told_apart_from_other_failures() {
        let missing = anyhow::Error::new(std::io::Error::from(ErrorKind::NotFound)).context("starting claude.cmd");
        assert_eq!(classify(&missing), Failure::MissingCli);
        assert_eq!(classify(&anyhow!("claude.cmd exited with 1")), Failure::Other);
        let no_file = anyhow::Error::new(std::io::Error::from(ErrorKind::NotFound)).context("reading summary.json");
        assert_eq!(classify(&no_file), Failure::Other);
    }

    #[test]
    fn settings_default_to_claude_and_persist() {
        let data = data_dir("settings");
        assert_eq!(settings(&data).provider, Provider::Claude);
        save_settings(&data, &Settings { provider: Provider::Codex }).unwrap();
        assert_eq!(settings(&data).provider, Provider::Codex);
        fs::remove_dir_all(&data).unwrap();
    }
}
