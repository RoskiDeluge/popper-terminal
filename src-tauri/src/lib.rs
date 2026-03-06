use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{path::BaseDirectory, AppHandle, Emitter, Manager, State};
use uuid::Uuid;

const HISTORY_FILENAME: &str = "session-history.json";
const MAX_HISTORY_SESSIONS: usize = 100;
const MAX_TRANSCRIPT_BYTES_PER_SESSION: usize = 160_000;
const MAX_TRANSCRIPT_BYTES_TOTAL: usize = 1_200_000;
const PREVIEW_BYTES: usize = 1_500;

#[derive(Default, Clone)]
struct PtyState {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

#[derive(Default, Clone)]
struct HistoryState {
    lock: Arc<Mutex<()>>,
}

struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    runtime: Arc<Mutex<SessionRuntimeMetadata>>,
}

#[derive(Default)]
struct SessionRuntimeMetadata {
    started_at_ms: i64,
    initial_cwd: Option<String>,
    last_known_cwd: Option<String>,
    transcript: String,
    terminated_by_user: bool,
    persisted: bool,
}

#[derive(Serialize)]
struct PtyDataPayload {
    session_id: String,
    data: String,
}

#[derive(Serialize)]
struct PtyExitPayload {
    session_id: String,
    status: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionRecord {
    record_id: String,
    started_at_ms: i64,
    ended_at_ms: i64,
    status: String,
    exit_code: Option<i32>,
    initial_cwd: Option<String>,
    last_known_cwd: Option<String>,
    preview: String,
    transcript: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionHistoryData {
    history_enabled: bool,
    sessions: Vec<SessionRecord>,
}

impl Default for SessionHistoryData {
    fn default() -> Self {
        Self {
            history_enabled: true,
            sessions: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct SessionHistoryItem {
    record_id: String,
    started_at_ms: i64,
    ended_at_ms: i64,
    status: String,
    exit_code: Option<i32>,
    initial_cwd: Option<String>,
    last_known_cwd: Option<String>,
    preview: String,
}

#[derive(Serialize)]
struct SessionTranscriptPayload {
    record_id: String,
    transcript: String,
}

#[derive(Serialize)]
struct HistorySettingsPayload {
    history_enabled: bool,
}

#[derive(Serialize)]
struct ResumeSessionPayload {
    session_id: String,
    used_fallback: bool,
    notice: Option<String>,
}

fn default_size(cols: Option<u16>, rows: Option<u16>) -> PtySize {
    PtySize {
        cols: cols.unwrap_or(80),
        rows: rows.unwrap_or(24),
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn trim_to_max_bytes(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }

    let mut start = input.len() - max_bytes;
    while start < input.len() && !input.is_char_boundary(start) {
        start += 1;
    }
    input[start..].to_string()
}

fn redact_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    let sensitive_markers = [
        "password",
        "passwd",
        "secret",
        "api_key",
        "apikey",
        "token",
        "authorization",
        "bearer ",
    ];

    if sensitive_markers.iter().any(|k| lower.contains(k))
        && (line.contains('=') || line.contains(':') || lower.contains("bearer "))
    {
        "[redacted sensitive output]".to_string()
    } else {
        line.to_string()
    }
}

fn redact_chunk(chunk: &str) -> String {
    let mut redacted = String::with_capacity(chunk.len());
    for segment in chunk.split_inclusive('\n') {
        let has_newline = segment.ends_with('\n');
        let line = if has_newline {
            &segment[..segment.len() - 1]
        } else {
            segment
        };
        redacted.push_str(&redact_line(line));
        if has_newline {
            redacted.push('\n');
        }
    }
    redacted
}

fn strip_ansi_and_control(chunk: &str) -> String {
    let mut cleaned = String::with_capacity(chunk.len());
    let mut chars = chunk.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            match chars.peek().copied() {
                Some('[') => {
                    let _ = chars.next();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    let _ = chars.next();
                    let mut previous = '\0';
                    for next in chars.by_ref() {
                        if next == '\u{7}' || (previous == '\u{1b}' && next == '\\') {
                            break;
                        }
                        previous = next;
                    }
                }
                Some(_) => {
                    let _ = chars.next();
                }
                None => {}
            }
            continue;
        }

        if ch == '\r' {
            cleaned.push('\n');
            continue;
        }

        if ch.is_control() && ch != '\n' && ch != '\t' {
            continue;
        }

        cleaned.push(ch);
    }

    cleaned
}

fn append_transcript(runtime: &mut SessionRuntimeMetadata, chunk: &str) {
    let stripped = strip_ansi_and_control(chunk);
    let sanitized = redact_chunk(&stripped);
    runtime.transcript.push_str(&sanitized);
    if runtime.transcript.len() > MAX_TRANSCRIPT_BYTES_PER_SESSION {
        runtime.transcript = trim_to_max_bytes(&runtime.transcript, MAX_TRANSCRIPT_BYTES_PER_SESSION);
    }
}

fn preview_from_transcript(transcript: &str) -> String {
    trim_to_max_bytes(transcript, PREVIEW_BYTES)
}

fn history_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;

    fs::create_dir_all(&base)
        .map_err(|e| format!("failed to create app data dir {}: {e}", base.display()))?;

    Ok(base.join(HISTORY_FILENAME))
}

fn load_history_from_path(path: &Path) -> Result<SessionHistoryData, String> {
    if !path.exists() {
        return Ok(SessionHistoryData::default());
    }

    let contents = fs::read_to_string(path)
        .map_err(|e| format!("failed to read history file {}: {e}", path.display()))?;
    if contents.trim().is_empty() {
        return Ok(SessionHistoryData::default());
    }

    serde_json::from_str::<SessionHistoryData>(&contents)
        .map_err(|e| format!("failed to parse history file {}: {e}", path.display()))
}

fn save_history_to_path(path: &Path, data: &SessionHistoryData) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("failed to encode history json: {e}"))?;
    fs::write(path, json).map_err(|e| format!("failed to write history file {}: {e}", path.display()))
}

