# Chat 1

Done:
- PRD interview finished; all sections in `docs/PRD.md`.
- Initial-prompt glossary measured on two episodes: no gain, dropped. Name fix at 0.75 wrong on a second show; 0.85 chosen (`docs/research/name-correction.md`).
- M1 speed numbers collected (`docs/research/stt-speed-m1.md`).
- ROADMAP created; phase 1 items 1-6 written in Rust (feed, audio, transcribe, names, run).

Flags opened:
- First Ep. 34 run went to the CPU; GPU fix committed, rerun not done.
- Rust name fix not yet compared with `phonetic_fix.py` on real transcripts.

Flags closed:
- Vulkan SDK installed; whisper-rs builds on Windows with the wrapper script.
