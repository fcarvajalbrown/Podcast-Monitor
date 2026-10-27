# Podcast Monitor PRD

Status: interview in progress. Sections 1-6 hold decisions Felipe made; everything not yet asked is in Open Questions.

## 1. Summary

A portable desktop app, written in Rust, that turns Chilean political podcasts into a PDF report: what each show talked about, for how long, who was named, and verbatim quotes with timestamps. It replaces the Python prototype `monitor.py`, which is untrusted and serves only as a list of ideas.

## 2. Problem

The report's reader needs to know what specialized political podcasts are putting on the agenda, alongside a weekly poll monitoring she already does, but there is no time to listen to them all. Today nobody covers it.

## 3. Users

- **Operator:** Felipe runs the app and sends the report.
- **Reader:** the report's reader, who reads the PDF on phone or PC. She uses an M1 Mac.

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
| Must | Two run modes: back-catalogue brief (every episode of a show) and latest-week report across the saved show list; runs on demand |
| Must | Add a show by name search in Apple's public podcast directory with a confirm step, or by pasting a feed URL; saved show list |
| Must | Audio from RSS first; YouTube fallback when a show has no feed |
| Must | Report sections: topics ranked by airtime, short summary per episode, people and parties named, verbatim quotes with timestamps |
| Must | PDF output in the "Desde el escritorio de Felipe Carvajal Brown" layout (`felipe_skill_voice/scripts/gen_pdf_desk.py` is the reference) |
| Must | Names glossary with phonetic post-correction of transcripts |
| Should | Glossary passed to Whisper as initial prompt (measurement pending) |

Hand-picked episode or date-range runs are out of v1.

## 6. Technical Architecture

- **Language and packaging:** Rust. Windows portable exe and macOS on Apple Silicon.
- **Speech-to-text:** Whisper large-v3-turbo through whisper.cpp (whisper-rs). GPU on Vulkan for Windows and Metal for Mac. Voxtral Mini was dropped from consideration.
- **Name correction:** glossary plus Spanish phonetic key and similarity threshold, on capitalized words only, skipping words that also appear in lowercase in the same transcript.
- **Sources:** RSS (all 17 chart shows and 20 of the 23 pilot shows have one), YouTube via yt-dlp as fallback.

Evidence:
- `docs/research/stt-bench-ap-ep34.md`: turbo on the RTX 2050 took 4 min 17 s for a 78 min episode (18x real time); faster-whisper small on CPU took 35 min and mangled Kast and Boric.
- `docs/research/name-correction.md`: phonetic correction raised correct "Kast" from 7 to 19 on turbo with no wrong change found.
- `docs/research/feeds-chile.md`, `docs/research/feeds-test-list.md`: feed availability.
- `docs/research/stt-alternatives.md`: other speech-to-text engines.

## 7-11

Not yet interviewed: UX, Risks, Testing and acceptance criteria, Phases and milestones.

## 12. Open Questions

- Who runs the app on the Mac: does the reader run it herself on her M1, or does Felipe still run everything?
- Which model writes the summaries, topics and people lists? No local LLM is installed; the Ep. 34 demo was written by hand from the transcript.
- Use publisher transcripts (Spreaker shows) when present, or always transcribe?
- Initial-prompt glossary: measure on Ep. 34 once whisper.cpp can be built (Vulkan SDK install was cancelled at the administrator prompt).
- Where the glossary comes from: feed text, Congress open data, a hand-kept list.
- Where Cómo te lo explico, Contracorriente and Comentarios de Tomás Mosciatti are published.
- Weekly cadence in addition to on demand.