fn with_history_read<R>(
    app: &AppHandle,
    history_state: &HistoryState,
    op: impl FnOnce(&SessionHistoryData) -> Result<R, String>,
) -> Result<R, String> {
    let _guard = history_state
        .lock
        .lock()
        .map_err(|_| "failed to lock history state".to_string())?;
    let path = history_file_path(app)?;
    let data = load_history_from_path(&path)?;
    op(&data)
}

fn with_history_write<R>(
    app: &AppHandle,
    history_state: &HistoryState,
    op: impl FnOnce(&mut SessionHistoryData) -> Result<R, String>,
) -> Result<R, String> {
    let _guard = history_state
        .lock
        .lock()
        .map_err(|_| "failed to lock history state".to_string())?;
    let path = history_file_path(app)?;
    let mut data = load_history_from_path(&path)?;
    let result = op(&mut data)?;
    save_history_to_path(&path, &data)?;
    Ok(result)
}

fn enforce_retention_limits(data: &mut SessionHistoryData) {
    while data.sessions.len() > MAX_HISTORY_SESSIONS {
        data.sessions.remove(0);
    }

    let mut total_bytes: usize = data.sessions.iter().map(|s| s.transcript.len()).sum();
    while total_bytes > MAX_TRANSCRIPT_BYTES_TOTAL && !data.sessions.is_empty() {
        let removed = data.sessions.remove(0);
        total_bytes = total_bytes.saturating_sub(removed.transcript.len());
    }
}

fn persist_session_record(
    app: &AppHandle,
    history_state: &HistoryState,
    mut record: SessionRecord,
) -> Result<(), String> {
    record.transcript = trim_to_max_bytes(&record.transcript, MAX_TRANSCRIPT_BYTES_PER_SESSION);
    record.preview = preview_from_transcript(&record.transcript);

    with_history_write(app, history_state, |data| {
        if !data.history_enabled {
            return Ok(());
        }

        data.sessions.push(record);
        enforce_retention_limits(data);
        Ok(())
    })
}

