# Reader feedback

## On the report after phase 4

Her words, as relayed by Felipe:

> Encuentro que el informe anterior estaba más completo. Recuperaría el resumen y los temas por tiempo al aire (aunque sólo por porcentaje). Además, recuerda que son muchos podcasts, entonces tiene que ser bien corto por cada uno porque sino será una biblia.

Asked by Felipe for an ideal date for the first reports:

> que sea actual, o sea, los de esta semana por ejemplo

What it asks for, read against the current PDF (Ep. 34 is 3 pages: summary, 29 topics with minutes, 43 people, 5 quotes):

- Keep the summary and the topics by airtime; airtime as a percentage only, not minutes.
- Each show short, because the report covers many shows.
- The first reports are this week's episodes, which is the latest-week report across the show list that the PRD left out of v1.
- "El informe anterior" is the hand-made Ep. 34 report of 26 October (`work/sent/2026-10-26-animales-politicos-ep34-hand.pdf`, 3 pages), confirmed by Felipe. The window PDF is kept beside it as `2026-10-27-animales-politicos-ep34-window.pdf`.

What the hand-made report had that the window PDF lacks:

- A header line with hosts and guest, weekday of publication, recording time, and duration in minutes.
- "En una línea": a one-sentence gist.
- 9 grouped topics with minutes and percent of the episode (the window PDF has 29 fine-grained topics, minutes only).
- A summary in paragraphs per segment, each with its time span.
- People and parties grouped by sector, names only, instead of 43 people with mention counts.
- 10 quotes in a minute and quote table, some marked with the speaker.
- "Cómo se hizo": how the report was made.

Show list she sent with it, 20 shows, as written:

1. Los Columnistas de Mesa Central – Tele13 Radio
2. Rat Pack de Mesa Central – Tele13 Radio
3. Columnistas Página 13 – Tele13 Radio
4. Hablemos en Off – Radio Duna
5. Nada Personal – Radio Duna
6. Terapia Chilensis – Radio Duna
7. El Primer Café – Radio Cooperativa
8. Panelistas de Pauta de Análisis – Radio Pauta
9. Polos Opuestos – Radio El Conquistador
10. Comentarios de Tomás Mosciatti – Radio Bío Bío
11. Animales Políticos – Daniel Matamala y Cristóbal Bellolio
12. Lo Que Importa – Daniel Matamala
13. Sin Llorar – The Clinic
14. Cómo te lo explico – Mónica Rincón y Paula Catena
15. El Villegas: Actualidad y esas cosas – Fernando Villegas
16. Democracia en LSD – Ximena Jara y Davor Mimica
17. CIPER en tus Oídos – CIPER Chile
18. Expediente Mónica – CIPER Chile
19. El Shot – Juan Manuel Astorga
20. Contracorriente – The Clin (cut off in the message)

Against `feeds-test-list.md`: these are the 23 test shows minus Entender es Todo, A mí nunca me han encuestado and Con Diálogo Es Distinto. 17 of the 20 have an official feed; Cómo te lo explico, Contracorriente and Comentarios de Tomás Mosciatti still have none found.

## Size of a week

`tools/feed_check/check_feeds.py tools/feed_check/shows-reader.json --since 2026-10-26` (her 20 shows; raw output in `tools/feed_check/results-reader-week.json`), checked on Wednesday 28 October:

- Monday 26 to Wednesday 28 October: 25 episodes, 18.7 h of audio, from 13 shows.
- 0 episodes so far this week from Lo Que Importa, Sin Llorar, CIPER en tus Oídos and Expediente Mónica.
- Cómo te lo explico, Contracorriente and Comentarios de Tomás Mosciatti still resolve to the wrong show or the stale re-upload listed in `feeds-test-list.md`, so their 0 means nothing.
- At the Ep. 34 window speed on the RTX 2050 (4,685 s of audio in 395 s, about 12x), 18.7 h of audio is about 1.6 h of transcription, before the summaries.

## On the first weekly report

The report she read: `work/sent/2026-10-28-semanal-semana-2026-10-26-window.pdf`. Her words, 28 October 16:17, as relayed by Felipe:

> Los encuentro iguales kjahsa
>
> igual, la distribución debería ser en total de todos los programas hablados
>
> porque así se sabe cuál es el tema de la semana

What it asks for:

- The topic distribution should be one total across all the shows, so the report shows the topic of the week. Today each show block has its own topics in percent.
- "Los encuentro iguales" (she finds them the same): read by Felipe as the weekly report being no different from the earlier on-demand reports.
