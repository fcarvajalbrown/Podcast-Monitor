# Roadmap

Scope and decisions: `docs/PRD.md`. Findings: `docs/research/`. Finished items: `docs/ROADMAP-HISTORY.md`.

## Open items

- Episode report not yet reshaped to the reader's comments: topics in minutes (she asked percent only), 28 topics, 41 people with 10 at 0, 4 pages for Ep. 34 (`docs/research/reader-feedback.md`). Scope to be set with Felipe.
- The weekly report the reader approved (`work/sent/2026-10-29-semanal-semana-2026-10-26-cli-conclusiones.pdf`) stays as it is: Felipe set it as the report that matters most, read every Monday early morning.
- Glossary improvements get a session of their own to study them, set by Felipe.
- Preprint of the measured results (speech-to-text speed, heard-as name fix, quote checks) on TechRxiv and engrXiv, chosen by Felipe; arXiv is out for lack of a sponsor. A session for it is set for 30 October 10:00.
- Preprint measurements set by Felipe before writing, on the 28 saved episodes of the week of 26 October: every quote checked against its transcript, every name-fix change judged by hand by Felipe, 10 episodes timed in both speed settings plus 3 repeats of one. No word error rate, so the paper claims names, quotes and speed only.

- Release exe (114 MB, one file) needs the Visual C++ runtime (MSVCP140, VCRUNTIME140, VCRUNTIME140_1), not present on every Windows; Felipe chose to bundle Microsoft's vc_redist.x64.exe in the installer, run silently only when the runtime is missing.
- Release exe links vulkan-1.dll, which only a GPU driver with Vulkan installs; a PC without it may not start the app. Felipe: most readers have integrated graphics, the release must be ready for them. Intel added Vulkan on Windows from 6th-gen Core (Skylake) on, per 2017-2018 reports; older chips have none.
- Integrated-graphics plan chosen by Felipe: the installer ships Khronos's vulkan-1.dll next to the exe; check transcription falls back to the CPU when no Vulkan driver works; time Ep. 34 on integrated graphics. This laptop (Ryzen 7 7435HS, RTX 2050) has no integrated GPU, so that timing needs another PC; the CPU timing runs here.
- CPU fallback checked: with Vulkan's loader pointed at a missing driver file (VK_DRIVER_FILES), whisper.cpp logs "no GPU found", loads the model on the CPU and the run goes on (log `2026-11-04_10-09-57_29212.log`).
- CPU-only speed: Ep. 34 (78 min) in 4,295 s compute, 1.1x real time, greedy with flash attention, Ryzen 7 7435HS, release exe with no Vulkan driver visible (`tools/stt_speed/bench.sh` into `work/cpu-fallback`); the RTX 2050 takes 158 s. A reader on the CPU waits about as long as the episode lasts.
- Installer 0.1.0 built by `tools/build/release-windows.sh` at a2efd0e: `work/installer/PodcastMonitor-windows-setup.exe`, 45,809,576 bytes, sha256 e5023f8c0aa821264129a3c97d7cb792c98a6ec8ddf8db6e37940f424f65b048; the exe carries the icon, vulkan-1.dll built from Khronos tag vulkan-sdk-1.4.363.0 (its version field reads "Dev Build"). Not yet installed on any PC.
- vulkan-1.dll source chosen by Felipe: built from Khronos's open-source Vulkan-Loader (Apache 2.0), since the SDK only has it inside LunarG's Qt installer VulkanRT.exe, which 7-Zip cannot unpack.
- Release exe carries no icon (no Windows resource is embedded); the `.ico` exists in `assets/brand/`. Felipe chose to embed the current icon in 0.1.0 (exe resource through winresource, window icon from the 256 px PNG); phase 8 swaps the image.

## Phases

| Phase | What | Status |
|---|---|---|
| 1 | Pipeline as a Windows CLI | Done |
| 2 | Summarizer through her assistant CLI, quote matching, limit-error pause | In Progress |
| 3 | PDF report in the escritorio layout | Done |
| 4 | Desktop window in Spanish | Done |
| 5 | macOS build with Metal, reference episodes, first on-demand report | Not Started |
| 6 | Free-AI path for a reader with no paid assistant | Not Started |
| 7 | Weekly report across the show list | Done |
| 8 | App icon and window design | Not Started |
| 9 | Download site at podcastmonitor.copiapolabs.cl | Not Started |

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
| 6 | Acceptance on Ep. 34 and Nada Personal driven from the window | Done |


## Phase 5: macOS build

