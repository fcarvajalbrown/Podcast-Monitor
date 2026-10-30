import argparse
import csv
import json
import os
import subprocess
import sys
from pathlib import Path

VARIANTS = {"beam5": [], "flash-greedy": ["--flash-attn", "--greedy"]}


def app_data():
    return Path(os.environ["APPDATA"]) / "Podcast-Monitor"


def episodes(runs, since, feeds):
    found = []
    for folder in sorted(runs.glob("*/*")):
        meta = folder / "episode.json"
        show = folder.parent / "show.json"
        if folder.name.endswith(".bak-weekly") or not meta.is_file() or not show.is_file():
            continue
        episode = json.loads(meta.read_text(encoding="utf-8"))
        title = json.loads(show.read_text(encoding="utf-8"))["title"]
        audio = next(iter(sorted(folder.glob("audio.*"))), None)
        if (episode.get("published") or "") >= since and audio and title in feeds:
            found.append({"relative": folder.relative_to(runs), "title": episode["title"], "feed": feeds[title], "audio": audio})
    return found


def transcribe(exe, model, work, episode, variant, log):
    folder = work / episode["relative"]
    transcript = folder / "transcript.json"
    if transcript.is_file():
        return transcript
    folder.mkdir(parents=True, exist_ok=True)
    audio = folder / episode["audio"].name
    if not audio.exists():
        os.link(episode["audio"], audio)
    command = [str(exe), "run", episode["feed"], "--title", episode["title"], "--latest", "1", "--work", str(work), "--model", str(model), *VARIANTS[variant]]
    with open(log, "a", encoding="utf-8") as handle:
        subprocess.run(command, stdout=handle, stderr=subprocess.STDOUT, check=False)
    if not transcript.is_file():
        print(f"  no transcript for {episode['relative']} ({variant}), see {log}", flush=True)
    return transcript


def record(rows, kind, variant, episode, transcript):
    if transcript.is_file():
        data = json.loads(transcript.read_text(encoding="utf-8"))
        rows.append([kind, variant, str(episode["relative"]).replace("\\", "/"), f"{data['audio_s']:.1f}", f"{data['compute_s']:.1f}", f"{data['audio_s'] / data['compute_s']:.2f}", len(data["segments"])])


def main():
    parser = argparse.ArgumentParser(description="Time transcription of every episode since a date in both decoding settings (beam 5, greedy with flash attention), alternating the order per episode, plus repeats of one episode; resumes from finished transcripts when rerun")
    parser.add_argument("out", type=Path)
    parser.add_argument("--since", default="2026-10-26", help="first publication date, ISO")
    parser.add_argument("--repeat-title", default="Ep. 34", help="text in the title of the episode to repeat")
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--exe", type=Path, default=Path(os.environ["USERPROFILE"]) / ".pmt" / "release" / "podcast-monitor.exe")
    parser.add_argument("--model", type=Path, default=app_data() / "models" / "ggml-large-v3-turbo.bin")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    feeds = {show["title"]: show["feed"] for show in json.loads((app_data() / "shows.json").read_text(encoding="utf-8"))}
    found = episodes(app_data() / "runs", args.since, feeds)
    repeated = next(episode for episode in found if args.repeat_title in episode["title"])
    print(f"{len(found)} episodes since {args.since}; repeating {repeated['title']} x{args.repeats}", flush=True)
    log = args.out / "runs.log"
    args.out.mkdir(parents=True, exist_ok=True)
    rows = []
    for index, episode in enumerate(found):
        order = list(VARIANTS) if index % 2 == 0 else list(reversed(VARIANTS))
        for variant in order:
            print(f"[{index + 1}/{len(found)}] {variant}  {episode['title'][:70]}", flush=True)
            record(rows, "week", variant, episode, transcribe(args.exe, args.model, args.out / variant, episode, variant, log))
    for repeat in range(1, args.repeats + 1):
        for variant in VARIANTS if repeat % 2 else reversed(VARIANTS):
            print(f"[repeat {repeat}/{args.repeats}] {variant}", flush=True)
            record(rows, f"repeat{repeat}", variant, repeated, transcribe(args.exe, args.model, args.out / f"repeat{repeat}-{variant}", repeated, variant, log))
    with open(args.out / "results.tsv", "w", encoding="utf-8", newline="") as handle:
        csv.writer(handle, delimiter="\t").writerows([["kind", "variant", "episode", "audio_s", "compute_s", "x_real_time", "segments"], *rows])
    print(f"{len(rows)} timings in {args.out / 'results.tsv'}", flush=True)


if __name__ == "__main__":
    main()
