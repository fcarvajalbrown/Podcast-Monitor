use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::assemble;
use crate::chunk;
use crate::feed::{self, Episode};
use crate::memory;
use crate::names::Glossary;
use crate::run::{self, ShowInfo, Transcription, read_json, write_json};
use crate::shows::SavedShow;
use crate::summarize::{self, Assistant, LimitReached, Provider};
use crate::transcribe::{Progress, Transcriber};

pub const MODEL_FILE: &str = "ggml-large-v3-turbo.bin";
pub const REPORT_FILE: &str = "informe.pdf";
const MODELS_FOLDER: &str = "models";
const RUNS_FOLDER: &str = "runs";
const UNFINISHED_FILE: &str = "current-run.json";
const UNFINISHED_WEEK_FILE: &str = "current-week.json";
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
    WaitingForMemory(u64),
    Summarizing,
    Report,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Failure {
    MissingModel(PathBuf),
    Limit,
    MissingCli,
    LowMemory,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub provider: Provider,
    #[serde(default = "debug_log_on")]
    pub debug_log: bool,
}

fn debug_log_on() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self { provider: Provider::Claude, debug_log: debug_log_on() }
    }
}

pub type StageSink = Arc<dyn Fn(Stage) + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QueueOutcome {
    pub reports: Vec<PathBuf>,
    pub failed: Vec<(Request, Failure)>,
    pub stopped: Option<Failure>,
}

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

pub fn unfinished_week(data: &Path) -> Result<Option<Vec<Request>>> {
    read_json(&data.join(UNFINISHED_WEEK_FILE))
}

pub fn mark_unfinished_week(data: &Path, requests: &[Request]) -> Result<()> {
    fs::create_dir_all(data)?;
    write_json(&data.join(UNFINISHED_WEEK_FILE), &requests)
}

pub fn clear_unfinished_week(data: &Path) -> Result<()> {
    match fs::remove_file(data.join(UNFINISHED_WEEK_FILE)) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}

pub fn finished(data: &Path, request: &Request) -> bool {
    let folder = episode_folder(data, request);
    let summary: Option<summarize::Summary> = read_json(&folder.join("summary.json")).unwrap_or(None);
    folder.join(REPORT_FILE).is_file() && summary.is_some_and(|done| done.chunks.len() == done.chunk_count)
}

pub type Choice<'a> = (Provider, &'a dyn Assistant);

pub fn run_queue(data: &Path, requests: &[Request], first: Choice, mut fallback: Option<Choice>, on_episode: &dyn Fn(usize, &Request), stage: StageSink) -> QueueOutcome {
    let (mut provider, mut assistant) = first;
    if let Err(error) = mark_unfinished_week(data, requests) {
        log::error!("saving the unfinished week: {error:#}");
    }
    log::info!("queue of {} episode(s) with {provider:?}", requests.len());
    let mut transcriber = None;
    let mut outcome = QueueOutcome::default();
    for (index, request) in requests.iter().enumerate() {
        on_episode(index, request);
        if finished(data, request) {
            log::info!("already finished: {} / {}", request.show.title, request.episode.title);
            outcome.reports.push(episode_folder(data, request).join(REPORT_FILE));
            continue;
        }
        let mut result = run_with(data, request, provider, assistant, stage.clone(), &mut transcriber);
        if result == Err(Failure::Limit)
            && let Some((next, next_assistant)) = fallback.take()
        {
            log::warn!("{provider:?} hit its limit; switching to {next:?} {} for the rest of the queue", next_assistant.model().unwrap_or("default model"));
            (provider, assistant) = (next, next_assistant);
            result = run_with(data, request, provider, assistant, stage.clone(), &mut transcriber);
        }
        match result {
            Ok(pdf) => outcome.reports.push(pdf),
            Err(Failure::Other) => outcome.failed.push((request.clone(), Failure::Other)),
            Err(stop) => {
                outcome.stopped = Some(stop);
                return outcome;
            }
        }
    }
    if let Err(error) = clear_unfinished_week(data) {
        log::error!("clearing the unfinished week: {error:#}");
    }
    outcome
}

pub fn settings(data: &Path) -> Settings {
    read_json(&data.join(SETTINGS_FILE)).unwrap_or_else(|error| {
        log::warn!("reading settings: {error:#}");
        None
    }).unwrap_or_default()
}

pub fn save_settings(data: &Path, settings: &Settings) -> Result<()> {
    fs::create_dir_all(data)?;
    write_json(&data.join(SETTINGS_FILE), settings)
}

pub fn run(data: &Path, request: &Request, provider: Provider, assistant: &dyn Assistant, stage: StageSink) -> Result<PathBuf, Failure> {
    run_with(data, request, provider, assistant, stage, &mut None)
}

