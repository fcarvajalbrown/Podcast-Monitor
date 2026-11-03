# Which regional podcasts publish today

Question: of the RSS podcasts found by eight web searches per region, how many still publish, and
how much audio do they put out each week?

Checked 3 November 2026 with `tools/region_discovery/check_candidates.py` over the 146 candidates
in `tools/region_discovery/websearch/`. Each candidate was looked up in Apple's Chilean directory,
matched by name (word overlap of at least 0.6, plus 0.25 when the publisher agrees), and its feed
read for episodes published since 4 September, a 60-day window. Results:
`tools/region_discovery/websearch-feeds.json`.

## Result

Of 146 candidates, 22 published at least once in the 60 days. The rest either stopped publishing
(their newest episode dates from 2017 to mid 2026) or have no show of that name in the directory.
Two shows rejected by the name rule were real matches and are counted here: Radio UdeC, listed as
"Radio UdeC | Programas", and Sin Escamas, listed with a long subtitle.

| Region | Show | Publisher | Episodes in 60 days | Audio hours in 60 days |
| --- | --- | --- | --- | --- |
| Biobío | Radio UdeC | Radio Universidad de Concepción | 150 | 103.8 |
| Biobío | Audiocuentos | Marta RD (Cañete) | 11 | 1.9 |
| Los Lagos | PL Podcast | Paislobo Prensa | 114 | 28.2 |
| Los Lagos | Sin Escamas | Salmonexpert | 1 | 0.8 |
| Magallanes | Voces de Alianza | P. Gonzalo Illanes | 2 | 1.0 |
| Metropolitana | Santiago Adicto | Radio Duna | 40 | 42.5 |
| Metropolitana | Traficantes de Cultura | Traficantes de Cultura | 22 | 12.8 |
| Metropolitana | Caldo de cultivo | Centro Cultural de España en Santiago | 1 | 0.8 |
| Ñuble | Radio LA FONTANA | La Fontana | 5 | 1.2 |
| Ñuble | Nación X | Frente Astral | 1 | 0.3 |
| O'Higgins | Aquí lo conversamos | Diario El Marino | 16 | 19.4 |
| O'Higgins | FUSAT Contigo | Hospital Clínico FUSAT | 7 | 1.5 |
| Valparaíso | Radio Ritoque | Radio Ritoque | 88 | 67.5 |
| Valparaíso | Programas de ChilenaFM de San Antonio | ChilenaFM San Antonio | 4 | 1.3 |

The Metropolitana search also found ten live national shows, which do not count for the region
because the national show list already covers national programming: Punto Aparte, Entender es Todo,
Una Nueva Mañana, Esa es la Idea, Radio Universidad de Chile, Primer Click, Money Talks, Con Peras y
Finanzas, Tomás Va A Morir and Expertas en Nada.

Nine regions have no live show: Arica y Parinacota, Tarapacá, Antofagasta, Atacama, Coquimbo, Maule,
La Araucanía, Los Ríos and Aysén.

## Regional fit

Felipe confirmed all 14 shows as regional on 3 November ("they are what people are talking on the
region, everything is politics"). Four were in doubt from their own feed descriptions: Voces de
Alianza, a Schoenstatt talk show on faith and national topics; Traficantes de Cultura, interviews
with authors and filmmakers; Nación X, a comics talk show; and FUSAT Contigo, a Rancagua hospital's
weekly health interviews on Radio Rancagua 99.5. A show counts for the region where the web search
found it, whatever its genre.
