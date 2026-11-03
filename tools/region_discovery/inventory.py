import argparse
import json

ACCENTS = (("a", "á"), ("e", "é"), ("i", "í"), ("o", "ó"), ("u", "úü"), ("n", "ñ"))


def load(path):
    return json.loads(open(path, encoding="utf-8").read())


def fold(text):
    text = (text or "").lower()
    for bare, marked in ACCENTS:
        for char in marked:
            text = text.replace(char, bare)
    return text


def relevant_genre(row, allowed):
    folded = {fold(genre) for genre in allowed}
    return any(fold(genre) in folded for genre in row.get("genres", []))


def funnel(rows, country, min_per_week, allowed, genre_label):
    levels = []
    stage = rows
    levels.append(("discovered in the directory", stage))
    stage = [row for row in stage if row["reachable"]]
    levels.append(("feed reachable and parsed", stage))
    stage = [row for row in stage if row["episodes_with_audio"] > 0]
    levels.append(("carries audio enclosures", stage))
    stage = [row for row in stage if row["live"]]
    levels.append(("published at least once in the window", stage))
    stage = [row for row in stage if row["episodes_per_week"] >= min_per_week]
    levels.append(("publishes at least %s episodes a week" % min_per_week, stage))
    stage = [row for row in stage if row["country"] == country]
    levels.append(("listed in the %s storefront" % country, stage))
    stage = [row for row in stage if relevant_genre(row, allowed)]
    levels.append(("%s genre set" % genre_label, stage))
    stage = [row for row in stage if row["text_regions"]]
    levels.append(("names a region in its own title, publisher or description", stage))
    return levels


def totals(rows):
    return {
        "sources": len(rows),
        "episodes_per_week": round(sum(row["episodes_per_week"] for row in rows), 1),
        "audio_hours_per_week": round(
            sum(row["episodes_per_week"] * row["mean_duration_s"] / 3600.0 for row in rows if row["mean_duration_s"] > 0), 1
        ),
        "mean_duration_min": round(
            sum(row["mean_duration_s"] for row in rows if row["mean_duration_s"] > 0)
            / max(1, sum(1 for row in rows if row["mean_duration_s"] > 0))
            / 60.0,
            1,
        ),
    }


def by_region(rows, spec):
    names = {region["id"]: region["name"] for region in spec["regions"]}
    out = {}
    for region in spec["regions"]:
        found = [row for row in rows if region["id"] in row["text_regions"]]
        out[region["id"]] = {"name": names[region["id"]], **totals(found)}
    unplaced = [row for row in rows if not row["text_regions"]]
    out["none"] = {"name": "no region named in its own metadata", **totals(unplaced)}
    return out


def main():
    parser = argparse.ArgumentParser(description="Build the qualification funnel and the per-region inventory from a discovery report")
    parser.add_argument("--report", required=True)
    parser.add_argument("--spec", required=True)
    parser.add_argument("--country", default="CHL")
    parser.add_argument("--min-per-week", type=float, default=0.25)
    parser.add_argument("--genres", default="strict")
    parser.add_argument("--out", default=None)
    args = parser.parse_args()

    report = load(args.report)
    spec = load(args.spec)
    rows = report["sources"]
    allowed = spec["genre_sets"][args.genres]
    levels = funnel(rows, args.country, args.min_per_week, allowed, args.genres)

    result = {
        "window_days": report["window_days"],
        "country_filter": args.country,
        "genre_set": args.genres,
        "genres_allowed": allowed,
        "min_episodes_per_week": args.min_per_week,
        "funnel": [{"level": name, **totals(stage)} for name, stage in levels],
        "qualified_by_region": by_region(levels[-1][1], spec),
        "live_by_region": by_region([row for row in rows if row["live"]], spec),
    }
    text = json.dumps(result, ensure_ascii=False, indent=2)
    if args.out:
        open(args.out, "w", encoding="utf-8").write(text)
    print(text)


if __name__ == "__main__":
    main()
