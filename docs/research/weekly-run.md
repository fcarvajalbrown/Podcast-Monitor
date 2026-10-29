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

## Stage times (phase 7 item 8)

Read from the debug logs with `tools/log_stages/stages.py <log>`. Second run, 18 episodes, 12.6 h of audio, 82.3 min in all:

| Stage | Time | Share | Median per episode |
|---|---|---|---|
| Download | 1.7 min | 2 % | 6 s |
| Decode to 16 kHz | 18.3 min | 22 % | 64 s |
| Transcription | 52.4 min | 64 % | 171 s (12 to 16x real time) |
| Codex summary | 9.9 min | 12 % | 31 s |
| Name fix and episode PDF | under 1 s | 0 % | 0 s |

The first run's 9 episodes split the same way (21 %, 66 %, 11 %).

Decode was the resampler, not the MP3 decoder: on El Villegas "Kast y Lula" (2,681 s at 44.1 kHz) symphonia decoded in 2.7 s, and the resampler took 55.3 s because it worked out sine and cosine terms for 64 taps of every output sample. Its weights repeat (160 phases for 44.1 kHz, 1 for 48 kHz), so they are now computed once per phase: 1.6 s, output within 6e-8 of the old filter (`audio::tests::times_decode_and_both_resamplers_on_a_real_episode`, run with `PM_AUDIO=<file> -- --ignored`). On this week's audio that is about 17 of the 82 minutes saved.

Transcription is now about 80 % of the run and the next lever; the Codex call is the one after it.

## Week-wide topic table (phase 7 item 11)

Built Thursday 29 October from the 28 episodes already finished, with `podcast-monitor week --report`: the 13 per-show week summaries were reused from their saved `week-2026-10-26.json` (none redone), and one Codex call merged the 13 shows' theme groups into the week table, saved as `weeks/2026-10-26/themes.json`. No label was left out of a group. The PDF is 4 pages, down from 7, and the table adds up to 99 %: Justicia, penas y caso Rabat 30 %, Elecciones y política brasileña 16 %, Gobierno y política chilena 15 %, Presentaciones y segmentos 10 %, then four themes between 5 and 8 %. Intros, ads and segment breaks take one row of 10 % (the open no-ad-filter flag). Nine episodes published since Wednesday were not transcribed and are left out. Copy: `work/sent/2026-10-29-semanal-semana-2026-10-26-cli.pdf`.
