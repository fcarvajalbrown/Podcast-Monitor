# Weekly run from the window (phase 7 items 7 to 9)

First run, Wednesday 28 October, started 13:10 local time from the window with Codex, on the RTX 2050 laptop (15.8 GB RAM).

- The queue held 28 episodes (26 at 11:26 that morning, two more published since). Animales Políticos Ep. 34 was skipped as already finished.
- First new episode, Columnistas Página 13, 47 min of audio, 285 s in all: download 7 s, decode to 16 kHz 61 s, transcription 182 s (about 16x real time), Codex summary 34 s.
- 9 episodes finished by 13:53. The run stopped at about 13:55 while transcribing El Villegas "Kast y Lula: ¿pragmatismo o traición?": the session's background-task guard killed the window because the machine was low on memory. It was not a crash of the app. `current-week.json` was kept, so Continuar informe semanal picks up the same list.
- Memory of the app, sampled every 10 s by `tools/mem_sample/sample.py`: private memory rose to 1.9 GB on the first transcription, 2.7 GB at about 10 min, then stayed between 3.1 and 3.4 GB; working set peaked at 1.5 GB. It levels off, so it does not grow per episode. 5.1 GB of the 15.8 GB was free right after the stop.
- Decoding takes a third as long as the transcription, the first candidate for item 8.
