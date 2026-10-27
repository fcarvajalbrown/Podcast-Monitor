# Fixing misheard names in transcripts

Problem: Whisper turns names into other real words or names ("Kast" as Kass, Cass, Castro, cast; "Boric" as Boris, Borich). See `stt-bench-ap-ep34.md`. Felipe asked whether `small` plus a Word-style corrector would do, then asked for the glossary prompt and phonetic correction to be measured.

## Why a spelling or grammar checker does not help

Spelling and grammar checkers flag words that do not exist or do not agree. Speech-to-text errors are sound-alike substitutions that are valid words, so they pass. Options checked anyway:

| Library | Language | Spanish | Licence | Fit |
|---|---|---|---|---|
| LanguageTool | Java | yes | LGPL | no single exe; does not catch valid-word substitutions |
| nlprule | Rust | experimental, data from LanguageTool 5.2 | MIT/Apache code, LGPL data | same limitation |
| Hunspell / spellbook | Rust | dictionaries exist | varies | same limitation |

## What does work: a names glossary

Two steps, usable together:

1. **Prompt before decoding.** Whisper takes an initial prompt; names listed there bias the spelling. OpenAI's cookbook calls it "not especially reliable"; one informal whisper.cpp test reports proper-noun recovery going from 45.9 % to 76.0 % with a hint list. Whisper reads only the last ~224 tokens of the prompt, so the list must be short and ordered by importance. Hotword biasing proper is not supported for Whisper in sherpa-onnx.
2. **Phonetic correction after decoding.** Match each word against the glossary by sound and edit distance and replace near misses. This is the common baseline in the research (double metaphone plus n-grams in MindMeld, 2019); no standalone library does it, it has to be assembled. Large glossaries raise false matches.

## Building blocks for the Rust exe

| Need | Crate | Notes |
|---|---|---|
| Initial prompt | whisper-rs `FullParams::set_initial_prompt` | seen in a fork; check upstream |
| GPU without CUDA | whisper-rs `vulkan` feature | Windows build needs the Vulkan SDK (`VULKAN_SDK`), not installed on Felipe's machine |
| Phonetic key | rphonetic (Double Metaphone, port of Apache commons-codec) | designed for English names; a Spanish Metaphone (Mosquera) exists in Python (abydos) and would need porting |
| Edit distance | strsim (MIT) or rapidfuzz | Jaro-Winkler, Levenshtein |
| Spanish phonemes | espeak-ng (pure Rust port) | GPL-3.0, which would make the exe GPL |

## Glossary sources

- Each feed's episode titles and descriptions name guests and topics.
- Chamber of Deputies and Senate open data (XML and JSON), also mirrored by OpenSanctions (free for non-commercial use).
- A hand-kept list of current government figures.

## Tooling found for the measurement

- FFmpeg's `whisper` filter has no prompt option, so the prompt test needs whisper.cpp's own `whisper-cli --prompt`.
- Official whisper.cpp Windows binaries (release b5454) ship CPU, BLAS and CUDA 11.8/12.4 builds, no Vulkan build.
- Visual Studio 18 Community and Rust (msvc) are installed; the Vulkan SDK is not.

## Sources

- https://developers.openai.com/cookbook/examples/whisper_prompting_guide
- https://developers.openai.com/cookbook/examples/whisper_correct_misspelling
- https://www.mintlify.com/XDcobra/react-native-sherpa-onnx/advanced/hotwords
- https://preview.aclanthology.org/author-url/D19-3011.pdf
- https://arxiv.org/pdf/2508.20700v2
- https://github.com/amsqr/Spanish-Metaphone
- https://abydos.readthedocs.io/en/v0.3.5/_modules/abydos/phonetic/_es.html
- https://docs.rs/rphonetic
- https://docs.rs/crate/rapidfuzz/latest
- https://docs.rs/espeak-ng
- https://docs.rs/crate/nlprule
- https://git.hoyer.xyz/harald/wyoming-whisper-rs/commit/e33cc3eb2206df53a533bba0b177f0f5036f9fa7
- https://www.opensanctions.org/datasets/cl_chamber_deputies/
- https://www.opensanctions.org/datasets/cl_senate/
