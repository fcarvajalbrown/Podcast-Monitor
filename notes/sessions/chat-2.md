# Chat 2

Done:
- Reran Ep. 34 through `podcast-monitor.exe run` on the GPU: Vulkan0 in the log, 3,232 MiB in nvidia-smi, 424 s compute for 4,685 s of audio; audio reused, transcript skipped on a second run.
- Item 7 passed: Kast 17 and Boric 9 after the fix, above 16 and 7; Rust replacements identical to `phonetic_fix.py --threshold 0.85`.
- `phonetic_fix.py` now reads segment transcripts.
- Phase 1 closed in `ROADMAP.md`, items moved to `docs/ROADMAP-HISTORY.md`.

Flags opened:
- whisper-rs 424 s against whisper-cli 354 s on the same GPU, not investigated.

Flags closed:
- Ep. 34 GPU rerun; Rust against Python name fix comparison.
