# Extractive trial on Ep. 34

Script: `tools/extractive_pick/pick.py`. Stdlib Python, TF-IDF per sentence, centroid scoring, maximal marginal relevance (`--trade-off`), Spanish stopword list in `stopwords-es.txt`. Input: `transcript.json` or `transcript.fixed.json`. Output: picked sentences in time order with the minute. Run on the beam-5 fixed transcript from `beam-vs-greedy-ep34.md`.

## Finding: the transcript loses its punctuation after minute 30

Both runs (beam 5 and greedy, RTX 2050, `ggml-large-v3-turbo.bin`) change character part way through. Segments and segments ending in sentence punctuation, per ten minutes:

| Minutes | Beam 5 segments | ending in . ! ? | Words per segment | Greedy segments | ending in . ! ? |
|---|---|---|---|---|---|
| 0-10 | 157 | 145 | 12.3 | 138 | 133 |
| 10-20 | 144 | 134 | 13.3 | 165 | 158 |
| 20-30 | 180 | 139 | 10.4 | 197 | 171 |
| 30-40 | 542 | 6 | 2.7 | 475 | 90 |
| 40-50 | 572 | 4 | 2.4 | 500 | 49 |
| 50-60 | 601 | 2 | 2.4 | 575 | 67 |
| 60-70 | 607 | 6 | 2.4 | 571 | 84 |
| 70-78 | 423 | 1 | 2.7 | 399 | 20 | 

From minute 30 the segments are two to three words, for example `en el que tal vez`, `tuvo lugar`, `por ejemplo`, `figuras`, with almost no punctuation in beam 5 and sparse punctuation in greedy. Greedy keeps more punctuation than beam 5 but shows the same fragmentation.

Consequence for the picker: sentences are rebuilt by joining segments up to a full stop, so after minute 30 beam 5 yields sentences of 400 to 1,200 words. They fall outside the 12 to 60 word window, and every pick (15 of 15, at any trade-off) lands before minute 32 of a 78 minute episode. The picker is not the limit; the transcript is.

Also seen: a replacement character in `ten?a` (a segment text with U+FFFD) in both runs around 40:00. Not yet counted across the episode.

Not investigated: why. The earlier `whisper-cli` run of the same episode (`name-correction.md`) may not show it; not checked.

## What the first 30 minutes show

Fifteen sentences picked at trade-off 0.7 are readable Spanish and cover the Brazilian election, the Kast comparison and the prison-benefits decree. Whether they are the passages a reader wants is for Felipe to judge; not measured.

## Cause search: chunking, 10 minutes (beam 5, RTX 2050)

