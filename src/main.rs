mod assemble;
mod audio;
mod chunk;
mod debug_log;
mod directory;
mod feed;
mod gui;
mod job;
mod memory;
mod names;
mod quotes;
mod report;
mod run;
mod section;
mod shows;
mod summarize;
mod tally;
mod transcribe;
mod week;
mod weekly;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "podcast-monitor", about = "Turn podcast episodes into corrected, timestamped transcripts; with no command, open the window")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "List the newest episodes of a feed")]
    Episodes {
        feed: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    #[command(about = "Download, transcribe and name-fix the newest episodes of a feed, resuming finished steps")]
    Run {
        feed: String,
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        glossary: Option<PathBuf>,
        #[arg(long, default_value = "work/runs")]
        work: PathBuf,
        #[arg(long, default_value_t = 1)]
        latest: usize,
        #[arg(long, default_value_t = names::DEFAULT_THRESHOLD)]
        threshold: f64,
        #[arg(long, help = "only episodes whose title contains this text")]
        title: Option<String>,
        #[arg(long, default_value_t = run::DEFAULT_BEAM_SIZE, conflicts_with = "greedy", help = "beam search width; 1 is greedy")]
        beam_size: usize,
        #[arg(long, help = "greedy decoding, same as --beam-size 1")]
        greedy: bool,
        #[arg(long, default_value_t = run::DEFAULT_CHUNK_MINUTES, help = "transcribe in chunks of this many minutes, each in a fresh state; 0 is one pass")]
        chunk_minutes: usize,
        #[arg(long, default_value_t = run::DEFAULT_TEXT_CONTEXT, help = "most tokens of earlier text fed to each 30 s window; 0 turns it off, 16384 is whisper.cpp's own default")]
        text_context: i32,
        #[arg(long, help = "feed the glossary names to whisper as its initial prompt; has no effect while --text-context is 0")]
        glossary_prompt: bool,
        #[arg(long, help = "flash attention on the GPU")]
        flash_attn: bool,
        #[arg(long, help = "Silero VAD model; whisper skips the stretches it marks as silence")]
        vad_model: Option<PathBuf>,
    },
    #[command(about = "Show how a transcript is split into chunks for the summarizer")]
    Chunks {
        transcript: PathBuf,
        #[arg(long, default_value_t = chunk::DEFAULT_MAX_CHARS)]
        max_chars: usize,
    },
    #[command(about = "Summarize a transcript through the assistant CLI, saving each part as it finishes")]
    Summarize {
        transcript: PathBuf,
        #[arg(long)]
        glossary: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "claude")]
        provider: summarize::Provider,
        #[arg(long, default_value_t = chunk::DEFAULT_MAX_CHARS)]
        max_chars: usize,
        #[arg(long, help = "where the summary is saved; defaults to summary.json next to the transcript")]
        out: Option<PathBuf>,
    },
    #[command(about = "Add shows to the window's saved show list by feed URL, under the feed's title and author")]
    AddShow {
        #[arg(required = true)]
        feeds: Vec<String>,
    },
    #[command(about = "List this week's episodes, Monday to now in local time, across the saved show list")]
    Week {
        #[arg(long, help = "run every listed episode through transcription, summary and PDF, skipping finished ones; uses the window's assistant choice")]
        run: bool,
        #[arg(long, help = "build the weekly PDF from this week's finished episodes, one block per show")]
        report: bool,
    },
    #[command(about = "Build one PDF report from every finished summary under a run folder")]
    Report {
        #[arg(default_value = "work/runs")]
        folder: PathBuf,
        #[arg(long, help = "where the PDF is saved; defaults to informe.pdf in the run folder")]
        out: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let data = shows::data_dir();
    match debug_log::start(&data, job::settings(&data).debug_log) {
        Ok(Some(path)) => log::debug!("log file {}, free memory {:?} MB", path.display(), memory::free_mb()),
        Ok(None) => {}
        Err(error) => eprintln!("debug log not started: {error:#}"),
    }
    let outcome = dispatch();
    if let Err(error) = &outcome {
        log::debug!("exiting with error: {error:#}");
    }
    outcome
}

#[cfg(windows)]
fn free_own_console() {
    use windows_sys::Win32::System::Console::{FreeConsole, GetConsoleProcessList};
    let mut attached = [0u32; 2];
    let count = unsafe { GetConsoleProcessList(attached.as_mut_ptr(), attached.len() as u32) };
    if count == 1 {
        log::debug!("the console belongs to the window alone; freeing it");
        unsafe { FreeConsole() };
    } else {
        log::debug!("console shared with {} other process(es); keeping it", count.saturating_sub(1));
    }
}

#[cfg(not(windows))]
fn free_own_console() {}

