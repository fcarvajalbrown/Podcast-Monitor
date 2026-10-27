# Podcast Monitor PRD

Status: interview in progress. Sections 1-6 hold decisions Felipe made; everything not yet asked is in Open Questions.

## 1. Summary

A portable desktop app, written in Rust, that turns Chilean political podcasts into a PDF report: what each show talked about, for how long, who was named, and verbatim quotes with timestamps. It replaces the Python prototype `monitor.py`, which is untrusted and serves only as a list of ideas.

## 2. Problem

The report's reader needs to know what specialized political podcasts are putting on the agenda, alongside a weekly poll monitoring she already does, but there is no time to listen to them all. Today nobody covers it.

## 3. Users

- **Reader and operator:** the report's reader runs the app herself on her M1 Mac with 8 GB of memory and reads the PDF on phone or PC.
- **Maintainer:** Felipe builds the app, keeps the show list and glossary seeds, and runs it on Windows for testing.

## 4. Goals and Non-goals

Goals:
- Any show the reader asks for can be added; Animales Políticos is the pilot.
- A report she can read instead of listening, with every quote checkable by minute.

Non-goals:
- No political affiliation appears anywhere: code, docs, commits, reports, sample data (project rule 1, public repo).
- Not a public web service.

## 5. Scope and Features

| Priority | Feature |
|---|---|
| Must | Two run modes: back-catalogue brief (every episode of a show) and latest-week report across the saved show list; on demand only, no schedule in v1 |
| Must | Add a show by name search in Apple's public podcast directory with a confirm step, or by pasting a feed URL; saved show list |
| Must | Audio from the show's RSS feed |
| Must | Report sections: topics ranked by airtime, short summary per episode, people and parties named, verbatim quotes with timestamps |
| Must | Small native desktop window in Spanish: show list with add and remove, one button per run mode, progress per episode, Open PDF at the end |
| Must | Resumable runs: each episode's transcript and summary is saved as soon as it is done; running again continues where it stopped, and the PDF is built from what is finished |
| Must | PDF output in the "Desde el escritorio de Felipe Carvajal Brown" layout (`felipe_skill_voice/scripts/gen_pdf_desk.py` is the reference) |
| Must | Names glossary with phonetic post-correction of transcripts at similarity 0.85; the same glossary is sent to the summarizer with each chunk so summaries and people lists use the right names where the transcript stays wrong; quotes keep the transcript's words; glossary built from an editable hand list, names in each show's episode titles and descriptions, and Chamber of Deputies and Senate open data, with a per-show exception list |

Out of v1: Whisper initial-prompt glossary (no gain on two episodes, see `docs/research/name-correction.md`), the YouTube fallback (no v1 show needs it; it returns with the three no-feed shows), hand-picked episode or date-range runs, publisher transcripts (every episode is transcribed locally, even when the feed carries a transcript), and the three pilot shows with no official feed (Cómo te lo explico, Contracorriente, Comentarios de Tomás Mosciatti).

## 6. Technical Architecture

- **Language and packaging:** Rust. macOS on Apple Silicon is the primary target (the reader's 8 GB M1); Windows portable exe for Felipe's testing.
- **Speech-to-text:** Whisper large-v3-turbo through whisper.cpp (whisper-rs). GPU on Vulkan for Windows and Metal for Mac. Voxtral Mini was dropped from consideration.
- **Name correction:** glossary plus Spanish phonetic key and similarity threshold 0.85, on capitalized words only, skipping words that also appear in lowercase in the same transcript. At 0.75 it turned María into Arias 27 times on a Nada Personal episode.
- **Interface:** native Rust window (egui or similar, no web stack), Spanish UI strings; code, identifiers and logs in English.
- **Summaries, topics and quote picks:** the app calls the official command-line tool of the assistant subscription the reader already pays for, logged in once on her Mac (`claude -p` or `codex exec`), sends it transcript chunks and reads the reply. The app never reads or stores her login token. v1 supports both behind one interface, chosen in settings. People counts and quote timestamps come straight from the corrected transcript.
- **Sources:** RSS only in v1 (all 17 chart shows and the 20 remaining pilot shows have one).
- **Quotes:** the summarizer only points to passages; the app finds each one in the timestamped transcript and prints the transcript's own text and minute, dropping any it cannot find.

Evidence:
- `docs/research/stt-bench-ap-ep34.md`: turbo on the RTX 2050 took 4 min 17 s for a 78 min episode (18x real time); faster-whisper small on CPU took 35 min and mangled Kast and Boric.
- `docs/research/name-correction.md`: phonetic correction raised correct "Kast" from 7 to 19 on turbo; on a second show the 0.75 setting made mostly wrong changes and 0.85 made only correct ones; the initial prompt gave no gain on either episode.
- `docs/research/feeds-chile.md`, `docs/research/feeds-test-list.md`: feed availability.
- `docs/research/stt-alternatives.md`: other speech-to-text engines.

## 7. UX

- The reader runs everything from one small window; the only Terminal step is the one-time login to her assistant CLI.
- Interface text in Spanish only.
- Adding a show: search by name, confirm the match with its publisher shown, or paste a feed URL.
- Long runs show progress per episode and survive a closed app or lid: finished episodes are kept and the next run continues.

## 8. Business case

Not applicable: a tool for one reader, no revenue.

## 9. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Scripted use of her assistant CLI draws from a capped allowance; a back-catalogue run can hit it | Run stops cleanly on a limit error, keeps finished episodes and tells her in Spanish to run again later; one summary call per episode where it fits |
| Provider terms on subscription logins are written for their own apps; scripted use through the official CLI on her own Mac is not stated either way, and the terms can change | The app only calls the official CLI and never handles her token; supporting two providers means one can replace the other |
| The summarizer returns a quote that is not in the audio, or a wrong minute | Every quote is looked up in the transcript; text and minute come from the transcript, unmatched quotes are dropped |
| The name fix changes a correct word into a glossary name | Threshold 0.85, capitalized words only, skip words common in lowercase, per-show exception list; at 0.75 a second show produced 27 wrong changes |
| Whisper turbo on the 8 GB M1 is slower than estimated | Estimate from published numbers is a few times real time (`docs/research/stt-speed-m1.md`); runs are resumable, so a long catalogue can span several sessions |
| A publisher changes or removes a feed | Feed URL is saved per show and can be replaced by search or paste |
| The public repo ties the tool to a political side | Project rule 1; transcripts, reports and episode audio stay in the git-ignored `work/` |

## 10-11

Not yet interviewed: Testing and acceptance criteria, Phases and milestones.

## 12. Open Questions

- Which assistant subscription the reader has (decides which CLI is tested first).
- Whisper turbo speed on the 8 GB M1 with Metal: estimated, not measured.
