# Chat 8

Done:
- Silero VAD never ran (whisper-rs `whisper_full_with_state` skips it); now run through `WhisperVadContext` (`a958569`). On Ep. 34 it cut 5 % of the audio and took 222 s against 158 s; Felipe kept it off by default.
- Glossary names Mimica, Ximena, Kouyoumdjian, Huneeus, Lüders added from the feeds' episode descriptions; fuzzy fix caught 6 of 13.
- 32 searches on misheard-name replacement (`docs/research/heard-as-replacements.md`); heard-as glossary lines `Name = heard, heard` built (`25b3ec2`); 13 of 13 caught, no false hits in the week's 28 transcripts.
- `!Corte` and `!Cortes` keep-lines stop court words becoming Cortés (20 swaps this week).

Flags opened:
- The week of 26 October's saved fixed transcripts and summaries still carry the old spellings; redo with `tools/resummarize/resummarize.sh 2026-10-26 codex` only if a corrected copy is sent.

Flags closed:
- Silero VAD did not engage.
- Names missing from the glossary.
- Corte to Cortés.