Hypothesis from the whisper.cpp source in the cargo registry: inside one `whisper_full` call the decoded text of each 30 s window is carried into the next as context (`prompt_past1`, up to 224 tokens), so one drifted window can feed the following ones; each new call starts clean. `run` now takes `--chunk-minutes` (default 10), cutting at the quietest second within 20 s of each boundary, each chunk in a fresh state. Searches found the same mechanism discussed for Whisper generally ([WhisperX paper](https://arxiv.org/pdf/2303.00747), [faster-whisper guide](https://localaimaster.com/blog/whisper-hallucination-fix)) but nothing that matches this exact symptom.

Result, Ep. 34, 8 chunks, compute 395 s, no replacement characters:

| Minutes | Segments | ending in . ! ? | Words per segment |
|---|---|---|---|
| 0-10 | 157 | 146 | 12.3 |
| 10-20 | 587 | 6 | 3.2 |
| 20-30 | 150 | 104 | 12.5 |
| 30-40 | 560 | 5 | 2.6 |
| 40-50 | 237 | 62 | 5.7 |
| 50-60 | 114 | 93 | 12.3 |
| 60-70 | 107 | 99 | 13.4 |
| 70-78 | 87 | 66 | 12.9 |

Chunking recovers minutes 20-30 and 50-78 but not 10-20, 30-40 and 40-50. So cross-chunk context is not the whole cause: a chunk can still break within its own ten minutes. Names unchanged (Kast 17, Boric 9; Squella 4 as in greedy). The picker's 15 picks now spread over the episode (01:00 to 77:14).

## Cause search: no carried text context (beam 5, single pass, RTX 2050)

`--chunk-minutes 0 --text-context 0` sets whisper.cpp's `n_max_text_ctx` to 0, so no earlier text is fed to later windows. Compute 326 s (395 s with context), 1,503 segments, no replacement characters.

| Minutes | Segments | ending in . ! ? | Words per segment |
|---|---|---|---|
| 0-10 | 254 | 81 | 7.3 |
| 10-20 | 245 | 84 | 7.7 |
| 20-30 | 290 | 81 | 6.4 |
| 30-40 | 155 | 61 | 9.1 |
| 40-50 | 123 | 34 | 10.5 |
| 50-60 | 120 | 38 | 11.3 |
| 60-70 | 195 | 53 | 7.2 |
| 70-78 | 121 | 52 | 9.2 |

The fragmentation is gone across the whole episode, which points at the carried context as the cause. Segments are shorter and fewer end in punctuation than in the first 30 minutes with context (about 12 words, 90 %), so sentences span several segments. The picker spreads its 15 picks from 07:49 to 77:14.

Names after the fix: Kast 16 (target 16), Boric 6 (target 7, was 9 with context), Squella 3. Boric misses the target.

Not yet tried: a small cap such as 64 tokens, or the cap together with 10 minute chunks.

## Cause search: cap of 64 tokens with 10 minute chunks (beam 5, RTX 2050)

`--chunk-minutes 10 --text-context 64`. Compute 339 s, 1,764 segments, no replacement characters.

| Minutes | Segments | ending in . ! ? | Words per segment |
|---|---|---|---|
| 0-10 | 178 | 126 | 10.5 |
| 10-20 | 455 | 3 | 4.1 |
| 20-30 | 118 | 79 | 15.5 |
| 30-40 | 508 | 7 | 2.8 |
| 40-50 | 214 | 60 | 6.1 |
| 50-60 | 138 | 78 | 9.9 |
| 60-70 | 123 | 86 | 11.3 |
| 70-78 | 30 | 26 | 35.4 |

Fragmentation returns in 10-20 and 30-40, the same places as chunking alone. Any carried context, even 64 tokens inside a fresh chunk, lets the output break; only `--text-context 0` kept the whole episode intact. Names after the fix: Kast 17, Boric 8, Squella 4. The 70-78 row (30 segments of 35 words) is odd and not looked at.

Summary of the four Ep. 34 runs on the RTX 2050, beam 5 unless noted:

| Setting | compute_s | Fragmented minutes | Kast | Boric |
|---|---|---|---|---|
| Default (context carried, one pass) | 395 | 30-78 | 17 | 9 |
| Chunks of 10 min | 395 | 10-20, 30-50 | 17 | 9 |
| Context 64 + chunks of 10 min | 339 | 10-20, 30-40 | 17 | 8 |
| Context 0, one pass | 326 | none | 16 | 6 |

Only context 0 meets Kast at least 16 and avoids fragmentation, and it misses Boric at least 7 by one mention.

## Benchmark of the three options (beam 5, one pass, RTX 2050, one run each)

Option 1, context off: the `--text-context 0` run above.

Option 2, keep context and repair afterwards: the default run (395 s, Kast 17, Boric 9). The simplest repair, cutting sentences at pauses, cannot work: in minutes 35-75 of that run 2,341 of 2,342 gaps between segments are exactly 0 s (segment times are contiguous), against 333 of 383 in minutes 0-25. No punctuation-restoration model was tried, so a repair that works is not shown.

Option 3, glossary names as whisper's initial prompt (`--glossary-prompt`, the 21 names of `glossary-ap-ep34.txt` comma-separated):

- With context carried: compute 384 s, 1,399 segments, no replacement characters. Fragmentation is much milder than the default run: segments average 5.5 to 5.8 words in minutes 40-60 (default 2.4) and about 30 % end in punctuation there (default under 1 %); minutes 0-40 and 60-78 look like the first half of the default run. Kast 17, Boric 9, Squella 1. Text from minute 45 reads as comma-separated clauses.
- With context off: identical to option 1 (same segments, same counts). Whisper.cpp only uses the prompt when the text context is above 0, so the prompt does nothing with context off.

| Arm | compute_s | Segments | Sentences / eligible (picker) | Kast | Boric | Squella |
|---|---|---|---|---|---|---|
| Default (context, no prompt) | 395 | 3,226 | 438 / 205 | 17 | 9 | 0 |
| 1. Context off | 326 | 1,503 | 485 / 257 | 16 | 6 | 3 |
| 2. Repair afterwards | 395 | 3,226 | 438 / 205 | 17 | 9 | 0 |
| 3. Prompt, context on | 384 | 1,399 | 774 / 384 | 17 | 9 | 1 |
| 3b. Prompt, context off | 332 | 1,503 | same as 1 | 16 | 6 | 3 |

Sentences and eligible counts are from `tools/extractive_pick/pick.py` on the fixed transcripts.

The earlier `whisper-cli` prompt run (`name-correction.md`) did fragment after minute 20, so the prompt's effect is not consistent across builds; here one run per arm, no repeat to show the noise.

## Repeat of option 3

Same command a second time (`--chunk-minutes 0 --glossary-prompt`, context carried). The transcript is identical to the first run: 1,399 segments, the same per-ten-minute table, Kast 17, Boric 9, Squella 1, 774 sentences and 384 eligible. Only the compute time differs (400 s against 384 s). So whisper.cpp decoding is deterministic here, and a repeat on the same episode shows no run-to-run noise. It does not show whether the prompt holds on other episodes; the glossary is also built from this episode, a best case. Not tested.

## Second episode: Nada Personal, default against glossary prompt (beam 5, one pass, RTX 2050)

"La sorpresa detrás de la vuelta de María Corina Machado a Venezuela", Radio Duna, 3,563 s, glossary `glossary-np-latest.txt` (best case, built from this episode). Feed `https://mdstrm.com/feeds/65ea0c645728c0087b186ba5/67f56020881515c36adbe29f?property=itunes`, filter `--title Machado` (an accented title filter did not match from the shell).

| Minutes | Default: segments | ending in . ! ? | words per segment | Prompt: segments | ending in . ! ? | words per segment |
|---|---|---|---|---|---|---|
| 0-10 | 184 | 40 | 9.6 | 232 | 215 | 7.6 |
| 10-20 | 171 | 112 | 9.8 | 172 | 117 | 9.9 |
| 20-30 | 238 | 72 | 7.0 | 434 | 49 | 3.9 |
| 30-40 | 273 | 114 | 6.2 | 606 | 3 | 2.7 |
| 40-50 | 543 | 29 | 2.9 | 554 | 4 | 2.8 |
| 50-59 | 572 | 5 | 2.8 | 586 | 8 | 2.7 |

Compute 319 s default, 320 s prompt, 1,981 and 2,584 segments, no replacement characters. Picker: 372 sentences and 139 eligible (default), 397 and 115 (prompt).

The prompt does not prevent the fragmentation here: with it the text breaks from minute 20, without it from minute 40. So the Ep. 34 result for option 3 does not generalize, and the default fragments on this episode too (after minute 40). Names are the same in both runs except Kast 4 against 3 and Lula 26 against 24.

## Second episode: Nada Personal, context off

`--chunk-minutes 0 --text-context 0` on the same episode. Compute 248 s (319 s default), 1,332 segments, no replacement characters. Segments average 5.3 to 11.2 words in every ten minute block, so no fragmentation; punctuation is sparser than the first half of the default run (22 to 99 of 136 to 293 segments per block end in punctuation). Picker: 300 sentences and 120 eligible (default 372 and 139).

Names after the fix, against the default run: Machado 10 and 10, Kast 7 against 4, Lula 23 against 26, Ríos 2 against 3, everything else equal.

Across the two episodes, context off is the only setting that removed the fragmentation and it was also the fastest on both (Ep. 34 326 s against 395 s; Nada Personal 248 s against 319 s). The glossary prompt helped on Ep. 34 and made the fragmentation start earlier on Nada Personal. Names: Ep. 34 loses Boric (6 against 9, target 7); Nada Personal gains Kast and loses a few Lula. Two episodes, one run each (the Ep. 34 repeat was identical).

## Fallback check on the fixed default (context off), read by Felipe

`pick.py` at its defaults (15 picks, trade-off 0.7, 12 to 60 words) on `noctx-fixed.json` (Ep. 34) and `np-noctx-fixed.json` (Nada Personal).

- Ep. 34: 485 sentences, 257 eligible, picks from 07:49 to 77:14. One pick is speech-recognition garble (41:08, "mi ley ... mi ley dando más material", likely "Milei").
- Ep. 34, 23:44: "con el ministro de Justicia a su diestra". Felipe is 80 to 90 % sure the speaker said "a su derecha": nobody in a Chilean podcast, even a formal one, says "diestra". Not checked against the audio. All eight Ep. 34 runs write "diestra", but they share one model, so that is not independent support.
- Nada Personal: 300 sentences, 120 eligible, picks from 03:27 to 57:36. Two of the 15 are ads (40:44 and 41:07); the picker has no ad filter.

Felipe read the 15 Ep. 34 picks and judged them readable enough to keep the picker as the fallback for the limit-error pause.
