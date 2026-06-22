//! Minimal line-delimited JSON over the Unix socket (DISTRO.md IPC). One JSON
//! request line in, one JSON response line out. No HTTP, local-only.

use indiana_core::index::Index;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    /// Return the daemon's current index of its monitored folders.
    Scan,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub index: Index,
}
