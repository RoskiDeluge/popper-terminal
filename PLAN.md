# Popper Terminal MVP Plan

Goal: load the Popper shell inside a Tauri-based terminal UI and run Popper’s basic commands with a familiar bash-like experience.

1) Prereqs & discovery
- Bundle the Popper shell binary for both dev and prod by building the sibling repo at `~/dev/popper` as a workspace member/path dependency and shipping its `popper` binary as a Tauri sidecar (kept in sync via a scripted build/copy step).
- Capture Popper shell invocation contract (command name, args, env needs, config files, default working dir).
- Note OS targets for MVP (macOS only for first cut) and any platform-specific flags.

2) Backend bridge (Rust/Tauri)
- Add PTY-backed process management to run the Popper shell (e.g., portable-pty or a Tauri pty plugin) so interactive input/output works.
- Expose Tauri commands/events to create a session, stream stdout/stderr, send input, handle resize, and terminate/restart sessions; ensure sidecar path resolution works in dev/build.
- Normalize encoding, exit codes, and error surfaces so the UI can display clear session state (running/ended/error).

3) Frontend terminal UI (TS/HTML/CSS)
- Install a terminal emulator component (e.g., xterm.js) and wire it to the Tauri bridge for streaming output and user input.
- Implement resize propagation, scrollback, cursor behavior, and basic keyboard shortcuts (Enter, Ctrl+C, Ctrl+D).
- Add minimal chrome: window title, status indicator for session state, and a “restart shell” control.

4) Wiring Popper as the default shell
- On app load, automatically start a Popper shell session via the bundled sidecar (fallback to error view if spawn fails, with guidance on rebuilding if the sidecar is missing).
- Support manual restart/new session without restarting the app; ensure cleanup on app close.

5) Validation & dev loop
- Manual run: `npm run tauri dev`, verify Popper commands work, resize behavior, and that exit/interrupt works as expected.
- Add basic logging/metrics hooks (optional) to trace session lifecycle for debugging during MVP.

Open questions
- Exact sidecar layout for macOS bundle (e.g., `Resources/poppershell` vs default `sidecar` directory) and how we want to package debug symbols.
- Preferred automation to keep the sidecar fresh (Cargo workspace build step + copy in `build.rs` vs npm script that runs `cargo build -p popper --release` then copies).
