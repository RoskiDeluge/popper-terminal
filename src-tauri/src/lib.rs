use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use tauri::{path::BaseDirectory, AppHandle, Emitter, Manager, State};
use uuid::Uuid;

#[derive(Default, Clone)]
struct PtyState {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
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

fn default_size(cols: Option<u16>, rows: Option<u16>) -> PtySize {
    PtySize {
        cols: cols.unwrap_or(80),
        rows: rows.unwrap_or(24),
        pixel_width: 0,
        pixel_height: 0,
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

    // Bundled resources
    for name in &base_names {
        let rel = format!("bin/{name}");
        if let Ok(p) = app.path().resolve(&rel, BaseDirectory::Resource) {
            candidates.push(p);
        }
    }

    // Next to the executable (dev/runtime)
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        for name in &base_names {
            candidates.push(dir.join("bin").join(name));
        }
    }

    // Dev fallback: crate bin dir
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

#[tauri::command]
fn start_session(
    app: AppHandle,
    state: State<PtyState>,
    cols: Option<u16>,
    rows: Option<u16>,
) -> Result<String, String> {
    let size = default_size(cols, rows);
    let pty_system = native_pty_system();
    let mut pair = pty_system.openpty(size).map_err(|e| e.to_string())?;

    let sidecar_path = resolve_sidecar(&app)?;
    let child = pair
        .slave
        .spawn_command(CommandBuilder::new(sidecar_path))
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
            },
        );
    }

    let app_handle = app.clone();
    let sessions = state.sessions.clone();
    let session_for_thread = session_id.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(read) => {
                    let payload = PtyDataPayload {
                        session_id: session_for_thread.clone(),
                        data: String::from_utf8_lossy(&buf[..read]).to_string(),
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

        if let Ok(mut child_guard) = child.lock() {
            let status = match child_guard.try_wait() {
                Ok(Some(_)) => 0,
                Ok(None) => -1,
                Err(_) => -1,
            };
            let _ = app_handle.emit(
                "pty-exit",
                &PtyExitPayload {
                    session_id: session_for_thread.clone(),
                    status,
                },
            );
        }

        if let Ok(mut sessions) = sessions.lock() {
            sessions.remove(&session_for_thread);
        }
    });

    Ok(session_id)
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
    writer.flush().ok();
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
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "failed to lock session state".to_string())?;
    if let Some(session) = sessions.remove(&session_id) {
        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PtyState::default())
        .invoke_handler(tauri::generate_handler![
            start_session,
            write_to_session,
            resize_session,
            terminate_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
