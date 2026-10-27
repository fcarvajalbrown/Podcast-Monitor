# Session notes

Current state: phase 1 done; Ep. 34 checked end to end on the GPU (`docs/research/pipeline-ep34-check.md`). PRD rescoped: Felipe runs transcription on his RTX 3060 PC or RTX 2050 laptop, one episode per show on demand, summarizer through his own CLI login; the reader runs it herself on her M1 from phase 5. Phase 2 not started. See `ROADMAP.md`.

Open flags:
- Transcription speed on the RTX 3060 not measured; needs the 3060 PC (on hold). `run` takes `--greedy`, `--beam-size N`, `--chunk-minutes`, `--text-context`, `--glossary-prompt`. On the RTX 2050: beam 5 395 s, greedy 324 s, both meet the name targets (`docs/research/beam-vs-greedy-ep34.md`). Repeat both there.
- Fixed: whisper.cpp carried each window's text into the next and the transcript fragmented into 2 to 3 word pieces (Ep. 34 from minute 30, Nada Personal from minute 40). `run` now defaults to `--text-context 0 --chunk-minutes 0`; verified identical to the context-off run on Ep. 34. `docs/research/extractive-trial-ep34.md`.
- Open: with context off Boric is 6 on Ep. 34 (target 7); Kast 16. Glossary prompt (`--glossary-prompt`) only works with context on and fragmented earlier on Nada Personal; not used. Two episodes, one run each, glossaries built from the episodes themselves.
- Summarizer direction chosen: her assistant CLI primary, extractive picker (`tools/extractive_pick/pick.py`) as the trial and possible fallback. Local LLM and non-LLM research in `docs/research/`.
- whisper-rs 424 s against whisper-cli 354 s on the RTX 2050, not investigated.
- No CPU-only or integrated-GPU speed measured; left by Felipe's choice.
- PRD open question: which assistant subscription the reader has.
