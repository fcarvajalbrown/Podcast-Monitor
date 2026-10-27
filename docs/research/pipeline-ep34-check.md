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
