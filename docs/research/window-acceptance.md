# Window acceptance (phase 4 item 6)

Question: do Ep. 34 of Animales Políticos and the Nada Personal episode of 26 October 2026 run end to end from the window, with the same results as the CLI.

Setup: release build of `podcast-monitor.exe` (113.3 MB) with no command, so it opens the window; app-data folder `%APPDATA%\Podcast-Monitor` with the turbo model hard-linked into `models/` and `tools/stt_compare/glossary-ap-ep34.txt` as `glossary.txt` for Ep. 34. Felipe clicked through the window on the RTX 2050 laptop.

## Ep. 34

- Animales Políticos added by name search (Casa Editorial Lo Que Importa) and Nada Personal by pasted feed URL; both saved to `shows.json` with title and publisher.
- Whisper ran on Vulkan0, the NVIDIA GeForce RTX 2050 (the only Vulkan device found): 4,685 s of audio in 395 s (the CLI took 424 s).
- Name fix: same eight replacements as before (Cass, Kass and Cast to Kast; Esquella to Squella; Belolio to Bellolio; París to Parisi; Arena to Arenas; Contarto to Contardo). Kast 16 and Boric 6 in the corrected transcript, equal to the context-off CLI result.
- The log shows two runs of the episode: the first summarized and wrote `informe.pdf`; the second skipped the transcript, summarized again and rewrote the PDF. `settings.json` and `summary.json` say Codex, so the PDF on disk carries the Codex summary, not Claude's. A run on a finished episode with the other assistant replaces the earlier summary without asking.
- PDF (3 pages): `tools/pdf_check/quotes.py` finds all 5 quotes at their printed minute in `transcript.fixed.json`; `tools/pdf_check/measure.py` gives Latin Modern only, header baseline 86.4 pt, red rule 91.4 pt, title baseline 125.4 pt, text column from 76.9 pt, as in phase 3.
- Felipe ran the second pass on purpose to try Codex and accepted Ep. 34 with the Codex summary.
- Abrir PDF opened the file in the system viewer (a LibreOffice lock file appeared next to it).
- Memory: not sampled during the transcription; the idle window afterwards used 110 MB.

## Ep. 34 again, release build of 29 October

- The weekly run had left a finished summary and PDF in the episode folder, so the first press of Generar informe skipped every stage (0.0 s). The folder was renamed to `...-segunda-t.bak-weekly` and the run started fresh.
- 243 s from Generar informe to the PDF: download and decode 12 s, transcription 183 s (greedy with flash attention), names under 1 s, Codex 41 s, PDF under 1 s.
- PDF 4 pages; `tools/pdf_check/quotes.py` finds 5 of 5 quotes at their printed minute; Kast 17, Boric 8.
- The glossary in use no longer had Bellolio, so the summary printed "Belolio"; `Bellolio = Belolio`, Matamala and Contardo were added afterwards.
- Abrir PDF opened the report in the system viewer (Felipe).
- Started through Explorer as a double-click starts it: only the window, no console.

## Nada Personal of 26 October, release build of 29 October

- Folder renamed to `...-machado-a-ve.bak-weekly` first, so the run started fresh.
- 200 s from Generar informe to the PDF: download 8 s, decode 12 s, transcription 147 s, names under 1 s, Codex 33 s.
- Name fix: Cass and Kass to Kast (2 each), Rios to Ríos (1).
- PDF 3 pages; `tools/pdf_check/quotes.py` finds 4 of 4 quotes at their printed minute (the reply gave four of the five allowed).
- Lula 24, Machado 10, Kast 6, against 23, 10 and 7 in the beam-5 CLI run of phase 2.
- Seen in the PDF: ads rank as topics ("Publicidad y mensajes de auspiciadores" 05:47, "Publicidad de telefonía móvil"), as in the open flag; "Dan 1" in the people table.
