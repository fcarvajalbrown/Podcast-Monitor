mod audio;
mod feed;
mod names;
mod run;
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
        Command::Run { feed, model, glossary, work, latest, threshold, title, beam_size, greedy } => {
            let beam_size = if greedy { 1 } else { beam_size };
            run::run(&run::Options { feed, model, glossary, work, latest, threshold, title, beam_size })
        }
    }
}
