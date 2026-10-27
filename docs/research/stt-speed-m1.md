# Whisper turbo speed on a base M1: published numbers

Question: how fast will large-v3-turbo transcribe on the reader's 8 GB M1? Felipe has no Mac, so he chose published numbers over a test build.

## What exists

| Source | Machine | Model | Setup | Number |
|---|---|---|---|---|
| whisper.cpp bench thread (issue 89) | Mac mini M1 (base) | large (v1/v2 size) | NEON BLAS, 4 threads, no Metal listed, old commit | 7,979 ms encode per 30 s window |
| same thread | Mac mini M1 (base) | medium | same | 3,980 ms |
| same thread | M1 Ultra | large-v3-turbo Q8_0 | Metal + Core ML, 8 threads | 618 ms encode |
| PromptQuorum guide (no method given) | M1 | large-v3 | Metal | 2-3x real time |

No published row covers large-v3-turbo on a base M1, with or without Metal.

## Reading

- Turbo keeps large-v3's encoder and cuts the decoder from 32 layers to 4, so the encoder number of the large model is the relevant floor.
- The base-M1 CPU encode of about 8 s per 30 s window is roughly 3.75x real time for encoding alone, before decoding, on an old commit without Metal. Metal should do better; by how much on the 7- or 8-core M1 GPU is not published.
- Working estimate, not measured: a few times real time. A 78 min episode in roughly 20-30 min; Animales Políticos' 72.3 h catalogue in roughly a day of compute; the RTX 2050 did the same episode in 4-6 min.
- Memory: the turbo model file is 1.6 GB, which fits in 8 GB beside macOS; no 8 GB measurement found.

## Sources

- https://github.com/ggml-org/whisper.cpp/issues/89
- https://www.promptquorum.com/local-llms/apple-silicon-whisper-metal-benchmark
- https://zenn.dev/charlotte/scraps/afc1a0b37e8213
