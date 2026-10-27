# Roadmap history

Finished roadmap items, newest first. Git dates each entry.

- Phase 2, item 4: an assistant failure whose text holds a limit marker ("hit your", usage, spend or rate limit, 429, low credit balance) stops `summarize` with exit code 2, finished parts kept. Markers from the Claude Code errors page (code.claude.com/docs/en/errors) and Codex user reports ("You've hit your usage limit", "429 Too Many Requests"); neither CLI documents a limit exit code. Checked with a fake `claude.cmd` returning a session-limit reply: stopped at part 1 of 3, exit 2. Not yet seen against a real limit (item 6).
- Phase 2, item 5: extractive picker on both context-off transcripts; Felipe judged the Ep. 34 picks readable and kept it as the limit-error fallback; two ad picks on Nada Personal (`docs/research/extractive-trial-ep34.md`).
- Phase 2, item 3: quote lookup in `src/quotes.rs`; on Ep. 34 (claude) and Nada Personal (codex) 10 of 10 quotes found, none dropped; one model minute (31:12) corrected to the transcript's 31:10.
- Phase 2, item 2: `podcast-monitor summarize` sends each chunk with the glossary to `claude -p --json-schema` or `codex exec --output-schema` and saves each reply as it arrives (rerun resumes). Ep. 34 through claude: one call, 47 s, 27 topic spans from 00:00 to 78:05, 43 people, 5 quotes. Nada Personal through codex: one call, 25 s, 22 spans, 22 people, 5 quotes. Quotes not yet checked against the transcript (item 3).
- Phase 2, item 1: `podcast-monitor chunks` splits a transcript into `[mm:ss] text` chunks under a 120,000 character cap; Ep. 34 and Nada Personal are one chunk each, three even parts at a 30,000 cap.
- Phase 1, item 7: Ep. 34 end to end on the GPU, Kast 17 and Boric 9 after the fix, above the 0.85 targets of 16 and 7 (`docs/research/pipeline-ep34-check.md`).
- Phase 1, item 6: per-episode resume, checked on Ep. 34: audio reused, transcript skipped on rerun.
- Phase 1, item 5: name fix at 0.85 with glossary and per-show exceptions; same replacements as `phonetic_fix.py --threshold 0.85` on Ep. 34.
- Phase 1, item 4: transcription through whisper-rs on Vulkan, checked on Ep. 34: Vulkan0 in the log, 4,685 s of audio in 424 s on the RTX 2050.
- Phase 1, item 3: audio download and decode to 16 kHz mono in Rust, checked on Ep. 34.
- Phase 1, item 2: feed fetch and parse (`podcast-monitor episodes`), tested on the Nada Personal feed.
- Phase 1, item 1: Cargo crate and CLI skeleton.
- PRD interview complete (`docs/PRD.md`).