Back in scope: Felipe set the platforms to Windows and macOS on Apple Silicon, M1 and later (first deferred after the reader's feedback). Order set by Felipe: this phase starts after phase 4 item 6, so the Mac build ports a window already accepted on Windows. Items are approved by Felipe before they are written here. The new transcription default (greedy with flash attention, `docs/research/stt-speed-ep34-rtx2050.md`) is measured on Vulkan only, not yet on Metal.

No Mac needed to start, set by Felipe: the build runs on GitHub Actions macOS runners. GitHub's hosted Apple Silicon runners have no Metal device (Apple Virtualization Framework limit, GitHub larger-runner docs and runner-images discussion 6138), so Actions can build and test the Metal exe but cannot measure Metal speed; that needs a real Mac.

## Phase 7: Weekly report across the show list

Asked by the reader (`docs/research/reader-feedback.md`): this week's episodes of her show list in one PDF, short per show, keeping the summary and the topics by airtime in percent only. Priority set by Felipe: this phase comes next, ahead of phase 4 item 6 and phase 6. Items are approved by Felipe before they are written here.

Chosen by Felipe:

- The weekly run is built into the app first, then this week's episodes go through it.
- Summarizer fallback: when Codex hits its limit the run switches on its own to Claude pinned to Sonnet 5, and the report says which assistant summarized each episode.
- The week is the 7 days before Generar informe semanal is pressed (it was Monday to now, which found 0 episodes on a Monday). Shows posted Monday near 05:00 then fall in two reports; Felipe judged that minor. The PDF title reads "últimos 7 días al <date>"; `week --monday` keeps "semana del <monday>".
- The weekly report has one block per show, covering that show's episodes of the week together.
- Each block: a header line (show, publisher, episodes with their days, total duration), a 3 to 4 line summary of the week, and 5 to 8 grouped topics with percent of airtime only. No people, no quotes.
- A closing line in the PDF says which assistant summarized which show, an exception Felipe made for this report. The weekly run starts on Codex.
- After the reader's feedback on the first weekly report (`docs/research/reader-feedback.md`): one "Temas de la semana" table at the top, percent of all the week's airtime across every show, replaces the per-show topic tables. Built by one more assistant call that merges each show's saved theme groups into 5 to 8 week themes.

| # | Item | Status |
|---|---|---|
| 1 | Debug log: every event of every run, window or CLI (stages with time, assistant calls and replies, errors, panics, settings changes) to `logs/` in the app-data folder | Done |
| 2 | Her 17 shows with a feed loaded into the saved show list | Done |
| 3 | Week picker: the episodes published in a date range across the saved shows | Done |
| 4 | Run queue: those episodes one after another, resumable, progress per episode | Done |
| 5 | Automatic fallback: a Codex limit switches the run to Claude Sonnet 5, assistant noted per episode | Done |
| 6 | Short per-show layout: sections and length chosen by Felipe, summary and topics in percent | Done |
| 7 | Weekly PDF: every show of the week in one file, from a Generar informe semanal button | Done |
| 8 | Bottlenecks: stage times read from the debug log | Done |
| 10 | Low-memory guard, before item 9: under 2 GB free the run unloads the model and waits, showing free memory and a Reanudar ahora button; goes on at 3 GB; stops cleanly after 15 min | Done |
| 11 | Week-wide topic table replacing the per-show tables, from one call over the shows' theme groups | Done |
| 9 | Acceptance: this week's report sent to the reader, copy kept in `work/sent/` | Done |

## Phase 6: Free-AI path for a reader with no paid assistant

Asked by Felipe: a reader without Claude or ChatGPT subscriptions can use ChatGPT in the browser or any free AI chat by copy and paste. Both items wanted, for flexibility; their order is decided when the phase starts.

| # | Item | Status |
|---|---|---|
| 1 | Summary by copy and paste: the window copies the prompt and the `[mm:ss]` transcript text, she pastes them into any browser AI chat and pastes the reply back; the app checks quotes, ranks airtime and people, and builds the PDF as with the CLI | Not Started |
| 2 | Transcript from a browser AI: she uploads the audio to a browser AI and pastes the transcript back; the app skips Whisper and runs the name fix, summary and PDF | Not Started |

## Phase 8: App icon and window design

Asked by Felipe as future work, then pressed after seeing the model download in the window: "we really need a new UI and fucking favicon". Order set by Felipe: phase 9 releases first with today's window, this phase after. The app icon and a window design modelled on Cuña-Cutter (`C:\Projects\Cuña-Cutter`: `assets/brand/`, PRD section 7, ADR 0008). Items are ordered when the phase starts.

| # | Item | Status |
|---|---|---|
| 1 | App icon: the icons in `assets/brand/` embedded in the Windows exe and shown on the window | Not Started |
| 2 | Window design modelled on Cuña-Cutter's UX | Not Started |

## Phase 9: Download site

Asked by Felipe: a public download page at https://podcastmonitor.copiapolabs.cl/, modelled on Cuña-Cutter's (its ADR 0015). Chosen by Felipe:

- Windows build at launch; the Apple Silicon button reads Próximamente until phase 5 has a build.
- The app downloads the Whisper model on first run, from a Descargar modelo button, SHA-256 checked, resumable (`docs/research/model-download.md`); the site offers only the app.
- Windows is installed by an Inno Setup installer (exe, Start menu shortcut, uninstaller).
- Files on the Hostinger hosting next to the page, at the fixed name `/descargas/PodcastMonitor-windows-setup.exe`.
- The page and the app show the SemVer version; each release has a matching CHANGELOG entry and git tag.
- Same layout and colours as Cuña-Cutter's page, in Spanish; guide steps set with Felipe when item 4 starts.
- Out of search engines and AI crawlers: robots.txt, noindex meta tag, X-Robots-Tag header.

| # | Item | Status |
|---|---|---|
| 1 | Model download on first run: the window fetches `ggml-large-v3-turbo.bin` into the app-data folder, with progress | Done |
| 2 | Version shown in the window title, "Podcast-Monitor 0.1.0" from Cargo.toml; first public release is 0.1.0, set by Felipe | Done |
| 3 | First release: CHANGELOG entry, git tag, Windows installer built with Inno Setup | Not Started |
| 4 | Page in Spanish with Cuña-Cutter's layout and colours, guide, out of indexes | Not Started |
| 5 | Upload to podcastmonitor.copiapolabs.cl, every link checked | Not Started |
