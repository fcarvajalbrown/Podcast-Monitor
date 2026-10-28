<p align="center"><img src="assets/brand/podcast-monitor-icon.svg" alt="Podcast-Monitor logo" width="128"></p>

# Podcast-Monitor

A Rust desktop tool that turns Chilean podcast episodes into a report you can read instead of listening: what each show talked about and for how long, who was named, and verbatim quotes you can check by minute. Transcription runs locally with whisper.cpp.

Status: a Windows command-line tool. It transcribes an episode from its RSS feed, fixes names against a glossary, and summarizes the transcript with topics ranked by airtime, people ranked by mentions and quotes found in the transcript, and lays the finished episodes out in a Spanish PDF report. The desktop window in Spanish and the macOS build come next. Scope is in `docs/PRD.md`, phases and open items in `ROADMAP.md`, measured findings in `docs/research/`.

## Build (Windows)

Needs Rust, Visual Studio with the C++ tools, Ninja and the Vulkan SDK. The wrapper sets up the build environment and a short target folder:

```
tools/build/cargo-windows.cmd build --release
```

## Use

```
podcast-monitor
podcast-monitor episodes <feed-url>
podcast-monitor run <feed-url> --model <ggml-model.bin> [--glossary <names.txt>] [--latest 1]
podcast-monitor summarize <transcript.json> [--glossary <names.txt>] [--provider <cli>]
podcast-monitor report [run-folder] [--out <file.pdf>]
```

- With no command it opens the window in Spanish, which keeps the saved show list in the app-data folder (`%APPDATA%\Podcast-Monitor` on Windows, `~/Library/Application Support/Podcast-Monitor` on the Mac). A show is added by searching its name in Apple's podcast directory and confirming the match, which shows the publisher, or by pasting its RSS feed address.
- `episodes` lists the newest episodes of a feed.
- `run` downloads, transcribes and name-fixes the newest episodes into `work/runs`. Finished steps are kept, so running it again picks up where it stopped.
- `summarize` sends the transcript to an assistant CLI you are logged into, saves each part as it finishes, then prints the topics, people and quotes.
- `report` builds one PDF from every finished summary under the run folder (`work/runs` by default), each episode starting on a new page, in an A4 layout set in Latin Modern Roman.

Every command takes `--help` for the rest of its options.

## Layout

- `src/`: the app.
- `tools/`: helper scripts (feed checks, transcription comparisons, PDF layout and quote checks, the icon renderer, the build wrapper).
- `assets/brand/`: logo SVGs and the Windows (`.ico`) and macOS (`.icns`) icon sets.
- `assets/fonts/`: Latin Modern Roman, compiled into the app for the report (GUST Font License, `LICENSE-latin-modern.txt`).
- `monitor.py`: the original Python prototype, kept for reference only.
