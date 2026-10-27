# Chat 6

Done:
- Phase 3 items approved by Felipe and written to ROADMAP.md, then all six built and closed.
- Crate pick: test page in Typst, krilla and printpdf (`tools/pdf_spike/`); printpdf failed the table and rules; Felipe picked Typst after viewing the PDFs (`docs/research/pdf-crates.md`). Latin Modern TTFs copied to `assets/fonts/` and compiled into the app.
- Header rule drawn in the logo red #B70002 (Felipe); no red rule exists in any felipe_skill_voice script, the reference draws grey.
- Page frame and text blocks matched to `gen_pdf_desk.py` output within 0.5 pt (`tools/pdf_check/measure.py`).
- Episode section layout A, report order (shows alphabetical, newest first, new page per episode), both chosen by Felipe.
- `report` command; `run` saves `show.json`.
- Acceptance on copies of the two saved replies: 10 of 10 quotes at their minute (`tools/pdf_check/quotes.py`), layout metrics equal to the reference; a bug that left page 1 with only the header was found and fixed.

Flags opened:
- Older run folders lack `show.json` (folder-name fallback).
- macOS app size unmeasured; Windows 103.3 MB.

Flags closed:
- Phase 3 item list awaiting approval.
