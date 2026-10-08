#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${PORT:-18923}"

for cand in "$ROOT/src-tauri/target/debug/agentrouter-desktop" "$ROOT/target/debug/agentrouter-desktop"; do
  if [ -x "$cand" ]; then BIN="$cand"; break; fi
done
if [ -z "${BIN:-}" ]; then
  echo "smoke: binary not found, run scripts/build.sh first" >&2
  exit 1
fi

"$BIN" --serve --port "$PORT" &
pid=$!
trap 'kill $pid 2>/dev/null || true' EXIT

for _ in $(seq 1 50); do
  if curl -fsS "http://127.0.0.1:$PORT/api/health" >/dev/null 2>&1; then break; fi
  sleep 0.2
done

curl -fsS "http://127.0.0.1:$PORT/api/health" | grep -q '"status"' || { echo "smoke: bad /api/health" >&2; exit 1; }
title="$(curl -fsS "http://127.0.0.1:$PORT/" | grep -o '<title>[^<]*</title>' || true)"
echo "smoke: health ok, title: $title"
echo "$title" | grep -q 'AgentRouter' || { echo "smoke: unexpected title" >&2; exit 1; }
echo "smoke: PASS"
