//! Where Indiana keeps its socket and config (IN_DAEMON.md, DISTRO.md).
//! `INDIANA_HOME` overrides the default `~/.indiana` — used by tests so they
//! never touch the real daemon's directory.

use std::path::PathBuf;

pub fn indiana_dir() -> PathBuf {
    if let Ok(d) = std::env::var("INDIANA_HOME") {
        return PathBuf::from(d);
    }
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".indiana")
}

pub fn socket_path() -> PathBuf {
    indiana_dir().join("indiana.sock")
}

pub fn config_path() -> PathBuf {
    indiana_dir().join("config.json")
}
