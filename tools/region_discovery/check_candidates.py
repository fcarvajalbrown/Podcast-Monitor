import argparse
import json
import re
import sys
import time
import unicodedata
import urllib.error
from datetime import date, timedelta
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "feed_check"))
from check_feeds import apple_search, summarize_feed

PAUSE_SECONDS = 4.0
MATCH_THRESHOLD = 0.6
STOPWORDS = {"el", "la", "los", "las", "de", "del", "en", "y", "a", "podcast", "radio", "the", "fm", "con"}


def tokens(text):
    plain = unicodedata.normalize("NFKD", text or "").encode("ascii", "ignore").decode().lower()
    return {word for word in re.findall(r"[a-z0-9]+", plain) if word not in STOPWORDS}


def name_agreement(candidate, found):
    wanted, got = tokens(candidate), tokens(found)
    if not wanted or not got:
        return 0.0
    return round(len(wanted & got) / len(wanted | got), 2)


def polite_search(name):
    for attempt in range(5):
        time.sleep(PAUSE_SECONDS * (attempt + 1))
        try:
            return apple_search(name)
        except urllib.error.HTTPError as error:
            if error.code not in (403, 429):
                raise
    return apple_search(name)


def best_match(name, publisher):
    matches = polite_search(name)
    if not matches:
        return None, 0.0
    scored = []
    for match in matches:
        score = name_agreement(name, match.get("collectionName", ""))
        if publisher and tokens(publisher) & tokens(match.get("artistName", "")):
            score += 0.25
        scored.append((score, match))
    scored.sort(key=lambda pair: pair[0], reverse=True)
    score, match = scored[0]
    return match, round(min(score, 1.0), 2)


def check_candidate(candidate, since):
    row = {"name": candidate["name"], "publisher_seen": candidate.get("publisher", "")}
    try:
        match, agreement = best_match(candidate["name"], candidate.get("publisher", ""))
    except Exception as error:
        row["status"] = f"directory failed: {error}"
        return row
    if match is None:
        row["status"] = "not in Apple directory"
        return row
    row.update({"apple_name": match.get("collectionName"), "apple_publisher": match.get("artistName"), "feed": match.get("feedUrl"), "name_agreement": agreement})
    if agreement < MATCH_THRESHOLD:
        row["status"] = "no matching show in Apple directory"
        return row
    if not row["feed"]:
        row["status"] = "no feed URL"
        return row
    try:
        row.update(summarize_feed(row["feed"], since))
        row["status"] = "ok"
    except Exception as error:
        row["status"] = f"feed failed: {error}"
    return row


def main():
    parser = argparse.ArgumentParser(description="Look up every web-search candidate in the Apple directory and measure its RSS feed, region by region, resuming from saved results")
    parser.add_argument("--candidates", required=True, help="folder of per-region candidate JSON files")
    parser.add_argument("--out", required=True, help="results JSON, rewritten after each region")
    parser.add_argument("--today", type=date.fromisoformat, default=date.today())
    parser.add_argument("--window-days", type=int, default=60)
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")

    since = args.today - timedelta(days=args.window_days)
    out = Path(args.out)
    results = json.loads(out.read_text(encoding="utf-8")) if out.exists() else {"window_days": args.window_days, "since": since.isoformat(), "regions": {}}

    for path in sorted(Path(args.candidates).glob("*.json")):
        spec = json.loads(path.read_text(encoding="utf-8"))
        region = spec["region"]
        saved = {row["name"]: row for row in results["regions"].get(region, [])}
        if saved and not any(row["status"].startswith("directory failed") for row in saved.values()):
            continue
        rows = []
        for candidate in spec["candidates"]:
            previous = saved.get(candidate["name"])
            if previous and not previous["status"].startswith("directory failed"):
                rows.append(previous)
            else:
                rows.append(check_candidate(candidate, since))
        results["regions"][region] = rows
        out.write_text(json.dumps(results, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        live = sum(1 for row in rows if row.get("since_episodes"))
        print(f"{region}: {len(rows)} candidates, {live} published in the last {args.window_days} days")


if __name__ == "__main__":
    main()
