# Project Context

## Purpose
Tauri-based desktop terminal that bundles and runs the Popper shell as a sidecar, exposing it through a PTY bridge and xterm.js UI so Popper can be used as the default interactive session.

## Tech Stack
- Frontend: TypeScript, Vite, xterm.js (+ fit addon), vanilla DOM/CSS
- Desktop shell: Tauri 2.x
- Backend: Rust with portable-pty for PTY handling, tauri-plugin-opener for OS integrations
- Build tools: npm scripts for Vite/Tauri, Rust toolchain for the sidecar and backend

## Project Conventions

### Code Style
- TypeScript with strict compiler settings (`strict`, `noUnused*`, `noFallthroughCasesInSwitch`); ES modules
- Minimal framework usage on the frontend (direct DOM APIs); prefers explicit state variables and early returns
- Rust backend favors small functions, Result-based error strings for Tauri command surfaces

### Architecture Patterns
- Tauri app with a Rust backend exposing commands (`start_session`, `write_to_session`, `resize_session`, `terminate_session`) that manage PTY sessions via portable-pty
- Popper shell built as a sidecar binary during `build.rs`; copied into `src-tauri/bin` (and target-suffixed variants) for bundling and runtime lookup
- Frontend single-window Vite app: xterm.js terminal wired to Tauri events (`pty-data`, `pty-exit`) and invokes backend commands for IO and resize; FitAddon keeps terminal sized to the container
- Sidecar resolution checks bundled resources, runtime `bin/`, then dev `src-tauri/bin`; exit code 0 closes the window, non-zero leaves UI up with restart affordance

### Testing Strategy
- No automated tests yet; validation is manual via `POPPER_PATH=/path/to/popper npm run tauri dev` or `npm run dev` + `tauri dev`
- Manual checks: Popper builds as sidecar, terminal renders and streams data, resize propagates, restart flow works after non-zero exit, `exit` closes the app

### Git Workflow
- Not formally documented; default to feature branches with small commits and clear messages targeting the main branch

## Domain Context
- Popper shell is an external project; this app only launches the Popper binary and relays PTY IO
- Build script requires access to the Popper repo (set `POPPER_PATH` or place `../popper` relative to `src-tauri`); failures to build/copy Popper skip sidecar bundling, causing runtime launch errors
- UI shows status text for start/failure/restart and auto-closes on clean shell exit

## Important Constraints
- Requires Rust toolchain and Node 18+; Tauri mobile features not used
- Sidecar lookup depends on `TAURI_ENV_TARGET_TRIPLE`/`TARGET` naming for bundled binaries; permissions are set executable on Unix during build
- Terminal currently single-session; status/state kept in-memory in the frontend and Rust `PtyState`

## External Dependencies
- Popper shell source (local checkout built as sidecar)
- Tauri APIs (`@tauri-apps/api`, Tauri 2 runtime) and tauri-plugin-opener
- portable-pty for PTY management
- xterm.js for terminal UI
