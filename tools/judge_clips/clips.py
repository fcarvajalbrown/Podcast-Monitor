import argparse
import csv
import json
import subprocess
import sys
from pathlib import Path

PAD_S = 1.0


def segment_end(transcript, start):
    segments = json.loads(transcript.read_text(encoding="utf-8"))["segments"]
    return min(segments, key=lambda segment: abs(segment["start_s"] - start))["end_s"]


def main():
    parser = argparse.ArgumentParser(description="Cut one numbered MP3 clip per name-fix change listed by names::tests::lists_every_change_the_fix_makes_on_saved_transcripts, for judging by ear")
    parser.add_argument("changes", help="TSV printed by the audit test")
    parser.add_argument("runs", help="run folder the audit read")
    parser.add_argument("out", help="folder for the clips and index.tsv")
    parser.add_argument("--skip", nargs="*", default=[], help="corrected names whose changes need no clip")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    with open(args.changes, encoding="utf-8") as handle:
        rows = [row for row in csv.reader(handle, delimiter="\t") if len(row) == 6 and row[0] != "folder" and row[3] not in args.skip]
    index = []
    for number, (folder, start, heard, fixed, kind, text) in enumerate(rows, 1):
        episode = Path(args.runs) / folder
        begin = float(start)
        end = segment_end(episode / "transcript.json", begin)
        clip = out / f"{number:02}.mp3"
        if not clip.exists():
            subprocess.run(["ffmpeg", "-v", "error", "-y", "-ss", f"{max(begin - PAD_S, 0):.2f}", "-to", f"{end + PAD_S:.2f}", "-i", str(episode / "audio.mp3"), "-ac", "1", "-b:a", "96k", str(clip)], check=True)
        index.append([f"{number:02}", heard, fixed, kind, folder, start, text])
        print(f"{number:02}  {heard} -> {fixed}  {text[:100]}")
    with open(out / "index.tsv", "w", encoding="utf-8", newline="") as handle:
        csv.writer(handle, delimiter="\t").writerows([["number", "heard", "fixed", "kind", "folder", "start_s", "segment"], *index])


if __name__ == "__main__":
    main()
