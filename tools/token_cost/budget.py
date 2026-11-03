import argparse
import json

WEEKS_PER_MONTH = 52.0 / 12.0


def load(path):
    return json.loads(open(path, encoding="utf-8").read())


def tokens(chars, cpt):
    return chars / cpt


def text_cost(model, input_tokens, output_tokens, cached_tokens=0.0):
    if model.get("input") is None or model.get("output") is None:
        return None
    fresh = (input_tokens - cached_tokens) / 1_000_000 * model["input"]
    cached = cached_tokens / 1_000_000 * (model.get("cached_input") or 0.0)
    out = output_tokens / 1_000_000 * model["output"]
    return fresh + cached + out


def episode_volumes(corpus, scenarios, scenario, cpt):
    per = corpus["per_episode"]
    overhead = scenarios["instruction_overhead_chars"]["value"]
    meta_out = scenarios["metadata_output_chars"]["value"]
    share = scenario["transcript_share"]

    transcript_in = tokens(per["input_chars"]["mean"], cpt)
    transcript_out = tokens(per["output_chars"]["mean"], cpt)
    metadata_in = tokens(per["metadata_chars"]["mean"] + overhead, cpt)
    metadata_out = tokens(meta_out, cpt)

    analysis_in = share * transcript_in + (1 - share) * metadata_in
    analysis_out = share * transcript_out + (1 - share) * metadata_out

    passes = scenario["providers_per_episode"] * (1.0 + scenario["verification_share"])
    retry = 1.0 + scenario["retry_share"]
    return {
        "analysis_input_tokens": analysis_in * passes * retry,
        "analysis_output_tokens": analysis_out * passes * retry,
        "audio_minutes": per["duration_s"]["mean"] / 60.0 * share,
        "analysis_output_chars": analysis_out * cpt,
    }


def report_volumes(corpus, scenarios, episodes_per_week, analysis_output_chars, cpt):
    spec = scenarios["reports"]
    out = scenarios["report_output_chars"]
    regions = spec["regions"]
    overhead = scenarios["instruction_overhead_chars"]["value"]

    daily_units = (regions if spec["daily_per_region"] else 0) + (1 if spec["daily_national"] else 0)
    weekly_units = (regions if spec["weekly_per_region"] else 0) + (1 if spec["weekly_national"] else 0)
    cross = 1 if spec["weekly_cross_regional"] else 0

    episodes_per_day = episodes_per_week / spec["days_per_week"]
    daily_input_chars = daily_units * (episodes_per_day / max(daily_units, 1) * analysis_output_chars + overhead)
    daily_output_chars = daily_units * out["per_unit_synthesis"]

    weekly_input_chars = weekly_units * (episodes_per_week / max(weekly_units, 1) * analysis_output_chars + overhead)
    weekly_output_chars = weekly_units * out["per_unit_synthesis"] + cross * out["cross_cutting"]
    weekly_input_chars += cross * (weekly_units * out["per_unit_synthesis"] + overhead)

    input_chars = daily_input_chars * spec["days_per_week"] + weekly_input_chars
    output_chars = daily_output_chars * spec["days_per_week"] + weekly_output_chars
    return {
        "report_input_tokens": tokens(input_chars, cpt),
        "report_output_tokens": tokens(output_chars, cpt),
        "daily_reports_per_week": daily_units * spec["days_per_week"],
        "weekly_reports": weekly_units + cross,
    }


def asr_cost(prices, episodes_per_week, audio_minutes_per_episode, model_id):
    for model in prices["asr_models"]:
        if model["id"] == model_id:
            return episodes_per_week * audio_minutes_per_episode * model["asr_usd_per_minute"], model
    raise SystemExit("unknown asr model " + model_id)


def run(corpus, prices, scenarios, episodes_per_week, asr_id, cpt_name):
    cpt = scenarios["chars_per_token"][cpt_name]
    clp = prices["fx"]["clp_per_usd"]
    rows = []
    for scenario in scenarios["scenarios"]:
        volumes = episode_volumes(corpus, scenarios, scenario, cpt)
        reports = report_volumes(corpus, scenarios, episodes_per_week, volumes["analysis_output_chars"], cpt)
        audio = volumes["audio_minutes"]
        transcription, asr_model = (0.0, None)
        if scenario["asr"]:
            transcription, asr_model = asr_cost(prices, episodes_per_week, audio, asr_id)

        analysis_in = volumes["analysis_input_tokens"] * episodes_per_week
        analysis_out = volumes["analysis_output_tokens"] * episodes_per_week

        for model in prices["text_models"]:
            analysis = text_cost(model, analysis_in, analysis_out)
            report = text_cost(model, reports["report_input_tokens"], reports["report_output_tokens"])
            if analysis is None or report is None:
                rows.append(
                    {
                        "scenario": scenario["id"],
                        "provider": model["provider"],
                        "model": model["id"],
                        "tier": model["tier"],
                        "weekly_usd": None,
                        "note": "no first-party price established",
                    }
                )
                continue
            weekly = analysis + report + transcription
            rows.append(
                {
                    "scenario": scenario["id"],
                    "provider": model["provider"],
                    "model": model["id"],
                    "tier": model["tier"],
                    "confirmed_price": model.get("confirmed", False),
                    "analysis_usd": round(analysis, 2),
                    "reports_usd": round(report, 2),
                    "transcription_usd": round(transcription, 2),
                    "transcription_model": asr_model["id"] if asr_model else None,
                    "weekly_usd": round(weekly, 2),
                    "monthly_usd": round(weekly * WEEKS_PER_MONTH, 2),
                    "monthly_clp": round(weekly * WEEKS_PER_MONTH * clp),
                    "per_episode_usd": round(weekly / max(episodes_per_week, 1), 4),
                    "analysis_input_tokens_weekly": round(analysis_in),
                    "analysis_output_tokens_weekly": round(analysis_out),
                }
            )
    return rows


