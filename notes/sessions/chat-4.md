# Chat 4

Done:
- Decisions with Felipe: merge, airtime and people counts go at the end of phase 2, in that order; people counted by exact surname as `count_names.py` does; people with 0 mentions kept at the bottom; split-episode summaries joined in order with no extra call.
- Phase 2 item 7: `Summary::merged` merges the parts of an episode; unit test only, no saved episode splits.
- Phase 2 item 8: `tally::airtime`; Ep. 34 21 topics over 78:05, Nada Personal 22 topics over 58:27 of 59:19.
- Phase 2 item 9: `tally::mentions`; every count equals a Python surname count on both transcripts (Ep. 34 Kast 16, Boric 6).
- All checks run on copies of the saved replies with a fake `claude.cmd`/`codex.cmd` first on PATH; no assistant call made.

Flags opened:
- Ads rank as the top airtime topic on Nada Personal (5:52); no ad filter.
- Shared surnames credit both people (Noguera, Bolsonaro), as decided.

Flags closed:
- Placement of airtime, people counts and merge.

Still open: phase 2 item 6 (real limit text), blocked until a real limit is hit.
