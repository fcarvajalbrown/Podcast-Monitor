mod audio;
mod chunk;
mod feed;
mod names;
mod quotes;
mod run;
mod summarize;
mod tally;
mod transcribe;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "podcast-monitor", about = "Turn podcast episodes into corrected, timestamped transcripts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
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
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Episodes { feed, limit } => {
            let show = feed::fetch(&feed)?;
            println!("{}", show.title);
            for episode in feed::newest(show.episodes, limit) {
                println!("{}  {:>4} min  {}", episode.id(), episode.duration_s / 60, episode.title);
            }
            Ok(())
        }
        Command::Run { feed, model, glossary, work, latest, threshold, title, beam_size, greedy, chunk_minutes, text_context, glossary_prompt } => {
            let beam_size = if greedy { 1 } else { beam_size };
            run::run(&run::Options { feed, model, glossary, work, latest, threshold, title, beam_size, chunk_minutes, text_context, glossary_prompt })
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
            let summary = match summarize::summarize(&chunks, glossary.as_ref(), provider, &summarize::Cli { provider }, &out) {
                Err(error) if error.downcast_ref::<summarize::LimitReached>().is_some() => {
                    eprintln!("{error}");
                    eprintln!("finished parts are saved in {}; run the same command again later to continue", out.display());
                    std::process::exit(2);
                }
                other => other?,
            };
            eprintln!("{} part(s) saved to {}", summary.chunks.len(), out.display());
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
    }
}
