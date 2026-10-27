# PDF crate for the report

Question: which open-source Rust crate builds the escritorio-layout PDF (phase 3 item 1).

Candidates picked with Felipe: Typst as a library (`typst` 0.15.1 through `typst-as-lib` 0.17.0, Apache-2.0 and MIT), `krilla` 0.8.2 (MIT or Apache-2.0), `printpdf` 0.12.8 (MIT). `genpdf` 0.2.0 left out: no release since 2021 and no justified alignment.

Method: `tools/pdf_spike/` builds the same two-page test (header line and rule, title, headings, four justified Spanish paragraphs with every accent, ¿ ¡ « » and curly quotes, a four-column table with a wrapping cell, centred page numbers) in each crate, plus `gen_pdf_desk.py` output from the same text through the `markdown` binary. Fonts: the Latin Modern Roman TTFs in `assets/fonts/` (GUST licence). Checked by text and drawing extraction with PyMuPDF and by rendering the pages.

| | Typst | krilla | printpdf (HTML layout) |
|---|---|---|---|
| Latin Modern embedded, subset | yes | yes | yes, regular embedded twice |
| Spanish characters | all correct | all correct | all correct |
| Justified text | yes, built in, with punctuation hanging into the right margin | yes, written by hand (greedy line break, space stretch) | yes |
| Table | yes, built in, booktabs rules | yes, written by hand | column widths ignored, table runs off the page; no rules drawn |
| Header rule | yes | yes | not drawn (CSS border ignored) |
| Page numbers | built in | written by hand | added by hand as extra page operations |
| Heading kept with next block | built in | written by hand | not tested |
| Run time, release build | 0.72 s | 0.25 s | 1.46 s |
| Code for the test page | 70 lines, mostly a template | 210 lines of layout code | 100 lines |

Measured against the reference: body baseline-to-baseline 15.0 pt in the reference, Typst, krilla; 14.9 pt in printpdf. Gap between paragraphs 23 pt in the reference and krilla, 18.5 pt in Typst with `par.spacing` 8 pt (12.5 pt matches; a template setting). Reference text column starts at 76.9 pt, not 70.9: reportlab's frame adds 6 pt padding inside the 2.5 cm margin, and its table spans the frame width, so it ends 12 pt right of the text. The reference breaks the "Inicio" header cell as "Inici / o"; it keeps each paragraph whole on one page, where the approved item asks for paragraphs that flow across pages.

Result: printpdf fails the table and the rules. Typst and krilla both pass; Typst does wrapping, justification, tables, page breaks and keep-with-next itself, krilla needs all of that written and tested in this repo.

Fonts on the reader's Mac: all three crates embed a subset of Latin Modern in the PDF itself (checked by PyMuPDF font listing), so the PDF reads the same on any machine without the font installed. The spike reads the TTFs from `assets/fonts/` at run time through a source-tree path, which would not exist on her Mac; the report code must compile the four TTFs into the binary (`include_bytes!`, about 850 KB) so the app needs nothing installed. Not yet checked on a Mac build (phase 5).

Size: release test-page exe 39.5 MB with Typst against 5.1 MB with krilla (Windows, same package), so Typst adds roughly 35 MB to the app; the current `podcast-monitor.exe` is 63.2 MB, most of it whisper.cpp.

Decision: Felipe picked Typst after viewing the four test PDFs.

Page frame (item 2), measured with `tools/pdf_check/measure.py` on the `writes_sample` test output against `gen_pdf_desk.py`: header baseline 86.4 pt, rule 91.4 pt (red #B70002 instead of grey), text column 76.9 to 518.4 pt, first body baseline on later pages 87.4 pt, page number baseline 802.2 pt centred at x 295.4 to 299.9; all equal to the reference. The frame is 2.5 cm plus reportlab's 6 pt frame padding. Typst's hanging punctuation is off (`overhang: false`) because the reference has none.

Text blocks (item 3), same check on the spike's test content: every heading, paragraph and table rule baseline is within 0.5 pt of the reference (reportlab collapses the space between blocks to the larger of space-after and space-before, then adds the line's leading below the baseline; Typst collapses the same way but from the baseline, so the only residue is 0.5 pt where a level-2 heading follows the title). Differences kept on purpose: the table spans the text column (the reference table runs 12 pt past it), so "Tiempo al aire" wraps where the reference wraps "Inicio"; Typst breaks justified lines by whole paragraph, so line breaks differ from reportlab's line-by-line breaks. Unit tests in `src/report.rs`: a heading is never the last thing on a page (20 to 49 lines before it), a long paragraph continues on the next page, the table header repeats on every page, short table rows are padded.

Size with the report built in (item 5): `podcast-monitor.exe` release on Windows is 103.3 MB, against 63.2 MB before Typst (40.1 MB more, fonts included). No macOS build measured yet.
