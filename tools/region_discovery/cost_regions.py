import argparse
import json
import subprocess
import sys
from pathlib import Path

BUDGET = Path(__file__).resolve().parent.parent / "token_cost" / "budget.py"


def region_load(rows, window_days):
    live = [row for row in rows if row.get("since_episodes") and not row.get("national")]
    weeks = window_days / 7
    return {
        "shows": len(live),
        "episodes_per_week": round(sum(row["since_episodes"] for row in live) / weeks, 2),
        "audio_hours_per_week": round(sum(row["since_hours"] for row in live) / weeks, 2),
    }


def run_budget(args, load):
    command = [
        sys.executable, str(BUDGET),
        "--corpus", args.corpus, "--prices", args.prices, "--scenarios", args.scenarios,
        "--episodes-per-week", str(load["episodes_per_week"]),
        "--audio-hours-per-week", str(load["audio_hours_per_week"]),
        "--regions", "1", "--no-national",
    ]
    return json.loads(subprocess.run(command, capture_output=True, text=True, encoding="utf-8", check=True).stdout)


def provider_grid(report, scenario):
    grid = {}
    for row in report["rows"]:
        if row["scenario"] != scenario:
            continue
        grid.setdefault(row["provider"], {})[row["tier"]] = {
            "model": row["model"],
            "monthly_usd": row.get("monthly_usd"),
            "confirmed_price": row.get("confirmed_price", False),
        }
    transcription = next(row["transcription_usd"] for row in report["rows"] if row["scenario"] == scenario and row.get("transcription_usd") is not None)
    return {"transcription_monthly_usd": round(transcription * 52 / 12, 2), "monthly_usd_by_provider_and_tier": grid}


def main():
    parser = argparse.ArgumentParser(description="Cost each region's own daily and weekly reports on every provider and model tier, from the live regional shows in a feed-check results file; a budget sample, not a monitoring selection")
    parser.add_argument("--feeds", required=True, help="results JSON written by check_candidates.py")
    parser.add_argument("--corpus", required=True)
    parser.add_argument("--prices", required=True)
    parser.add_argument("--scenarios", required=True)
    parser.add_argument("--scenario", default="high", help="budget scenario to report; high transcribes every episode")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")

    feeds = json.loads(Path(args.feeds).read_text(encoding="utf-8"))
    regions = []
    for region, rows in feeds["regions"].items():
        load = region_load(rows, feeds["window_days"])
        entry = {"region": region, **load}
        if load["shows"]:
            entry.update(provider_grid(run_budget(args, load), args.scenario))
        regions.append(entry)
        print(json.dumps(entry, ensure_ascii=False))
    Path(args.out).write_text(json.dumps({"scenario": args.scenario, "regions": regions}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