fn build_session_record(
    runtime: &SessionRuntimeMetadata,
    ended_at_ms: i64,
    status: &str,
    exit_code: Option<i32>,
) -> SessionRecord {
    SessionRecord {
        record_id: Uuid::new_v4().to_string(),
        started_at_ms: runtime.started_at_ms,
        ended_at_ms,
        status: status.to_string(),
        exit_code,
        initial_cwd: runtime.initial_cwd.clone(),
        last_known_cwd: runtime.last_known_cwd.clone(),
        preview: String::new(),
        transcript: runtime.transcript.clone(),
    }
}

fn persist_and_terminate_all_sessions(app: &AppHandle, pty_state: &PtyState, history_state: &HistoryState) {
    let drained_sessions = match pty_state.sessions.lock() {
        Ok(mut sessions) => sessions.drain().map(|(_, session)| session).collect::<Vec<_>>(),
        Err(_) => return,
    };

    for session in drained_sessions {
        if let Ok(mut runtime) = session.runtime.lock() {
            runtime.terminated_by_user = true;
            if !runtime.persisted {
                runtime.persisted = true;
                let record = build_session_record(&runtime, now_ms(), "terminated", None);
                let _ = persist_session_record(app, history_state, record);
            }
        }

        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
        }
    }
}

fn resolve_sidecar(app: &AppHandle) -> Result<PathBuf, String> {
    let target_triple = std::env::var("TAURI_ENV_TARGET_TRIPLE")
        .ok()
        .or_else(|| std::env::var("TARGET").ok());

    let mut candidates = Vec::new();
    let base_names = ["popper".to_string()]
        .into_iter()
        .chain(
            target_triple
                .as_ref()
                .map(|t| format!("popper-{t}"))
                .into_iter(),
        )
        .collect::<Vec<_>>();

    for name in &base_names {
        let rel = format!("bin/{name}");
        if let Ok(p) = app.path().resolve(&rel, BaseDirectory::Resource) {
            candidates.push(p);
        }
    }

    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        for name in &base_names {
            candidates.push(dir.join("bin").join(name));
        }
    }

    let dev_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("bin");
    for name in &base_names {
        candidates.push(dev_root.join(name));
    }

    for cand in &candidates {
        if cand.exists() {
            return Ok(cand.clone());
        }
    }

    Err(format!(
        "Popper sidecar not found. Tried: {}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn start_session_internal(
    app: AppHandle,
    state: &PtyState,
    history_state: &HistoryState,
    cols: Option<u16>,
    rows: Option<u16>,
    cwd_override: Option<PathBuf>,
) -> Result<String, String> {
    let size = default_size(cols, rows);
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(size).map_err(|e| e.to_string())?;

    let sidecar_path = resolve_sidecar(&app)?;
    let initial_cwd = cwd_override
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .map(|p| p.display().to_string());

    let mut cmd = CommandBuilder::new(sidecar_path);
    if let Some(cwd) = cwd_override {
        cmd.cwd(cwd);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("failed to start popper: {e}"))?;

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("failed to clone pty reader: {e}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("failed to take pty writer: {e}"))?;
    let master = pair.master;

    let runtime = Arc::new(Mutex::new(SessionRuntimeMetadata {
        started_at_ms: now_ms(),
        initial_cwd: initial_cwd.clone(),
        last_known_cwd: initial_cwd,
        transcript: String::new(),
        terminated_by_user: false,
        persisted: false,
    }));

    let writer = Arc::new(Mutex::new(writer));
    let child = Arc::new(Mutex::new(child));
    let session_id = Uuid::new_v4().to_string();

    {
        let mut sessions = state
            .sessions
            .lock()
            .map_err(|_| "failed to lock session state".to_string())?;
        sessions.insert(
            session_id.clone(),
            PtySession {
                master,
                writer: writer.clone(),
                child: child.clone(),
                runtime: runtime.clone(),
            },
        );
    }

    let app_handle = app.clone();
    let sessions = state.sessions.clone();
    let history_for_thread = history_state.clone();
    let session_for_thread = session_id.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(read) => {
                    let data = String::from_utf8_lossy(&buf[..read]).to_string();
                    if let Ok(mut runtime_guard) = runtime.lock() {
                        append_transcript(&mut runtime_guard, &data);
                    }

                    let payload = PtyDataPayload {
                        session_id: session_for_thread.clone(),
                        data,
                    };
                    if let Err(err) = app_handle.emit("pty-data", &payload) {
                        eprintln!("Failed to emit pty-data: {}", err);
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("PTY read error: {}", err);
                    break;
                }
            }
        }

        let mut exit_code: Option<i32> = None;
        if let Ok(mut child_guard) = child.lock() {
            if let Ok(status) = child_guard.wait() {
                exit_code = Some(status.exit_code() as i32);
            }
        }

        let mut status_for_ui = exit_code.unwrap_or(-1);
        let ended_at_ms = now_ms();
        let mut record_to_persist: Option<SessionRecord> = None;
        if let Ok(mut runtime_guard) = runtime.lock() {
            if !runtime_guard.persisted {
                runtime_guard.persisted = true;
                if runtime_guard.terminated_by_user {
                    status_for_ui = -1;
                    record_to_persist = Some(build_session_record(
                        &runtime_guard,
                        ended_at_ms,
                        "terminated",
                        None,
                    ));
                } else {
                    record_to_persist = Some(build_session_record(
                        &runtime_guard,
                        ended_at_ms,
                        "exited",
                        exit_code,
                    ));
                }
            }
        }

        if let Some(record) = record_to_persist {
            let _ = persist_session_record(&app_handle, &history_for_thread, record);
        }
        let _ = app_handle.emit(
            "pty-exit",
            &PtyExitPayload {
                session_id: session_for_thread.clone(),
                status: status_for_ui,
            },
        );

        if let Ok(mut sessions) = sessions.lock() {
            sessions.remove(&session_for_thread);
        }
    });

    Ok(session_id)
}

