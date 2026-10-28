# Session notes

Current state: phases 1 and 3 done; phase 5 deferred; phase 7 (weekly report) done except item 9, the reader receiving this week's report, which Felipe handles himself. The window has a Generar / Continuar informe semanal button; `podcast-monitor week [--run] [--report]` does the same from a terminal. Week of 26 October: PDF at `%APPDATA%\Podcast-Monitor\weeks\2026-10-26\informe-semanal.pdf` (13 shows, 7 pages), copy in `work/sent/`. Debug logs in `%APPDATA%\Podcast-Monitor\logs\`; `tools/log_stages/stages.py <log>` times each stage. Build with `tools/build/cargo-windows.cmd <args> -j 2` from bash. See `ROADMAP.md`, `docs/ROADMAP-HISTORY.md`, `docs/research/weekly-run.md`.

Open flags:
- Console fix (3644824) waits for Felipe's double-click check; ROADMAP open item.
- Phase 7 item 9: done when Felipe says the reader has the report; copy goes to `work/sent/`.
- Phase 4 item 6: acceptance on Ep. 34 and Nada Personal from the window, not started.
- Transcription is about 80 % of a run after the resampler fix; next speed lever, not started.
- Weekly grouping can split one story across two themes.
- Phase 2 item 6: limit markers not checked against a real limit message.
- No ad filter: ads rank as topics and as picker picks.
- Rerunning an episode with another provider overwrites its summary.
- Boric 6 on Ep. 34 with context off (target 7); do not change the target.
- whisper-rs 424 s against whisper-cli 354 s on the RTX 2050, not investigated.
- No CPU-only or integrated-GPU speed measured.
- Older run folders without `show.json` fall back to the folder name.
