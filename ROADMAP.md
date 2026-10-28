# Roadmap

Scope and decisions: `docs/PRD.md`. Findings: `docs/research/`. Finished items: `docs/ROADMAP-HISTORY.md`.

## Open items

- None.

## Phases

| Phase | What | Status |
|---|---|---|
| 1 | Pipeline as a Windows CLI | Done |
| 2 | Summarizer through her assistant CLI, quote matching, limit-error pause | In Progress |
| 3 | PDF report in the escritorio layout | Done |
| 4 | Desktop window in Spanish | In Progress |
| 5 | macOS build with Metal, reference episodes, first on-demand report | Deferred |
| 6 | Free-AI path for a reader with no paid assistant | Not Started |
| 7 | Weekly report across the show list | In Progress |

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

Episode section layout chosen by Felipe (layout A): show title as the level-1 heading, episode title as level 2, one line "Publicado el 26 de octubre de 2026. Duración: 1 h 18 min." (date in the reader's local time), then level-3 headings Resumen, Temas por tiempo al aire (table Tema, Tiempo al aire), Personas mencionadas (table Persona, Menciones) and Citas (one paragraph per quote, `[mm:ss] «text»`); an empty part is left out.

Report order chosen by Felipe: shows alphabetically, newest episode first within a show, each episode on a new page. `run` now saves the feed's title as `show.json` in the show folder; an older folder without it falls back to the folder name with a warning.

All items done; see `docs/ROADMAP-HISTORY.md`.

## Phase 4: Desktop window in Spanish

Native Rust window in iced 0.14, Spanish UI strings, code and logs in English (`docs/PRD.md` sections 5 to 7). Items approved by Felipe in this order. Chosen by Felipe:

- GUI crate: iced (`docs/research/gui-crates.md`).
- Data: show list and run folders in the OS app-data folder (`~/Library/Application Support/Podcast-Monitor`, `%APPDATA%\Podcast-Monitor`); the CLI keeps `work/`.
- Run mode, for the PRD's "one button per run mode": select a saved show, pick one of its newest episodes, press one run button.
- Whisper model copied once to `models/ggml-large-v3-turbo.bin` in the app-data folder; the window says where if it is missing; download on first run left to phase 5.
- Summarizer: Claude or Codex, chosen in the window and saved in the app-data folder.
- Glossary: one `glossary.txt` in the app-data folder, if present, serves every run.

| # | Item | Status |
|---|---|---|
| 1 | GUI crate pick: one test window per open-source native Rust candidate (egui, iced, Slint) with Spanish text, a list, buttons, a progress bar and an Open PDF action; Felipe picks from screenshots | Done |
| 2 | Saved show list: add by pasted feed URL, remove, kept on disk | Done |
| 3 | Add by name search in Apple's directory, confirm step showing the publisher | Done |
| 4 | On-demand episode run in the background: transcribe, summarize, report; progress per episode; resume after a closed app; limit-error stop explained in Spanish | Done |
| 5 | Open PDF at the end of a run | Done |
| 6 | Acceptance on Ep. 34 and Nada Personal driven from the window | Not Started |

## Phase 5: macOS build

Deferred indefinitely by Felipe after the reader's feedback: he runs the tool on his own PCs.

## Phase 7: Weekly report across the show list

Asked by the reader (`docs/research/reader-feedback.md`): this week's episodes of her show list in one PDF, short per show, keeping the summary and the topics by airtime in percent only. Priority set by Felipe: this phase comes next, ahead of phase 4 item 6 and phase 6. Items are approved by Felipe before they are written here.

Chosen by Felipe:

- The weekly run is built into the app first, then this week's episodes go through it.
- Summarizer fallback: when Codex hits its limit the run switches on its own to Claude pinned to Sonnet 5, and the report says which assistant summarized each episode.
- The week is Monday to now, in local time.

| # | Item | Status |
|---|---|---|
| 1 | Debug log: every event of every run, window or CLI (stages with time, assistant calls and replies, errors, panics, settings changes) to `logs/` in the app-data folder | Done |
| 2 | Her 17 shows with a feed loaded into the saved show list | Done |
| 3 | Week picker: the episodes published in a date range across the saved shows | Done |
| 4 | Run queue: those episodes one after another, resumable, progress per episode | Not Started |
| 5 | Automatic fallback: a Codex limit switches the run to Claude Sonnet 5, assistant noted per episode | Not Started |
| 6 | Short per-show layout: sections and length chosen by Felipe, summary and topics in percent | Not Started |
| 7 | Weekly PDF: every show of the week in one file, from a Generar informe semanal button | Not Started |
| 8 | Bottlenecks: stage times read from the debug log | Not Started |
| 9 | Acceptance: this week's report sent to the reader, copy kept in `work/sent/` | Not Started |

## Phase 6: Free-AI path for a reader with no paid assistant

Asked by Felipe: a reader without Claude or ChatGPT subscriptions can use ChatGPT in the browser or any free AI chat by copy and paste. Both items wanted, for flexibility; their order is decided when the phase starts.

| # | Item | Status |
|---|---|---|
| 1 | Summary by copy and paste: the window copies the prompt and the `[mm:ss]` transcript text, she pastes them into any browser AI chat and pastes the reply back; the app checks quotes, ranks airtime and people, and builds the PDF as with the CLI | Not Started |
| 2 | Transcript from a browser AI: she uploads the audio to a browser AI and pastes the transcript back; the app skips Whisper and runs the name fix, summary and PDF | Not Started |
