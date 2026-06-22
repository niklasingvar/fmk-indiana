//! The monitored-folders list (IN_DAEMON.md): input, not derived state. It is
//! the one legitimate non-source persistence — user choice, kept across
//! restarts (IN_PRINCIPLES.md carve-out). Lives in `~/.indiana/config.json`.

use crate::paths::{config_path, indiana_dir};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub folders: Vec<PathBuf>,
}

impl Config {
    /// Load config, or an empty default if absent / unreadable.
    pub fn load() -> Config {
        std::fs::read_to_string(config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        std::fs::create_dir_all(indiana_dir())?;
        std::fs::write(config_path(), serde_json::to_string_pretty(self)?)
    }

    /// Add a folder (absolute, de-duplicated). Returns false if already present.
    pub fn add_folder(&mut self, path: &Path) -> bool {
        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if self.folders.contains(&abs) {
            return false;
        }
        self.folders.push(abs);
        true
    }
}
