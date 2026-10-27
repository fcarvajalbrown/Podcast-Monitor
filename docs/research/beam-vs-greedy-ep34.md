# Beam 5 against greedy: Ep. 34 on the RTX 2050

Episode: "Ep. 34 -La crisis de los decretos - Animales Políticos - Segunda Temporada", 4,685 s. Model `ggml-large-v3-turbo.bin`, glossary `tools/stt_compare/glossary-ap-ep34.txt`, fix threshold 0.85, Vulkan0 on the RTX 2050 (4 GB), laptop idle, Firefox closed. Each run in its own folder, so both decoded from scratch. Built from commit with `--greedy` and `--beam-size N` on `run`.

This is the RTX 2050, not the RTX 3060 PC. The 3060 measurement asked for is still not taken.

## Speed

| Decoding | compute_s | Real-time factor |
|---|---|---|
| Beam 5 (`--beam-size 5`, default) | 395 | 11.9x |
| Greedy (`--greedy`) | 324 | 14.5x |

Greedy is 71 s (18 %) faster. The earlier beam-5 run in `pipeline-ep34-check.md` took 424 s; this one 395 s on the same machine.

## Names

`count_names.py`, exact spellings, raw and after the Rust fix:

| Name | Beam raw | Beam fixed | Greedy raw | Greedy fixed |
|---|---|---|---|---|
| Kast | 10 | 17 | 7 | 16 |
| Boric | 9 | 9 | 9 | 9 |
| Squella | 0 | 0 | 0 | 4 |
| Bellolio | 0 | 2 | 0 | 2 |
| Contardo | 1 | 2 | 1 | 2 |
| Mamdani | 0 | 1 | 0 | 1 |
| Parisi | 3 | 3 | 0 | 4 |
| Rabat | 4 | 4 | 3 | 3 |
| Bachelet | 1 | 1 | 2 | 2 |

Target after the fix: Kast at least 16, Boric at least 7. Both runs meet it (beam 17 and 9, greedy 16 and 9). Greedy heard Kast 7 times raw against 10, so it leans harder on the fix; Squella and Parisi appear only through the fix in greedy.

Not compared: whether the words around the names are right. Name counts do not measure overall accuracy.