def sensitivity(corpus, prices, scenarios, episodes_per_week, asr_id, model_id, scenario_id):
    cpt = scenarios["chars_per_token"]["central"]
    model = next(m for m in prices["text_models"] if m["id"] == model_id)
    scenario = next(s for s in scenarios["scenarios"] if s["id"] == scenario_id)
    base = episode_volumes(corpus, scenarios, scenario, cpt)
    reports = report_volumes(corpus, scenarios, episodes_per_week, base["analysis_output_chars"], cpt)
    transcription = 0.0
    if scenario["asr"]:
        transcription, _ = asr_cost(prices, episodes_per_week, base["audio_minutes"], asr_id)
    out = []
    for in_mult in scenarios["sensitivity_multipliers"]:
        for out_mult in scenarios["sensitivity_multipliers"]:
            analysis = text_cost(
                model,
                base["analysis_input_tokens"] * episodes_per_week * in_mult,
                base["analysis_output_tokens"] * episodes_per_week * out_mult,
            )
            report = text_cost(
                model, reports["report_input_tokens"] * in_mult, reports["report_output_tokens"] * out_mult
            )
            weekly = analysis + report + transcription
            out.append(
                {
                    "input_multiplier": in_mult,
                    "output_multiplier": out_mult,
                    "weekly_usd": round(weekly, 2),
                    "monthly_usd": round(weekly * WEEKS_PER_MONTH, 2),
                }
            )
    return out


def cpt_spread(corpus, prices, scenarios, episodes_per_week, asr_id, model_id, scenario_id):
    rows = []
    for name in ("low", "central", "high"):
        found = [
            row
            for row in run(corpus, prices, scenarios, episodes_per_week, asr_id, name)
            if row["model"] == model_id and row["scenario"] == scenario_id
        ]
        rows.append({"chars_per_token": scenarios["chars_per_token"][name], "named": name, "weekly_usd": found[0]["weekly_usd"], "monthly_usd": found[0]["monthly_usd"]})
    return rows


def inventory_from(path, min_episodes_per_week):
    report = load(path)
    live = [row for row in report["sources"] if row["live"] and row["episodes_per_week"] >= min_episodes_per_week]
    return {
        "sources_live": len(live),
        "episodes_per_week": round(sum(row["episodes_per_week"] for row in live), 1),
        "mean_duration_s": int(
            sum(row["mean_duration_s"] for row in live if row["mean_duration_s"] > 0)
            / max(1, sum(1 for row in live if row["mean_duration_s"] > 0))
        ),
    }


def main():
    parser = argparse.ArgumentParser(description="Cost the podcast monitoring workload against provider price tables")
    parser.add_argument("--corpus", required=True)
    parser.add_argument("--prices", required=True)
    parser.add_argument("--scenarios", required=True)
    parser.add_argument("--inventory", default=None)
    parser.add_argument("--episodes-per-week", type=float, default=None)
    parser.add_argument("--min-episodes-per-week", type=float, default=0.25)
    parser.add_argument("--asr", default="qwen3-asr-flash")
    parser.add_argument("--sensitivity-model", default="gpt-6-luna")
    parser.add_argument("--sensitivity-scenario", default="medium")
    parser.add_argument("--regions", type=int, default=None)
    parser.add_argument("--out", default=None)
    args = parser.parse_args()

    corpus = load(args.corpus)
    prices = load(args.prices)
    scenarios = load(args.scenarios)
    if args.regions is not None:
        scenarios["reports"]["regions"] = args.regions
        scenarios["reports"]["daily_per_region"] = args.regions > 0
        scenarios["reports"]["weekly_per_region"] = args.regions > 0
        scenarios["reports"]["weekly_cross_regional"] = args.regions > 1

    inventory = None
    if args.inventory:
        inventory = inventory_from(args.inventory, args.min_episodes_per_week)
    episodes = args.episodes_per_week if args.episodes_per_week else (inventory or {}).get("episodes_per_week")
    if not episodes:
        raise SystemExit("give --episodes-per-week or --inventory")

    result = {
        "episodes_per_week": episodes,
        "inventory": inventory,
        "measured_corpus": corpus["totals"],
        "chars_per_token": scenarios["chars_per_token"],
        "fx": prices["fx"],
        "rows": run(corpus, prices, scenarios, episodes, args.asr, "central"),
        "chars_per_token_spread": cpt_spread(
            corpus, prices, scenarios, episodes, args.asr, args.sensitivity_model, args.sensitivity_scenario
        ),
        "sensitivity": {
            "model": args.sensitivity_model,
            "scenario": args.sensitivity_scenario,
            "grid": sensitivity(
                corpus, prices, scenarios, episodes, args.asr, args.sensitivity_model, args.sensitivity_scenario
            ),
        },
    }
    text = json.dumps(result, ensure_ascii=False, indent=2)
    if args.out:
        open(args.out, "w", encoding="utf-8").write(text)
    print(text)


if __name__ == "__main__":
    main()
