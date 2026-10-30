import argparse
import csv
import difflib
import json
import random
import re
import sys
from pathlib import Path

CONTEXT_WORDS = 8
SHOWN_WORDS = 30
LONG_HUNK = 10


def words(transcript):
    raw, plain, starts = [], [], []
    for segment in json.loads(transcript.read_text(encoding="utf-8"))["segments"]:
        for token in segment["text"].split():
            folded = re.sub(r"[^\w]", "", token.lower())
            if folded:
                raw.append(token)
                plain.append(folded)
                starts.append(segment["start_s"])
    return raw, plain, starts


def shown(tokens):
    return " ".join(tokens[:SHOWN_WORDS]) + (" ..." if len(tokens) > SHOWN_WORDS else "") or "(nothing)"


def main():
    parser = argparse.ArgumentParser(description="Word agreement between two transcript sets of the same episodes (case and punctuation folded, accents kept), and a blinded sample of the places they differ for judging from text")
    parser.add_argument("first", type=Path, help="run folder of the first setting, such as work/speed-week/beam5")
    parser.add_argument("second", type=Path)
    parser.add_argument("out", type=Path)
    parser.add_argument("--sample", type=int, default=60)
    parser.add_argument("--seed", type=int, default=20261030)
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    args.out.mkdir(parents=True, exist_ok=True)
    per_episode, hunks = [], []
    for transcript in sorted(args.first.glob("*/*/transcript.json")):
        folder = transcript.parent.relative_to(args.first)
        other = args.second / folder / "transcript.json"
        if not other.is_file():
            continue
        a_raw, a_plain, a_starts = words(transcript)
        b_raw, b_plain, _ = words(other)
        edits = long_edits = 0
        for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a_plain, b_plain, autojunk=False).get_opcodes():
            if tag == "equal":
                continue
            size = max(i2 - i1, j2 - j1)
            edits += size
            long_edits += size if size > LONG_HUNK else 0
            hunks.append({"folder": str(folder).replace("\\", "/"), "start_s": a_starts[min(i1, len(a_starts) - 1)], "before": a_raw[max(0, i1 - CONTEXT_WORDS):i1], "first": a_raw[i1:i2], "second": b_raw[j1:j2], "after": a_raw[i2:i2 + CONTEXT_WORDS], "size": size})
        per_episode.append([str(folder).replace("\\", "/"), len(a_plain), len(b_plain), edits, long_edits, f"{100 * edits / len(a_plain):.2f}"])
        print(f"{per_episode[-1][-1]:>6} %  {folder}", flush=True)
    with open(args.out / "per-episode.tsv", "w", encoding="utf-8", newline="") as handle:
        csv.writer(handle, delimiter="\t").writerows([["folder", "first_words", "second_words", "differing_words", "in_hunks_over_10", "differing_pct"], *per_episode])
    total_words = sum(row[1] for row in per_episode)
    total_edits = sum(row[3] for row in per_episode)
    total_long = sum(row[4] for row in per_episode)
    print(f"episodes={len(per_episode)} first_words={total_words} differing={total_edits} ({100 * total_edits / total_words:.2f} %) hunks={len(hunks)} words_in_hunks_over_{LONG_HUNK}={total_long} hunks_over_{LONG_HUNK}={sum(h['size'] > LONG_HUNK for h in hunks)}")
    chooser = random.Random(args.seed)
    picked = chooser.sample(hunks, min(args.sample, len(hunks)))
    sheet, key = [], []
    for number, hunk in enumerate(picked, 1):
        swap = chooser.random() < 0.5
        a, b = (hunk["second"], hunk["first"]) if swap else (hunk["first"], hunk["second"])
        sheet.append([number, " ".join(hunk["before"]), shown(a), shown(b), " ".join(hunk["after"])])
        key.append([number, hunk["folder"], f"{hunk['start_s']:.2f}", "second" if swap else "first", hunk["size"]])
    with open(args.out / "sheet.tsv", "w", encoding="utf-8", newline="") as handle:
        csv.writer(handle, delimiter="\t").writerows([["number", "before", "A", "B", "after"], *sheet])
    with open(args.out / "key.tsv", "w", encoding="utf-8", newline="") as handle:
        csv.writer(handle, delimiter="\t").writerows([["number", "folder", "start_s", "A_is", "size"], *key])


if __name__ == "__main__":
    main()
