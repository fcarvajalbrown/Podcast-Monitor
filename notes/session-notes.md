# Session notes

Current state: phases 1 and 3 done; phase 2 done except item 6 (text of a real usage-limit message, blocked until a real limit is hit). `podcast-monitor report [run-folder]` builds the Spanish PDF in the escritorio layout through Typst (`src/report.rs`, `src/report.typ`, `src/section.rs`, `src/assemble.rs`); checks in `tools/pdf_check/` (`measure.py` positions, `quotes.py` quotes at their minute). Saved replies: `work/summary-ep34-claude.json`, `work/summary-np-codex.json`. Next: phase 4, the desktop window in Spanish (not planned yet). See `ROADMAP.md` and `docs/ROADMAP-HISTORY.md`.

Open flags:
- Phase 2 item 6: limit markers not yet checked against a real limit message from either CLI.
- Ads rank as a topic: "Publicidad y mensajes de patrocinadores" is the top airtime topic on Nada Personal (5:52); no ad filter.
- Merge of parts tested only by unit test; no saved episode splits.
- Picker fallback is Python (`tools/extractive_pick/pick.py`); how the Rust app uses it on a limit error not decided. Two ad picks on Nada Personal; no ad filter.
- Ep. 34 23:44 "a su diestra" likely "a su derecha" (Felipe 80 to 90 % sure), not checked against audio.
- Open: with context off Boric is 6 on Ep. 34 (target 7); Kast 16. Do not change the target; ask Felipe if it comes up.
- whisper-rs 424 s against whisper-cli 354 s on the RTX 2050, not investigated.
- No CPU-only or integrated-GPU speed measured; left by Felipe's choice.
- App size: Windows release exe 103.3 MB with the report (63.2 MB before); no macOS build measured.
- Older run folders have no `show.json`; `report` falls back to the folder name until `run` is rerun on that feed.
- PRD open question: which assistant subscription the reader has.
