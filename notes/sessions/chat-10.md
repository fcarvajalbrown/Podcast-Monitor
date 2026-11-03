# Chat 10

Done:
- Cost study made RSS-only: SUBTEL and live-radio capture removed everywhere, `subtel_stations.py` deleted, GPU and API comparison recomputed on RSS hours (`docs/research/api-costs.md`; SYNCOAI studies 04 and 05 in `credentials-synco`).
- Per-region budget sample: 8 web searches per region, 146 candidates, feed check, 14 live regional shows in 7 regions, national shows excluded, Felipe kept all 14 (`docs/research/regional-feeds.md`, `tools/region_discovery/check_candidates.py`, `cost_regions.py`).
- Costed on OpenAI, Qwen and Kimi at low, medium and high tiers: per region, per show and per average episode. Stated everywhere as a budget sample, not the monitoring selection.
- Kimi k3 (3.00 / 15.00) and k3.1-pro (2.00 / 10.00) prices supplied by Felipe from a search-engine summary, in `tools/token_cost/prices.json`.
- Current IONOS plan is the basic one: S+ at USD 6, so L+ adds 19 and XL+ adds 41.
- SYNCOAI changes in PR #1 on Forgejo, branch `docs/budget-per-region`; PDF re-rendered with a PENDIENTE watermark (`felipe_skill_voice/scripts/watermark_pdf.py`).
- Rule 4 added to `CLAUDE.md`: RSS only, any show-list genre, national shows never count for a region.

Flags opened:
- PR #1 waiting for Felipe's merge.
- Kimi prices to check against the console.
- IONOS invoice price: measured VPS (2 cores, 1.8 GiB, 89 GB) does not match the S+ listing.
- Inspiron 15 Gaming 5577 instead of a paid VPS, still to analyse.

Flags closed:
- Radio-stream contamination of the cost study.
- Per-region cost and the Kimi and IONOS inputs.
