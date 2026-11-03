use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::job;

pub const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin";
pub const MODEL_SHA256: &str = "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69";
pub const MODEL_SIZE: u64 = 1_624_555_275;
const USER_AGENT: &str = "Podcast-Monitor";
const REPORT_EVERY: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Progress {
    Bytes(u64, u64),
    Checking,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Failure {
    Interrupted,
    Damaged,
}

pub struct Source<'a> {
    pub url: &'a str,
    pub size: u64,
    pub sha256: &'a str,
}

pub const HUGGING_FACE: Source<'static> = Source { url: MODEL_URL, size: MODEL_SIZE, sha256: MODEL_SHA256 };

pub fn present(data: &Path) -> bool {
    job::model_path(data).is_file()
}

pub fn partial_path(target: &Path) -> PathBuf {
    let mut name = target.file_name().map(|name| name.to_os_string()).unwrap_or_default();
    name.push(".part");
    target.with_file_name(name)
}

pub fn fetch(target: &Path, source: &Source, on_progress: &dyn Fn(Progress)) -> Result<(), Failure> {
    let partial = partial_path(target);
    if let Err(error) = download(&partial, source, on_progress) {
        log::warn!("model download interrupted: {error:#}");
        return Err(Failure::Interrupted);
    }
    on_progress(Progress::Checking);
    match sha256_of(&partial) {
        Ok(hash) if hash == source.sha256 => {}
        Ok(hash) => {
            log::error!("model hash {hash}, expected {}; deleting {}", source.sha256, partial.display());
            let _ = fs::remove_file(&partial);
            return Err(Failure::Damaged);
        }
        Err(error) => {
            log::error!("hashing the model: {error:#}");
            return Err(Failure::Interrupted);
        }
    }
    fs::rename(&partial, target).map_err(|error| {
        log::error!("moving the model into place: {error}");
        Failure::Interrupted
    })?;
    log::info!("model ready at {}", target.display());
    Ok(())
}

fn download(partial: &Path, source: &Source, on_progress: &dyn Fn(Progress)) -> Result<()> {
    if let Some(folder) = partial.parent() {
        fs::create_dir_all(folder).with_context(|| format!("creating {}", folder.display()))?;
    }
    let mut done = fs::metadata(partial).map(|meta| meta.len()).unwrap_or(0);
    if done > source.size {
        fs::remove_file(partial)?;
        done = 0;
    }
    on_progress(Progress::Bytes(done, source.size));
    if done == source.size {
        return Ok(());
    }
    log::info!("downloading the model from {} at byte {done}", source.url);
    let mut request = ureq::get(source.url).header("User-Agent", USER_AGENT);
    if done > 0 {
        request = request.header("Range", format!("bytes={done}-"));
    }
    let mut response = request.call().with_context(|| format!("requesting {}", source.url))?;
    let resumed = response.status().as_u16() == 206;
    if done > 0 && !resumed {
        log::warn!("server ignored the range request; starting the model over");
        done = 0;
    }
    let mut out = OpenOptions::new().create(true).write(true).append(resumed).truncate(!resumed).open(partial).with_context(|| format!("opening {}", partial.display()))?;
    let mut reader = response.body_mut().with_config().limit(source.size + 1).reader();
    let mut buffer = vec![0u8; 256 * 1024];
    let mut last_report = done;
    loop {
        let read = reader.read(&mut buffer).context("reading the model download")?;
        if read == 0 {
            break;
        }
        out.write_all(&buffer[..read]).context("writing the model")?;
        done += read as u64;
        if done - last_report >= REPORT_EVERY {
            out.flush()?;
            on_progress(Progress::Bytes(done, source.size));
            last_report = done;
        }
    }
    out.flush()?;
    on_progress(Progress::Bytes(done, source.size));
    if done != source.size {
        bail!("download ended at {done} of {} bytes", source.size);
    }
    Ok(())
}

