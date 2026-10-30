# Preprint measurements: the 28 episodes of the week of 26 October

Plan set by Felipe (ROADMAP open items): every quote checked against its transcript, every name-fix change judged by hand, 10 episodes timed in both speed settings plus 3 repeats of one. Corpus: the 28 episode folders of the weekly run in `%APPDATA%\Podcast-Monitor\runs` (the two `.bak-weekly` folders are the weekly run's copies of Ep. 34 and Nada Personal; the window reruns of 29 October are counted apart).

## Quotes

`assemble::tests::lists_every_summary_quote_against_its_transcript`, run with `PM_RUNS=<runs folder> [PM_SUMMARY=summary.previous.json] -- --ignored --nocapture`, applies the app's quote rule (`quotes::Index::find`) to every quote the summarizer returned, against `transcript.fixed.json`, and prints the minute asked, the start of the matched segment and the gap.

| Summaries | Episodes | Quotes returned | Dropped | Largest gap to the stated minute |
|---|---|---|---|---|
| Codex, conclusion prompt (`summary.json`) | 28 | 140 | 0 | 8 s |
| Codex, first prompt (`summary.previous.json`) | 28 | 140 | 0 | 5 s |
| Codex, window reruns of 29 October | 2 | 9 | 0 | 1 s |

- Every quote returned was found verbatim (after folding case, accents and punctuation) and within 8 s of the minute the model gave. The check never had to drop a quote on this corpus, so it shows that Codex quoted faithfully here, not that the check catches errors; the unit test with an invented quote is the only drop seen.
- With 0 drops in 280 quotes, the rule of three puts the drop rate under about 1.1 % at 95 % confidence for this summarizer and prompt.
- Every summary in the corpus is from Codex; Claude was not measured.
