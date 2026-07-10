//! Socket client — talks to the Indiana daemon over the Unix socket.
//! Also provides Tauri commands (thin glue, no logic).

use indiana_protocol::*;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

fn indiana_home() -> PathBuf {
    std::env::var("INDIANA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut h = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
            h.push(".indiana");
            h
        })
}

fn socket_path() -> PathBuf {
    let mut p = indiana_home();
    p.push("indiana.sock");
    p
}

fn focus_path() -> PathBuf {
    let mut p = indiana_home();
    p.push("focus.txt");
    p
}

fn send_recv(req: &Request) -> Option<String> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let body = serde_json::to_string(req).ok()?;
    writer.write_all(body.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    Some(line.trim().to_string())
}

pub fn status() -> Option<StatusResponse> {
    let raw = send_recv(&Request::Status)?;
    serde_json::from_str(&raw).ok()
}

pub fn add_folder(path: &Path) -> Option<bool> {
    let raw = send_recv(&Request::Add {
        path: path.to_path_buf(),
    })?;
    serde_json::from_str::<AddResponse>(&raw)
        .map(|r| r.added)
        .ok()
}

pub fn remove_folder(path: &Path) -> Option<bool> {
    let raw = send_recv(&Request::Remove {
        path: path.to_path_buf(),
    })?;
    serde_json::from_str::<RemoveResponse>(&raw)
        .map(|r| r.removed)
        .ok()
}

pub fn copy_folder(path: &Path, kind: Option<&str>, group: Option<u64>) -> Option<String> {
    let raw = send_recv(&Request::Copy {
        path: path.to_path_buf(),
        kind: kind.map(|k| k.to_string()),
        group,
    })?;
    serde_json::from_str::<CopyResponse>(&raw)
        .map(|r| r.text)
        .ok()
}

pub fn run_group(path: &Path, group: u64) -> Option<RunGroupResponse> {
    let raw = send_recv(&Request::RunGroup {
        path: path.to_path_buf(),
        group,
    })?;
    serde_json::from_str(&raw).ok()
}

pub fn shutdown() -> bool {
    send_recv(&Request::Shutdown).is_some()
}
/// Spawn the bundled daemon sidecar if not already running, and wait for it to
/// answer `Status`. The single source of truth for "did it come up" — the caller
/// (and UI) just awaits this. On failure it returns the real reason, not a bare
/// `false`, so the panel can show why instead of a generic "failed to start".
pub fn spawn_daemon(app: &tauri::AppHandle) -> Result<(), String> {
    // Already alive — nothing to do.
    if status().is_some() {
        return Ok(());
    }

    let (_rx, _child) = app
        .shell()
        .sidecar("indiana")
        .map_err(|e| format!("locate sidecar: {e}"))?
        .args(["serve"])
        .spawn()
        .map_err(|e| format!("spawn sidecar: {e}"))?;

    // Poll for the daemon to bind its socket and answer Status.
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if status().is_some() {
            return Ok(());
        }
    }
    Err("daemon did not respond within 10s".into())
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub fn status() -> Result<StatusResponse, String> {
        super::status().ok_or_else(|| "daemon not running".into())
    }

    #[tauri::command]
    pub fn add_folder(path: String) -> Result<bool, String> {
        Ok(super::add_folder(Path::new(&path)).unwrap_or(false))
    }

    #[tauri::command]
    pub fn remove_folder(path: String) -> Result<bool, String> {
        Ok(super::remove_folder(Path::new(&path)).unwrap_or(false))
    }

    #[tauri::command]
    pub fn copy_folder(
        path: String,
        kind: Option<String>,
        group: Option<u64>,
    ) -> Result<String, String> {
        super::copy_folder(Path::new(&path), kind.as_deref(), group)
            .ok_or_else(|| "daemon not running".into())
    }

    #[tauri::command]
    pub fn run_group(path: String, group: u64) -> Result<RunGroupResponse, String> {
        super::run_group(Path::new(&path), group).ok_or_else(|| "daemon not running".into())
    }

    #[tauri::command]
    pub fn shutdown() -> Result<bool, String> {
        Ok(super::shutdown())
    }

    #[tauri::command]
    pub async fn spawn_sidecar(app: tauri::AppHandle) -> Result<(), String> {
        // spawn_daemon blocks up to 10s polling for the socket. Run it on the
        // blocking pool so it doesn't park an async worker and freeze other IPC
        // (the 3s status poll, copy) while the daemon comes up.
        tauri::async_runtime::spawn_blocking(move || super::spawn_daemon(&app))
            .await
            .map_err(|e| e.to_string())?
    }

    /// Signal that a native dialog is about to open (prevents auto-hide on focus loss).
    #[tauri::command]
    pub fn set_dialog_open(open: bool, app: tauri::AppHandle) {
        if let Some(state) = app.try_state::<std::sync::Mutex<crate::DaemonState>>() {
            state.lock().unwrap().dialog_open = open;
        }
    }

    /// Run `indiana templates refresh <path>` via the bundled sidecar.
    /// Creates missing `.indiana/<command>/prompt.md` files without overwriting existing ones.
    #[tauri::command]
    pub async fn refresh_templates(app: tauri::AppHandle, path: String) -> Result<bool, String> {
        run_templates_sidecar(&app, "refresh", &path).await
    }

    /// Run `indiana templates replace <path>` via the bundled sidecar.
    /// Overwrites every `.indiana/indianas/<command>/prompt.md` with the
    /// embedded default — discards user edits to command templates.
    #[tauri::command]
    pub async fn replace_templates(app: tauri::AppHandle, path: String) -> Result<bool, String> {
        run_templates_sidecar(&app, "replace", &path).await
    }

    /// Spawn `indiana templates <subcmd> <path>` and surface a real reason on
    /// failure. The sidecar's stderr is captured so a non-zero exit (e.g. an
    /// unknown subcommand in a stale sidecar, or a unwritable folder) reaches
    /// the UI instead of a silent `false`.
    async fn run_templates_sidecar(
        app: &tauri::AppHandle,
        subcmd: &str,
        path: &str,
    ) -> Result<bool, String> {
        use tauri_plugin_shell::process::CommandEvent;

        let (mut rx, _child) = app
            .shell()
            .sidecar("indiana")
            .map_err(|e| format!("locate sidecar: {e}"))?
            .args(["templates", subcmd, path])
            .spawn()
            .map_err(|e| format!("spawn sidecar: {e}"))?;

        let mut stderr = String::new();
        let mut exit_code: Option<i32> = None;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stderr(bytes) => stderr.push_str(&String::from_utf8_lossy(&bytes)),
                CommandEvent::Error(e) => return Err(format!("sidecar error: {e}")),
                CommandEvent::Terminated(status) => {
                    exit_code = status.code;
                    break;
                }
                _ => {}
            }
        }
        match exit_code {
            Some(0) => Ok(true),
            Some(code) => Err(format!(
                "indiana templates {subcmd} exited {code}: {}",
                stderr.trim()
            )),
            None => Err(format!(
                "indiana templates {subcmd} produced no exit event: {}",
                stderr.trim()
            )),
        }
    }

    #[tauri::command]
    pub fn home_dir() -> String {
        std::env::var("HOME").unwrap_or_default()
    }

    #[tauri::command]
    pub fn read_focus() -> String {
        std::fs::read_to_string(focus_path()).unwrap_or_default()
    }

    #[tauri::command]
    pub fn save_focus(text: String) -> Result<(), String> {
        let path = focus_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, text).map_err(|e| e.to_string())
    }
}
