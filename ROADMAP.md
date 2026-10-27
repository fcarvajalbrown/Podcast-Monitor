# Roadmap

Scope and decisions: `docs/PRD.md`. Findings: `docs/research/`. Finished items: `docs/ROADMAP-HISTORY.md`.

## Open items

- Phase 1, item 1: Cargo crate and CLI skeleton (In Progress)

## Phases

| Phase | What | Status |
|---|---|---|
| 1 | Pipeline as a Windows CLI | In Progress |
| 2 | Summarizer through her assistant CLI, quote matching, limit-error pause | Not Started |
| 3 | PDF report in the escritorio layout | Not Started |
| 4 | Desktop window in Spanish | Not Started |
| 5 | macOS build with Metal, reference episodes, first weekly run | Not Started |

## Phase 1: Pipeline as a Windows CLI

| # | Item | Status |
|---|---|---|
| 1 | Cargo crate and CLI skeleton | In Progress |
| 2 | Feed fetch and parse: episodes with title, date, duration, audio URL, description | Not Started |
| 3 | Audio download and decode to 16 kHz mono in Rust, no FFmpeg at run time | Not Started |
| 4 | Transcription through whisper-rs on Vulkan, timestamped segments | Not Started |
| 5 | Name fix at 0.85 on segments, glossary file and per-show exceptions | Not Started |
| 6 | Per-episode resume: each finished step saved to disk and skipped on rerun | Not Started |
| 7 | Check: Ep. 34 end to end, Kast and Boric counts at least the 0.85 results in `docs/research/name-correction.md` | Not Started |
