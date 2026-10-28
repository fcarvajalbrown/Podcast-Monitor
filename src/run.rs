use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::audio;
use crate::feed::{self, Episode};
use crate::names::{self, Glossary, Replacements};
use crate::transcribe::{Decoding, Progress, Segment, Transcriber, print_progress};

const LANGUAGE: &str = "es";
pub const DEFAULT_BEAM_SIZE: usize = 5;
pub const DEFAULT_CHUNK_MINUTES: usize = 0;
pub const DEFAULT_TEXT_CONTEXT: i32 = 0;

pub struct Options {
    pub feed: String,
    pub glossary: Option<PathBuf>,
    pub work: PathBuf,
    pub latest: usize,
    pub title: Option<String>,
    pub transcription: Transcription,
}

pub struct Transcription {
    pub model: PathBuf,
    pub threshold: f64,
    pub beam_size: usize,
    pub chunk_minutes: usize,
    pub text_context: i32,
    pub glossary_prompt: bool,
}

impl Transcription {
    pub fn with_defaults(model: PathBuf) -> Self {
        Self {
            model,
            threshold: names::DEFAULT_THRESHOLD,
            beam_size: DEFAULT_BEAM_SIZE,
            chunk_minutes: DEFAULT_CHUNK_MINUTES,
            text_context: DEFAULT_TEXT_CONTEXT,
            glossary_prompt: false,
        }
    }
}

fn default_beam_size() -> usize {
    DEFAULT_BEAM_SIZE
}

#[derive(Serialize, Deserialize)]
pub struct Transcript {
    pub model: String,
    #[serde(default = "default_beam_size")]
    pub beam_size: usize,
    pub language: String,
    pub audio_s: f64,
    pub compute_s: f64,
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize)]
pub struct ShowInfo {
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct FixedTranscript {
    pub threshold: f64,
    pub replacements: Vec<Replacement>,
    pub segments: Vec<Segment>,
}

#[derive(Serialize, Deserialize)]
pub struct Replacement {
    pub heard: String,
    pub name: String,
    pub count: usize,
}

pub fn run(options: &Options) -> Result<()> {
    let show = feed::fetch(&options.feed)?;
    let show_dir = options.work.join(feed::slug(&show.title, 60));
    let glossary = options.glossary.as_deref().map(Glossary::load).transpose()?;
    let mut transcriber = None;
    let matching = show.episodes.into_iter().filter(|episode| options.title.as_deref().is_none_or(|title| episode.title.contains(title))).collect();
    let episodes = feed::newest(matching, options.latest);
    fs::create_dir_all(&show_dir)?;
    write_json(&show_dir.join("show.json"), &ShowInfo { title: show.title.clone() })?;
    log::info!("{}: {} episode(s) into {}", show.title, episodes.len(), show_dir.display());
    for episode in &episodes {
        let folder = show_dir.join(episode.id());
        fs::create_dir_all(&folder)?;
        log::info!("{}", episode.title);
        process(episode, &folder, &options.transcription, glossary.as_ref(), &mut transcriber, &print_progress()).with_context(|| format!("episode {}", episode.id()))?;
    }
    Ok(())
}

pub fn process(episode: &Episode, folder: &Path, options: &Transcription, glossary: Option<&Glossary>, transcriber: &mut Option<Transcriber>, progress: &Progress) -> Result<()> {
    write_json(&folder.join("episode.json"), episode)?;
    let transcript_path = folder.join("transcript.json");
    let transcript: Transcript = match read_json(&transcript_path)? {
        Some(done) => {
            log::info!("  transcript already done");
            done
        }
        None => {
            let audio_path = audio::download(&episode.audio_url, folder)?;
            log::info!("  decoding {}", audio_path.display());
            let samples = audio::load_for_whisper(&audio_path)?;
            if transcriber.is_none() {
                *transcriber = Some(Transcriber::load(&options.model)?);
            }
            let decoding = Decoding {
                beam_size: options.beam_size,
                chunk_minutes: options.chunk_minutes,
                text_context: options.text_context,
                prompt: glossary.filter(|_| options.glossary_prompt).map(Glossary::prompt),
            };
            let started = Instant::now();
            let segments = transcriber.as_ref().map(|t| t.transcribe(&samples, LANGUAGE, &decoding, progress)).transpose()?.unwrap_or_default();
            let transcript = Transcript {
                model: options.model.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default(),
                beam_size: options.beam_size,
                language: LANGUAGE.into(),
                audio_s: samples.len() as f64 / audio::WHISPER_RATE as f64,
                compute_s: started.elapsed().as_secs_f64(),
                segments,
            };
            write_json(&transcript_path, &transcript)?;
            log::info!("  transcribed {:.0} s of audio in {:.0} s", transcript.audio_s, transcript.compute_s);
            transcript
        }
    };
    if let Some(glossary) = glossary {
        let (segments, replacements) = names::correct(&transcript.segments, glossary, options.threshold);
        let fixed = FixedTranscript { threshold: options.threshold, replacements: listed(replacements), segments };
        for replacement in &fixed.replacements {
            log::info!("  {} -> {} x{}", replacement.heard, replacement.name, replacement.count);
        }
        write_json(&folder.join("transcript.fixed.json"), &fixed)?;
    }
    Ok(())
}

fn listed(replacements: Replacements) -> Vec<Replacement> {
    let mut list: Vec<Replacement> = replacements.into_iter().map(|((heard, name), count)| Replacement { heard, name, count }).collect();
    list.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.heard.cmp(&b.heard)));
    list
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
    }
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let partial = path.with_extension("json.part");
    fs::write(&partial, serde_json::to_string_pretty(value)?).with_context(|| format!("writing {}", partial.display()))?;
    fs::rename(&partial, path).with_context(|| format!("saving {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip_and_missing_file() {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("value.json");
        assert!(read_json::<Vec<u32>>(&path).unwrap().is_none());
        write_json(&path, &vec![1u32, 2, 3]).unwrap();
        assert_eq!(read_json::<Vec<u32>>(&path).unwrap(), Some(vec![1, 2, 3]));
        assert!(!dir.join("value.json.part").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
