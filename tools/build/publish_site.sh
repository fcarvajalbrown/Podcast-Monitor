#!/bin/bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
host="${PM_SSH_HOST:-ffa}"
folder="${PM_SITE:-domains/copiapolabs.cl/public_html/podcastmonitor}"
installer="$root/work/installer/PodcastMonitor-windows-setup.exe"
name=$(basename "$installer")

ssh -o BatchMode=yes "$host" "mkdir -p $folder/descargas $folder/tour"
scp -q "$root"/site/index.html "$root"/site/cambios.html "$root"/site/robots.txt "$root"/site/icono.svg "$root"/site/.htaccess "$host:$folder/"
scp -q "$root"/site/tour/*.png "$host:$folder/tour/"
scp -q "$installer" "$host:$folder/descargas/$name.partial"
remote=$(ssh -o BatchMode=yes "$host" "cd $folder/descargas && mv $name.partial $name && sha256sum $name" | cut -d' ' -f1)
local_hash=$(sha256sum "$installer" | cut -d' ' -f1)
if [ "$remote" != "$local_hash" ]; then
  echo "hash mismatch for $name" >&2
  exit 1
fi
echo "$name published, $(stat -c %s "$installer") bytes, sha256 $remote"
