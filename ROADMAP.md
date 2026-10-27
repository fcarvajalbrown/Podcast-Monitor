# Roadmap

Scope and decisions: `docs/PRD.md`. Findings: `docs/research/`. Finished items: `docs/ROADMAP-HISTORY.md`.

## Open items

- Phase 1, item 7: Ep. 34 GPU run not yet verified; first run used the CPU (whisper-rs GPU default off), fixed in code, rerun pending

## Phases

| Phase | What | Status |
|---|---|---|
| 1 | Pipeline as a Windows CLI | In Progress |
| 2 | Summarizer through her assistant CLI, quote matching, limit-error pause | Not Started |
| 3 | PDF report in the escritorio layout | Not Started |
| 4 | Desktop window in Spanish | Not Started |
| 5 | macOS build with Metal, reference episodes, first weekly run | Not Started |

## Phase 1: Pipeline as a Windows CLI

Build and test on Windows through `tools/build/cargo-windows.cmd` (VS dev env, Ninja, short target dir `%USERPROFILE%\.pmt` for the 260-char path limit, Vulkan SDK).

| # | Item | Status |
|---|---|---|
| 3 | Audio download and decode to 16 kHz mono in Rust, no FFmpeg at run time | Done in code, unverified on a real episode |
| 4 | Transcription through whisper-rs on Vulkan, timestamped segments | Done in code, GPU use unverified |
| 5 | Name fix at 0.85 on segments, glossary file and per-show exceptions | Done in code, unit tests pass |
| 6 | Per-episode resume: each finished step saved to disk and skipped on rerun | Done in code, audio reuse unverified |
| 7 | Check: Ep. 34 end to end, Kast and Boric counts at least the 0.85 results in `docs/research/name-correction.md` | In Progress |
