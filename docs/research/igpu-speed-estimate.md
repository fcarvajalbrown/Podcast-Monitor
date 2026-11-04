# Transcription speed on integrated graphics: an estimate until a real PC is timed

No integrated-GPU PC has been timed. This laptop (Ryzen 7 7435HS, RTX 2050) has no integrated GPU. This file holds what published sources allow inferring, read 2026-11-04.

## Measured here

- CPU only, no Vulkan driver visible: Ep. 34 (78 min, 4,685 s of audio) in 4,295 s, 1.1x real time, greedy with flash attention, release exe.
- RTX 2050 on Vulkan: 158 s (`stt-speed-ep34-rtx2050.md`).

## Published

- whisper.cpp PR #3492 (merged 1 November 2025, commit 999a7e0) lets `whisper_backend_init_gpu()` pick integrated GPUs (`GGML_BACKEND_DEVICE_TYPE_IGPU`) as well as discrete ones. Before it, whisper.cpp skipped an integrated GPU and ran on the CPU. The bundled whisper.cpp in whisper-rs-sys 0.15.0 is 1.8.3 and has that check (`src/whisper.cpp` lines 1302 and 1372), so this app uses an integrated GPU when its driver offers Vulkan.
- The PR tested a Ryzen 7 6800H with Radeon 680M and a Core Ultra 7 155H with Intel Arc graphics: "3-4x better realtime factor compared to CPU-only processing (CPU realtime factor: ~0.3)". The PR title says about 12x; the page does not reconcile the two, does not name the model, and does not define realtime factor.
- A January 2026 comment on the PR: the 1.8.3 Windows binary did not detect an Intel Iris Xe; the author put it down to drivers.
- No published timing found for older Intel UHD graphics (UHD 620 appears in the PR only as a detected device in a log line).

## Inference

The 6800H's processor is the same Zen 3+ eight-core family as this laptop's 7435HS, so its CPU-only time should be close to the 4,295 s measured here. Taking the PR's 3-4x at face value, a 680M or Arc-class integrated GPU would transcribe Ep. 34 in roughly 18 to 24 minutes, a quarter to a third of the episode's length. This rests on one PR with an undefined measure and an unnamed model. Older or weaker integrated graphics (Intel UHD before Arc) are expected to sit between that and the CPU time; no number supports a figure for them.

## Sources

- ggml-org/whisper.cpp PR #3492: https://github.com/ggml-org/whisper.cpp/pull/3492
- Phoronix, "Whisper.cpp 1.8.3 Delivers A 12x Performance Boost With Integrated Graphics": https://www.phoronix.com/news/Whisper-cpp-1.8.3-12x-Perf
