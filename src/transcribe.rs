use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub start_s: f64,
    pub end_s: f64,
    pub text: String,
}

pub struct Transcriber {
    context: WhisperContext,
}

impl Transcriber {
    pub fn load(model: &Path) -> Result<Self> {
        let path = model.to_str().ok_or_else(|| anyhow!("model path is not valid UTF-8"))?;
        let context = WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .with_context(|| format!("loading model {}", model.display()))?;
        Ok(Self { context })
    }

    pub fn transcribe(&self, audio: &[f32], language: &str) -> Result<Vec<Segment>> {
        let mut state = self.context.create_state().context("creating whisper state")?;
        let mut params = FullParams::new(SamplingStrategy::BeamSearch { beam_size: 5, patience: -1.0 });
        params.set_language(Some(language));
        params.set_n_threads(threads());
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_progress_callback_safe(|percent: i32| {
            eprint!("\r  transcribing {percent:>3}%");
            let _ = std::io::stderr().flush();
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

fn threads() -> i32 {
    std::thread::available_parallelism().map(|n| n.get().min(4) as i32).unwrap_or(4)
}
