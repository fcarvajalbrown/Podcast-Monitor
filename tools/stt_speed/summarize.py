import argparse
import csv
import math
import statistics
import sys
from pathlib import Path


def describe(values):
    ordered = sorted(values)
    return f"n={len(values)} median={statistics.median(values):.2f} mean={statistics.mean(values):.2f} sd={statistics.stdev(values) if len(values) > 1 else 0:.2f} min={ordered[0]:.2f} max={ordered[-1]:.2f}"


def main():
    parser = argparse.ArgumentParser(description="Summarise results.tsv from week_bench.py: speed per setting, paired speed-up per episode, repeat spread, GPU temperatures, failures")
    parser.add_argument("results", type=Path)
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    with open(args.results, encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    good = [row for row in rows if not row["flag"]]
    print(f"rows={len(rows)} flagged={len(rows) - len(good)}")
    for kind in ("week", "repeat"):
        subset = [row for row in good if row["kind"].startswith(kind)]
        for variant in sorted({row["variant"] for row in subset}):
            mine = [row for row in subset if row["variant"] == variant]
            audio = sum(float(row["audio_s"]) for row in mine)
            compute = sum(float(row["compute_s"]) for row in mine)
            print(f"{kind} {variant}: audio_h={audio / 3600:.2f} compute_h={compute / 3600:.2f} pooled_x={audio / compute:.2f} per_episode {describe([float(row['x_real_time']) for row in mine])}")
            temperatures = [int(row["gpu_c_before"]) for row in mine if row["gpu_c_before"]]
            if temperatures:
                print(f"  gpu_c_before {describe(temperatures)}")
    pairs = {}
    for row in good:
        if row["kind"] == "week":
            pairs.setdefault(row["episode"], {})[row["variant"]] = row
    ratios = [float(pair["beam5"]["compute_s"]) / float(pair["flash-greedy"]["compute_s"]) for pair in pairs.values() if len(pair) == 2]
    print(f"paired speed-up flash-greedy over beam5: {describe(ratios)} geo_mean={math.exp(statistics.mean(math.log(r) for r in ratios)):.2f} slower_cases={sum(r < 1 for r in ratios)}")
    segments = [int(pair["flash-greedy"]["segments"]) / int(pair["beam5"]["segments"]) for pair in pairs.values() if len(pair) == 2]
    print(f"segment count ratio flash-greedy/beam5: {describe(segments)}")


if __name__ == "__main__":
    main()
