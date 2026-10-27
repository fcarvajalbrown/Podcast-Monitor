# Whisper benchmark: Animales Políticos Ep. 34

Episode: "Ep. 34 - La crisis de los decretos", 4,685 s (78 min), publisher MP3 from the show's RSS feed. Machine: Felipe's laptop, RTX 2050 4 GB, 16 logical CPUs. Felipe dropped Voxtral Mini from the benchmark and asked for a faster Whisper setup taken from his existing repos.

| Engine | How | Compute | RTF | Times real time | Disagreement with turbo |
|---|---|---|---|---|---|
| whisper large-v3-turbo | whisper.cpp through FFmpeg 9.0.2 `whisper` filter, `language=es`, `queue=30`, GPU on | 257 s | 0.055 | 18x | reference |
| faster-whisper small | CPU int8, 4 threads, Cuña-Cutter `tools/stt_bench/bench.py` | 2,117 s | 0.452 | 2.2x | 14.4 % WER |

Peak memory of the small CPU run: 4,461 MB.

## Surnames

Counted with `tools/stt_compare/names.py` and a spelling count on both transcripts.

| Name as spoken | turbo | small |
|---|---|---|
| Kast | Kast 7, Kass 5, Cass 3, Cast 1, cast 1 | Caz 4, Cast 2, Cas 2, cast 7 |
| Boric | Boric 8 | Boris 4, Borich 2, Boric 1 |

## Findings

- On this machine turbo on the GPU is 8 times faster than small on the CPU and more accurate; small on CPU only makes sense on a machine with no usable GPU.
- The FFmpeg build used has `--enable-vulkan` and no CUDA toolkit is installed; GPU load was 99 % during the turbo run. The backend whisper.cpp picked is not printed in the log, so Vulkan is likely but unconfirmed.
- small mangles the two most important surnames of the episode; turbo gets Boric right every time and Kast right less than half the time. A names glossary (prompt or post-correction) is needed whatever the engine.
- The 14.4 % figure is disagreement with turbo, not error against a human transcript.

## Reproduce

```
ffmpeg -i audio.mp3 -ar 16000 -ac 1 work/pool/ap-ep34/audio.wav
python Corpofobia/tools/video_review/transcribe.py audio.mp3 --model ../Corpofobia/work/models/ggml-large-v3-turbo.bin --out whisper-turbo.jsonl --language es --queue 30
python tools/stt_compare/ffmpeg_whisper_to_bench.py whisper-turbo.jsonl --id ap-ep34 --engine whisper-turbo-ffmpeg --compute-s 257 --audio-s 4685.1
python Cuña-Cutter/tools/stt_bench/bench.py small --pool work/pool --out work/bench --length 4700 --offset 0 --threads 4
python Cuña-Cutter/tools/stt_bench/score.py whisper-turbo-ffmpeg --bench work/bench
python tools/stt_compare/names.py work/bench/whisper-turbo-ffmpeg/ap-ep34.json work/bench/whisper-small-cpu/ap-ep34.json
```

The model path to the FFmpeg filter must be relative: an absolute `C:` path breaks the filter's option parser.
