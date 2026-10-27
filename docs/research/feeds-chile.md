# Chilean political podcast feeds

Source list: Chilean-publisher shows on the Apple Podcasts Chile Politics chart (top 200, genre 1527), plus Lo Que Importa con Daniel Matamala and Informe:Chile. Felipe chose the Apple chart after Spotify's chart proved unreadable without a browser (its API returns Internal Server Error). Felipe: all of them are useful.

Checked with `tools/feed_check/check_feeds.py tools/feed_check/shows-chile.json`; raw output in `tools/feed_check/results-chile.json`.

| Apple rank | Show | Publisher | Episodes | Hours | Publisher transcripts | Oldest | Newest |
|---|---|---|---|---|---|---|---|
| 1 | Polos Opuestos | El Conquistador FM | 1000 (feed cap) | 699.0 | yes | 2022-06-08 | 2026-10-26 |
| 2 | Animales Políticos | Casa Editorial Lo Que Importa | 60 | 72.3 | no | 2025-06-28 | 2026-10-26 |
| 5 | Nada Personal | Radio Duna | 368 | 363.5 | no | 2025-04-09 | 2026-10-26 |
| 7 | Ideas con Fuerza | El Conquistador FM | 377 | 279.6 | yes | 2025-03-03 | 2026-10-22 |
| 8 | Panelistas de Pauta de Análisis | Radio Pauta | 599 | 209.8 | no | 2023-10-20 | 2026-10-26 |
| 13 | Visionarios | Radio Duna | 313 | 16.2 | no | 2025-04-08 | 2026-10-23 |
| 20 | Sentido Común | El Conquistador FM | 930 | 596.6 | yes | 2023-01-03 | 2026-10-26 |
| 26 | SIN LLORAR | The Clinic | 60 | 47.6 | no | 2025-08-13 | 2026-10-21 |
| 36 | Reporte Minero y Energético | Radio Duna | 72 | 57.9 | no | 2025-04-09 | 2026-10-21 |
| 62 | Sintonía Crónica | Radio Duna | 307 | 146.2 | no | 2025-05-20 | 2026-10-26 |
| 80 | Ahora en Duna | Radio Duna | 252 | 96.2 | no | 2025-04-08 | 2026-10-26 |
| 113 | Crónicas de Ruperto Concha | Resumen.cl | 137 | 56.0 | no | 2019-10-20 | 2024-05-21 |
| 120 | Aire Fresco | Radio Duna | 361 | 316.6 | no | 2025-04-11 | 2026-10-26 |
| 130 | Charlas de política | Juntos por Chile | 3 | 3.3 | no | 2020-09-10 | 2021-07-25 |
| 200 | Cambios Que Transforman | Radio Duna | 233 | 212.3 | no | 2025-10-06 | 2026-10-26 |
| - | Lo Que Importa con Daniel Matamala | Casa Editorial Lo Que Importa | 102 | 107.3 | no | 2024-04-15 | 2026-10-21 |
| - | Informe:Chile | El Ideálogo | 190 | 77.8 | partial | 2019-12-11 | 2026-02-24 |

## Findings

- All 17 shows have a working public RSS feed with audio enclosures. RSS-only sourcing holds for this list.
- Spreaker-hosted shows (El Conquistador FM, Informe:Chile) publish Spanish `podcast:transcript` files in SRT, TXT and VTT, so they need no local speech-to-text. Their accuracy is unmeasured.
- Total back catalogue is about 3,160 hours; Animales Políticos alone is 72.3 hours.
- Spreaker returns at most 1000 items per feed; Polos Opuestos has older episodes not reachable through RSS.
- Crónicas de Ruperto Concha (last 2024-05), Charlas de política (last 2021-07) and Informe:Chile (last 2026-02) are inactive or paused.
- "Sentido Común" appears twice on the chart (ranks 20 and 91, both El Conquistador FM); only one feed was checked.
