# Session notes

Current state: phases 1, 3 and 7 done; the reader liked the weekly report with the week-wide topic table and conclusion summaries (`work/sent/2026-10-29-semanal-semana-2026-10-26-cli-conclusiones.pdf`). Platforms are Windows and macOS on Apple Silicon M1 and later; phase 5 (macOS) starts after phase 4 item 6. The app transcribes greedy with flash attention (334 s to 158 s on Ep. 34, `docs/research/stt-speed-ep34-rtx2050.md`); `tools/stt_speed/bench.sh` compares variants, `tools/resummarize/resummarize.sh <monday>` redoes a week's summaries from saved transcripts. The window has a Generar / Continuar informe semanal button; `podcast-monitor week [--run] [--report]` does the same from a terminal. Week of 26 October: PDF at `%APPDATA%\Podcast-Monitor\weeks\2026-10-26\informe-semanal.pdf` (13 shows, 7 pages), copy in `work/sent/`. Debug logs in `%APPDATA%\Podcast-Monitor\logs\`; `tools/log_stages/stages.py <log>` times each stage. Build with `tools/build/cargo-windows.cmd <args> -j 2` from bash. See `ROADMAP.md`, `docs/ROADMAP-HISTORY.md`, `docs/research/weekly-run.md`.

Open flags:
- Phase 4 item 6: Ep. 34 passed from the window (see ROADMAP); Nada Personal and the Abrir PDF viewer launch still to do.
- "Belolio" for Bellolio in the Ep. 34 people table; not in the glossary.
- The old Ep. 34 run folder sits as `runs/animales-politicos/...-segunda-t.bak-weekly`; check whether the weekly report picks it up.
- Greedy with flash attention not yet measured on Metal. Silero VAD now runs but is 40 % slower on Ep. 34; kept off by default.
- Glossary takes heard-as lines (`Name = heard, heard`); this week's five misspelled names are in, but the saved fixed transcripts and summaries of the week were not redone.
- Weekly grouping can split one story across two themes.
- Phase 2 item 6: limit markers not checked against a real limit message.
- No ad filter: ads rank as topics and as picker picks.
- Rerunning an episode with another provider overwrites its summary.
- Boric 6 on Ep. 34 with context off (target 7); do not change the target.
- whisper-rs 424 s against whisper-cli 354 s on the RTX 2050, not investigated.
- No CPU-only or integrated-GPU speed measured.
- Older run folders without `show.json` fall back to the folder name.
