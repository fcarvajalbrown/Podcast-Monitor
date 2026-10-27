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
