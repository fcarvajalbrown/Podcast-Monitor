use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

const SAMPLE_RATE: usize = 16_000;
const SEARCH_S: usize = 20;
const SPEECH_GAP: usize = SAMPLE_RATE / 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub start_s: f64,
    pub end_s: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Decoding {
    pub beam_size: usize,
    pub chunk_minutes: usize,
    pub text_context: i32,
    pub prompt: Option<String>,
    pub vad_model: Option<String>,
}

pub type Progress = Arc<dyn Fn(i32) + Send + Sync>;

pub fn print_progress() -> Progress {
    Arc::new(|percent| {
        eprint!("\r  transcribing {percent:>3}%");
        let _ = std::io::stderr().flush();
    })
}

pub struct Transcriber {
    context: WhisperContext,
}

impl Transcriber {
    pub fn load(model: &Path, flash_attn: bool) -> Result<Self> {
        let path = model.to_str().ok_or_else(|| anyhow!("model path is not valid UTF-8"))?;
        let mut params = WhisperContextParameters::default();
        params.use_gpu(true);
        params.flash_attn(flash_attn);
        let context = WhisperContext::new_with_params(path, params)
            .with_context(|| format!("loading model {}", model.display()))?;
        Ok(Self { context })
    }

    pub fn transcribe(&self, audio: &[f32], language: &str, decoding: &Decoding, progress: &Progress) -> Result<Vec<Segment>> {
        let Some(vad_model) = decoding.vad_model.as_deref() else {
            return self.transcribe_all(audio, language, decoding, progress);
        };
        let pieces = speech_pieces(vad_model, audio)?;
        let speech = join_speech(audio, &pieces);
        log::info!("  speech {:.0} s of {:.0} s in {} pieces", speech.len() as f64 / SAMPLE_RATE as f64, audio.len() as f64 / SAMPLE_RATE as f64, pieces.len());
        let segments = self.transcribe_all(&speech, language, decoding, progress)?;
        Ok(segments
            .into_iter()
            .map(|segment| Segment { start_s: original_time(&pieces, segment.start_s), end_s: original_time(&pieces, segment.end_s), text: segment.text })
            .collect())
    }

    fn transcribe_all(&self, audio: &[f32], language: &str, decoding: &Decoding, progress: &Progress) -> Result<Vec<Segment>> {
        let cuts = boundaries(audio, decoding.chunk_minutes * 60 * SAMPLE_RATE);
        let mut segments = Vec::new();
        for (index, pair) in cuts.windows(2).enumerate() {
            log::info!("  chunk {}/{}", index + 1, cuts.len() - 1);
            let offset_s = pair[0] as f64 / SAMPLE_RATE as f64;
            let part = self.transcribe_chunk(&audio[pair[0]..pair[1]], language, decoding, progress.clone())?;
            segments.extend(part.into_iter().map(|segment| Segment { start_s: segment.start_s + offset_s, end_s: segment.end_s + offset_s, text: segment.text }));
        }
        Ok(segments)
    }

