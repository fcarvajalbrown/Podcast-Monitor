import argparse
import re
import statistics
import sys
from datetime import datetime
from pathlib import Path

STAMP = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+[-+]\d\d:\d\d) +\w+ +(\S+): (.*)$")
MARKS = [
    ("download", re.compile(r"^podcast_monitor::job$"), re.compile(r"^run (.+?) with ")),
    ("decode", re.compile(r"^podcast_monitor::run$"), re.compile(r"^\s*decoding ")),
    ("transcribe", re.compile(r"^podcast_monitor::transcribe$"), re.compile(r"^\s*chunk 1/")),
    ("names", re.compile(r"^podcast_monitor::run$"), re.compile(r"^\s*transcribed (\d+) s of audio")),
    ("summary", re.compile(r"^podcast_monitor::job$"), re.compile(r"^stage Summarizing")),
    ("report", re.compile(r"^podcast_monitor::job$"), re.compile(r"^stage Report")),
    ("end", re.compile(r"^podcast_monitor::job$"), re.compile(r"^run ended after")),
]
STAGES = ["download", "decode", "transcribe", "names", "summary", "report"]


def episodes(lines):
    current = None
    for line in lines:
        match = STAMP.match(line)
        if not match:
            continue
        when, target, message = datetime.fromisoformat(match[1]), match[2], match[3]
        for name, target_pattern, message_pattern in MARKS:
            hit = message_pattern.match(message) if target_pattern.match(target) else None
            if not hit:
                continue
            if name == "download":
                current = {"title": hit[1], "marks": {"download": when}}
            elif current is not None:
                current["marks"][name] = when
                if name == "names":
                    current["audio_s"] = int(hit[1])
                if name == "end":
                    yield current
                    current = None
            break


def durations(episode):
    marks = episode["marks"]
    order = STAGES + ["end"]
    result = {}
    for stage, following in zip(order, order[1:]):
        if stage in marks and following in marks:
            result[stage] = (marks[following] - marks[stage]).total_seconds()
    return result


def main():
    parser = argparse.ArgumentParser(description="Time each pipeline stage per episode from a Podcast-Monitor debug log")
    parser.add_argument("log", type=Path)
    args = parser.parse_args()
    rows = []
    for episode in episodes(args.log.read_text(encoding="utf-8", errors="replace").splitlines()):
        rows.append((episode["title"], episode.get("audio_s", 0), durations(episode)))
    if not rows:
        sys.exit("no finished episode in the log")
    print("episode\taudio_min\t" + "\t".join(STAGES) + "\ttotal\trealtime_x")
    for title, audio_s, spans in rows:
        total = sum(spans.values())
        speed = audio_s / spans["transcribe"] if spans.get("transcribe") else 0
        print(f"{title[:60]}\t{audio_s / 60:.0f}\t" + "\t".join(f"{spans.get(stage, 0):.0f}" for stage in STAGES) + f"\t{total:.0f}\t{speed:.1f}")
    totals = {stage: sum(spans.get(stage, 0) for _, _, spans in rows) for stage in STAGES}
    grand = sum(totals.values())
    audio = sum(audio_s for _, audio_s, _ in rows)
    print(f"\n{len(rows)} episodes, {audio / 3600:.1f} h of audio, {grand / 60:.1f} min in all")
    for stage in STAGES:
        each = [spans.get(stage, 0) for _, _, spans in rows]
        print(f"{stage}\t{totals[stage] / 60:.1f} min\t{100 * totals[stage] / grand:.0f} %\tmedian {statistics.median(each):.0f} s")


if __name__ == "__main__":
    main()
