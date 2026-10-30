import argparse
import csv
import ctypes
import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

VARIANTS = {"beam5": [], "flash-greedy": ["--flash-attn", "--greedy"]}
SLOWEST_X_REAL_TIME = 3.0
RUN_OVERHEAD_S = 300
ATTEMPTS = 2
HOT_C = 85
COOL_C = 70
MIN_FREE_RAM_GB = 2.0
MIN_FREE_DISK_GB = 2.0
POLL_S = 60
ES_CONTINUOUS = 0x80000000
ES_SYSTEM_REQUIRED = 0x00000001


class PowerStatus(ctypes.Structure):
    _fields_ = [("ac_line", ctypes.c_byte), ("flags", ctypes.c_byte), ("percent", ctypes.c_byte), ("saver", ctypes.c_byte), ("life_s", ctypes.c_ulong), ("full_s", ctypes.c_ulong)]


class MemoryStatus(ctypes.Structure):
    _fields_ = [("length", ctypes.c_ulong), ("load", ctypes.c_ulong), ("total", ctypes.c_ulonglong), ("available", ctypes.c_ulonglong), ("total_page", ctypes.c_ulonglong), ("available_page", ctypes.c_ulonglong), ("total_virtual", ctypes.c_ulonglong), ("available_virtual", ctypes.c_ulonglong), ("available_extended", ctypes.c_ulonglong)]


def app_data():
    return Path(os.environ["APPDATA"]) / "Podcast-Monitor"


def say(text):
    print(f"{datetime.now():%H:%M:%S} {text}", flush=True)


def on_ac_power():
    status = PowerStatus()
    return not ctypes.windll.kernel32.GetSystemPowerStatus(ctypes.byref(status)) or status.ac_line != 0


def free_ram_gb():
    status = MemoryStatus()
    status.length = ctypes.sizeof(MemoryStatus)
    ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status))
    return status.available / 2**30


def gpu_temperature():
    try:
        output = subprocess.run(["nvidia-smi", "--query-gpu=temperature.gpu", "--format=csv,noheader"], capture_output=True, text=True, timeout=30).stdout
        return int(output.split()[0])
    except (OSError, ValueError, IndexError, subprocess.TimeoutExpired):
        return None


def wait_until_safe(out):
    if shutil.disk_usage(out).free / 2**30 < MIN_FREE_DISK_GB:
        raise SystemExit(f"under {MIN_FREE_DISK_GB} GB free on the disk; stopping, rerun the same command to resume")
    while not on_ac_power():
        say("on battery, waiting for AC power")
        time.sleep(POLL_S)
    while free_ram_gb() < MIN_FREE_RAM_GB:
        say(f"{free_ram_gb():.1f} GB RAM free, waiting")
        time.sleep(POLL_S)
    temperature = gpu_temperature()
    if temperature is not None and temperature >= HOT_C:
        while temperature is not None and temperature > COOL_C:
            say(f"GPU at {temperature} C, waiting to cool under {COOL_C} C")
            time.sleep(POLL_S)
            temperature = gpu_temperature()
    return temperature


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
            found.append({"relative": folder.relative_to(runs), "title": episode["title"], "feed": feeds[title], "audio": audio, "duration_s": episode.get("duration_s") or 3600})
    return found


def transcribe(exe, model, work, episode, variant, log, out):
    folder = work / episode["relative"]
    transcript = folder / "transcript.json"
    if transcript.is_file():
        return transcript, None
    folder.mkdir(parents=True, exist_ok=True)
    audio = folder / episode["audio"].name
    if not audio.exists():
        os.link(episode["audio"], audio)
    command = [str(exe), "run", episode["feed"], "--title", episode["title"], "--latest", "1", "--work", str(work), "--model", str(model), *VARIANTS[variant]]
    limit = episode["duration_s"] / SLOWEST_X_REAL_TIME + RUN_OVERHEAD_S
    temperature = None
    for attempt in range(1, ATTEMPTS + 1):
        temperature = wait_until_safe(out)
        with open(log, "a", encoding="utf-8") as handle:
            handle.write(f"\n=== {datetime.now():%Y-%m-%d %H:%M:%S} {variant} attempt {attempt} {episode['relative']}\n")
            handle.flush()
            try:
                subprocess.run(command, stdout=handle, stderr=subprocess.STDOUT, timeout=limit, check=False)
            except subprocess.TimeoutExpired:
                say(f"  killed after {limit:.0f} s (attempt {attempt})")
        if transcript.is_file():
            return transcript, temperature
        say(f"  no transcript (attempt {attempt}), see {log}")
    return transcript, temperature


