# <img src="src-tauri/icons/icon.png" alt="Popper Terminal icon" width="32" height="32" /> Popper Terminal

Popper Terminal is a Tauri-based desktop terminal that bundles and runs the [popper](https://github.com/RoskiDeluge/popper) shell as its default session. It uses a PTY bridge and xterm.js frontend for interactive input/output.

## Features
- Bundled Popper shell sidecar (built from your local Popper repo)
- Interactive PTY-backed session with xterm.js UI
- Window controls for restart; `exit` in the shell closes the app

## Prerequisites
- Rust toolchain (for Tauri backend and Popper)
- Node 18+ (for Vite/Tauri frontend tooling)
- A local checkout of the [popper](https://github.com/RoskiDeluge/popper) shell source

## Dev setup
From a fresh checkout:
1) `git clone https://github.com/RoskiDeluge/popper-terminal`
2) `cd popper-terminal`
3) `cd ..`
4) `git clone https://github.com/RoskiDeluge/popper`
5) `cd popper-terminal`
6) `npm install`
7) `npm run tauri dev`

- `POPPER_PATH` should point to your Popper repo root; if omitted, the build script looks for a sibling `popper` repo next to `popper-terminal`.
- If your Popper repo lives elsewhere, launch with `POPPER_PATH=/path/to/popper npm run tauri dev`.
- Quit and relaunch the app to pick up icon or sidecar changes.

## Sidecar rebuilds
- Changes in the Popper repo's `src/`, `Cargo.toml`, or `Cargo.lock` trigger a fresh sidecar rebuild the next time you run `npm run tauri dev` or build from `src-tauri`.
- Popper is built into `src-tauri/target/popper-sidecar`, then copied into `src-tauri/bin` for bundling.
- On macOS Apple Silicon, the app writes both `src-tauri/bin/popper` and `src-tauri/bin/popper-aarch64-apple-darwin`.
- To force a fresh sidecar rebuild manually, run `cargo build` in `src-tauri`.
