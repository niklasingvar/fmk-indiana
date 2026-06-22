//! Minimal line-delimited JSON over the Unix socket (DISTRO.md IPC). One JSON
//! request line in, one JSON response line out. No HTTP, local-only.

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
