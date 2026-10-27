# Roadmap

Scope and decisions: `docs/PRD.md`. Findings: `docs/research/`. Finished items: `docs/ROADMAP-HISTORY.md`.

## Open items

- None.

## Phases

| Phase | What | Status |
|---|---|---|
| 1 | Pipeline as a Windows CLI | Done |
| 2 | Summarizer through her assistant CLI, quote matching, limit-error pause | In Progress |
| 3 | PDF report in the escritorio layout | Not Started |
| 4 | Desktop window in Spanish | Not Started |
| 5 | macOS build with Metal, reference episodes, first on-demand report | Not Started |

## Phase 1: Pipeline as a Windows CLI

Build and test on Windows through `tools/build/cargo-windows.cmd` (VS dev env, Ninja, short target dir `%USERPROFILE%\.pmt` for the 260-char path limit, Vulkan SDK).

All items done; see `docs/ROADMAP-HISTORY.md`.

## Phase 2: Summarizer through her assistant CLI

Chunking decided with Felipe: the whole episode goes in one call when it is under a size cap, split at segment boundaries into equal parts only above it; each line is sent as `[mm:ss] text`. With the minute prefix Ep. 34 is 80,402 characters and Nada Personal 64,535, both one chunk at the 120,000 cap.

Reply shape decided with Felipe, enforced by a JSON schema (`claude -p --json-schema`, `codex exec --output-schema`): a Spanish summary, topics as name with `[mm:ss]` start and end spans, people named (names only), quotes as verbatim text with an approximate minute. The app sums spans for airtime, counts people in the corrected transcript and looks up every quote.

Quote rule decided with Felipe: exact match after folding case, accents and punctuation, across segment lines; the hit nearest the model's minute wins; the transcript's words and first segment's minute are printed; anything else is dropped.

| # | Item | Status |
|---|---|---|
| 1 | Transcript chunking: one chunk under the cap, equal parts at segment boundaries above it | Done |
| 2 | CLI wrapper: `claude -p` and `codex exec` behind one interface, glossary sent with each chunk | Done |
| 3 | Quote matching: find each quote in the transcript, print its text and minute, drop unmatched | Done |
| 4 | Limit-error pause: stop cleanly, keep finished episodes | Not Started |
| 5 | Extractive picker on `noctx-fixed.json` and `np-noctx-fixed.json` as the fallback check, read by Felipe | Not Started |
