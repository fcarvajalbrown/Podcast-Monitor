# Window acceptance (phase 4 item 6)

Question: do Ep. 34 of Animales Políticos and the Nada Personal episode of 26 October 2026 run end to end from the window, with the same results as the CLI.

Setup: release build of `podcast-monitor.exe` (113.3 MB) with no command, so it opens the window; app-data folder `%APPDATA%\Podcast-Monitor` with the turbo model hard-linked into `models/` and `tools/stt_compare/glossary-ap-ep34.txt` as `glossary.txt` for Ep. 34. Felipe clicked through the window on the RTX 2050 laptop.

## Ep. 34

- Animales Políticos added by name search (Casa Editorial Lo Que Importa) and Nada Personal by pasted feed URL; both saved to `shows.json` with title and publisher.
- Whisper ran on Vulkan0, the NVIDIA GeForce RTX 2050 (the only Vulkan device found): 4,685 s of audio in 395 s (the CLI took 424 s).
- Name fix: same eight replacements as before (Cass, Kass and Cast to Kast; Esquella to Squella; Belolio to Bellolio; París to Parisi; Arena to Arenas; Contarto to Contardo). Kast 16 and Boric 6 in the corrected transcript, equal to the context-off CLI result.
- The log shows two runs of the episode: the first summarized and wrote `informe.pdf`; the second skipped the transcript, summarized again and rewrote the PDF. `settings.json` and `summary.json` say Codex, so the PDF on disk carries the Codex summary, not Claude's. A run on a finished episode with the other assistant replaces the earlier summary without asking.
- PDF (3 pages): `tools/pdf_check/quotes.py` finds all 5 quotes at their printed minute in `transcript.fixed.json`; `tools/pdf_check/measure.py` gives Latin Modern only, header baseline 86.4 pt, red rule 91.4 pt, title baseline 125.4 pt, text column from 76.9 pt, as in phase 3.
- Abrir PDF opened the file in the system viewer (a LibreOffice lock file appeared next to it).
- Memory: not sampled during the transcription; the idle window afterwards used 110 MB.
