#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT/scripts/sync-frontend.sh"

cargo build --manifest-path "$ROOT/src-tauri/Cargo.toml" "$@"