pub fn sha256_of(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).context("hashing")?;
    Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;
    use std::sync::Mutex;

    use super::*;

    fn serve(body: Vec<u8>, honour_range: bool, cut_after: Option<usize>, requests: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming().take(requests) {
                let mut stream = stream.unwrap();
                let mut start = 0;
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line.trim().is_empty() {
                        break;
                    }
                    if let Some(range) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
                        start = range.trim().trim_end_matches('-').parse().unwrap();
                    }
                }
                let start = if honour_range { start } else { 0 };
                let rest = &body[start..];
                let status = if start > 0 { "206 Partial Content" } else { "200 OK" };
                let header = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", rest.len());
                stream.write_all(header.as_bytes()).unwrap();
                let sent = cut_after.map_or(rest.len(), |cut| cut.min(rest.len()));
                let _ = stream.write_all(&rest[..sent]);
            }
        });
        format!("http://{address}/model.bin")
    }

    fn body() -> Vec<u8> {
        (0..3_000_000u32).map(|index| (index % 251) as u8).collect()
    }

    fn hash(bytes: &[u8]) -> String {
        Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-model-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_cut_download_resumes_from_the_partial_file_and_passes_the_hash() {
        let dir = folder("resume");
        let target = dir.join("model.bin");
        let bytes = body();
        let sha = hash(&bytes);
        let first = serve(bytes.clone(), true, Some(1_000_000), 1);
        let source = Source { url: &first, size: bytes.len() as u64, sha256: &sha };
        assert_eq!(fetch(&target, &source, &|_| {}), Err(Failure::Interrupted));
        assert_eq!(fs::metadata(partial_path(&target)).unwrap().len(), 1_000_000);
        let second = serve(bytes.clone(), true, None, 1);
        let seen = Mutex::new(Vec::new());
        let source = Source { url: &second, size: bytes.len() as u64, sha256: &sha };
        assert_eq!(fetch(&target, &source, &|progress| seen.lock().unwrap().push(progress)), Ok(()));
        assert_eq!(fs::read(&target).unwrap(), bytes);
        assert!(!partial_path(&target).exists());
        let seen = seen.into_inner().unwrap();
        assert_eq!(seen.first(), Some(&Progress::Bytes(1_000_000, 3_000_000)));
        assert_eq!(seen.last(), Some(&Progress::Checking));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_server_that_ignores_the_range_restarts_the_file() {
        let dir = folder("norange");
        let target = dir.join("model.bin");
        let bytes = body();
        fs::write(partial_path(&target), &bytes[..500_000]).unwrap();
        let url = serve(bytes.clone(), false, None, 1);
        let sha = hash(&bytes);
        let source = Source { url: &url, size: bytes.len() as u64, sha256: &sha };
        assert_eq!(fetch(&target, &source, &|_| {}), Ok(()));
        assert_eq!(fs::read(&target).unwrap(), bytes);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_wrong_hash_deletes_the_file() {
        let dir = folder("damaged");
        let target = dir.join("model.bin");
        let bytes = body();
        let url = serve(bytes.clone(), true, None, 1);
        let wrong = hash(b"something else");
        let source = Source { url: &url, size: bytes.len() as u64, sha256: &wrong };
        assert_eq!(fetch(&target, &source, &|_| {}), Err(Failure::Damaged));
        assert!(!target.exists());
        assert!(!partial_path(&target).exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[ignore = "downloads the last megabyte of the real model from Hugging Face and needs the model in the app-data folder"]
    fn live_resume_of_the_last_megabyte_from_hugging_face() {
        let local = job::model_path(&crate::shows::data_dir());
        let dir = folder("live");
        let target = dir.join(job::MODEL_FILE);
        let keep = MODEL_SIZE - 1_048_576;
        let mut source_file = File::open(&local).unwrap();
        let mut partial = File::create(partial_path(&target)).unwrap();
        io::copy(&mut (&mut source_file).take(keep), &mut partial).unwrap();
        drop(partial);
        let seen = Mutex::new(Vec::new());
        assert_eq!(fetch(&target, &HUGGING_FACE, &|progress| seen.lock().unwrap().push(progress)), Ok(()));
        let seen = seen.into_inner().unwrap();
        assert_eq!(seen.first(), Some(&Progress::Bytes(keep, MODEL_SIZE)));
        assert!(seen.iter().all(|progress| !matches!(progress, Progress::Bytes(done, _) if *done < keep)));
        assert_eq!(fs::metadata(&target).unwrap().len(), MODEL_SIZE);
        fs::remove_dir_all(&dir).unwrap();
    }
}
