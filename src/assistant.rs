use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::summarize::{Provider, hide_console};

const CLAUDE_INSTALLER: &str = "https://claude.ai/install.ps1";
const CODEX_INSTALLER: &str = "https://chatgpt.com/codex/install.ps1";
const STEP_LIMIT: Duration = Duration::from_secs(15 * 60);
const POLL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Missing,
    SignedOut,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    Installing,
    SigningIn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Failure {
    Install,
    SignIn,
    Cancelled,
}

fn file_names(provider: Provider, windows: bool) -> &'static [&'static str] {
    match (provider, windows) {
        (Provider::Claude, true) => &["claude.exe", "claude.cmd"],
        (Provider::Claude, false) => &["claude"],
        (Provider::Codex, true) => &["codex.exe", "codex.cmd"],
        (Provider::Codex, false) => &["codex"],
    }
}

fn find_in(names: &[&str], folders: &[PathBuf]) -> Option<PathBuf> {
    folders.iter().flat_map(|folder| names.iter().map(move |name| folder.join(name))).find(|path| path.is_file())
}

fn install_folders() -> Vec<PathBuf> {
    let under = |variable: &str, tail: &[&str]| env::var_os(variable).map(|root| tail.iter().fold(PathBuf::from(root), |path, part| path.join(part)));
    [
        under("USERPROFILE", &[".local", "bin"]),
        under("HOME", &[".local", "bin"]),
        under("LOCALAPPDATA", &["Programs", "OpenAI", "Codex", "bin"]),
        under("APPDATA", &["npm"]),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn locate(provider: Provider) -> Option<PathBuf> {
    let mut folders: Vec<PathBuf> = env::var_os("PATH").map(|path| env::split_paths(&path).collect()).unwrap_or_default();
    folders.extend(install_folders());
    find_in(file_names(provider, cfg!(windows)), &folders)
}

fn signed_in(provider: Provider, program: &Path) -> bool {
    let mut command = Command::new(program);
    hide_console(&mut command);
    match provider {
        Provider::Claude => command.args(["auth", "status"]),
        Provider::Codex => command.args(["login", "status"]),
    };
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|status| status.success())
}

pub fn status(provider: Provider) -> Status {
    match locate(provider) {
        None => Status::Missing,
        Some(program) if signed_in(provider, &program) => Status::Ready,
        Some(_) => Status::SignedOut,
    }
}

fn run_until_done(mut command: Command, label: &str, cancel: &AtomicBool, spawn_failure: Failure) -> Result<bool, Failure> {
    hide_console(&mut command);
    let output_path = env::temp_dir().join(format!("podcast-monitor-{label}-{}.txt", std::process::id()));
    let output = File::create(&output_path).and_then(|file| Ok((file.try_clone()?, file)));
    let (stdout, stderr) = output.map_or((Stdio::null(), Stdio::null()), |(first, second)| (first.into(), second.into()));
    let mut child = command.stdin(Stdio::null()).stdout(stdout).stderr(stderr).spawn().map_err(|error| {
        log::error!("starting {label}: {error}");
        spawn_failure
    })?;
    let started = Instant::now();
    let outcome = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            break Err(Failure::Cancelled);
        }
        if started.elapsed() > STEP_LIMIT {
            let _ = child.kill();
            let _ = child.wait();
            break Ok(false);
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status.success()),
            Ok(None) => sleep(POLL),
            Err(error) => {
                log::error!("waiting for {label}: {error}");
                break Ok(false);
            }
        }
    };
    log::debug!("{label} ended after {:.1} s, {outcome:?}, output: {}", started.elapsed().as_secs_f64(), fs::read_to_string(&output_path).unwrap_or_default());
    let _ = fs::remove_file(&output_path);
    outcome
}

fn install(provider: Provider, cancel: &AtomicBool) -> Result<PathBuf, Failure> {
    if !cfg!(windows) {
        return Err(Failure::Install);
    }
    let script = match provider {
        Provider::Claude => CLAUDE_INSTALLER,
        Provider::Codex => CODEX_INSTALLER,
    };
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &format!("irm {script} | iex")]);
    command.env("CODEX_NON_INTERACTIVE", "1");
    run_until_done(command, "assistant-install", cancel, Failure::Install)?;
    locate(provider).ok_or(Failure::Install)
}

fn sign_in(provider: Provider, program: &Path, cancel: &AtomicBool) -> Result<(), Failure> {
    let mut command = Command::new(program);
    match provider {
        Provider::Claude => command.args(["auth", "login"]),
        Provider::Codex => command.arg("login"),
    };
    run_until_done(command, "assistant-sign-in", cancel, Failure::SignIn)?;
    if signed_in(provider, program) { Ok(()) } else { Err(Failure::SignIn) }
}

pub fn set_up(provider: Provider, cancel: &AtomicBool, on_step: &dyn Fn(Step)) -> Result<(), Failure> {
    let program = match locate(provider) {
        Some(program) => program,
        None => {
            on_step(Step::Installing);
            install(provider, cancel)?
        }
    };
    if !signed_in(provider, &program) {
        on_step(Step::SigningIn);
        sign_in(provider, &program, cancel)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("podcast-monitor-assistant-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn exiting_with(code: u8) -> Command {
        let mut command = Command::new("powershell");
        command.args(["-NoProfile", "-Command", &format!("exit {code}")]);
        command
    }

    #[test]
    fn windows_looks_for_the_native_exe_and_the_npm_cmd() {
        assert_eq!(file_names(Provider::Claude, true), ["claude.exe", "claude.cmd"]);
        assert_eq!(file_names(Provider::Codex, true), ["codex.exe", "codex.cmd"]);
        assert_eq!(file_names(Provider::Codex, false), ["codex"]);
    }

    #[test]
    fn finds_a_cmd_when_no_exe_exists_and_prefers_the_exe() {
        let dir = folder("order");
        fs::write(dir.join("claude.cmd"), "").unwrap();
        let names = file_names(Provider::Claude, true);
        assert_eq!(find_in(names, std::slice::from_ref(&dir)), Some(dir.join("claude.cmd")));
        fs::write(dir.join("claude.exe"), "").unwrap();
        assert_eq!(find_in(names, std::slice::from_ref(&dir)), Some(dir.join("claude.exe")));
    }

    #[test]
    fn finds_nothing_in_empty_folders() {
        let dir = folder("empty");
        assert_eq!(find_in(file_names(Provider::Codex, true), &[dir]), None);
    }

    #[cfg(windows)]
    #[test]
    fn a_step_reports_its_exit_code_and_can_be_cancelled() {
        let keep_going = AtomicBool::new(false);
        assert_eq!(run_until_done(exiting_with(0), "test-ok", &keep_going, Failure::Install), Ok(true));
        assert_eq!(run_until_done(exiting_with(3), "test-fail", &keep_going, Failure::Install), Ok(false));
        let stop = AtomicBool::new(true);
        assert_eq!(run_until_done(exiting_with(0), "test-stop", &stop, Failure::Install), Err(Failure::Cancelled));
    }
}
