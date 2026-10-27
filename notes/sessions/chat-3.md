# Chat 3

Done:
- Phase 2 chunking: `podcast-monitor chunks`; whole episode in one call under 120,000 characters, equal parts at segment boundaries above. Ep. 34 80,402 and Nada Personal 64,535 characters, one chunk each.
- Phase 2 CLI wrapper: `podcast-monitor summarize`, `claude -p --json-schema` and `codex exec --output-schema` behind the `Assistant` trait, glossary in each prompt, each part saved as it finishes. Ep. 34 through claude 47 s, Nada Personal through codex 25 s.
- Phase 2 quote matching: exact after folding, nearest hit to the model's minute, transcript words and minute printed; 10 of 10 found on both episodes.
- Phase 2 limit pause: limit markers in the error text stop the run with exit code 2; checked with a fake `claude.cmd`.
- Extractive picker on both context-off transcripts; Felipe judged it readable and kept it as the fallback.
- Decisions with Felipe: chunking, reply shape, quote rule, picker kept.

Flags opened:
- Real limit text not yet seen on either CLI (ROADMAP phase 2 item 6).
- Ep. 34 23:44 "a su diestra" likely "a su derecha" (Felipe 80 to 90 % sure), not checked against audio.
- Nada Personal picks include two ads; picker has no ad filter.
- Airtime per topic and people counts from the corrected transcript are decided but not built.

Flags closed:
- Summarizer interface, quote matching, limit-error pause.

Note: a fake-CLI test first ran the real `claude` three times (Windows path in the bash PATH); redone with the fake.
