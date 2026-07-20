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
    /// The ACP agent the daemon drives for auto-run (IN_AUTORUN.md). The
    /// default when a repo names no `provider` in its Casablanca settings.
    #[serde(default)]
    pub agent: AgentConfig,
    /// Named agents selectable per repo via the `provider` Casablanca setting.
    /// Overrides the built-in providers (claude, opencode, cursor) on name
    /// collision; a name found in neither fails the turn.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub agents: HashMap<String, AgentConfig>,
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
    /// ACP auth method id to pass to `authenticate` after `initialize`.
    /// Adapters that gate sessions behind login (Cursor CLI: `cursor_login`)
    /// need it; the default claude-code-acp does not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<String>,
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
            auth_method: None,
        }
    }
}

/// The launch config for a built-in provider name, all verified against the
/// real adapters (IN_AUTORUN.md). `claude` is the same as the global default.
fn builtin_provider(name: &str) -> Option<AgentConfig> {
    let (command, args, auth_method) = match name {
        "claude" => (default_agent_command(), default_agent_args(), None),
        "opencode" => ("opencode".into(), vec!["acp".into()], None),
        "cursor" => (
            "agent".into(),
            vec!["acp".into()],
            Some("cursor_login".into()),
        ),
        _ => return None,
    };
    Some(AgentConfig {
        command,
        args,
        env: HashMap::new(),
        auth_method,
    })
}

impl Config {
    /// The agent behind a provider name: the user's `agents` map first, then
    /// the built-ins. `None` means unknown — the caller fails the turn rather
    /// than silently substituting another agent.
    pub fn provider_agent(&self, name: &str) -> Option<AgentConfig> {
        self.agents
            .get(name)
            .cloned()
            .or_else(|| builtin_provider(name))
    }

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
    fn test_provider_agent_builtins() {
        let cfg = Config::default();
        let opencode = cfg.provider_agent("opencode").unwrap();
        assert_eq!(opencode.command, "opencode");
        assert_eq!(opencode.args, vec!["acp"]);
        assert_eq!(opencode.auth_method, None);
        let cursor = cfg.provider_agent("cursor").unwrap();
        assert_eq!(cursor.command, "agent");
        assert_eq!(cursor.args, vec!["acp"]);
        assert_eq!(cursor.auth_method.as_deref(), Some("cursor_login"));
        let claude = cfg.provider_agent("claude").unwrap();
        assert_eq!(claude.command, "npx");
        assert!(cfg.provider_agent("goose").is_none(), "unknown → None");
    }

    #[test]
    fn test_provider_agent_config_overrides_builtin() {
        let cfg: Config = serde_json::from_str(
            r#"{ "agents": { "cursor": { "command": "/opt/agent", "args": [] },
                             "mine": { "command": "/opt/mine", "args": ["acp"] } } }"#,
        )
        .unwrap();
        assert_eq!(cfg.provider_agent("cursor").unwrap().command, "/opt/agent");
        assert_eq!(cfg.provider_agent("mine").unwrap().command, "/opt/mine");
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