#[tauri::command]
fn start_session(
    app: AppHandle,
    state: State<PtyState>,
    history_state: State<HistoryState>,
    cols: Option<u16>,
    rows: Option<u16>,
) -> Result<String, String> {
    start_session_internal(app, &state, &history_state, cols, rows, None)
}

#[tauri::command]
fn start_session_from_history(
    app: AppHandle,
    state: State<PtyState>,
    history_state: State<HistoryState>,
    record_id: String,
    cols: Option<u16>,
    rows: Option<u16>,
) -> Result<ResumeSessionPayload, String> {
    let mut used_fallback = false;
    let preferred_cwd = with_history_read(&app, &history_state, |data| {
        let Some(record) = data.sessions.iter().find(|entry| entry.record_id == record_id) else {
            return Err("history record not found".to_string());
        };

        Ok(record
            .last_known_cwd
            .clone()
            .or_else(|| record.initial_cwd.clone()))
    })?;

    let cwd_override = preferred_cwd
        .as_ref()
        .and_then(|value| {
            let candidate = PathBuf::from(value);
            if candidate.is_dir() {
                Some(candidate)
            } else {
                used_fallback = true;
                None
            }
        });

    if preferred_cwd.is_none() {
        used_fallback = true;
    }

    let session_id = start_session_internal(app, &state, &history_state, cols, rows, cwd_override)?;
    let notice = if used_fallback {
        Some("Recorded working directory was unavailable; used default launch directory.".to_string())
    } else {
        None
    };

    Ok(ResumeSessionPayload {
        session_id,
        used_fallback,
        notice,
    })
}

#[tauri::command]
fn write_to_session(
    state: State<PtyState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock session state".to_string())?;
    let Some(session) = sessions.get_mut(&session_id) else {
        return Err("session not found".into());
    };

    let mut writer = session
        .writer
        .lock()
        .map_err(|_| "failed to lock writer".to_string())?;
    writer
        .write_all(data.as_bytes())
        .map_err(|e| format!("write error: {e}"))?;
    let _ = writer.flush();
    Ok(())
}

