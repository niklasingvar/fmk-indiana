//! The monitored-folders list (IN_DAEMON.md): input, not derived state. It is
//! the one legitimate non-source persistence — user choice, kept across
//! restarts (IN_PRINCIPLES.md carve-out). Lives in `~/.indiana/config.json`.

use crate::paths::{config_path, indiana_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub folders: Vec<PathBuf>,
    /// Global kill-switch for auto-run dispatch (IN_AUTORUN.md). Default off:
    /// a `::fix -a` marker dispatches only when this is enabled. This is the
    /// roadmap's "pausable" control — flip it off to batch instead of run.
    #[serde(default)]
    pub auto_run: bool,
    /// The ACP agent the daemon drives for auto-run (IN_AUTORUN.md).
    #[serde(default)]
    pub agent: AgentConfig,
}

/// How to launch the ACP agent adapter the daemon speaks to over stdio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// The adapter command. Default: `npx`, which fetches and caches Claude
    /// Code's ACP adapter on first use (see `args`) — no separate install step
    /// beyond Node (DISTRO.md). Set to an installed bin (e.g. `claude-code-acp`)
    /// with empty `args` to skip npx.
    #[serde(default = "default_agent_command")]
    pub command: String,
    /// Args passed before the ACP handshake. Default runs the adapter via npx.
    #[serde(default = "default_agent_args")]
    pub args: Vec<String>,
    /// Extra environment for the adapter process (e.g. auth). Merged onto the
    /// daemon's own environment.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_agent_command() -> String {
    "npx".to_string()
}

fn default_agent_args() -> Vec<String> {
    vec![
        "-y".to_string(),
        "@zed-industries/claude-code-acp".to_string(),
    ]
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            command: default_agent_command(),
            args: default_agent_args(),
            env: HashMap::new(),
        }
    }
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

    /// Remove a folder by its canonical path. Returns false if it wasn't present.
    pub fn remove_folder(&mut self, path: &Path) -> bool {
        let len_before = self.folders.len();
        self.folders.retain(|p| p != path);
        self.folders.len() < len_before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_old_config_parses_with_defaults() {
        // A pre-auto-run config.json has only `folders`. New fields default:
        // auto-run off, adapter = claude-code-acp.
        let cfg: Config = serde_json::from_str(r#"{"folders":["/tmp/x"]}"#).unwrap();
        assert_eq!(cfg.folders, vec![PathBuf::from("/tmp/x")]);
        assert!(!cfg.auto_run, "auto-run defaults off (opt-in)");
        assert_eq!(cfg.agent.command, "npx");
        assert_eq!(
            cfg.agent.args,
            vec!["-y", "@zed-industries/claude-code-acp"]
        );
    }

    #[test]
    fn test_auto_run_and_agent_round_trip() {
        let mut cfg = Config::default();
        cfg.auto_run = true;
        cfg.agent.command = "/opt/bin/my-acp".to_string();
        cfg.agent.args = vec!["--flag".to_string()];
        cfg.agent
            .env
            .insert("ANTHROPIC_API_KEY".to_string(), "x".to_string());
        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert!(back.auto_run);
        assert_eq!(back.agent.command, "/opt/bin/my-acp");
        assert_eq!(back.agent.args, vec!["--flag".to_string()]);
        assert_eq!(back.agent.env.get("ANTHROPIC_API_KEY").unwrap(), "x");
    }

    #[test]
    fn test_default_config_is_off() {
        let cfg = Config::default();
        assert!(!cfg.auto_run);
        assert_eq!(cfg.agent.command, "npx");
        assert_eq!(
            cfg.agent.args,
            vec!["-y", "@zed-industries/claude-code-acp"]
        );
    }
}
