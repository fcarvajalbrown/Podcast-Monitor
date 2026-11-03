# What the pipeline would cost through paid APIs

Question: the summaries run today through an assistant CLI subscription at no cost per episode. If
that route closed, or the pipeline ran unattended on a server, what would the same work cost through
paid APIs, and what does the hardware allow?

Measured 2 November 2026 with `tools/token_cost/measure.py` over the 59 saved episodes in
`%APPDATA%\Podcast-Monitor\runs`, and priced with `tools/token_cost/budget.py`.

## What the pipeline actually sends and receives

| Per episode | Minimum | Median | Mean | p90 | Maximum |
| --- | --- | --- | --- | --- | --- |
| Characters sent | 3,304 | 42,338 | 46,560 | 69,370 | 81,531 |
| Characters returned | 1,727 | 4,311 | 4,417 | 5,585 | 6,365 |
| Title and description only | 153 | 465 | 862 | 2,419 | 2,568 |

The measurement agrees with the figure already recorded for Ep. 34 in `ROADMAP.md`: 80,402
transcript characters plus 1,129 of instructions and glossary gives the 81,531 maximum.

Totals across the 59 episodes: 16 shows, 42.2 hours of audio, 2,747,021 characters sent, 260,580
returned. Every episode fitted in one call, so the 120,000-character chunk cap has never yet split
an episode. The Monday to Sunday week of 26 October held 57 episodes and 41.1 hours of audio, mean
episode 43.2 minutes.

The weekly report costs one extra call per show, returning 5,378 characters on average across the 16
shows, plus one call for the week-wide theme table, which returned 9,441.

Reading descriptions instead of transcripts cuts the input by a factor of 54, from 46,560 characters
to 862.

## Storage

Audio weighs 1.336 MB per minute, or 80.2 MB per hour of programme, measured over 60 saved episodes.
A saved transcript is 96 KB. At the current national rate that is 3.2 GB of audio a week, 167 GB a
year if nothing is deleted.

## Prices, first party, read 2 November 2026

| Provider | Tier | Model | Input per 1M | Output per 1M |
| --- | --- | --- | --- | --- |
| OpenAI | low | gpt-6-luna | 0.10 | 0.50 |
| OpenAI | medium | gpt-6.1-terra | 1.00 | 5.00 |
| OpenAI | high | gpt-6-astra | 10.00 | 50.00 |
| Qwen | low | qwen3.7-flash | 0.0339 | 0.1465 |
| Qwen | medium | qwen3.7-plus | 0.4510 | 1.8037 |
| Qwen | high | qwen3.8-max | 2.2547 | 6.7642 |
| Kimi | low | kimi-k3 | 3.00 | 15.00 |

USD per million tokens. Qwen is published in yuan for the Singapore deployment and converted at
6.6475 CNY per USD, derived from the European Central Bank reference rates of 2 November 2026
(EUR/USD 1.1216, EUR/CNY 7.4567). Pesos use 994.29 CLP per USD, the dolar observado of 27 October
from the SII table.

Three cautions. OpenAI's middle model changed since an October reading: it is gpt-6.1-terra at 1.00
and 5.00, where gpt-6.1-sol was 2.00 and 10.00. Qwen's plus tier is flagged on its own page as a
limited-time 20 per cent discount. Kimi's prices could not be re-read, because its pricing tables
render in the browser on every first-party page; the figures above are an 8 October reading and need
confirming in the console. Kimi also closed the cheaper K2 series to new registrations on 31 October
2026 and retires it at the end of December, and it sells no speech recognition at all.

Speech to text, per minute of audio: qwen3-asr-flash 0.002347, gpt-4o-mini-transcribe 0.003,
gpt-transcribe 0.0045, whisper-1 0.006. OpenAI labels its per-minute figures as estimates on the
page itself.

## What it would cost

For the 57 episodes a week of the current show list, with a national report:

| Scenario | Cheapest model | Dearest model |
| --- | --- | --- |
| Descriptions only, no audio fetched | 0.04 | 12.86 |
| Transcribing half the episodes | 12.56 | 51.23 |
| Transcribing all, three providers, verified | 25.88 | 339.06 |

USD a month. In the middle scenario on the cheapest models, transcription is 12.44 of the 12.82
total: the analysis of every episode plus a daily and a weekly report comes to 0.38. That ratio holds
across providers at the low tier. The cost of this pipeline is audio minutes, not tokens.

Characters per token stays an estimate, carried from the earlier Propuesta-Tokens range of 3.0 to
4.0, since no provider tokenizer was run. Moving it across that whole range changes the middle
scenario by about five cents a month, so measuring it is not worth the effort unless a figure has to
be defended precisely.

## Whisper on a small server

It does not fit. `ggml-large-v3-turbo.bin` is 1.51 GiB, against 1.0 GiB free on a 1.8 GiB two-core
VPS with no swap. The smaller model is worse: the measured `faster-whisper small` CPU run in
`stt-bench-ap-ep34.md` peaked at 4,461 MB, took 35 minutes for a 78-minute episode on a 16-thread
laptop, and scored 14.4 per cent word error rate. Local transcription needs the GPU machine it
already runs on, where the same episode takes 158 seconds.

## Finding shows by region

`tools/region_discovery/` searched Apple's directory in the Chilean storefront, 72 searches, then
checked every feed found.

| Stage | Sources | Episodes a week | Audio hours a week |
| --- | --- | --- | --- |
| Found in the directory | 1,976 | 2,406 | 2,024 |
| Feed reachable and parsed | 1,956 | 2,406 | 2,024 |
| Published at least once in 60 days | 553 | 2,406 | 2,024 |
| News, politics or commentary genre | 213 | 1,457 | 1,055 |
| Names a region in its own metadata | 16 | 53 | 40 |

The directory is a poor instrument for finding local shows. Searching a city name returns national
programmes and unrelated content: the Araucania searches found 75 live feeds, none naming Temuco or
Araucania anywhere in their own metadata. Five regions returned no live source that names them.
Most regional Chilean media broadcasts rather than publishing RSS, which is why the regional podcast
counts are low and those five regions return zero.