fn run_with(data: &Path, request: &Request, provider: Provider, assistant: &dyn Assistant, stage: StageSink, transcriber: &mut Option<Transcriber>) -> Result<PathBuf, Failure> {
    let folder = episode_folder(data, request);
    let model = model_path(data);
    if !folder.join("transcript.json").is_file() && !model.is_file() {
        return Err(Failure::MissingModel(model));
    }
    log::info!("run {} / {} with {provider:?} into {}", request.show.title, request.episode.title, folder.display());
    let probe = memory::system_probe();
    if memory::low(&probe).is_some() && transcriber.take().is_some() {
        log::info!("unloaded the whisper model to free memory");
    }
    let waiting = stage.clone();
    if memory::wait(&probe, &|free| waiting(Stage::WaitingForMemory(free))) == memory::Wait::TimedOut {
        return Err(Failure::LowMemory);
    }
    let started = std::time::Instant::now();
    let logged: StageSink = Arc::new(move |current| {
        if !matches!(current, Stage::Transcribing(_)) {
            log::debug!("stage {current:?} at {:.1} s", started.elapsed().as_secs_f64());
        }
        stage(current)
    });
    let outcome = steps(data, &folder, model, request, provider, assistant, logged, transcriber).map_err(|error| classify(&error));
    log::info!("run ended after {:.1} s: {}", started.elapsed().as_secs_f64(), match &outcome { Ok(pdf) => pdf.display().to_string(), Err(failure) => format!("{failure:?}") });
    outcome
}

fn steps(data: &Path, folder: &Path, model: PathBuf, request: &Request, provider: Provider, assistant: &dyn Assistant, stage: StageSink, transcriber: &mut Option<Transcriber>) -> Result<PathBuf> {
    fs::create_dir_all(folder).with_context(|| format!("creating {}", folder.display()))?;
    if let Some(show_dir) = folder.parent() {
        write_json(&show_dir.join("show.json"), &ShowInfo { title: request.show.title.clone() })?;
    }
    let glossary_path = data.join(GLOSSARY_FILE);
    let glossary = glossary_path.is_file().then(|| Glossary::load(&glossary_path)).transpose()?;

    stage(Stage::Preparing);
    let transcribing = stage.clone();
    let progress: Progress = Arc::new(move |percent| transcribing(Stage::Transcribing(percent)));
    run::process(&request.episode, folder, &Transcription::with_defaults(model), glossary.as_ref(), transcriber, &progress)?;

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
    log::error!("run failed: {error:#}");
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

    fn second_request() -> Request {
        let mut second = request();
        second.episode.guid = "ep-2".into();
        second.episode.title = "Ep. 2".into();
        second
    }

    #[test]
    fn queue_runs_every_episode_then_skips_finished_ones_on_a_rerun() {
        let data = data_dir("queue");
        let requests = [request(), second_request()];
        requests.iter().for_each(|request| place_transcript(&data, request));
        let seen = Mutex::new(Vec::new());
        let (sink, _) = recorder();
        let outcome = run_queue(&data, &requests, (Provider::Codex, &Reply(true)), None, &|index, _| seen.lock().unwrap().push(index), sink.clone());
        assert_eq!(outcome.reports.len(), 2);
        assert_eq!(outcome.stopped, None);
        assert_eq!(*seen.lock().unwrap(), [0, 1]);
        assert_eq!(unfinished_week(&data).unwrap(), None);
        let rerun = run_queue(&data, &requests, (Provider::Codex, &Reply(false)), None, &|_, _| {}, sink);
        assert_eq!(rerun.reports, outcome.reports);
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn queue_stops_at_a_limit_and_keeps_the_week_marker() {
        let data = data_dir("queue-limit");
        let requests = vec![request(), second_request()];
        requests.iter().for_each(|request| place_transcript(&data, request));
        let (sink, _) = recorder();
        let outcome = run_queue(&data, &requests, (Provider::Codex, &Reply(false)), None, &|_, _| {}, sink);
        assert_eq!(outcome.stopped, Some(Failure::Limit));
        assert!(outcome.reports.is_empty());
        assert_eq!(unfinished_week(&data).unwrap(), Some(requests));
        fs::remove_dir_all(&data).unwrap();
    }

    struct Pinned;

    impl Assistant for Pinned {
        fn ask(&self, prompt: &str, schema: &Value) -> Result<Value> {
            Reply(true).ask(prompt, schema)
        }

        fn model(&self) -> Option<&str> {
            Some(summarize::FALLBACK_MODEL)
        }
    }

    #[test]
    fn a_limit_switches_the_rest_of_the_queue_to_the_fallback() {
        let data = data_dir("queue-fallback");
        let requests = [request(), second_request()];
        requests.iter().for_each(|request| place_transcript(&data, request));
        let (sink, _) = recorder();
        let outcome = run_queue(&data, &requests, (Provider::Codex, &Reply(false)), Some((Provider::Claude, &Pinned)), &|_, _| {}, sink);
        assert_eq!(outcome.stopped, None);
        assert_eq!(outcome.reports.len(), 2);
        for request in &requests {
            let summary: summarize::Summary = read_json(&episode_folder(&data, request).join("summary.json")).unwrap().unwrap();
            assert_eq!((summary.provider, summary.model.as_deref()), (Provider::Claude, Some("claude-sonnet-5")));
        }
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
        save_settings(&data, &Settings { provider: Provider::Codex, debug_log: false }).unwrap();
        assert_eq!(settings(&data).provider, Provider::Codex);
        assert!(!settings(&data).debug_log);
        fs::write(data.join(SETTINGS_FILE), r#"{"provider":"codex"}"#).unwrap();
        assert!(settings(&data).debug_log);
        fs::remove_dir_all(&data).unwrap();
    }
}
