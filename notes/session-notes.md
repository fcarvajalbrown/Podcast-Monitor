# Session notes

Current state: phase 1 done; phase 2 items 1 to 5 and 7 to 9 done (chunking, `summarize` through claude or codex, quote matching, limit-error pause with exit code 2, extractive picker kept as fallback, merge of parts, airtime per topic, people counts by surname); only item 6 is open. See `ROADMAP.md` and `docs/ROADMAP-HISTORY.md`. Saved replies: `work/summary-ep34-claude.json`, `work/summary-np-codex.json`. Phase 3 items approved in chat 6 and written to ROADMAP.md. Brand assets in `assets/brand/`.

Open flags:
- Phase 3 item 1 done: Typst picked. Felipe says the escritorio layout had a red line; gen_pdf_desk.py draws a grey #999999 rule, source of the red not found yet.
- Phase 2 item 6: limit markers not yet checked against a real limit message from either CLI.
- Ads rank as a topic: "Publicidad y mensajes de patrocinadores" is the top airtime topic on Nada Personal (5:52); no ad filter.
- Merge of parts tested only by unit test; no saved episode splits.
- Picker fallback is Python (`tools/extractive_pick/pick.py`); how the Rust app uses it on a limit error not decided. Two ad picks on Nada Personal; no ad filter.
- Ep. 34 23:44 "a su diestra" likely "a su derecha" (Felipe 80 to 90 % sure), not checked against audio.
- Open: with context off Boric is 6 on Ep. 34 (target 7); Kast 16. Do not change the target; ask Felipe if it comes up.
- whisper-rs 424 s against whisper-cli 354 s on the RTX 2050, not investigated.
- No CPU-only or integrated-GPU speed measured; left by Felipe's choice.
- PRD open question: which assistant subscription the reader has.
