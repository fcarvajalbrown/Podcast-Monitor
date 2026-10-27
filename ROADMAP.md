# Roadmap

Scope and decisions: `docs/PRD.md`. Findings: `docs/research/`. Finished items: `docs/ROADMAP-HISTORY.md`.

## Open items

- None.

## Phases

| Phase | What | Status |
|---|---|---|
| 1 | Pipeline as a Windows CLI | Done |
| 2 | Summarizer through her assistant CLI, quote matching, limit-error pause | Not Started |
| 3 | PDF report in the escritorio layout | Not Started |
| 4 | Desktop window in Spanish | Not Started |
| 5 | macOS build with Metal, reference episodes, first on-demand report | Not Started |

## Phase 1: Pipeline as a Windows CLI

Build and test on Windows through `tools/build/cargo-windows.cmd` (VS dev env, Ninja, short target dir `%USERPROFILE%\.pmt` for the 260-char path limit, Vulkan SDK).

All items done; see `docs/ROADMAP-HISTORY.md`.
