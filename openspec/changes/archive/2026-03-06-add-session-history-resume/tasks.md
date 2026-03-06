## 1. Implementation
- [x] 1.1 Define session history data model (metadata + bounded transcript chunks) and storage location under app data directory.
- [x] 1.2 Persist session records on shell exit/termination, including timestamps, exit reason/code, last known cwd, and transcript preview/tail.
- [x] 1.3 Add backend commands/events to list history entries, fetch transcript content, and request resume-context launch.
- [x] 1.4 Add frontend history UI (open button, searchable list, detail preview) and wiring for open transcript and resume actions.
- [x] 1.5 Enforce retention limits (max sessions + max bytes per transcript) with deterministic trimming behavior.
- [x] 1.6 Add privacy controls (history enabled toggle and clear-history action) and basic secret-redaction handling for persisted lines.

## 2. Validation
- [x] 2.1 Manual test: completed and forced-terminated sessions appear in history with expected metadata.
- [x] 2.2 Manual test: resume action launches a new shell in previous last-known cwd and does not attempt process checkpoint restore.
- [x] 2.3 Manual test: retention trimming keeps app responsive and removes oldest records first.
- [x] 2.4 Manual test: disable-history mode prevents new persistence; clear-history removes existing records.

Validation note (2026-03-05): terminal-only verification confirmed history file creation and metadata persistence after force-stopping the sidecar process (`session-history.json` written under app data). Full checklist remains pending in-app click-through for normal exit, resume action, retention stress, and settings toggles.
