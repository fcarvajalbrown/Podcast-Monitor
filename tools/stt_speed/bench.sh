#!/usr/bin/env bash
set -euo pipefail

usage() { echo "usage: bench.sh <out dir> <variant>... (baseline flash greedy q8 vad, or all)" >&2; exit 1; }
[ $# -ge 2 ] || usage
out=$1
shift
[ "$1" = all ] && set -- baseline flash greedy q8 vad

root=$(cd "$(dirname "$0")/../.." && pwd)
exe=${PM_EXE:-$(cygpath -u "$USERPROFILE")/.pmt/release/podcast-monitor.exe}
models=${PM_MODELS:-$root/work/models}
full=${PM_MODEL:-$(cygpath -u "$APPDATA")/Podcast-Monitor/models/ggml-large-v3-turbo.bin}
feed=${PM_FEED:-https://anchor.fm/s/106ac41b0/podcast/rss}
title=${PM_TITLE:-Ep. 34}
glossary=${PM_GLOSSARY:-$root/tools/stt_compare/glossary-ap-ep34.txt}

for variant in "$@"; do
  args=(--model "$full")
  case $variant in
    baseline) ;;
    flash) args+=(--flash-attn) ;;
    greedy) args+=(--greedy) ;;
    q8) args=(--model "$models/ggml-large-v3-turbo-q8_0.bin") ;;
    vad) args+=(--vad-model "$models/ggml-silero-v5.1.2.bin") ;;
    *) usage ;;
  esac
  work=$out/$variant
  if compgen -G "$work/*/*/transcript.json" > /dev/null; then
    echo "$variant: already done"
  else
    echo "$variant: running"
    "$exe" run "$feed" --title "$title" --glossary "$glossary" --work "$work" "${args[@]}" > "$out/$variant.log" 2>&1
  fi
done

for variant in "$@"; do
  for transcript in "$out/$variant"/*/*/transcript.json; do
    [ -f "$transcript" ] || continue
    python -I -c 'import json,sys; t=json.load(open(sys.argv[2],encoding="utf-8")); print(f"{sys.argv[1]}: compute_s {t[\"compute_s\"]:.0f}, audio_s {t[\"audio_s\"]:.0f}, {t[\"audio_s\"]/t[\"compute_s\"]:.1f}x, {len(t[\"segments\"])} segments")' "$variant" "$transcript"
    python -I "$root/tools/stt_compare/count_names.py" "$glossary" "$transcript" "${transcript%.json}.fixed.json"
  done
done
