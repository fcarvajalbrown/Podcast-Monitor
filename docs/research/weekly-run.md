# Weekly run from the window (phase 7 items 7 to 9)

First run, Wednesday 28 October, started 13:10 local time from the window with Codex, on the RTX 2050 laptop (15.8 GB RAM).

- The queue held 28 episodes (26 at 11:26 that morning, two more published since). Animales Políticos Ep. 34 was skipped as already finished.
- First new episode, Columnistas Página 13, 47 min of audio, 285 s in all: download 7 s, decode to 16 kHz 61 s, transcription 182 s (about 16x real time), Codex summary 34 s.
- 9 episodes finished by 13:53. The run stopped at about 13:55 while transcribing El Villegas "Kast y Lula: ¿pragmatismo o traición?": the session's background-task guard killed the window because the machine was low on memory. It was not a crash of the app. `current-week.json` was kept, so Continuar informe semanal picks up the same list.
- Memory of the app, sampled every 10 s by `tools/mem_sample/sample.py`: private memory rose to 1.9 GB on the first transcription, 2.7 GB at about 10 min, then stayed between 3.1 and 3.4 GB; working set peaked at 1.5 GB. It levels off, so it does not grow per episode. 5.1 GB of the 15.8 GB was free right after the stop.
- Decoding takes a third as long as the transcription, the first candidate for item 8.

## Second run, from Continuar informe semanal

Started 14:09 from the window launched on its own (not from the agent shell), Codex, log `logs/2026-10-28_14-08-54_15188.log` in the app-data folder.

- The 10 episodes finished in the first run were skipped; the other 18 ran one after another from 14:09 to 15:31 (82 min, 119 to 431 s per episode, 274 s on average), none failed, no Codex limit, no fallback, no low-memory wait (5.3 GB free at the start). The Whisper model was loaded once for the whole queue.
- Codex summaries took 23 to 51 s each; the 13 per-show week calls and the PDF took 5.5 min (15:31 to 15:36).
- The PDF, `weeks/2026-10-26/informe-semanal.pdf`, is 6 pages: 13 show blocks in alphabetical order, the 4 shows with no episode this week listed at the end, closing line "Resúmenes hechos con Codex."
- Three blocks have no topic table (Panelistas de Pauta de Análisis, Polos Opuestos, Rat Pack de Mesa Central): the prompt lists each topic as `E1: name (mm:ss)` and for those shows Codex copied the `(mm:ss)` into the group labels, so no label matched and every topic was left out (95 warnings in the log).
- El Villegas lists its days as "lunes 26, lunes 26 y martes 27": two episodes on the same day are named twice.
- One topic table (Columnistas Página 13) breaks across pages 1 and 2, leaving one row and its header on page 2.
- Each Codex call read Felipe's own instruction files before answering: `~/.claude/CLAUDE.md` 74 times and the humanizer, felipe-outward-writing and ponytail skills, through Codex's global instructions. It worked, but the summaries ran under instructions written for Felipe's own work, and the reader's machine will not have them.
- The two `mpa: invalid main_data_begin` and `xing tag ... crc mismatch` warnings come from the MP3 decoder on damaged frames and did not stop any decode; the one ERROR line is Codex's own model-list refresh timing out.
