# Podcast-Monitor

## Rules

1. **No political affiliation anywhere.** The repo is public. No party, organisation, person or chat that ties this tool or Felipe to a political side appears in code, docs, commits, reports, sample data or test fixtures. Shows are described by name and publisher only, never by leaning.
2. **`monitor.py` is an untrusted prototype.** It was generated elsewhere and Felipe does not trust it. Read it for ideas only; no behaviour carries into the Rust exe unless the PRD records Felipe choosing it.
3. **Keep every report sent to the reader.** Copy each PDF the reader receives into the git-ignored `work/sent/` as `<date sent>-<show>-<episode>-<how made>.pdf`, so her feedback can be checked against the exact file she read.

## Layout

- `docs/PRD.md`: product requirements (interview in progress).
- `docs/research/`: verified findings, one file per question.
- `tools/feed_check/`: finds shows in the Apple directory and checks their RSS feeds.
