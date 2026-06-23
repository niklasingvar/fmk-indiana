//! Protocol types — duplicates of the daemon's wire format (primitives only).
//! Never imports `indiana_core`; keeps the Tauri build independent.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    Status,
    Add { path: PathBuf },
    Remove { path: PathBuf },
    Copy { path: PathBuf },
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub path: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub folders: Vec<FolderInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddResponse {
    pub added: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveResponse {
    pub removed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopyResponse {
    pub text: String,
}
