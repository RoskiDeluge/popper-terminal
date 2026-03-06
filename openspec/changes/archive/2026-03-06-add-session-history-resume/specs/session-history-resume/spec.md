## ADDED Requirements

### Requirement: Persist Session Records
The system SHALL persist a session record whenever a terminal session exits normally or is terminated by the application.

#### Scenario: Persist normal exit session
- **WHEN** the Popper shell exits with a process exit code
- **THEN** the system stores a session record with start time, end time, exit code, initial working directory, and last known working directory if available
- **AND** the record includes a bounded transcript preview suitable for list/detail display

#### Scenario: Persist terminated session
- **WHEN** the user or app terminates a running session
- **THEN** the system stores a session record marked terminated with no fabricated exit code
- **AND** the stored metadata still includes available timing and working-directory fields

### Requirement: Provide Searchable Session History
The system SHALL provide a history view that lists recent session records in reverse chronological order and supports text search.

#### Scenario: Open history list
- **WHEN** the user opens the history interface
- **THEN** the system displays persisted sessions newest-first with timestamp, status, and working-directory summary

#### Scenario: Filter sessions by query
- **WHEN** the user enters a search query
- **THEN** the system filters visible sessions by matching query text against indexed transcript preview and metadata fields

### Requirement: Support Resume Context Launch
The system SHALL allow users to start a new shell session from a selected historical session context.

#### Scenario: Resume from historical session
- **WHEN** the user chooses resume on a historical record with a known last working directory
- **THEN** the system starts a new Popper shell session using that working directory as the launch directory
- **AND** the system does not attempt process checkpoint restoration of the original shell process

#### Scenario: Resume fallback when directory is unavailable
- **WHEN** the recorded last working directory is missing or inaccessible
- **THEN** the system starts a new session in the default launch directory
- **AND** the UI surfaces a non-blocking notice that fallback was used

### Requirement: Enforce Retention and Privacy Controls
The system SHALL bound persisted history size and provide user controls for history persistence and deletion.

#### Scenario: Retention trim on write
- **WHEN** persisting a new session would exceed configured retention limits (record count or transcript bytes)
- **THEN** the system removes oldest records first until limits are satisfied

#### Scenario: History disabled
- **WHEN** history persistence is disabled by user setting
- **THEN** the system does not write new session records
- **AND** existing records remain unchanged unless explicit clear-history is requested

#### Scenario: Clear history
- **WHEN** the user requests clear-history
- **THEN** the system deletes all persisted session records and transcript data
- **AND** the history view updates to empty state without requiring app restart
