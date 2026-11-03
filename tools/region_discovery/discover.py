import argparse
import json
import os
import time
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timedelta, timezone
from email.utils import parsedate_to_datetime
from xml.etree import ElementTree as ET

USER_AGENT = "PodcastMonitorRegionDiscovery/0.1"
ITUNES_NS = "{http://www.itunes.com/dtds/podcast-1.0.dtd}"
SEARCH = "https://itunes.apple.com/search"
ACCENTS = (("a", "á"), ("e", "é"), ("i", "í"), ("o", "ó"), ("u", "úü"), ("n", "ñ"))


def get(url, timeout=30):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return response.read()


def search(term, country, limit):
    query = urllib.parse.urlencode(
        {"term": term, "media": "podcast", "entity": "podcast", "country": country, "limit": limit}
    )
    payload = json.loads(get(SEARCH + "?" + query))
    return payload.get("results", [])


def fold(text):
    text = (text or "").lower()
    for bare, marked in ACCENTS:
        for char in marked:
            text = text.replace(char, bare)
    return text


def duration_seconds(text):
    if not text:
        return 0
    try:
        total = 0.0
        for part in str(text).strip().split(":"):
            total = total * 60 + float(part)
        return int(total)
    except ValueError:
        return 0


def published(item):
    raw = item.findtext("pubDate")
    if not raw:
        return None
    try:
        stamp = parsedate_to_datetime(raw)
    except (TypeError, ValueError):
        return None
    if stamp.tzinfo is None:
        stamp = stamp.replace(tzinfo=timezone.utc)
    return stamp.astimezone(timezone.utc)


def feed_health(feed_url, window_days):
    try:
        root = ET.fromstring(get(feed_url, timeout=25))
    except Exception as error:
        return {"ok": False, "error": type(error).__name__}
    channel = root.find("channel")
    if channel is None:
        return {"ok": False, "error": "NoChannel"}
    cutoff = datetime.now(timezone.utc) - timedelta(days=window_days)
    recent = []
    with_audio = 0
    for item in channel.findall("item"):
        if item.find("enclosure") is None:
            continue
        with_audio += 1
        stamp = published(item)
        if stamp and stamp >= cutoff:
            recent.append(duration_seconds(item.findtext(ITUNES_NS + "duration")))
    stated = [value for value in recent if value > 0]
    parts = [channel.findtext("title"), channel.findtext("description"), channel.findtext(ITUNES_NS + "author")]
    return {
        "ok": True,
        "episodes_with_audio": with_audio,
        "episodes_in_window": len(recent),
        "episodes_per_week": round(len(recent) * 7.0 / window_days, 2),
        "mean_duration_s": int(sum(stated) / len(stated)) if stated else 0,
        "durations_stated": len(stated),
        "text": " ".join([part for part in parts if part])[:4000],
    }


def region_hits(spec, text):
    folded = fold(text)
    hits = []
    for region in spec["regions"]:
        for needle in region["match"]:
            if fold(needle) in folded:
                hits.append(region["id"])
                break
    return hits


def term_list(spec):
    terms = []
    for region in spec["regions"]:
        for pattern in spec["term_patterns"]:
            terms.append({"term": pattern.format(place=region["term_place"]), "region": region["id"]})
    for term in spec["national_terms"]:
        terms.append({"term": term, "region": None})
    return terms


