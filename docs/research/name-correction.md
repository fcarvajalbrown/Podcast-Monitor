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

## Measured: phonetic correction on Ep. 34

`tools/stt_compare/phonetic_fix.py` with `tools/stt_compare/glossary-ap-ep34.txt` (21 names picked by hand from the episode, so this is a best case: a real glossary would come from feed text and lists). Phonetic key is a simple Spanish rule set plus a similarity ratio.

| Setting | Result |
|---|---|
| All words, 0.75 | unusable: "hacer" to Kaiser x25, "hasta" to Kast x14, "primera" to Piñera x13 |
| Capitalized only, 0.85 | 17 changes on turbo, all correct; misses Boris/Borich on small |
| Capitalized only, 0.75 | wrong on sentence-starters ("Hasta", "Casa" to Kast) |
| Capitalized only, 0.75, skip words seen twice in lowercase in the same text | 20 changes each on turbo and small, no wrong change found on review |

Correct spellings counted before and after (last setting):

| Name | turbo | turbo + fix | small | small + fix |
|---|---|---|---|---|
| Kast | 7 | 19 | 0 | 1 |
| Boric | 8 | 8 | 1 | 7 |
| Squella | 0 | 3 | 0 | 0 |
| Bellolio | 0 | 2 | 0 | 2 |
| Contardo | 1 | 2 | 1 | 2 |
| Arenas | 2 | 3 | 1 | 4 |
| Larraín | 2 | 2 | 2 | 4 |
| Parisi | 2 | 3 | 0 | 3 |

Findings: the fix lifts turbo close to complete on these names. It cannot fix small's lowercase "cast" and "caz" without touching ordinary words, so small still loses Kast almost entirely. "Castro" to Kast was right in this episode but would be wrong for a real Castro; the glossary step needs a per-show exception list.

## Measured: initial-prompt glossary on Ep. 34

whisper.cpp built from source (commit 231638a) with Vulkan (Vulkan SDK 1.4.363.0) on the RTX 2050; the log confirms the Vulkan0 backend. Two `whisper-cli` passes on `work/pool/ap-ep34/audio.wav`, `-l es`, large-v3-turbo, one with `--prompt` set to the 21 glossary names comma-separated. Converted with `tools/stt_compare/ffmpeg_whisper_to_bench.py` (now reads `whisper-cli -oj` JSON) and counted with `tools/stt_compare/count_names.py`.

| Name | FFmpeg filter | cli, no prompt | cli, prompt | cli no prompt + fix | cli prompt + fix |
|---|---|---|---|---|---|
| Kast | 7 | 8 | 8 | 19 | 18 |
| Boric | 8 | 7 | 9 | 9 | 9 |
| Squella | 0 | 0 | 0 | 0 | 1 |
| Bellolio | 0 | 0 | 0 | 2 | 2 |
| Contardo | 1 | 1 | 2 | 2 | 2 |
| Rabat | 4 | 5 | 3 | - | - |
| Parisi | 2 | 0 | 0 | - | - |
| Bachelet | 2 | 2 | 1 | - | - |

Wrong spellings of Kast: no prompt Cass 6, Cast 2; with prompt Kass 4, Cass 6.

Findings:
- The prompt brought no measurable gain on these names: Kast stayed at 8, Boric rose by 2, Rabat and Bachelet fell. Phonetic post-correction does the work either way.
- With the prompt, `whisper-cli` split the episode into 4,173 segments instead of 2,544 (similar word count, 12,444 against 12,427).
- Speed: `whisper-cli` on Vulkan took 354 s without the prompt and 335 s with it, against 257 s for the FFmpeg filter run in `stt-bench-ap-ep34.md`. Both are well above real time.
- One episode and a hand-picked glossary; not a general result.

## Measured: second episode, Nada Personal

"La sorpresa detrás de la vuelta de María Corina Machado a Venezuela", Radio Duna, 2026-10-26, 3,563 s (59 min). Fetched with `tools/feed_check/fetch_episode.py`. Glossary `tools/stt_compare/glossary-np-latest.txt`: 16 surnames from the episode description and the no-prompt transcript, so again a best case. Same `whisper-cli` build and settings as above; fix is `phonetic_fix.py` at its defaults (capitalized only, 0.75, skip words seen twice in lowercase).

| Name | no prompt | prompt | no prompt + fix | prompt + fix |
|---|---|---|---|---|
| Machado | 10 | 10 | 10 | 10 |
| Kast | 2 | 3 | 5 | 3 |
| Lula | 24 | 24 | 24 | 26 |
| Milei | 0 | 0 | 0 | 0 |
| Faúndez | 2 | 2 | 7 | 7 |
| Arias | 3 | 3 | 33 | 33 |
| Ríos | 3 | 3 | 6 | 5 |
| Rabat | 3 | 2 | 3 | 2 |
| Sheinbaum | 0 | 0 | 0 | 0 |

Changes the fix made on the no-prompt transcript: María to Arias x27, Andrés to Faúndez x4, Kass to Kast x3, Arica to Arias x3, Dios to Ríos x1, Faundes to Faúndez x1, Pros to Ríos x1, Urroz to Ríos x1. With the prompt it also changed Luna to Lula x2.

Findings:
- The prompt again brought no gain: Milei ("Milley" x3) and Sheinbaum ("Shinjaps" x3) stayed wrong with both names in the prompt, Kast rose by one. Segment count moved the other way this time (861 with the prompt against 3,104 without), so the prompt's effect on segmentation is not consistent.
- The phonetic fix is not safe on a new show at these settings. Most of its changes here were wrong: first names (María, Andrés), a city (Arica) and ordinary words (Dios, Pros) became glossary surnames. Only Kass to Kast and Faundes to Faúndez were right. The Ep. 34 result ("no wrong change found") does not generalize.
- The fix also missed what was needed: "Milley" and "Shinjaps" were not matched to Milei and Sheinbaum.
- Speed: 412 s without the prompt and 422 s with it, for 59 min of audio.

## Measured: stricter threshold on both episodes

`phonetic_fix.py --threshold 0.85` on the `whisper-cli` no-prompt transcripts.

- Nada Personal: Kass to Kast x3, Faundes to Faúndez x1. 4 changes, all correct.
- Ep. 34: Cass to Kast x6, París to Parisi x3, Belolio to Bellolio x2, Cast to Kast x2, Contarto to Contardo x1, Arena to Arenas x1, Mabdani to Mamdani x1. 16 changes; the París contexts are about the candidate ("París es curioso porque es menos cartucho"), so all read as correct.
- 0.85 still misses Milley (Milei) and Shinjaps (Sheinbaum).

Felipe's decision: v1 runs the fix at 0.85 and also sends the glossary to the summarizer with each chunk, so summaries and people lists use the right names where the transcript stays wrong. Quotes keep the transcript's words. The initial prompt is dropped from v1.
