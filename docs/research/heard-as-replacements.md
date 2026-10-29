# Heard-as replacements for misspelled names

Question: how do transcription tools fix a name the recognizer spells wrong every time, and what should a heard-as line in `glossary.txt` look like? 32 web searches, run after the name fix caught 6 of 13 misspellings on the week of 26 October (`docs/research/weekly-run.md`, "Names added to the glossary").

## Two mechanisms, used together

Every vendor that documents both keeps them apart:

- Hints before or during recognition bias the model toward a spelling: Deepgram keyterms, Speechmatics `additional_vocab` with `sounds_like`, Google phrase sets and custom classes, Azure phrase lists, Amazon custom vocabulary, Trint Vocab Builder, Otter custom vocabulary, Whisper's initial prompt. The app already has the Whisper prompt (`--glossary-prompt`); OpenAI's cookbook calls prompting "not especially reliable", and only the last 224 tokens of a prompt are used.
- Exact replacement after recognition: Deepgram `replace=find:replacement` (find lowercase, up to 200 terms suggested), AssemblyAI custom spelling (`from` list to one `to`), Gladia custom spelling, Azure display-format rewrite rules, Corti `{ find, replace }`, MacWhisper Global Replace, VoiceInk, Spokenly and Voquill replacement rules, Beey lexicon rules.

Gladia's guidance states the split: phonetic matching catches sound-alike errors but needs tuning and can misfire; custom spelling is a literal, deterministic swap with no false positives, but a variant never listed is never caught, so every variant seen goes in. Its suggested workflow is to transcribe once, then route each error to one or the other. That is the case here: the fuzzy fix stays for unseen spellings, and heard-as lines take the five it misses.

## How replacement lists behave

- One correct form, several heard forms: AssemblyAI takes an array of `from` words for one `to`; VoiceInk takes comma-separated originals for one replacement.
- Whole words only, case ignored: VoiceInk and Spokenly match case-insensitively at word boundaries ("github" also catches "GITHUB"); MacWhisper makes both optional; Azure matches the original case-insensitively and inserts the replacement as written.
- Conflicts: Azure applies the rule with the longer original; Conversation Analyzer applies rules in list order. Overlap is the known risk of plain replacement (Lumevoice, GoTranscript: never list an ordinary word as a heard form).
- Order: VoiceInk and Voquill run replacements on the raw transcript before the AI step, so the language model reads corrected text. The app already runs its name fix before the summary.
- Existing transcripts: MacWhisper and Beey apply new rules only to future transcriptions. Descript adds a word to its glossary after the user corrects it three times.
- A name heard as several words ("XL8" as "Excel8", Trint entries of several words) needs a multi-word heard form; none of this week's five misses is split.

## Fuzzy matching, for comparison

- Spanish-specific phonetic keys exist (Spanish Metaphone, Phonetic Spanish by Amón and Echeverría; a record-linkage comparison found Phonetic Spanish best most of the time). Double Metaphone lumps all foreign rules together and misses misspellings that change the sound.
- Thresholds trade recall for false hits: Splink suggests Jaro-Winkler 0.8 for first names and warns lower values match unrelated names; one library suggests 0.85 for fewer false positives.
- LLM post-correction of names can invent names it does not know (arXiv 2506.10779, 2409.06062); published fixes restrict the model to candidates from a list.

## What this means for the glossary

- A heard-as line maps one or more exact heard words to one glossary name, matched as whole words, case ignored, the name inserted as written, and applied before the fuzzy fix so it never fights it.
- Heard forms come only from transcripts already seen, as Gladia advises, so the five misses of this week are the first entries: Mimisa, Coyuntian, Jimena, Uneos, Luthers.
- No common Spanish word should be a heard form; "Jimena" is itself a real first name, so a line for it would also rewrite any real Jimena.

Sources: deepgram.com/docs/find-and-replace, developers.deepgram.com/docs/keyterm, assemblyai.com/docs/speech-to-text/pre-recorded-audio/custom-spelling, gladia.io/blog/custom-vocabulary-vs-custom-spelling, docs.speechmatics.com/features/custom-dictionary, learn.microsoft.com/azure/ai-services/speech-service/how-to-custom-speech-display-text-format, docs.cloud.google.com/speech-to-text/docs/adaptation, docs.aws.amazon.com/transcribe/latest/dg/custom-vocabulary-create-table.html, docs.corti.ai/stt/guides/dictation-replacements, macwhisper.helpscoutdocs.com/article/37-find-and-replace-in-transcriptions, tryvoiceink.com/docs/word-replacements, spokenly.app/docs/word-replacements, docs.voquill.com/guides/dictionary, beey.io/en/docs/user-guide/editor/find-find-and-replace-replacement-rules, help.descript.com transcription glossary, info.trint.com vocab builder, developers.openai.com/cookbook/examples/whisper_prompting_guide, developers.openai.com/cookbook/examples/whisper_correct_misspelling, github.com/amsqr/Spanish-Metaphone, scitepress.org/Papers/2016/62277, moj-analytical-services.github.io/splink comparators guide, docs-vcc.atlassian.net substitution rules, lumevoice.com/blog/custom-vocabulary-dictation, arxiv.org/abs/2506.10779, arxiv.org/abs/2409.06062.