def run_searches(spec, country, limit, delay, state, path):
    for entry in term_list(spec):
        key = entry["term"]
        if key in state["searches"]:
            continue
        try:
            results = search(key, country, limit)
        except Exception as error:
            state["searches"][key] = {"region": entry["region"], "error": type(error).__name__, "count": 0}
            save(state, path)
            time.sleep(delay)
            continue
        state["searches"][key] = {"region": entry["region"], "count": len(results)}
        for result in results:
            feed = result.get("feedUrl")
            if not feed:
                continue
            found = state["candidates"].setdefault(
                feed,
                {
                    "name": result.get("collectionName", ""),
                    "publisher": result.get("artistName", ""),
                    "genres": result.get("genres", []),
                    "track_count": result.get("trackCount", 0),
                    "latest_release": result.get("releaseDate", ""),
                    "country": result.get("country", ""),
                    "terms": [],
                    "term_regions": [],
                },
            )
            if key not in found["terms"]:
                found["terms"].append(key)
            if entry["region"] and entry["region"] not in found["term_regions"]:
                found["term_regions"].append(entry["region"])
        save(state, path)
        total = len(state["candidates"])
        print("searched " + key + ": " + str(len(results)) + " results, " + str(total) + " unique feeds", flush=True)
        time.sleep(delay)


def run_feeds(state, path, window_days, workers):
    pending = [feed for feed in state["candidates"] if feed not in state["feeds"]]
    print("checking " + str(len(pending)) + " feeds", flush=True)
    done = 0
    with ThreadPoolExecutor(max_workers=workers) as pool:
        for feed, health in zip(pending, pool.map(lambda url: feed_health(url, window_days), pending)):
            state["feeds"][feed] = health
            done += 1
            if done % 25 == 0:
                save(state, path)
                print("  " + str(done) + "/" + str(len(pending)) + " feeds checked", flush=True)
    save(state, path)


def classify(spec, state):
    rows = []
    for feed, found in state["candidates"].items():
        health = state["feeds"].get(feed, {})
        text = " ".join([found["name"], found["publisher"], health.get("text", "")])
        rows.append(
            {
                "feed": feed,
                "name": found["name"],
                "publisher": found["publisher"],
                "country": found["country"],
                "genres": found.get("genres", []),
                "track_count": found.get("track_count", 0),
                "terms": found["terms"],
                "term_regions": found["term_regions"],
                "text_regions": region_hits(spec, text),
                "reachable": bool(health.get("ok")),
                "episodes_with_audio": health.get("episodes_with_audio", 0),
                "episodes_in_window": health.get("episodes_in_window", 0),
                "episodes_per_week": health.get("episodes_per_week", 0.0),
                "mean_duration_s": health.get("mean_duration_s", 0),
                "live": bool(health.get("ok")) and health.get("episodes_in_window", 0) > 0,
            }
        )
    return rows


def save(state, path):
    open(path, "w", encoding="utf-8").write(json.dumps(state, ensure_ascii=False, indent=1))


def main():
    parser = argparse.ArgumentParser(
        description="Find regional Chilean podcasts in Apple's public directory and measure their feeds"
    )
    parser.add_argument("--spec", required=True)
    parser.add_argument("--state", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--country", default="cl")
    parser.add_argument("--limit", type=int, default=200)
    parser.add_argument("--delay", type=float, default=1.5)
    parser.add_argument("--window-days", type=int, default=60)
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--stage", choices=["search", "feeds", "report", "all"], default="all")
    args = parser.parse_args()

    spec = json.loads(open(args.spec, encoding="utf-8").read())
    state = {"searches": {}, "candidates": {}, "feeds": {}}
    if os.path.exists(args.state):
        state.update(json.loads(open(args.state, encoding="utf-8").read()))

    if args.stage in ("search", "all"):
        run_searches(spec, args.country, args.limit, args.delay, state, args.state)
    if args.stage in ("feeds", "all"):
        run_feeds(state, args.state, args.window_days, args.workers)

    rows = classify(spec, state)
    report = {"window_days": args.window_days, "sources": rows}
    open(args.out, "w", encoding="utf-8").write(json.dumps(report, ensure_ascii=False, indent=1))
    reachable = sum(1 for row in rows if row["reachable"])
    live = sum(1 for row in rows if row["live"])
    print("candidates " + str(len(rows)) + ", reachable " + str(reachable) + ", live " + str(live))


if __name__ == "__main__":
    main()
