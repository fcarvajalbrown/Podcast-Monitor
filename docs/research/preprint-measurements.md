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

## Name-fix changes

`names::tests::lists_every_change_the_fix_makes_on_saved_transcripts`, run with `PM_RUNS=<runs folder> PM_GLOSSARY=<glossary> -- --ignored --nocapture`, reruns the fix at 0.85 on every saved `transcript.json` and prints each changed word with its segment.

With the app's current glossary (26 lines: 21 names, 6 of them with heard-as forms, two keep lines) the fix makes 70 changes across the 28 episodes: 59 fuzzy, 11 heard-as. Kast takes 42 of them (Kass 26, Cass 9, CAST 4, Cast 3), then Rabat 9, Bellolio 4, Huneeus 5, Lüders 3, Kouyoumdjian 2, Ximena 2, and one each of Mimica, Ríos, Lula, Faúndez and Contardo.

- 70 is under the 140 judged changes the plan aimed at for a ±5 point interval; at 70 the interval is about ±7 points if precision is near 90 %.
- These numbers are in-sample: Mimica, Ximena, Kouyoumdjian, Huneeus, Lüders and all six heard-as forms were added after reading these same transcripts (`weekly-run.md`), so they cannot show how the glossary does on episodes it has not seen.

## Held-out set