fn dispatch() -> Result<()> {
    let Some(command) = Cli::parse().command else {
        free_own_console();
        return Ok(gui::run()?);
    };
    match command {
        Command::Episodes { feed, limit } => {
            let show = feed::fetch(&feed)?;
            println!("{}", show.title);
            for episode in feed::newest(show.episodes, limit) {
                println!("{}  {:>4} min  {}", episode.id(), episode.duration_s / 60, episode.title);
            }
            Ok(())
        }
        Command::Run { feed, model, glossary, work, latest, threshold, title, beam_size, greedy, chunk_minutes, text_context, glossary_prompt, flash_attn, vad_model } => {
            let beam_size = if greedy { 1 } else { beam_size };
            let transcription = run::Transcription { model, threshold, beam_size, chunk_minutes, text_context, glossary_prompt, flash_attn, vad_model };
            run::run(&run::Options { feed, glossary, work, latest, title, transcription })
        }
        Command::Chunks { transcript, max_chars } => {
            for (index, part) in chunk::split(&chunk::load_segments(&transcript)?, max_chars).iter().enumerate() {
                println!("{}  {} to {}  {} chars  {} lines", index + 1, chunk::clock(part.start_s), chunk::clock(part.end_s), part.text.chars().count(), part.text.lines().count());
            }
            Ok(())
        }
        Command::Summarize { transcript, glossary, provider, max_chars, out } => {
            let glossary = glossary.as_deref().map(names::Glossary::load).transpose()?;
            let segments = chunk::load_segments(&transcript)?;
            let chunks = chunk::split(&segments, max_chars);
            let out = out.unwrap_or_else(|| transcript.with_file_name("summary.json"));
            let summary = match summarize::summarize(&chunks, glossary.as_ref(), provider, &summarize::Cli::new(provider), &out) {
                Err(error) if error.downcast_ref::<summarize::LimitReached>().is_some() => {
                    log::error!("{error}");
                    log::info!("finished parts are saved in {}; run the same command again later to continue", out.display());
                    std::process::exit(2);
                }
                other => other?,
            };
            log::info!("{} part(s) saved to {}", summary.chunks.len(), out.display());
            let episode = summary.merged();
            for topic in tally::airtime(&episode.topics) {
                println!("{}  {}", chunk::clock(topic.seconds), topic.name);
            }
            for person in tally::mentions(&episode.people, &segments) {
                println!("{:>4}  {}", person.count, person.name);
            }
            let index = quotes::Index::new(&segments);
            for quote in &episode.quotes {
                match index.find(&quote.text, &quote.minute) {
                    Some(found) => println!("{}  {}", chunk::clock(found.start_s), found.text),
                    None => println!("dropped  {}  {}", quote.minute, quote.text),
                }
            }
            Ok(())
        }
        Command::AddShow { feeds } => {
            let path = shows::list_path(&shows::data_dir());
            let mut saved = shows::load(&path)?;
            for url in feeds {
                let fetched = feed::fetch(&url).with_context(|| format!("fetching {url}"))?;
                if fetched.episodes.is_empty() {
                    bail!("{url} has no episodes with audio");
                }
                let show = shows::SavedShow { title: fetched.title, publisher: fetched.author, feed: url };
                let line = format!("{}  ·  {}", show.title, show.publisher);
                if shows::add(&mut saved, show) {
                    log::info!("added {line}");
                } else {
                    log::info!("already saved: {line}");
                }
                shows::save(&path, &saved)?;
            }
            log::info!("{} show(s) in {}", saved.len(), path.display());
            Ok(())
        }
        Command::Week { run, report } => {
            let saved = shows::load(&shows::list_path(&shows::data_dir()))?;
            let week = week::collect(&saved, chrono::Local::now());
            let mut seconds = 0;
            for request in &week.requests {
                seconds += request.episode.duration_s;
                let published = request.episode.published.map(|date| date.with_timezone(&chrono::Local).format("%a %d %H:%M").to_string()).unwrap_or_default();
                println!("{published}  {:>4} min  {}  ·  {}", request.episode.duration_s / 60, request.show.title, request.episode.title);
            }
            println!("{} episode(s), {:.1} h, since Monday {}", week.requests.len(), seconds as f64 / 3600.0, week.monday);
            for show in &week.failed {
                log::warn!("feed failed: {}", show.title);
            }
            if run {
                let data = shows::data_dir();
                let provider = job::settings(&data).provider;
                let requests = match job::unfinished_week(&data)? {
                    Some(stopped) => {
                        log::info!("continuing the stopped queue of {} episode(s)", stopped.len());
                        stopped
                    }
                    None => week.requests.clone(),
                };
                let count = requests.len();
                let stage: job::StageSink = std::sync::Arc::new(|_| {});
                let fallback = summarize::Cli::fallback_for(provider);
                let outcome = job::run_queue(&data, &requests, (provider, &summarize::Cli::new(provider)), fallback.as_ref().map(|cli| (cli.provider, cli as &dyn summarize::Assistant)), &|index, request| log::info!("[{}/{count}] {} · {}", index + 1, request.show.title, request.episode.title), stage);
                log::info!("{} report(s), {} failed", outcome.reports.len(), outcome.failed.len());
                for (request, failure) in &outcome.failed {
                    log::warn!("failed: {} · {} ({failure:?})", request.show.title, request.episode.title);
                }
                if let Some(stop) = outcome.stopped {
                    bail!("stopped: {stop:?}; run the same command again to continue");
                }
            }
            if report {
                let data = shows::data_dir();
                let provider = job::settings(&data).provider;
                let fallback = summarize::Cli::fallback_for(provider);
                let outcome = weekly::build(&data, &week, &saved, (provider, &summarize::Cli::new(provider)), fallback.as_ref().map(|cli| (cli.provider, cli as &dyn summarize::Assistant)))?;
                if let Some(stop) = outcome.stopped {
                    bail!("stopped: {stop:?}; run the same command again to continue");
                }
            }
            Ok(())
        }
        Command::Report { folder, out } => {
            let out = out.unwrap_or_else(|| folder.join("informe.pdf"));
            let count = assemble::build(&folder, &out)?;
            log::info!("{count} episode(s) in {}", out.display());
            Ok(())
        }
    }
}
