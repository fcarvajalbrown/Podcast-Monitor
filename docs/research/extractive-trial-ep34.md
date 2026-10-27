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
