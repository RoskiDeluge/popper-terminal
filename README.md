# <img src="src-tauri/icons/128x128.png" alt="Popper Terminal icon" width="32" height="32" /> Popper Terminal

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
To launch the terminal with the Popper shell, follow this sequence from a fresh checkout:
1) `git clone https://github.com/RoskiDeluge/popper-terminal`
2) `cd popper-terminal`
3) `git clone https://github.com/RoskiDeluge/popper`
4) From `popper-terminal`, stay at the repo root (the `popper` clone should live at `popper-terminal/popper`).
5) `npm install`
6) `npm run tauri dev`

- `POPPER_PATH` should point to your Popper repo root; if omitted, the build script looks for a sibling `../popper` relative to `src-tauri`.
- If your Popper repo lives elsewhere, launch with `POPPER_PATH=/path/to/popper npm run tauri dev`.
- The build script runs `cargo build` for Popper and copies the binary into `src-tauri/bin` as a sidecar.
- Quit/relaunch to pick up icon or sidecar changes; `cargo clean` in `src-tauri` can help when assets cache.

## Launching the app
After cloning both `popper-terminal` and `popper` and installing frontend deps, launch from the repo root with:
`npm run tauri dev`
