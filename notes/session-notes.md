# Session notes

Current state: phase 1 done; Ep. 34 checked end to end on the GPU (`docs/research/pipeline-ep34-check.md`). PRD rescoped: Felipe runs transcription on his RTX 3060 PC or RTX 2050 laptop, one episode per show on demand, summarizer through his own CLI login; the reader runs it herself on her M1 from phase 5. Phase 2 not started. See `ROADMAP.md`.

Open flags:
- Transcription speed on the RTX 3060 not measured; needs the 3060 PC. `run` now takes `--greedy` and `--beam-size N`. On the RTX 2050: beam 5 395 s, greedy 324 s, both meet the name targets (`docs/research/beam-vs-greedy-ep34.md`). Repeat both there.
- Transcript fragments after minute 30 in both beam and greedy runs (2 to 3 word segments, almost no punctuation); cause not found, blocks sentence-based summarizing. `docs/research/extractive-trial-ep34.md`.
- Summarizer direction chosen: her assistant CLI primary, extractive picker (`tools/extractive_pick/pick.py`) as the trial and possible fallback. Local LLM and non-LLM research in `docs/research/`.
- whisper-rs 424 s against whisper-cli 354 s on the RTX 2050, not investigated.
- No CPU-only or integrated-GPU speed measured; left by Felipe's choice.
- PRD open question: which assistant subscription the reader has.
