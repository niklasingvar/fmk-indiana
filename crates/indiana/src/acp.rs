//! Minimal synchronous ACP client (IN_AUTORUN.md). The daemon spawns an ACP
//! agent adapter (default: `npx -y @zed-industries/claude-code-acp`) and drives
//! one turn over newline-delimited JSON-RPC on the child's stdio — the same
//! manual JSON-RPC stance as `mcp.rs`, no async runtime pulled into the daemon.
//!
//! Protocol: Agent Client Protocol v1 (agentclientprotocol.com). We are the
//! *client*; the adapter is the *agent*. One turn is: `initialize` →
//! `session/new` → `session/prompt`. While the prompt is in flight the agent
//! streams `session/update` notifications and may call back into us with
//! `session/request_permission` (we auto-grant — full autonomy) or `fs/*`
//! (we serve directly). The turn ends when `session/prompt` returns a
//! `stopReason`.

use crate::config::AgentConfig;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const PROTOCOL_VERSION: i64 = 1;

/// Standard install locations checked before falling back to PATH — a daemon
/// launched by launchd inherits a bare PATH (docs/DISTRO.md, same lesson as the
/// menulet and Casablanca binary resolution).
fn resolve_command(command: &str) -> PathBuf {
    if command.contains('/') {
        return PathBuf::from(command);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(home) = &home {
        candidates.push(home.join(".local/bin").join(command));
        candidates.push(home.join(".bun/bin").join(command));
    }
    candidates.push(PathBuf::from("/opt/homebrew/bin").join(command));
    candidates.push(PathBuf::from("/usr/local/bin").join(command));
    candidates
        .into_iter()
        .find(|p| p.exists())
        // Fall back to the bare name; Command searches PATH.
        .unwrap_or_else(|| PathBuf::from(command))
}

/// A spawned ACP agent adapter and its stdio, mid-conversation.
pub struct AcpAgent<W: Write> {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    next_id: i64,
    log: W,
}

impl<W: Write> AcpAgent<W> {
    /// Spawn the adapter with piped stdio. `log` receives a trace of every
    /// JSON-RPC line (for `~/.indiana/dispatch/<id>.log`).
    pub fn spawn(cfg: &AgentConfig, mut log: W) -> io::Result<Self> {
        let bin = resolve_command(&cfg.command);
        let _ = writeln!(log, "# spawn {} {:?}", bin.display(), cfg.args);
        let mut child = Command::new(&bin)
            .args(&cfg.args)
            .envs(&cfg.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("cannot launch agent '{}': {e}", bin.display()),
                )
            })?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        Ok(Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
            next_id: 0,
            log,
        })
    }

    /// Drive one full turn. Returns the turn's `stopReason` (e.g. `end_turn`).
    /// `on_update` receives every `session/update` notification's params so the
    /// caller can project the agent's streamed work (transcript).
    pub fn run_turn<F, U>(
        &mut self,
        cwd: &Path,
        prompt: &str,
        model: Option<&str>,
        on_elicitation: &mut F,
        on_update: &mut U,
    ) -> io::Result<String>
    where
        F: FnMut(&Value) -> io::Result<Value>,
        U: FnMut(&Value),
    {
        self.call(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "clientCapabilities": {
                    "fs": { "readTextFile": true, "writeTextFile": true },
                    "elicitation": { "form": {} },
                },
            }),
            on_elicitation,
            on_update,
        )?;
        let session = self.call(
            "session/new",
            json!({ "cwd": cwd.display().to_string(), "mcpServers": [] }),
            on_elicitation,
            on_update,
        )?;
        let session_id = session
            .get("sessionId")
            .and_then(Value::as_str)
            .ok_or_else(|| io::Error::other("session/new returned no sessionId"))?
            .to_string();
        if let Some(model) = model {
            let config_id = session
                .get("configOptions")
                .and_then(Value::as_array)
                .and_then(|options| {
                    options.iter().find(|option| {
                        option.get("category").and_then(Value::as_str) == Some("model")
                            || option.get("id").and_then(Value::as_str) == Some("model")
                    })
                })
                .and_then(|option| option.get("id"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    io::Error::other(
                        "configured model cannot be selected: agent exposes no model option",
                    )
                })?;
            self.call(
                "session/set_config_option",
                json!({
                    "sessionId": session_id,
                    "configId": config_id,
                    "value": model,
                }),
                on_elicitation,
                on_update,
            )?;
        }
        let result = self.call(
            "session/prompt",
            json!({
                "sessionId": session_id,
                "prompt": [ { "type": "text", "text": prompt } ],
            }),
            on_elicitation,
            on_update,
        )?;
        Ok(result
            .get("stopReason")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string())
    }

    /// Send a request and pump the connection until its response arrives,
    /// servicing any agent-initiated requests in between.
    fn call<F, U>(
        &mut self,
        method: &str,
        params: Value,
        on_elicitation: &mut F,
        on_update: &mut U,
    ) -> io::Result<Value>
    where
        F: FnMut(&Value) -> io::Result<Value>,
        U: FnMut(&Value),
    {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))?;
        loop {
            let msg = self.read_message()?;
            // A message with a `method` is an inbound request/notification.
            if let Some(inbound) = msg.get("method").and_then(Value::as_str) {
                let m = inbound.to_string();
                // A request carries an id; a notification has none. The one
                // notification we project is `session/update` (agent chunks,
                // tool calls); everything is still logged either way.
                if let Some(req_id) = msg.get("id").cloned() {
                    self.handle_request(&m, req_id, msg.get("params").cloned(), on_elicitation)?;
                } else if m == "session/update" {
                    on_update(msg.get("params").unwrap_or(&Value::Null));
                }
                continue;
            }
            // Otherwise it is a response. Ours?
            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                if let Some(err) = msg.get("error") {
                    return Err(io::Error::other(format!("acp {method} error: {err}")));
                }
                return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
            }
            // A stray response to a request we no longer await — ignore.
        }
    }

    /// Handle an agent→client request. Full autonomy: permission requests are
    /// auto-granted; `fs/*` are served against the working tree.
    fn handle_request<F>(
        &mut self,
        method: &str,
        id: Value,
        params: Option<Value>,
        on_elicitation: &mut F,
    ) -> io::Result<()>
    where
        F: FnMut(&Value) -> io::Result<Value>,
    {
        let params = params.unwrap_or(Value::Null);
        let result = match method {
            "session/request_permission" => grant_permission(&params),
            "fs/read_text_file" => fs_read(&params),
            "fs/write_text_file" => fs_write(&params),
            "elicitation/create" => on_elicitation(&params),
            other => {
                // Unsupported capability (terminal/*): tell the agent so it
                // falls back to its own tools.
                return self.send(&json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("unsupported: {other}") },
                }));
            }
        };
        match result {
            Ok(value) => self.send(&json!({ "jsonrpc": "2.0", "id": id, "result": value })),
            Err(e) => self.send(&json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": e.to_string() },
            })),
        }
    }

    fn send(&mut self, msg: &Value) -> io::Result<()> {
        let line = msg.to_string();
        let _ = writeln!(self.log, "--> {line}");
        self.stdin.write_all(line.as_bytes())?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()
    }

    fn read_message(&mut self) -> io::Result<Value> {
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line)?;
            if n == 0 {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "agent closed stdio",
                ));
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let _ = writeln!(self.log, "<-- {trimmed}");
            return serde_json::from_str(trimmed).map_err(io::Error::other);
        }
    }
}