#[tauri::command]
fn resize_session(
    state: State<PtyState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock session state".to_string())?;
    let Some(session) = sessions.get_mut(&session_id) else {
        return Err("session not found".into());
    };

    session
        .master
        .resize(PtySize {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("resize error: {e}"))
}

#[tauri::command]
fn terminate_session(state: State<PtyState>, session_id: String) -> Result<(), String> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock session state".to_string())?;

    if let Some(session) = sessions.get(&session_id) {
        if let Ok(mut runtime) = session.runtime.lock() {
            runtime.terminated_by_user = true;
        }

        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
        }
    }

    Ok(())
}

fn history_item_matches_query(item: &SessionRecord, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let haystack = [
        item.status.to_ascii_lowercase(),
        item.preview.to_ascii_lowercase(),
        item.initial_cwd
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase(),
        item.last_known_cwd
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase(),
    ]
    .join(" ");

    haystack.contains(query)
}

#[tauri::command]
fn list_session_history(
    app: AppHandle,
    history_state: State<HistoryState>,
    query: Option<String>,
) -> Result<Vec<SessionHistoryItem>, String> {
    let query = query.unwrap_or_default().trim().to_ascii_lowercase();
    with_history_read(&app, &history_state, |data| {
        let items = data
            .sessions
            .iter()
            .rev()
            .filter(|entry| history_item_matches_query(entry, &query))
            .map(|entry| SessionHistoryItem {
                record_id: entry.record_id.clone(),
                started_at_ms: entry.started_at_ms,
                ended_at_ms: entry.ended_at_ms,
                status: entry.status.clone(),
                exit_code: entry.exit_code,
                initial_cwd: entry.initial_cwd.clone(),
                last_known_cwd: entry.last_known_cwd.clone(),
                preview: entry.preview.clone(),
            })
            .collect();
        Ok(items)
    })
}

#[tauri::command]
fn get_session_transcript(
    app: AppHandle,
    history_state: State<HistoryState>,
    record_id: String,
) -> Result<SessionTranscriptPayload, String> {
    with_history_read(&app, &history_state, |data| {
        let Some(record) = data.sessions.iter().find(|entry| entry.record_id == record_id) else {
            return Err("history record not found".to_string());
        };

        Ok(SessionTranscriptPayload {
            record_id: record.record_id.clone(),
            transcript: record.transcript.clone(),
        })
    })
}

#[tauri::command]
fn clear_session_history(app: AppHandle, history_state: State<HistoryState>) -> Result<(), String> {
    with_history_write(&app, &history_state, |data| {
        data.sessions.clear();
        Ok(())
    })
}

#[tauri::command]
fn get_history_settings(
    app: AppHandle,
    history_state: State<HistoryState>,
) -> Result<HistorySettingsPayload, String> {
    with_history_read(&app, &history_state, |data| {
        Ok(HistorySettingsPayload {
            history_enabled: data.history_enabled,
        })
    })
}

#[tauri::command]
fn set_history_enabled(
    app: AppHandle,
    history_state: State<HistoryState>,
    enabled: bool,
) -> Result<HistorySettingsPayload, String> {
    with_history_write(&app, &history_state, |data| {
        data.history_enabled = enabled;
        Ok(HistorySettingsPayload {
            history_enabled: data.history_enabled,
        })
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PtyState::default())
        .manage(HistoryState::default())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                let pty_state = app.state::<PtyState>();
                let history_state = app.state::<HistoryState>();
                persist_and_terminate_all_sessions(&app, &pty_state, &history_state);
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_session,
            start_session_from_history,
            write_to_session,
            resize_session,
            terminate_session,
            list_session_history,
            get_session_transcript,
            clear_session_history,
            get_history_settings,
            set_history_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
