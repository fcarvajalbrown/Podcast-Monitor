use std::f64::consts::PI;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error as DecodeError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::feed::USER_AGENT;

pub const WHISPER_RATE: u32 = 16_000;
const SINC_HALF_TAPS: i64 = 32;

pub fn download(url: &str, folder: &Path) -> Result<PathBuf> {
    let target = folder.join(format!("audio.{}", extension(url)));
    if target.exists() {
        return Ok(target);
    }
    let partial = target.with_extension("part");
    let mut response = ureq::get(url).header("User-Agent", USER_AGENT).call().with_context(|| format!("downloading {url}"))?;
    let mut out = File::create(&partial).with_context(|| format!("creating {}", partial.display()))?;
    io::copy(&mut response.body_mut().as_reader(), &mut out).context("writing audio")?;
    drop(out);
    fs::rename(&partial, &target)?;
    Ok(target)
}

fn extension(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .filter(|ext| matches!(ext.as_str(), "mp3" | "m4a" | "mp4" | "aac"))
        .unwrap_or_else(|| "mp3".into())
}

pub fn load_for_whisper(path: &Path) -> Result<Vec<f32>> {
    let (samples, rate) = decode_mono(path)?;
    Ok(resample(&samples, rate, WHISPER_RATE))
}

pub fn decode_mono(path: &Path) -> Result<(Vec<f32>, u32)> {
    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        hint.with_extension(ext);
    }
    let mut format = symphonia::default::get_probe()
        .probe(&hint, stream, FormatOptions::default(), MetadataOptions::default())
        .context("probing audio format")?;
    let track = format.default_track(TrackType::Audio).ok_or_else(|| anyhow!("no audio track"))?;
    let track_id = track.id;
    let params = track.codec_params.as_ref().and_then(|params| params.audio()).ok_or_else(|| anyhow!("no audio codec parameters"))?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(params, &AudioDecoderOptions::default())
        .context("creating decoder")?;
    let mut mono = Vec::new();
    let mut rate = 0;
    let mut interleaved: Vec<f32> = Vec::new();
    while let Some(packet) = format.next_packet().context("reading audio packet")? {
        if packet.track_id != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(buffer) => {
                rate = buffer.spec().rate();
                let channels = buffer.spec().channels().count().max(1);
                interleaved.resize(buffer.samples_interleaved(), 0.0);
                buffer.copy_to_slice_interleaved(&mut interleaved);
                mono.extend(interleaved.chunks_exact(channels).map(|frame| frame.iter().sum::<f32>() / channels as f32));
            }
            Err(DecodeError::DecodeError(_)) => {}
            Err(error) => return Err(error).context("decoding audio"),
        }
    }
    if rate == 0 {
        return Err(anyhow!("no audio decoded from {}", path.display()));
    }
    Ok((mono, rate))
}

pub fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || input.is_empty() {
        return input.to_vec();
    }
    let step = from as f64 / to as f64;
    let cutoff = (to as f64 / from as f64).min(1.0) * 0.95;
    let output_len = (input.len() as f64 / step).floor() as usize;
    let last = input.len() as i64 - 1;
    (0..output_len)
        .map(|n| {
            let position = n as f64 * step;
            let center = position.floor() as i64;
            let mut sum = 0.0;
            let mut weight = 0.0;
            for k in (center - SINC_HALF_TAPS + 1)..=(center + SINC_HALF_TAPS) {
                let distance = position - k as f64;
                let tap = cutoff * sinc(cutoff * distance) * blackman(distance / SINC_HALF_TAPS as f64);
                sum += tap * input[k.clamp(0, last) as usize] as f64;
                weight += tap;
            }
            (sum / weight) as f32
        })
        .collect()
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-9 { 1.0 } else { (PI * x).sin() / (PI * x) }
}

fn blackman(x: f64) -> f64 {
    if x.abs() >= 1.0 {
        return 0.0;
    }
    let t = (x + 1.0) / 2.0;
    0.42 - 0.5 * (2.0 * PI * t).cos() + 0.08 * (4.0 * PI * t).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frequency: f64, rate: u32, seconds: f64) -> Vec<f32> {
        (0..(rate as f64 * seconds) as usize).map(|i| (2.0 * PI * frequency * i as f64 / rate as f64).sin() as f32).collect()
    }

    fn rms(samples: &[f32]) -> f64 {
        (samples.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / samples.len() as f64).sqrt()
    }

    #[test]
    fn keeps_speech_band_tone() {
        let output = resample(&tone(1000.0, 44_100, 1.0), 44_100, WHISPER_RATE);
        assert_eq!(output.len(), 16_000);
        let level = rms(&output[1000..15_000]);
        assert!((level - 0.5f64.sqrt()).abs() < 0.02, "rms {level}");
    }

    #[test]
    fn removes_tone_above_new_nyquist() {
        let output = resample(&tone(12_000.0, 48_000, 1.0), 48_000, WHISPER_RATE);
        assert!(rms(&output[1000..15_000]) < 0.02);
    }

    #[test]
    fn extension_ignores_query() {
        assert_eq!(extension("https://example.com/a/ep.M4A?x=1"), "m4a");
        assert_eq!(extension("https://example.com/stream?id=3"), "mp3");
    }
}
