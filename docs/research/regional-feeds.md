# Which regional podcasts publish today

Question: of the RSS podcasts found by eight web searches per region, how many still publish, and
how much audio do they put out each week, and what would each region's own reports cost?

This is not the final podcast selection. The shows below are a sample used only to size the budget
of SYNCOAI's daily and weekly regional reports for content creators, priced on the three API
providers (OpenAI, Qwen, Kimi) at the low, medium and high tiers of their models. Which shows are
monitored is decided separately.

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

Felipe confirmed all 14 shows as regional ("they are what people are talking on the
region, everything is politics"). Four were in doubt from their own feed descriptions: Voces de
Alianza, a Schoenstatt talk show on faith and national topics; Traficantes de Cultura, interviews
with authors and filmmakers; Nación X, a comics talk show; and FUSAT Contigo, a Rancagua hospital's
weekly health interviews on Radio Rancagua 99.5. A show counts for the region where the web search
found it, whatever its genre.

## Budget per region

USD a month for each region's own daily and weekly report, every episode transcribed with
qwen3-asr-flash and analysed by the model in the column. National reports are not included. Run with
`tools/region_discovery/cost_regions.py` (scenario `high` of `tools/token_cost/scenarios.json`);
results in `tools/region_discovery/region-costs.json`. Episodes and hours a week are the 60-day
counts divided by 60/7.

| Region | Shows | Episodes a week | Audio hours a week | OpenAI low | OpenAI medium | OpenAI high | Qwen low | Qwen medium | Qwen high | Kimi low | Kimi high |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Biobío | 2 | 18.8 | 12.3 | 8.58 | 18.06 | 112.90 | 7.86 | 11.96 | 28.13 | 39.14 | 28.60 |
| Los Lagos | 2 | 13.4 | 3.4 | 2.82 | 9.67 | 78.16 | 2.31 | 5.26 | 16.89 | 24.89 | 17.28 |
| Magallanes | 1 | 0.2 | 0.1 | 0.11 | 0.48 | 4.11 | 0.09 | 0.23 | 0.71 | 1.28 | 0.88 |
| Metropolitana | 3 | 7.3 | 6.5 | 4.42 | 8.28 | 46.92 | 4.13 | 5.79 | 12.29 | 16.87 | 12.58 |
| Ñuble | 2 | 0.7 | 0.2 | 0.18 | 0.77 | 6.71 | 0.13 | 0.37 | 1.25 | 2.09 | 1.43 |
| O'Higgins | 2 | 2.7 | 2.4 | 1.66 | 3.23 | 18.91 | 1.54 | 2.21 | 4.76 | 6.71 | 4.97 |
| Valparaíso | 2 | 10.7 | 8.0 | 5.51 | 11.04 | 66.30 | 5.10 | 7.48 | 16.84 | 23.32 | 17.18 |
| Seven regions | 14 | 53.9 | 33.0 | 23.28 | 51.53 | 334.01 | 21.16 | 33.30 | 80.87 | 114.30 | 82.92 |

The models are gpt-6-luna, gpt-6.1-terra and gpt-6-astra for OpenAI; qwen3.7-flash, qwen3.7-plus
and qwen3.8-max for Qwen; kimi-k3 and kimi-k3.1-pro for Kimi, which has no medium tier in the price
table. Kimi's prices (k3 at 3.00 and 15.00, k3.1-pro at 2.00 and 10.00 per million tokens) were
supplied by Felipe from a search-engine summary, since the official page renders its tables in the
browser; k3.1-pro is cheaper than k3 at these prices. Transcription is the same in every column:
7.54 a month for Biobío and 20.20 for the seven regions together, on 33.0 audio hours a week. The
nine regions with no live show cost nothing in this sample.
