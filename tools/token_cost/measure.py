import argparse
import json
import os
import re
import statistics
import sys

MAX_CHARS = 120_000

INSTRUCTIONS = (
    "You are reading part {part} of {count} of a Spanish-language podcast transcript, from {start} to {end}. "
    "Every line starts with its time as [mm:ss].\n"
    "\n"
    "Return:\n"
    "- summary: one paragraph in Spanish on what the speakers argued in this part: for each subject, the position "
    "each speaker took and the conclusion they reached, naming them. Write \"X sostuvo que la medida no resolvera "
    "el problema\", not \"hablaron de la medida\".Report the speakers' views as theirs, without adding your own "
    "judgement.\n"
    "- topics: every topic discussed, with a short Spanish name and the [mm:ss] start and end of each stretch where "
    "it is discussed. A topic that comes back later gets one entry per stretch, with the same name.\n"
    "- people: every person named in this part, spelled as in the names list below when they appear in it.\n"
    "- quotes: up to five short passages worth quoting, each copied word for word from the transcript lines, 8 to 40 "
    "words, with the mm:ss of the line where it starts. Do not correct the transcript's spelling inside a quote.\n"
    "\n"
    "The transcript comes from speech recognition and misspells names. The correct spellings are: {names}. Use them "
    "in the summary, topics and people even where the transcript has them wrong.\n"
    "\n"
    "Transcript:\n"
)


def clock(seconds):
    whole = max(int(seconds), 0)
    return f"{whole // 60:02}:{whole % 60:02}"


def glossary_names(path):
    if not path or not os.path.exists(path):
        return ""
    names = []
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        names.append(line.split("=")[0].strip())
    return ", ".join(names)


def transcript_chars(path):
    data = json.load(open(path, encoding="utf-8"))
    total = 0
    lines = 0
    for segment in data.get("segments", []):
        text = segment.get("text", "").strip()
        if not text:
            continue
        total += len(f"[{clock(segment.get('start_s', 0.0))}] {text}\n")
        lines += 1
    return total, lines


def plain_text(value):
    return re.sub(r"<[^>]+>", " ", value or "")


def reply_chars(path):
    if not os.path.exists(path):
        return 0, 0
    data = json.load(open(path, encoding="utf-8"))
    chunks = data.get("chunks", [])
    return len(json.dumps(chunks, ensure_ascii=False)), len(chunks)


def spread(values):
    if not values:
        return {}
    ordered = sorted(values)
    pick = lambda share: ordered[min(len(ordered) - 1, int(share * len(ordered)))]
    return {
        "min": ordered[0],
        "p10": pick(0.10),
        "median": statistics.median(ordered),
        "p90": pick(0.90),
        "max": ordered[-1],
        "mean": statistics.mean(ordered),
    }


def episodes(runs, overhead_names):
    found = []
    for show in sorted(os.listdir(runs)):
        show_dir = os.path.join(runs, show)
        if not os.path.isdir(show_dir):
            continue
        for name in sorted(os.listdir(show_dir)):
            if name.endswith(".bak-weekly"):
                continue
            folder = os.path.join(show_dir, name)
            fixed = os.path.join(folder, "transcript.fixed.json")
            meta = os.path.join(folder, "episode.json")
            if not (os.path.isdir(folder) and os.path.exists(fixed) and os.path.exists(meta)):
                continue
            info = json.load(open(meta, encoding="utf-8"))
            body, lines = transcript_chars(fixed)
            chunks = max(1, -(-body // MAX_CHARS))
            overhead = len(
                INSTRUCTIONS.format(part=1, count=chunks, start="00:00", end="00:00", names=overhead_names)
            )
            out_chars, reply_parts = reply_chars(os.path.join(folder, "summary.json"))
            description = plain_text(info.get("description", ""))
            found.append(
                {
                    "show": show,
                    "episode": name,
                    "published": info.get("published", ""),
                    "duration_s": info.get("duration_s", 0),
                    "transcript_chars": body,
                    "transcript_lines": lines,
                    "chunks": chunks,
                    "prompt_overhead_chars": overhead * chunks,
                    "input_chars": body + overhead * chunks,
                    "output_chars": out_chars,
                    "reply_parts": reply_parts,
                    "metadata_chars": len(info.get("title", "")) + len(description),
                }
            )
    return found


def main():
    parser = argparse.ArgumentParser(description="Measure what the summarizer sends and receives per episode")
    parser.add_argument("runs", help="the runs directory inside the app-data folder")
    parser.add_argument("--glossary", default=None)
    parser.add_argument("--since", default=None, help="keep episodes published on or after this ISO date")
    parser.add_argument("--out", default=None)
    args = parser.parse_args()

    names = glossary_names(args.glossary)
    rows = episodes(args.runs, names)
    if args.since:
        rows = [row for row in rows if row["published"][:10] >= args.since]
    if not rows:
        sys.exit("no episodes with a fixed transcript found")

    totals = {
        "episodes": len(rows),
        "shows": len({row["show"] for row in rows}),
        "audio_hours": sum(row["duration_s"] for row in rows) / 3600.0,
        "input_chars": sum(row["input_chars"] for row in rows),
        "output_chars": sum(row["output_chars"] for row in rows),
        "metadata_chars": sum(row["metadata_chars"] for row in rows),
        "chunks": sum(row["chunks"] for row in rows),
        "episodes_with_reply": sum(1 for row in rows if row["output_chars"] > 0),
    }
    report = {
        "glossary_names_chars": len(names),
        "max_chars_per_chunk": MAX_CHARS,
        "totals": totals,
        "per_episode": {
            "duration_s": spread([row["duration_s"] for row in rows]),
            "input_chars": spread([row["input_chars"] for row in rows]),
            "output_chars": spread([row["output_chars"] for row in rows if row["output_chars"] > 0]),
            "metadata_chars": spread([row["metadata_chars"] for row in rows]),
        },
        "episodes": rows,
    }
    text = json.dumps(report, ensure_ascii=False, indent=2)
    if args.out:
        open(args.out, "w", encoding="utf-8").write(text)
    summary = {k: v for k, v in report.items() if k != "episodes"}
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
