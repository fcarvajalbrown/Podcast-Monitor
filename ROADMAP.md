# Roadmap

Scope and decisions: `docs/PRD.md`. Findings: `docs/research/`. Finished items: `docs/ROADMAP-HISTORY.md`.

## Open items

- None.

## Phases

| Phase | What | Status |
|---|---|---|
| 1 | Pipeline as a Windows CLI | Done |
| 2 | Summarizer through her assistant CLI, quote matching, limit-error pause | In Progress |
| 3 | PDF report in the escritorio layout | In Progress |
| 4 | Desktop window in Spanish | Not Started |
| 5 | macOS build with Metal, reference episodes, first on-demand report | Not Started |

## Phase 1: Pipeline as a Windows CLI

Build and test on Windows through `tools/build/cargo-windows.cmd` (VS dev env, Ninja, short target dir `%USERPROFILE%\.pmt` for the 260-char path limit, Vulkan SDK).

All items done; see `docs/ROADMAP-HISTORY.md`.

## Phase 2: Summarizer through her assistant CLI

Chunking decided with Felipe: the whole episode goes in one call when it is under a size cap, split at segment boundaries into equal parts only above it; each line is sent as `[mm:ss] text`. With the minute prefix Ep. 34 is 80,402 characters and Nada Personal 64,535, both one chunk at the 120,000 cap.

Reply shape decided with Felipe, enforced by a JSON schema (`claude -p --json-schema`, `codex exec --output-schema`): a Spanish summary, topics as name with `[mm:ss]` start and end spans, people named (names only), quotes as verbatim text with an approximate minute. The app sums spans for airtime, counts people in the corrected transcript and looks up every quote.

Quote rule decided with Felipe: exact match after folding case, accents and punctuation, across segment lines; the hit nearest the model's minute wins; the transcript's words and first segment's minute are printed; anything else is dropped.

Episode figures decided with Felipe, built at the end of phase 2 so the phase 3 report only lays them out: parts of a split episode merge first (summary paragraphs joined in order, no extra call; topics, people and quotes concatenated), then airtime per topic is the sum of its spans, ranked longest first, then each person is counted by the last word of their name as an exact, case-sensitive word in the corrected transcript (as `count_names.py` does; people sharing a surname are both credited), ranked by count, people with 0 mentions kept at the bottom.

| # | Item | Status |
|---|---|---|
| 1 | Transcript chunking: one chunk under the cap, equal parts at segment boundaries above it | Done |
| 2 | CLI wrapper: `claude -p` and `codex exec` behind one interface, glossary sent with each chunk | Done |
| 3 | Quote matching: find each quote in the transcript, print its text and minute, drop unmatched | Done |
| 4 | Limit-error pause: stop cleanly, keep finished episodes | Done |
| 6 | Limit text seen from a real limit on either CLI, matched by the markers | Not Started |
| 5 | Extractive picker on `noctx-fixed.json` and `np-noctx-fixed.json` as the fallback check, read by Felipe | Done |
| 7 | Merge the parts of a split episode into one record | Done |
| 8 | Airtime per topic from the summed spans, ranked | Done |
| 9 | People counts by surname in the corrected transcript, ranked, zeros kept | Done |

## Phase 3: PDF report in the escritorio layout

Layout reference: `felipe_skill_voice/scripts/gen_pdf_desk.py` (A4, 2.5 cm margins, Latin Modern Roman with its GUST licence, grey italic header line over a thin rule, which Felipe chose to draw in the logo red #B70002 instead of grey, justified body, booktabs-style tables, centred page number). Items approved by Felipe in this order.

| # | Item | Status |
|---|---|---|
| 1 | PDF crate pick: one test page per open-source Rust candidate (Latin Modern embedded, Spanish accents, justified text, a table, page numbers); Felipe picked Typst (`docs/research/pdf-crates.md`) | Done |
| 2 | Page frame: A4, 2.5 cm margins, grey italic header line with thin rule, centred page number | Done |
| 3 | Text blocks: headings, paragraphs wrapping across pages, heading kept with the next block, booktabs table | Not Started |
| 4 | Episode section in Spanish: show, episode, date, duration, summary, topics with airtime, people with counts (zeros last), quotes with [mm:ss] | Not Started |
| 5 | `report` command: one PDF from every finished summary in a run folder | Not Started |
| 6 | Acceptance on Ep. 34 and Nada Personal from the saved replies: every printed quote found at its minute, layout checked against `gen_pdf_desk.py` output | Not Started |
