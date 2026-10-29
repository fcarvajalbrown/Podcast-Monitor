#!/usr/bin/env bash
set -euo pipefail

usage() { echo "usage: resummarize.sh <monday YYYY-MM-DD> [codex|claude]" >&2; exit 1; }
[ $# -ge 1 ] || usage
monday=$1
provider=${2:-codex}
exe=${PM_EXE:-$(cygpath -u "$USERPROFILE")/.pmt/release/podcast-monitor.exe}
data=${PM_DATA:-$(cygpath -u "$APPDATA")/Podcast-Monitor}
glossary=$data/glossary.txt

for week in "$data"/runs/*/week-"$monday".json; do
  [ -f "$week" ] || continue
  show_dir=$(dirname "$week")
  for id in $(python -I -c 'import json,sys; print("\n".join(json.load(open(sys.argv[1], encoding="utf-8"))["episodes"]))' "$week" | tr -d '\r'); do
    folder=$show_dir/$id
    if [ ! -f "$folder/summary.previous.json" ] && [ -f "$folder/summary.json" ]; then
      mv "$folder/summary.json" "$folder/summary.previous.json"
    fi
    transcript=$folder/transcript.json
    args=()
    if [ -f "$glossary" ]; then
      args=(--glossary "$glossary")
      [ -f "$folder/transcript.fixed.json" ] && transcript=$folder/transcript.fixed.json
    fi
    echo "summarizing $id"
    "$exe" summarize "$transcript" "${args[@]}" --provider "$provider" --out "$folder/summary.json" > /dev/null
  done
  mv "$week" "${week%.json}.previous.json"
done

themes=$data/weeks/$monday/themes.json
[ -f "$themes" ] && mv "$themes" "${themes%.json}.previous.json"
echo "done; run: podcast-monitor week --report"
