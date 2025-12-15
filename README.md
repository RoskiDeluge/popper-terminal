# Popper Terminal

Popper Terminal is a Tauri-based desktop terminal that bundles and runs the Popper shell as its default session. It uses a PTY bridge and xterm.js frontend for interactive input/output.

## Features
- Bundled Popper shell sidecar (built from sibling repo `~/dev/popper` by default)
- Interactive PTY-backed session with xterm.js UI
- Window controls for restart; `exit` in the shell closes the app

## Prerequisites
- Rust toolchain (for Tauri backend and Popper)
- Node 18+ (for Vite/Tauri frontend tooling)
- Popper shell source checked out at `~/dev/popper` (override with `POPPER_PATH` if different)

## Dev setup
1) Install JS deps: `npm install`
2) Run dev app: `npm run tauri dev`  
   - The build script will `cargo build` the Popper shell and copy the binary into `src-tauri/bin` as a sidecar.
   - If your Popper repo is elsewhere, set `POPPER_PATH=/path/to/popper` before running.
3) Quit/relaunch to pick up icon or sidecar changes; `cargo clean` in `src-tauri` can help when assets cache.

## Notes
- Icons live in `src-tauri/icons`; replace `icon.icns` (macOS) and `icon.ico`/PNGs, then rebuild.
- Status/errors surface in the UI and console; adjust logging in `src/main.ts` if needed.
