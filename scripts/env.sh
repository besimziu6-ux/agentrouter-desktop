#!/usr/bin/env bash
set -euo pipefail
export PKG_CONFIG_PATH=/tmp/opencode/pc-override:/tmp/opencode/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig:/tmp/opencode/sysroot/usr/share/pkgconfig:${PKG_CONFIG_PATH:-}
ver="$(pkg-config --modversion webkit2gtk-4.1)"
if [ "$ver" != "2.52.6" ]; then
  echo "env.sh: expected webkit2gtk-4.1 2.52.6, got $ver" >&2
  exit 1
fi
exec "$@"
