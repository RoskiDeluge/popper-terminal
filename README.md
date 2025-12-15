# Popper Terminal

Popper Terminal is a Tauri-based desktop terminal that bundles and runs the [Popper shell](https://github.com/RoskiDeluge/popper) as its default session. It uses a PTY bridge and xterm.js frontend for interactive input/output.

## Features
- Bundled Popper shell sidecar (built from your local Popper repo)
- Interactive PTY-backed session with xterm.js UI
- Window controls for restart; `exit` in the shell closes the app

## Prerequisites
- Rust toolchain (for Tauri backend and Popper)
- Node 18+ (for Vite/Tauri frontend tooling)
- Popper shell source checked out locally (set `POPPER_PATH` to its path)

## Dev setup
1) Install JS deps: `npm install`
2) Run dev app: `POPPER_PATH=/full/path/to/popper npm run tauri dev`  
   - `POPPER_PATH` should point to your Popper repo root; if omitted, the build script looks for a sibling `../popper` relative to `src-tauri`.
   - The build script runs `cargo build` for Popper and copies the binary into `src-tauri/bin` as a sidecar.
3) Quit/relaunch to pick up icon or sidecar changes; `cargo clean` in `src-tauri` can help when assets cache.
