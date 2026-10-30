import argparse
import ctypes
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path

POLL_S = 30


def say(text):
    print(f"{datetime.now():%H:%M:%S} {text}", flush=True)


def next_clock(hhmm, after):
    hour, minute = map(int, hhmm.split(":"))
    moment = after.replace(hour=hour, minute=minute, second=0, microsecond=0)
    return moment if moment >= after else moment + timedelta(days=1)


def main():
    parser = argparse.ArgumentParser(description="Put Windows to sleep once a marker file appears, after a grace period and never before a clock time; skip sleeping if the job ended too soon after this started")
    parser.add_argument("marker", type=Path, help="file the job writes when it ends")
    parser.add_argument("--not-before", default="05:00", help="local HH:MM, the next one from now")
    parser.add_argument("--grace-min", type=float, default=20)
    parser.add_argument("--min-job-min", type=float, default=10, help="if the marker appears sooner than this after start, do not sleep")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    started = datetime.now()
    earliest = next_clock(args.not_before, started)
    say(f"waiting for {args.marker}; sleep not before {earliest:%Y-%m-%d %H:%M}, {args.grace_min:g} min after the marker")
    while not args.marker.exists():
        time.sleep(POLL_S)
    ended = datetime.now()
    say(f"marker found: {args.marker.read_text(encoding='utf-8', errors='replace').strip()[:200]}")
    if ended - started < timedelta(minutes=args.min_job_min):
        say(f"job ended {(ended - started).total_seconds() / 60:.1f} min after start, under {args.min_job_min:g}; not sleeping")
        return 1
    target = max(earliest, ended + timedelta(minutes=args.grace_min))
    say(f"sleeping the PC at {target:%H:%M}")
    while datetime.now() < target:
        time.sleep(POLL_S)
    if args.dry_run:
        say("dry run, not sleeping")
        return 0
    say("sleeping now")
    ctypes.windll.powrprof.SetSuspendState(False, False, False)
    return 0


if __name__ == "__main__":
    sys.exit(main())
