# AgentRouter Desktop

Tauri 2 shell around the AgentRouter web UI. The Rust backend serves
`dist/` over HTTP on 127.0.0.1; window mode points a Tauri webview at it.

Standard Tauri layout: Rust crate lives in `src-tauri/`, synced frontend in
`dist/`, helper scripts in `scripts/`.

## Prereqs

- Rust stable toolchain
- Tauri Linux deps (Ubuntu 22.04):
  `libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev
  libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
- Tauri CLI: `cargo install tauri-cli --version "^2"`
- Local override for this machine: prefix commands with `scripts/env.sh`
  (sets `PKG_CONFIG_PATH` for the vendored webkit sysroot).

## Scripts

- `scripts/sync-frontend.sh` — copy UI from
  `../agentrouter-cli/src/agentrouter/static` into `dist/`
- `scripts/build.sh` — sync + `cargo build` (pass extra cargo args through)
- `scripts/smoke.sh` — start `--serve --port 18923`, check `/api/health`
  and `/` title, kill. Override port with `PORT=...`
- `scripts/env.sh` — sysroot wrapper, usage: `scripts/env.sh <cmd>`

## Flags

- `--serve` — headless HTTP mode, no window (used by smoke tests and CI)
- `--port <n>` — bind port, default `8787`

Default (no flags) is reserved for Tauri window mode: bind 127.0.0.1:0,
then open the `main` webview at the assigned URL. Window wiring lives in
`src-tauri/src/main.rs` (pending).

## Icons

Placeholder at `src-tauri/icons/icon.svg`. Before a real release, generate
the full set:

```
cd src-tauri && cargo tauri icon icons/icon.svg
```

This writes the `.png`/`.icns` files `tauri.conf.json` bundling expects.

## CI

`.github/workflows/build.yml` runs on push and manual dispatch: installs
system deps, syncs the frontend, builds the `.deb` via
`tauri-apps/tauri-action@v0`, uploads it as an artifact. Tags matching
`v*` also create a GitHub release with the `.deb` attached.

## Phase scope

Window shell + CI only. Backend API, auth, and multi-window work land in
later phases.