    fn transcribe_chunk(&self, audio: &[f32], language: &str, decoding: &Decoding, progress: Progress) -> Result<Vec<Segment>> {
        let mut state = self.context.create_state().context("creating whisper state")?;
        let strategy = if decoding.beam_size <= 1 {
            SamplingStrategy::Greedy { best_of: 1 }
        } else {
            SamplingStrategy::BeamSearch { beam_size: decoding.beam_size as i32, patience: -1.0 }
        };
        let mut params = FullParams::new(strategy);
        params.set_language(Some(language));
        params.set_n_max_text_ctx(decoding.text_context);
        if let Some(prompt) = decoding.prompt.as_deref() {
            params.set_initial_prompt(prompt);
        }
        params.set_n_threads(threads());
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        let mut logged = -1;
        params.set_progress_callback_safe(move |percent: i32| {
            if crate::debug_log::progress_step(logged, percent) {
                logged = percent;
                log::debug!("transcribing {percent}%");
            }
            progress(percent)
        });
        state.full(params, audio).context("running whisper")?;
        eprintln!();
        state
            .as_iter()
            .map(|segment| {
                Ok(Segment {
                    start_s: segment.start_timestamp() as f64 / 100.0,
                    end_s: segment.end_timestamp() as f64 / 100.0,
                    text: segment.to_str_lossy()?.trim().to_string(),
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Piece {
    joined_start: usize,
    original_start: usize,
    len: usize,
}

fn speech_pieces(vad_model: &str, audio: &[f32]) -> Result<Vec<Piece>> {
    let mut params = WhisperVadContextParams::new();
    params.set_n_threads(threads());
    let mut context = WhisperVadContext::new(vad_model, params).with_context(|| format!("loading VAD model {vad_model}"))?;
    let spans = context
        .segments_from_samples(WhisperVadParams::new(), audio)
        .context("running VAD")?
        .map(|span| (cs_to_sample(span.start), cs_to_sample(span.end)))
        .collect::<Vec<_>>();
    Ok(layout(&spans, audio.len()))
}

fn cs_to_sample(centiseconds: f32) -> usize {
    (centiseconds.max(0.0) as f64 * SAMPLE_RATE as f64 / 100.0).round() as usize
}

fn layout(spans: &[(usize, usize)], audio_len: usize) -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    for &(start, end) in spans {
        let start = start.min(audio_len);
        let end = end.min(audio_len);
        if end <= start {
            continue;
        }
        let joined_start = pieces.last().map_or(0, |last| last.joined_start + last.len + SPEECH_GAP);
        pieces.push(Piece { joined_start, original_start: start, len: end - start });
    }
    pieces
}

fn join_speech(audio: &[f32], pieces: &[Piece]) -> Vec<f32> {
    let mut joined = Vec::with_capacity(pieces.last().map_or(0, |last| last.joined_start + last.len));
    for piece in pieces {
        joined.resize(piece.joined_start, 0.0);
        joined.extend_from_slice(&audio[piece.original_start..piece.original_start + piece.len]);
    }
    joined
}

fn original_time(pieces: &[Piece], joined_s: f64) -> f64 {
    let sample = (joined_s * SAMPLE_RATE as f64).max(0.0) as usize;
    let index = pieces.partition_point(|piece| piece.joined_start <= sample);
    let Some(piece) = index.checked_sub(1).map(|index| pieces[index]) else {
        return joined_s;
    };
    let within = (sample - piece.joined_start).min(piece.len);
    (piece.original_start + within) as f64 / SAMPLE_RATE as f64
}

fn boundaries(audio: &[f32], chunk_samples: usize) -> Vec<usize> {
    let mut cuts = vec![0];
    if chunk_samples > 0 {
        let reach = SEARCH_S * SAMPLE_RATE;
        let mut target = chunk_samples;
        while target + reach < audio.len() {
            let cut = quietest(audio, target - reach, target + reach);
            cuts.push(cut);
            target = cut + chunk_samples;
        }
    }
    cuts.push(audio.len());
    cuts
}

fn quietest(audio: &[f32], from: usize, to: usize) -> usize {
    let window = SAMPLE_RATE;
    (from..to - window)
        .step_by(SAMPLE_RATE / 10)
        .min_by(|&a, &b| energy(&audio[a..a + window]).total_cmp(&energy(&audio[b..b + window])))
        .map_or(to, |start| start + window / 2)
}

fn energy(window: &[f32]) -> f32 {
    window.iter().map(|sample| sample * sample).sum()
}

fn threads() -> i32 {
    std::thread::available_parallelism().map(|n| n.get().min(4) as i32).unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuts_land_in_the_quietest_second_near_each_target() {
        let mut audio = vec![1.0f32; 200 * SAMPLE_RATE];
        let quiet_start = 55 * SAMPLE_RATE;
        audio[quiet_start..quiet_start + 2 * SAMPLE_RATE].fill(0.0);
        let cuts = boundaries(&audio, 60 * SAMPLE_RATE);
        assert_eq!(cuts.first(), Some(&0));
        assert_eq!(cuts.last(), Some(&audio.len()));
        assert!((quiet_start..quiet_start + 2 * SAMPLE_RATE).contains(&cuts[1]));
        assert!(cuts.windows(2).all(|pair| pair[1] > pair[0]));
    }

    #[test]
    fn speech_is_joined_with_short_gaps_and_times_map_back() {
        let audio: Vec<f32> = (0..20 * SAMPLE_RATE).map(|i| i as f32).collect();
        let spans = [(2 * SAMPLE_RATE, 4 * SAMPLE_RATE), (10 * SAMPLE_RATE, 13 * SAMPLE_RATE), (19 * SAMPLE_RATE, 25 * SAMPLE_RATE), (5, 5)];
        let pieces = layout(&spans, audio.len());
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[1].joined_start, 2 * SAMPLE_RATE + SPEECH_GAP);
        assert_eq!(pieces[2].len, SAMPLE_RATE);
        let joined = join_speech(&audio, &pieces);
        assert_eq!(joined.len(), 6 * SAMPLE_RATE + 2 * SPEECH_GAP);
        assert_eq!(joined[pieces[1].joined_start], audio[10 * SAMPLE_RATE]);
        assert_eq!(joined[2 * SAMPLE_RATE], 0.0);
        assert_eq!(original_time(&pieces, 1.0), 3.0);
        assert_eq!(original_time(&pieces, 2.05), 4.0);
        assert_eq!(original_time(&pieces, 2.1 + 1.5), 11.5);
        assert_eq!(original_time(&pieces, 5.2 + 0.5), 19.5);
    }

    #[test]
    fn zero_chunk_length_keeps_one_piece() {
        let audio = vec![0.5f32; 10 * SAMPLE_RATE];
        assert_eq!(boundaries(&audio, 0), vec![0, audio.len()]);
    }
}
