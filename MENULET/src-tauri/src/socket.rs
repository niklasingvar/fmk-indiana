//! Socket client — talks to the Indiana daemon over the Unix socket.
//! Also provides Tauri commands (thin glue, no logic).

use crate::protocol::*;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

fn indiana_home() -> PathBuf {
    std::env::var("INDIANA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut h = std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_default();
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

pub fn copy_folder(path: &Path) -> Option<String> {
    let raw = send_recv(&Request::Copy {
        path: path.to_path_buf(),
    })?;
    serde_json::from_str::<CopyResponse>(&raw)
        .map(|r| r.text)
        .ok()
}

pub fn shutdown() -> bool {
    send_recv(&Request::Shutdown).is_some()
}

pub mod commands {
    use super::*;
    use tauri_plugin_shell::ShellExt;

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
    pub fn copy_folder(path: String) -> Result<String, String> {
        super::copy_folder(Path::new(&path))
            .ok_or_else(|| "daemon not running".into())
    }

    #[tauri::command]
    pub fn shutdown() -> Result<bool, String> {
        Ok(super::shutdown())
    }

    #[tauri::command]
    pub async fn spawn_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
        let shell = app
            .shell()
            .sidecar("indiana")
            .map_err(|e| format!("sidecar not found: {e}"))?;
        let (_rx, _child) = shell
            .args(["serve"])
            .spawn()
            .map_err(|e| format!("spawn failed: {e}"))?;
        Ok(true)
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