Chosen by Felipe: the glossary is frozen as `preprint-glossary-frozen.txt` (a copy of the app's glossary on 30 October), and the 16 episodes of the week the weekly run never touched (published Wednesday 28 to Thursday 29 October, plus SIN LLORAR and Lo Que Importa) go through `week --run` with it. No name is added to the glossary from those transcripts before their changes are judged, so they test the glossary on episodes it has not seen.

### "Lunes" and Huneeus

Felipe asked whether a heard "Lunes" (Monday) could be turned into Huneeus. The fix only looks at capitalized words, so the 49 "lunes" in the saved transcripts are never candidates; none of the 30 transcripts has a capitalized "Lunes", so the corpus never tested it. With the app's phonetic key, "Lunes" scores 0.800 against Huneeus, under the 0.85 threshold, while "Luneus" scores 0.909; a sentence-opening "Lunes" is left alone (`names::tests::sentence_opening_lunes_stays_while_luneus_becomes_huneeus`). The margin is 0.05, so a lower threshold would start taking it.

### In-sample verdicts

Judged by Felipe: Kast (42) and Rabat (7) from their segments, the other 21 by ear from `tools/judge_clips/clips.py` (one file, a spoken number before each clip). Every row is in `preprint-judged-insample.tsv`.

- 69 of 70 changes correct: precision 98.6 %, Wilson 95 % interval 92.3 % to 99.7 %.
- The one wrong change is fuzzy: "El diputado republicano Felipe Ross" became Ríos. Ross is the deputy's real name. A `!Ross` keep line would stop it, but the glossary stays frozen until the held-out set is judged.
- Ximena Jara and Davor Mimica checked as the show's co-hosts on its Patreon page and Rephonic listing. Mimisa to Mimica is counted correct because the rule is the right person's name, not the sound (Felipe's call).

## Held-out set results

`week --run` with the frozen glossary finished all 44 episodes of the week, 0 failed; 16 of them are new (the held-out set). Both audits rerun over the runs folder and filtered to those 16:

- Quotes: 80 returned by Codex, 0 dropped, largest gap to the stated minute 5 s. Across all runs that is 369 quotes and 0 drops (rule of three: under about 0.8 % at 95 %).
- Name fix: 30 changes (26 fuzzy, 4 heard-as): Kast 20, Ximena 3, Rabat 4, Ríos 2, Bellolio 1, Arias 1. Both "Ricos" to Ríos changes sit in the telenovela title "Los Ricos También Lloran". Felipe judges them by ear (`tools/judge_clips/clips.py`, `work/preprint/clips-heldout/all.mp3`).

### Held-out verdicts

Judged by Felipe from the segment list (he did not listen to this set); every row is in `preprint-judged-heldout.tsv`.

- 25 of 30 changes correct: precision 83.3 %, Wilson 95 % interval 66.4 % to 92.7 %, against 98.6 % in-sample.
- Fuzzy: 24 of 26 correct. The two wrong ones are "Ricos" to Ríos inside the telenovela title "Los Ricos También Lloran".
- Heard-as: 1 of 4 correct. The heard-as line `Ximena = Jimena` turned a real Jimena Jara, named in Los Columnistas de Mesa Central, into Ximena three times. She is Jimena Jara Quilodrán, lawyer and former Undersecretary of Cultures and the Arts (es.wikipedia.org/wiki/Jimena_Jara, link from Felipe), a different person from the Democracia en LSD co-host Ximena Jara. `heard-as-replacements.md` had flagged this risk ("Jimena" is itself a real first name); the held-out set shows it happening.
- "Pepe Aria" to Arias counted correct by Felipe: Chilean speech drops the final s. Web searches did not identify the person.
- In-sample and held-out pooled: 94 of 100, 94.0 % (87.5 % to 97.2 %), but the two sets should be reported apart, since the in-sample one was used to build the glossary.

## Speed: overnight plan

Scaled up by Felipe from 10 episodes to the whole week, run overnight when he says: `python -I tools/stt_speed/week_bench.py work/speed-week` transcribes all 44 episodes published since 26 October (33.0 h of audio) with beam 5 and with greedy plus flash attention, the order alternating per episode so heat does not favour one setting, then Ep. 34 five times in each setting for the run-to-run spread. Audio is hard-linked from the app-data runs (no download), no glossary, summary or PDF; every transcript is saved as it finishes and a rerun of the same command skips finished ones. Timings go to `work/speed-week/results.tsv`. Estimate about 4.5 h on the RTX 2050. Smoke test: El Shot of 29 October, 1,360 s of audio in 52 s (26.1x) with greedy and flash attention.

Safety nets for the unattended night, asked by Felipe: each run is killed past audio/3 + 300 s and tried twice, then marked failed and skipped; `results.tsv` and `progress.json` are rewritten after every run (written to a part file and renamed); Windows is kept awake through `SetThreadExecutionState`; the script waits while the laptop is on battery, while free RAM is under 2 GB, or when the GPU reaches 85 C until it is back under 70 C, and records the GPU temperature before each run; it stops cleanly under 2 GB of free disk; empty transcripts and speeds outside 3x to 80x are flagged. The kill was tested with a 5 s limit: the run was stopped, marked failed, and no process was left.

## Speed: overnight result

Run of 29 to 30 October, 02:25 to 06:44, RTX 2050, large-v3-turbo, release build. 98 runs, 0 failed, 0 flagged (no empty transcript, no speed outside 3x to 80x). Summary by `python -I tools/stt_speed/summarize.py work/speed-week/results.tsv`.

| Setting | Episodes | Audio | Compute | Pooled speed | Per episode (median, range) |
|---|---|---|---|---|---|
| beam 5 | 44 | 33.13 h | 2.26 h | 14.67x | 14.59x (13.45 to 16.24), sd 0.62 |
| greedy + flash attention | 44 | 33.13 h | 1.21 h | 27.44x | 27.49x (25.28 to 30.86), sd 1.12 |

Paired by episode, greedy with flash attention was faster in all 44: compute time ratio median 1.87 (1.79 to 1.94, sd 0.03). Segment count, greedy over beam 5: median 0.99 (0.84 to 1.20); this says nothing about accuracy, which this run did not measure.

Repeats, Ep. 34 (4,685 s of audio) five times each: beam 5 14.71x to 14.76x (sd 0.02), greedy + flash attention 28.24x to 28.65x (sd 0.17). Run-to-run spread is well under 2 %, so the spread across the 44 episodes comes from the episodes, not noise. The first week run of Ep. 34 with beam 5 took 323.1 s against 317.3 to 318.6 s in the repeats; it started with the GPU at 48 C, the only run below 57 C.

GPU temperature before each run: 57 C to 59 C for every run but that first one; the 85 C wait never triggered.

## Accuracy of greedy + flash attention against beam 5

No human reference transcript exists, so the two settings are compared with each other (Felipe's choice). `python -I tools/stt_compare/agreement.py work/speed-week/beam5 work/speed-week/flash-greedy work/preprint/agreement` aligns the word sequences of the 44 overnight episodes with difflib, case and punctuation folded and accents kept. Of 318,009 beam 5 words, 13,133 differ (4.13 %), in 6,913 places; 30 places run over 10 words, holding 823 of the differing words. Per episode in `work/preprint/agreement/per-episode.tsv`.

Which one is right where they differ: 60 places drawn at random (seed 20261030), A and B shuffled per place, judged by Felipe from the words around them, not from audio (his choice). The key is in `key.tsv`.
