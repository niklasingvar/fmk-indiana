//! Test-only mock ACP agent (IN_AUTORUN.md). Speaks the agent side of the
//! Agent Client Protocol over stdio so the auto-run dispatch path can be driven
//! end-to-end without a real Claude Code adapter. Built only under the
//! `test-support` feature; never shipped.
//!
//! Behaviour is controlled by `MOCK_ACP_MODE`:
//!   - `succeed` (default): on prompt, request one edit permission, then
//!     "resolve" the marker by deleting every `:working]` line under the
//!     session cwd and committing, and end the turn.
//!   - `fail`: request permission but leave the marker line in place, so the
//!     daemon observes a surviving `:working` marker and records `:failed`.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let mode = std::env::var("MOCK_ACP_MODE").unwrap_or_else(|_| "succeed".to_string());
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut out = io::stdout();
    let mut cwd = String::new();

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let method = msg.get("method").and_then(Value::as_str);
        let id = msg.get("id").cloned();
        match (method, id) {
            (Some("initialize"), Some(id)) => {
                respond(
                    &mut out,
                    id,
                    json!({ "protocolVersion": 1, "agentCapabilities": {} }),
                );
            }
            (Some("session/new"), Some(id)) => {
                cwd = msg["params"]["cwd"].as_str().unwrap_or("").to_string();
                respond(&mut out, id, json!({ "sessionId": "mock-session" }));
            }
            (Some("session/prompt"), Some(id)) => {
                notify(
                    &mut out,
                    json!({
                        "sessionId": "mock-session",
                        "update": {
                            "sessionUpdate": "agent_message_chunk",
                            "content": { "type": "text", "text": "mock agent working" }
                        }
                    }),
                );
                request_permission(&mut out);
                let granted = read_permission_response(&mut reader);
                if granted && mode == "succeed" {
                    resolve_markers(Path::new(&cwd));
                    commit(Path::new(&cwd));
                }
                respond(&mut out, id, json!({ "stopReason": "end_turn" }));
            }
            // Any other request (authenticate, session/cancel, …) → empty ok.
            (Some(_), Some(id)) => respond(&mut out, id, json!({})),
            _ => {} // notification, or a response we don't need
        }
    }
}

fn send(out: &mut impl Write, msg: Value) {
    let _ = writeln!(out, "{msg}");
    let _ = out.flush();
}

fn respond(out: &mut impl Write, id: Value, result: Value) {
    send(out, json!({ "jsonrpc": "2.0", "id": id, "result": result }));
}

fn notify(out: &mut impl Write, params: Value) {
    send(
        out,
        json!({ "jsonrpc": "2.0", "method": "session/update", "params": params }),
    );
}

fn request_permission(out: &mut impl Write) {
    send(
        out,
        json!({
            "jsonrpc": "2.0",
            "id": 9999,
            "method": "session/request_permission",
            "params": {
                "sessionId": "mock-session",
                "toolCall": { "toolCallId": "t1", "title": "edit", "kind": "edit", "status": "pending" },
                "options": [
                    { "optionId": "always", "name": "Allow for session", "kind": "allow_always" },
                    { "optionId": "no", "name": "Reject", "kind": "reject_once" }
                ]
            }
        }),
    );
}

/// Read messages until the client answers our permission request (id 9999).
fn read_permission_response(reader: &mut impl BufRead) -> bool {
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return false;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if msg.get("id").and_then(Value::as_i64) == Some(9999) {
            return msg["result"]["outcome"]["outcome"].as_str() == Some("selected");
        }
    }
}

/// Delete every line bearing a `:working]` bracket under `root` — the mock's
/// stand-in for the agent removing the resolved marker line.
fn resolve_markers(root: &Path) {
    for path in md_files(root) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !text.contains(":working]") {
            continue;
        }
        let kept: Vec<&str> = text.lines().filter(|l| !l.contains(":working]")).collect();
        let mut out = kept.join("\n");
        if text.ends_with('\n') && !out.is_empty() {
            out.push('\n');
        }
        let _ = std::fs::write(&path, out);
    }
}

fn commit(root: &Path) {
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["add", "-A"])
        .output();
    // Inline identity so the commit works without global git config (CI).
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "user.email=mock@indiana.test",
            "-c",
            "user.name=Mock Agent",
            "commit",
            "-m",
            "fix: auto-run resolved marker",
            "--no-gpg-sign",
        ])
        .output();
}

fn md_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if p.file_name().and_then(|n| n.to_str()) != Some(".git") {
                    stack.push(p);
                }
            } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
                out.push(p);
            }
        }
    }
    out
}
