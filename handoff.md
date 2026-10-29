# Handoff

Continue Podcast-Monitor (C:\users\beetlejuice\desktop\podcast-monitor). Read notes/session-notes.md, then the open items at the top of ROADMAP.md, the phase 4 table in ROADMAP.md and notes/sessions/chat-8.md.

State: Silero VAD is fixed and kept off by default (it made Ep. 34 slower, 158 s to 222 s). The glossary at %APPDATA%\Podcast-Monitor\glossary.txt now takes heard-as lines (`Name = heard, heard`) and holds Mimica, Ximena, Kouyoumdjian, Huneeus and Lüders, plus `!Corte` and `!Cortes`, so the name fix caught 13 of 13 misspellings this week and leaves court words alone. The week of 26 October's saved summaries still carry the old spellings; redo them with tools/resummarize/resummarize.sh 2026-10-26 codex only if Felipe wants a corrected copy sent.

Next steps, in order:
1) Phase 4 item 6: acceptance on Ep. 34 and Nada Personal from the window, which also covers the open console check (a double-click must show only the window, no console). Start only when Felipe double-clicks C:\Users\Beetlejuice\.pmt\release\podcast-monitor.exe himself; ask before opening any window. Record what he sees in ROADMAP.md and docs/ROADMAP-HISTORY.md.
2) Then phase 5 (macOS on Apple Silicon M1 and later, Metal), which starts after phase 4 item 6; greedy with flash attention is not yet measured on Metal.

Build with tools/build/cargo-windows.cmd <args> -j 2 run directly from bash. Copy any PDF sent to the reader into work/sent/ as CLAUDE.md rule 3 says.