def record(rows, kind, variant, episode, transcript, temperature):
    if not transcript.is_file():
        rows.append([kind, variant, str(episode["relative"]).replace("\\", "/"), "", "", "", "", temperature or "", "failed"])
        return
    data = json.loads(transcript.read_text(encoding="utf-8"))
    speed = data["audio_s"] / data["compute_s"] if data["compute_s"] else 0.0
    flag = "empty" if not data["segments"] else "suspect speed" if not 3 <= speed <= 80 else ""
    if flag:
        say(f"  {flag}: {speed:.1f}x, {len(data['segments'])} segments")
    rows.append([kind, variant, str(episode["relative"]).replace("\\", "/"), f"{data['audio_s']:.1f}", f"{data['compute_s']:.1f}", f"{speed:.2f}", len(data["segments"]), temperature if temperature is not None else "", flag])


def save(out, rows, step, total, current):
    with open(out / "results.tsv.part", "w", encoding="utf-8", newline="") as handle:
        csv.writer(handle, delimiter="\t").writerows([["kind", "variant", "episode", "audio_s", "compute_s", "x_real_time", "segments", "gpu_c_before", "flag"], *rows])
    os.replace(out / "results.tsv.part", out / "results.tsv")
    progress = {"updated": datetime.now().isoformat(timespec="seconds"), "step": step, "of": total, "current": current, "failed": sum(row[-1] == "failed" for row in rows)}
    (out / "progress.json").write_text(json.dumps(progress, ensure_ascii=False, indent=1), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description="Time transcription of every episode since a date in both decoding settings (beam 5, greedy with flash attention), alternating the order per episode, plus repeats of one episode; unattended-safe and resumes from finished transcripts when rerun")
    parser.add_argument("out", type=Path)
    parser.add_argument("--since", default="2026-10-26", help="first publication date, ISO")
    parser.add_argument("--repeat-title", default="Ep. 34", help="text in the title of the episode to repeat")
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--exe", type=Path, default=Path(os.environ["USERPROFILE"]) / ".pmt" / "release" / "podcast-monitor.exe")
    parser.add_argument("--model", type=Path, default=app_data() / "models" / "ggml-large-v3-turbo.bin")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    ctypes.windll.kernel32.SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED)
    feeds = {show["title"]: show["feed"] for show in json.loads((app_data() / "shows.json").read_text(encoding="utf-8"))}
    found = episodes(app_data() / "runs", args.since, feeds)
    repeated = next(episode for episode in found if args.repeat_title in episode["title"])
    jobs = []
    for index, episode in enumerate(found):
        for variant in VARIANTS if index % 2 == 0 else reversed(VARIANTS):
            jobs.append(("week", variant, episode, args.out / variant))
    for repeat in range(1, args.repeats + 1):
        for variant in VARIANTS if repeat % 2 else reversed(VARIANTS):
            jobs.append((f"repeat{repeat}", variant, repeated, args.out / f"repeat{repeat}-{variant}"))
    args.out.mkdir(parents=True, exist_ok=True)
    say(f"{len(found)} episodes since {args.since}, {len(jobs)} runs; repeating {repeated['title']} x{args.repeats}")
    log = args.out / "runs.log"
    rows = []
    try:
        for step, (kind, variant, episode, work) in enumerate(jobs, 1):
            say(f"[{step}/{len(jobs)}] {kind} {variant}  {episode['title'][:70]}")
            save(args.out, rows, step, len(jobs), f"{kind} {variant} {episode['relative']}")
            transcript, temperature = transcribe(args.exe, args.model, work, episode, variant, log, args.out)
            record(rows, kind, variant, episode, transcript, temperature)
            save(args.out, rows, step, len(jobs), "done" if step == len(jobs) else "between runs")
    finally:
        ctypes.windll.kernel32.SetThreadExecutionState(ES_CONTINUOUS)
    say(f"{len(rows)} timings in {args.out / 'results.tsv'}, {sum(row[-1] == 'failed' for row in rows)} failed")


if __name__ == "__main__":
    main()
