#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/../agentrouter-cli/src/agentrouter/static"
DST="$ROOT/dist"

if [ ! -f "$SRC/index.html" ]; then
  echo "sync-frontend: source not found: $SRC/index.html" >&2
  exit 1
fi

mkdir -p "$DST/static/css" "$DST/static/js"
cp -f "$SRC/index.html" "$DST/index.html"

cp -f "$SRC"/css/*.css "$DST/static/css/" 2>/dev/null || true
for f in "$SRC"/js/*.js; do
  base="$(basename "$f")"
  case "$base" in
    *.test.js) continue ;;
    package.json) continue ;;
    *) cp -f "$f" "$DST/static/js/$base" ;;
  esac
done
rm -f "$DST"/static/js/*.test.js "$DST"/static/js/package.json 2>/dev/null || true

echo "synced frontend -> $DST"
ls "$DST/index.html" "$DST/static" >/dev/null
