# Feeds for the pilot test list

23 shows supplied by the report's reader as the test set. Checked with `tools/feed_check/check_feeds.py tools/feed_check/shows-test.json`; raw output in `tools/feed_check/results-test.json`.

| Show | Publisher in feed | Episodes | Hours | Publisher transcripts | Newest | Status |
|---|---|---|---|---|---|---|
| Los Columnistas de Mesa Central | Tele 13 Radio | 2071 | 1276.7 | no | 2026-10-26 | ok |
| Rat Pack de Mesa Central | Tele 13 Radio | 2088 | 893.8 | no | 2026-10-23 | ok |
| Columnistas Página 13 | Tele 13 Radio | 2382 | 1779.6 | no | 2026-10-26 | ok |
| Hablemos en Off | Radio Duna | 370 | 329.0 | no | 2026-10-26 | ok |
| Nada Personal | Radio Duna | 368 | 363.5 | no | 2026-10-26 | ok |
| Terapia Chilensis | Radio Duna | 354 | 179.4 | no | 2026-10-26 | ok |
| El Primer Café | Cooperativa | 183 | 207.9 | yes | 2026-10-26 | ok |
| Panelistas de Pauta de Análisis | Radio Pauta | 599 | 209.8 | no | 2026-10-26 | ok |
| Polos Opuestos | El Conquistador FM | 1000 (cap) | 699.0 | yes | 2026-10-26 | ok |
| Comentarios de Tomás Mosciatti | AeseGATO (iVoox) | 380 | 242.0 | no | 2026-07-10 | third-party re-upload, stale; no official feed found |
| Animales Políticos | Casa Editorial Lo Que Importa | 60 | 72.3 | no | 2026-10-26 | ok |
| Lo Que Importa | Casa Editorial Lo Que Importa | 102 | 107.3 | no | 2026-10-21 | ok |
| Sin Llorar | The Clinic | 60 | 47.6 | no | 2026-10-21 | ok |
| Cómo te lo explico | - | - | - | - | - | not in Apple directory; the match found is a different show (Javier Jiménez, 2018-2020) |
| El Villegas: Actualidad y esas cosas | Fernando Villegas | 600 | 512.1 | no | 2026-10-27 | ok |
| Democracia en LSD | Democracia en LSD | 343 | 328.4 | no | 2026-10-21 | ok |
| CIPER en tus Oídos | CIPER | 85 | 19.0 | no | 2026-09-30 | ok |
| Expediente Mónica | CIPER Chile | 13 | 11.3 | no | 2026-10-20 | ok |
| El Shot | Juan Manuel Astorga | 273 | 97.0 | no | 2026-10-26 | ok |
| Contracorriente | - | - | - | - | - | not in Apple directory; the match found is a different show (Alboan) |
| Entender es Todo | The Clinic | 32 | 20.8 | no | 2026-10-22 | ok |
| A mí nunca me han encuestado | Paulina Valenzuela y Sergio Toro | 50 | 32.6 | no | 2026-10-15 | ok |
| Con Diálogo Es Distinto | Con Diálogo Es Distinto | 132 | 115.7 | no | 2026-10-25 | ok |

## Findings

- 20 of 23 have a working official RSS feed.
- 3 have none found: Cómo te lo explico (confirmed to exist by a 13.cl interview, platform unknown), Contracorriente (no trace found under The Clinic or Horizontal), Comentarios de Tomás Mosciatti (Bío Bío lists the program; only a third-party iVoox re-upload has a feed). These may be YouTube or Spotify only, which reopens the RSS-only source decision for them.
- Name search needs a publisher check: two of the 23 matched an unrelated show with the same title, so the exe must show the match for confirmation, as decided.
- The three Tele 13 Radio shows alone hold about 3,950 hours of back catalogue.

## Sources

- https://www.13.cl/c/programas/only-comparini/capitulos/monica-rincon-recordo-el-ataque-de-risa-que-sufrio-en-un
- https://www.biobiochile.cl/biobiotv/programacion
