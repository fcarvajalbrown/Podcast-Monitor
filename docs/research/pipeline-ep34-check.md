# Phase 1 check: Ep. 34 through the Rust pipeline

Episode: "Ep. 34 -La crisis de los decretos - Animales Políticos - Segunda Temporada", 4,685 s. Command:

```
podcast-monitor.exe run https://anchor.fm/s/106ac41b0/podcast/rss --title "Ep. 34" --model ../Corpofobia/work/models/ggml-large-v3-turbo.bin --glossary tools/stt_compare/glossary-ap-ep34.txt
```

Built with `tools/build/cargo-windows.cmd build --release` at commit 8d811f5.

## GPU

- Log: `ggml_vulkan: 0 = NVIDIA GeForce RTX 2050`, `whisper_model_load: Vulkan0 total size = 1623.92 MB`.
- `nvidia-smi`: 1,115 MiB used before the model loaded, 3,232 MiB and 59 % utilization during transcription.
- Compute: 4,685 s of audio in 424 s, 3,226 segments. For comparison, `whisper-cli` on Vulkan took 354 s on the same episode (`name-correction.md`).

## Resume

- First run: `audio.mp3` already in `work/runs/animales-politicos/`, kept its original timestamp, no download line in the log; the run went straight to decoding.
- Second run: printed `transcript already done`, loaded no model and only reran the name fix.

## Names (item 7)

`count_names.py` on `transcript.json` and `transcript.fixed.json` (Rust fix at 0.85):

| Name | Rust raw | Rust + fix | target |
|---|---|---|---|
| Kast | 10 | 17 | 16 |
| Boric | 9 | 9 | 7 |
| Bellolio | 0 | 2 | |
| Contardo | 1 | 2 | |
| Arenas | 3 | 4 | |
| Mamdani | 0 | 1 | |
| Parisi | 3 | 3 | |

Target is the `whisper-cli` no-prompt transcript with `phonetic_fix.py --threshold 0.85` in `name-correction.md`: Kast 8 raw plus Cass x6 and Cast x2, Boric 7 with no Boric change. Both names are above it.

Replacements, Rust and `phonetic_fix.py --threshold 0.85` on the same `transcript.json`: Cass to Kast x6, Belolio to Bellolio x2, Arena to Arenas x1, Contarto to Contardo x1, Kass to Kast x1, Mabdani to Mamdani x1. 12 each, identical pairs and counts.

The whisper-rs transcript differs from the `whisper-cli` one: it already has Parisi x3 where `whisper-cli` heard París, and no "Cast", so the fix had fewer words to change (12 against 16).
