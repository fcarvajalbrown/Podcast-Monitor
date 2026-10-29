# Transcription speed options: Ep. 34 on the RTX 2050

Episode: "Ep. 34 -La crisis de los decretos - Animales Políticos - Segunda Temporada", 4,685 s. Vulkan on the RTX 2050 (4 GB) laptop, glossary `tools/stt_compare/glossary-ap-ep34.txt`, fix threshold 0.85, each variant decoded from scratch in its own folder, one after another. Run with `tools/stt_speed/bench.sh work/stt-speed all` (flags `--flash-attn` and `--vad-model` added to `run` for this). Models: `ggml-large-v3-turbo.bin` (full precision, the app's model), `ggml-large-v3-turbo-q8_0.bin` from the whisper.cpp model page on Hugging Face, `ggml-silero-v5.1.2.bin` from ggml-org/whisper-vad.

| Variant | compute_s | Real-time factor | Against today | Kast raw / fixed | Boric fixed |
|---|---|---|---|---|---|
| Beam 5, full model (today) | 334 | 14.0x | | 10 / 18 | 7 |
| Flash attention | 250 | 18.7x | -25 % | 9 / 17 | 7 |
| Greedy | 243 | 19.3x | -27 % | 9 / 17 | 9 |
| Flash attention and greedy | 158 | 29.7x | -53 % | 10 / 17 | 8 |
| Q8_0 model | 306 | 15.3x | -8 % | 11 / 18 | 9 |
| Silero VAD | 318 | 14.7x | -5 % | 10 / 18 | 7 |

Target after the name fix: Kast at least 16, Boric at least 7. Every variant meets it.

- Flash attention is off by default in whisper-rs 0.16 (`WhisperContextParameters::default`), which likely explains the earlier gap against whisper-cli (354 s against 424 s).
- The VAD run gave the same 1,591 segments and the same name counts as today's setting, so VAD most likely did not engage; its 5 % is within run-to-run spread. Not investigated.
- The two gains stack: flash attention and greedy together halve the transcription time. On last week's 52.4 min of transcription that would be about 25 min.
- Name counts are not an accuracy measure; the words around the names were not compared.
