# Change: Add Session History and Resume Context

## Why
Developers and agents lose terminal context after shell exit or app restart, which slows iterative workflows. A lightweight history and resume flow reduces repeated setup and improves continuity without introducing multi-session complexity.

## What Changes
- Add persisted session records for completed/terminated terminal sessions.
- Add a searchable history UI to browse recent sessions and open transcript previews.
- Add a resume action that starts a new shell in the previous session's last known working directory.
- Add bounded retention and transcript size limits to control storage growth.
- Add privacy controls for transcript persistence and basic redaction safeguards.

## Reffy References
- `dev-agent-qol-features-v1.md` - initial ideation, scope guardrails, and ordering rationale for session history/resume.

## Impact
- Affected specs: `session-history-resume` (new capability)
- Affected code:
  - Frontend terminal controls and new history panel (`src/main.ts`, `src/styles.css`)
  - Rust PTY/session lifecycle + persistence service (`src-tauri/src/lib.rs` and new storage module)
  - Optional config wiring for retention defaults
