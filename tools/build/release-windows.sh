#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
version=$(grep -m1 '^version' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
exe=$(cygpath -u "$USERPROFILE")/.pmt/release/podcast-monitor.exe
iscc="/c/Program Files (x86)/Inno Setup 6/ISCC.exe"

cmd //c "$(cygpath -w tools/build/cargo-windows.cmd)" build --release -j "${PM_JOBS:-2}"
[ -f work/vulkan-loader/bin/vulkan-1.dll ] || cmd //c "$(cygpath -w tools/build/vulkan-loader.cmd)"
if [ ! -f work/redist/vc_redist.x64.exe ]; then
  mkdir -p work/redist
  curl -sSL -o work/redist/vc_redist.x64.exe https://aka.ms/vs/17/release/vc_redist.x64.exe
fi

"$iscc" /Q "/DAppVersion=$version" "/DAppExe=$(cygpath -w "$exe")" "$(cygpath -w tools/build/installer.iss)"
setup=work/installer/PodcastMonitor-windows-setup.exe
echo "version $version"
echo "$setup $(stat -c %s "$setup") bytes"
sha256sum "$setup"
