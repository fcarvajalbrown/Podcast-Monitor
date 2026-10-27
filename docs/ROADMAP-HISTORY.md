# Roadmap history

Finished roadmap items, newest first. Git dates each entry.

- Phase 2, item 1: `podcast-monitor chunks` splits a transcript into `[mm:ss] text` chunks under a 120,000 character cap; Ep. 34 and Nada Personal are one chunk each, three even parts at a 30,000 cap.
- Phase 1, item 7: Ep. 34 end to end on the GPU, Kast 17 and Boric 9 after the fix, above the 0.85 targets of 16 and 7 (`docs/research/pipeline-ep34-check.md`).
- Phase 1, item 6: per-episode resume, checked on Ep. 34: audio reused, transcript skipped on rerun.
- Phase 1, item 5: name fix at 0.85 with glossary and per-show exceptions; same replacements as `phonetic_fix.py --threshold 0.85` on Ep. 34.
- Phase 1, item 4: transcription through whisper-rs on Vulkan, checked on Ep. 34: Vulkan0 in the log, 4,685 s of audio in 424 s on the RTX 2050.
- Phase 1, item 3: audio download and decode to 16 kHz mono in Rust, checked on Ep. 34.
- Phase 1, item 2: feed fetch and parse (`podcast-monitor episodes`), tested on the Nada Personal feed.
- Phase 1, item 1: Cargo crate and CLI skeleton.
- PRD interview complete (`docs/PRD.md`).
