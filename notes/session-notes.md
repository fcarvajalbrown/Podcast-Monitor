# Session notes

Current state: phase 1 done; phase 2 items 1 to 5 done (chunking, `summarize` through claude or codex, quote matching, limit-error pause with exit code 2, extractive picker kept as fallback). See `ROADMAP.md` and `docs/ROADMAP-HISTORY.md`. Saved replies: `work/summary-ep34-claude.json`, `work/summary-np-codex.json`.

Open flags:
- Phase 2 item 6: limit markers not yet checked against a real limit message from either CLI.
- Not built yet: airtime per topic from the summed spans, people counts in the corrected transcript, merge of parts when an episode splits. Where they live (end of phase 2 or phase 3 report) not decided.
- Picker fallback is Python (`tools/extractive_pick/pick.py`); how the Rust app uses it on a limit error not decided. Two ad picks on Nada Personal; no ad filter.
- Ep. 34 23:44 "a su diestra" likely "a su derecha" (Felipe 80 to 90 % sure), not checked against audio.
- Open: with context off Boric is 6 on Ep. 34 (target 7); Kast 16. Do not change the target; ask Felipe if it comes up.
- whisper-rs 424 s against whisper-cli 354 s on the RTX 2050, not investigated.
- No CPU-only or integrated-GPU speed measured; left by Felipe's choice.
- PRD open question: which assistant subscription the reader has.