impl<W: Write> Drop for AcpAgent<W> {
    fn drop(&mut self) {
        // The turn is over; do not leak the adapter process.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Choose an allow option and return the `selected` outcome. Prefers a
/// session-wide grant, then a one-shot; never selects a reject.
fn grant_permission(params: &Value) -> io::Result<Value> {
    let options = params.get("options").and_then(Value::as_array);
    let pick = options.and_then(|opts| {
        let by_kind = |kind: &str| {
            opts.iter()
                .find(|o| o.get("kind").and_then(Value::as_str) == Some(kind))
        };
        by_kind("allow_always")
            .or_else(|| by_kind("allow_once"))
            .or_else(|| {
                opts.iter().find(|o| {
                    !matches!(
                        o.get("kind").and_then(Value::as_str),
                        Some("reject_once") | Some("reject_always")
                    )
                })
            })
    });
    match pick.and_then(|o| o.get("optionId")).and_then(Value::as_str) {
        Some(option_id) => Ok(json!({
            "outcome": { "outcome": "selected", "optionId": option_id }
        })),
        // No grantable option offered — cancel rather than hang.
        None => Ok(json!({ "outcome": { "outcome": "cancelled" } })),
    }
}

fn fs_read(params: &Value) -> io::Result<Value> {
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::other("fs/read_text_file: no path"))?;
    let content = std::fs::read_to_string(path)?;
    Ok(json!({ "content": content }))
}

fn fs_write(params: &Value) -> io::Result<Value> {
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::other("fs/write_text_file: no path"))?;
    let content = params.get("content").and_then(Value::as_str).unwrap_or("");
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grant_permission_prefers_allow_always() {
        let params = json!({
            "options": [
                { "optionId": "once", "name": "Once", "kind": "allow_once" },
                { "optionId": "always", "name": "Always", "kind": "allow_always" },
                { "optionId": "no", "name": "Reject", "kind": "reject_once" },
            ]
        });
        let out = grant_permission(&params).unwrap();
        assert_eq!(out["outcome"]["outcome"], "selected");
        assert_eq!(out["outcome"]["optionId"], "always");
    }

    #[test]
    fn test_grant_permission_falls_back_to_once() {
        let params = json!({
            "options": [
                { "optionId": "no", "name": "Reject", "kind": "reject_once" },
                { "optionId": "once", "name": "Once", "kind": "allow_once" },
            ]
        });
        let out = grant_permission(&params).unwrap();
        assert_eq!(out["outcome"]["optionId"], "once");
    }

    #[test]
    fn test_grant_permission_no_option_cancels() {
        let params = json!({ "options": [
            { "optionId": "no", "name": "Reject", "kind": "reject_once" },
        ] });
        let out = grant_permission(&params).unwrap();
        assert_eq!(out["outcome"]["outcome"], "cancelled");
    }

    #[test]
    fn test_resolve_command_absolute_passthrough() {
        assert_eq!(resolve_command("/opt/x/acp"), PathBuf::from("/opt/x/acp"));
    }
}
