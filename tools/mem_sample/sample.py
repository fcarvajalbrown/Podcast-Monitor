import argparse
import csv
import json
import subprocess
import sys
import time
from pathlib import Path

FIELDS = ["time_s", "working_set_mb", "private_mb", "peak_working_set_mb"]


def read(name):
    script = (
        f"Get-Process -Name '{name}' -ErrorAction SilentlyContinue | "
        "Select-Object WorkingSet64,PrivateMemorySize64,PeakWorkingSet64 | ConvertTo-Json -Compress"
    )
    out = subprocess.run(["powershell", "-NoProfile", "-Command", script], capture_output=True, text=True).stdout.strip()
    if not out:
        return None
    found = json.loads(out)
    found = found[0] if isinstance(found, list) else found
    return [found["WorkingSet64"] / 2**20, found["PrivateMemorySize64"] / 2**20, found["PeakWorkingSet64"] / 2**20]


def main():
    parser = argparse.ArgumentParser(description="Sample a Windows process's memory every few seconds into a CSV until it exits, then print the peaks")
    parser.add_argument("name", help="process name without .exe")
    parser.add_argument("--out", required=True, help="CSV file; rows are appended, so a rerun continues it")
    parser.add_argument("--every", type=float, default=5.0, help="seconds between samples")
    parser.add_argument("--wait", type=float, default=120.0, help="seconds to wait for the process to appear")
    args = parser.parse_args()
    out = Path(args.out)
    new = not out.exists()
    started = time.monotonic()
    while read(args.name) is None:
        if time.monotonic() - started > args.wait:
            sys.exit(f"{args.name} did not start within {args.wait:.0f} s")
        time.sleep(1)
    peaks = [0.0, 0.0, 0.0]
    with out.open("a", newline="") as handle:
        writer = csv.writer(handle)
        if new:
            writer.writerow(FIELDS)
        while (sample := read(args.name)) is not None:
            writer.writerow([round(time.monotonic() - started)] + [round(value, 1) for value in sample])
            handle.flush()
            peaks = [max(peak, value) for peak, value in zip(peaks, sample)]
            time.sleep(args.every)
    print(f"working set max {peaks[0]:.0f} MB, private max {peaks[1]:.0f} MB, peak working set {peaks[2]:.0f} MB")


if __name__ == "__main__":
    main()
