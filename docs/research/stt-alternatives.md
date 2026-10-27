# Speech-to-text options for a Rust exe

Constraints: Chilean Spanish, Windows, single portable exe, Felipe's machine has an NVIDIA RTX 2050 with 4 GB VRAM.

## Already on this machine

| What | Where | Usable from Rust |
|---|---|---|
| whisper.cpp `ggml-large-v3-turbo.bin` (1.6 GB) | `Desktop\Corpofobia\work\models\` | yes, whisper-rs |
| faster-whisper base, small, medium, large-v3-turbo (CTranslate2) | `~\.cache\huggingface\hub\` | not directly |
| FFmpeg 9.0.2 with the `whisper` filter (whisper.cpp) | WinGet | only by shelling out to ffmpeg.exe |

Prior measurement in Cuña-Cutter (`docs/adr/0003`): YouTube auto-captions on Chilean political speech, WER 24.5 %, 8 of 12 surnames right; Parakeet TDT 0.6b v3 cannot be locked to Spanish and drifted into English once.

## Candidates

| Engine | Spanish evidence | Size / VRAM | Licence | Rust path |
|---|---|---|---|---|
| Whisper large-v3-turbo (whisper.cpp) | Whisper large-v3 among the most accurate free models in an Iberian-language benchmark | 1.6 GB file | MIT | whisper-rs, `cuda` feature; Windows CUDA build needs CUDA, MSVC with Clang, CMake, `LIBCLANG_PATH` |
| Voxtral Mini 3B (Mistral) | Spanish listed as supported; no Spanish WER found | Q4_K_M GGUF 2.98 GB | Apache 2.0 | no Rust binding found; third-party transcribe.cpp CLI only |
| Voxtral Small 24B | 3.04 % Spanish WER on Open ASR Leaderboard | far beyond 4 GB VRAM | Apache 2.0 | out |
| Meta omniASR LLM 1B | among the most accurate free models in the Iberian benchmark | about 6 GiB VRAM | Apache 2.0 (suite) | no Rust binding found; out on 4 GB |
| Parakeet TDT 0.6b v3 (via sherpa-onnx) | no Spanish WER found; drifted to English in Cuña-Cutter | small | CC-BY-4.0 | sherpa-rs, prebuilt binaries option |
| Canary-Qwen 2.5B | English only | - | - | out |

## Findings

- No candidate beats Whisper on published Spanish evidence while fitting 4 GB VRAM and having a Rust binding.
- Voxtral Mini is the only plausible challenger; it has no Spanish number and no Rust binding, so it can only be judged by a local benchmark.
- Spreaker-hosted feeds publish their own Spanish transcripts (see `feeds-chile.md`), which skip speech-to-text entirely.
- Transcription speed on the RTX 2050 is unmeasured for every engine.

## Sources

- Open ASR Leaderboard paper: https://arxiv.org/pdf/2510.06961
- Benchmarking ASR tools for Iberian languages: https://arxiv.org/html/2609.36920v1
- whisper-rs: https://docs.rs/crate/whisper-rs/latest
- sherpa-rs: https://github.com/Limit-LAB/sherpa-rs
- Voxtral Mini 3B GGUF: https://huggingface.co/handy-computer/Voxtral-Mini-3B-2507-GGUF
- omniASR: https://huggingface.co/facebook/omniASR-CTC-1B
