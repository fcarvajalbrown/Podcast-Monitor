use std::backtrace::Backtrace;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};

const LOGS_FOLDER: &str = "logs";
const OWN_TARGET: &str = "podcast_monitor";
const WHISPER_TARGETS: [&str; 2] = ["whisper_rs", "ggml"];

struct FileLog {
    file: Option<Mutex<File>>,
}

impl FileLog {
    fn wanted(target: &str, level: Level) -> bool {
        target.starts_with(OWN_TARGET) || WHISPER_TARGETS.iter().any(|prefix| target.starts_with(prefix)) || level <= Level::Warn
    }
}

impl Log for FileLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        Self::wanted(metadata.target(), metadata.level())
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let own = record.target().starts_with(OWN_TARGET);
        if own && record.level() <= Level::Info {
            eprintln!("{}", record.args());
        }
        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            let stamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");
            let _ = writeln!(file, "{stamp} {:<5} {}: {}", record.level(), record.target(), record.args().to_string().trim_end());
            let _ = file.flush();
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            let _ = file.flush();
        }
    }
}

pub fn file_name(stamp: &str, pid: u32) -> String {
    format!("{stamp}_{pid}.log")
}

pub fn start(data: &Path, enabled: bool) -> Result<Option<PathBuf>> {
    let path = enabled.then(|| data.join(LOGS_FOLDER).join(file_name(&Local::now().format("%Y-%m-%d_%H-%M-%S").to_string(), std::process::id())));
    let file = match &path {
        Some(path) => {
            fs::create_dir_all(path.parent().unwrap_or(data)).with_context(|| format!("creating {}", data.join(LOGS_FOLDER).display()))?;
            Some(Mutex::new(File::create(path).with_context(|| format!("creating {}", path.display()))?))
        }
        None => None,
    };
    log::set_boxed_logger(Box::new(FileLog { file })).context("installing the logger")?;
    log::set_max_level(if enabled { LevelFilter::Trace } else { LevelFilter::Info });
    if enabled {
        whisper_rs::install_logging_hooks();
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            log::error!(target: "podcast_monitor::panic", "{info}\n{}", Backtrace::force_capture());
            previous(info);
        }));
        log::debug!(
            "podcast-monitor {} on {} {}, args {:?}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            std::env::args().collect::<Vec<_>>()
        );
    }
    Ok(path)
}

pub fn progress_step(previous: i32, percent: i32) -> bool {
    previous < 0 || percent / 10 != previous / 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_and_whisper_records_are_kept_and_other_crates_only_from_warn() {
        assert!(FileLog::wanted("podcast_monitor::run", Level::Trace));
        assert!(FileLog::wanted("whisper_rs::whisper_logging_hook", Level::Info));
        assert!(FileLog::wanted("ggml_logging_hook", Level::Debug));
        assert!(!FileLog::wanted("wgpu_core::device", Level::Info));
        assert!(FileLog::wanted("wgpu_core::device", Level::Warn));
    }

    #[test]
    fn progress_is_logged_every_ten_percent() {
        let logged: Vec<i32> = (0..=100).scan(-1, |last, percent| {
            let step = progress_step(*last, percent);
            if step {
                *last = percent;
            }
            Some(step.then_some(percent))
        }).flatten().collect();
        assert_eq!(logged, [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    }

    #[test]
    fn file_name_carries_time_and_process() {
        assert_eq!(file_name("2026-10-28_10-40-00", 42), "2026-10-28_10-40-00_42.log");
    }
}
