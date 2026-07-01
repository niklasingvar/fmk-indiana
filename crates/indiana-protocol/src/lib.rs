//! Wire protocol types shared between daemon and menulet.
//! One source of truth — no duplicated type definitions.

use indiana_core::compile::CompiledPayload;
use indiana_core::index::Index;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    /// Return the daemon's current index of its monitored folders.
    Scan,
    /// Return the daemon's compiled payload.
    Payload,
    /// Monitor a new folder: persist it, watch it, and rescan now.
    Add { path: PathBuf },
    /// Return per-folder status — paths and marker counts (menulet face).
    Status,
    /// Stop monitoring a folder: remove from config, unwatch, rebuild index.
    Remove { path: PathBuf },
    /// Return the compiled bundle for one folder as ready-to-paste text.
    /// `kind` limits to one marker kind (e.g. "action"); `None` → all kinds.
    Copy { path: PathBuf, #[serde(default)] kind: Option<String> },
    /// Graceful shutdown: ack, unlink the socket, exit.
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub index: Index,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddResponse {
    /// False when the folder was already monitored.
    pub added: bool,
    pub index: Index,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadResponse {
    pub payload: CompiledPayload,
}

/// A monitored folder + its live marker count. Computed by the daemon so the
/// menulet never counts (MENULET_PRD).
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub path: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub folders: Vec<FolderInfo>,
    /// Whether a face can cleanly stop this daemon. False when the daemon is
    /// supervised (launchd `KeepAlive`), since a `Shutdown` would be restarted.
    /// Computed by the daemon so faces never reason about lifecycle themselves.
    /// `default` keeps older daemons (no field) deserializing as not-stoppable.
    #[serde(default)]
    pub stoppable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveResponse {
    pub removed: bool,
    pub index: Index,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopyResponse {
    pub text: String,
}
