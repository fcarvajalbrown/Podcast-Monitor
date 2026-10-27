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
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Episodes { .. } => anyhow::bail!("episodes is not implemented yet"),
        Command::Run { .. } => anyhow::bail!("run is not implemented yet"),
    }
}
