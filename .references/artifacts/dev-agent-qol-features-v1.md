# Dev/Agent QoL Feature Exploration (v1)

## Metadata
- Date: 2026-03-05
- Owner: Codex
- Area: Terminal UX / Tauri + Popper sidecar
- Status: proposed

## Context
Popper Terminal currently launches a fresh Popper shell session and streams PTY I/O. For developer and agent workflows, repeated context setup (re-running commands, restoring cwd, recalling previous outputs) adds friction.

This note proposes two small quality-of-life features to review before formal OpenSpec planning.

## Feature 1: Session History + Resume

### Problem
When a session exits or the app is restarted, users lose quick access to previous command/output context and session metadata. Reconstructing context costs time.

### Proposal
Add a lightweight session archive and resume picker.

### UX
- On each session end, persist a `session record` containing:
  - start/end timestamp
  - exit code / terminated flag
  - initial cwd
  - shell binary/version metadata (if available)
  - bounded transcript tail (for quick preview)
- Add a `History` button in the window chrome.
- History panel shows recent sessions sorted newest-first with search by command text.
- Selecting a session offers:
  - `Open Transcript` (read-only)
  - `Resume Context` (start new shell in last known cwd and optionally replay selected commands)

### Scope Guardrails
- Transcript storage is bounded by size and count (for example, keep last N sessions, trim large output).
- Resume is a new shell process seeded by context, not process checkpointing.
- Local-only storage under app data dir.

### Why This Is Small
- No protocol changes required between frontend and shell.
- Reuses existing PTY output stream, adding persistence hooks and a simple selector UI.

### Risks / Questions
- Should command replay be opt-in per command vs full replay?
- Need clear redaction strategy for secrets in persisted transcripts.

## Feature 2: Command Snippets + One-Click Run

### Problem
Developers and agents repeatedly run the same setup/debug commands (build/test/log/watch) and often copy/paste from notes or previous sessions.

### Proposal
Add a per-project snippet palette with one-click execution into the active session.

### UX
- Add `Snippets` button near terminal controls.
- Snippet entry fields:
  - label
  - command text
  - optional cwd override
- Actions:
  - `Run` (send command + newline to active PTY)
  - `Insert` (paste into prompt without running)
  - `Edit` / `Delete`
- Include a tiny starter set generated from common scripts when available (for example from `package.json` scripts).

### Scope Guardrails
- Plain command execution only (no complex templating in v1).
- Snippets stored locally in app data dir.
- Explicit confirmation for commands marked as destructive by simple heuristics (for example `rm`, `git reset --hard`).

### Why This Is Small
- Mostly frontend state/UI plus a small command-injection action using existing input path.
- High daily utility with low backend complexity.

### Risks / Questions
- Heuristic safety prompts may miss edge cases.
- Need a simple import/export format if teams want sharing later.

## Comparison and Suggested Order
1. Session History + Resume
2. Command Snippets + One-Click Run

Rationale: history/resume directly addresses context continuity and naturally pairs with future agent workflows (inspect prior outputs, relaunch from known cwd). Snippets is lower risk and can ship quickly in parallel or immediately after.

## Success Signals (Pre-Spec)
- Developers can recover prior session context in under 15 seconds.
- Reduced repeated setup commands per restart.
- No measurable startup delay from history indexing at small scale.

## Next Step
If approved, convert these into an OpenSpec change proposal with:
- capability boundaries (storage, privacy, replay semantics)
- acceptance criteria for history listing, transcript viewing, and resume behavior
- acceptance criteria for snippet creation, run/insert flows, and safety prompts
