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
//!   - `question`: ask one ACP form question, then resolve only after the
//!     client accepts it.
//!
//! `MOCK_ACP_REQUIRE_AUTH=1` makes `session/new` fail with the ACP
//! auth-required error (-32000) until the client calls `authenticate` —
//! the behaviour of `agent acp` (Cursor CLI).

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let mode = std::env::var("MOCK_ACP_MODE").unwrap_or_else(|_| "succeed".to_string());
    let require_auth = std::env::var("MOCK_ACP_REQUIRE_AUTH").is_ok();
    let mut authenticated = false;
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut out = io::stdout();
    let mut cwd = String::new();
    let mut selected_model: Option<String> = None;

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
                    json!({
                        "protocolVersion": 1,
                        "agentCapabilities": {},
                        "authMethods": [ { "id": "mock_login", "name": "Mock Login" } ],
                    }),
                );
            }
            (Some("authenticate"), Some(id)) => {
                authenticated = msg["params"]["methodId"].as_str() == Some("mock_login");
                respond(&mut out, id, json!({}));
            }
            (Some("session/new"), Some(id)) => {
                if require_auth && !authenticated {
                    send(
                        &mut out,
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32000, "message": "Authentication required" },
                        }),
                    );
                    continue;
                }
                cwd = msg["params"]["cwd"].as_str().unwrap_or("").to_string();
                respond(
                    &mut out,
                    id,
                    json!({
                        "sessionId": "mock-session",
                        "configOptions": [model_option(selected_model.as_deref().unwrap_or("default"))],
                    }),
                );
            }
            (Some("session/set_config_option"), Some(id)) => {
                if msg["params"]["configId"].as_str() == Some("model") {
                    selected_model = msg["params"]["value"].as_str().map(str::to_string);
                }
                respond(
                    &mut out,
                    id,
                    json!({
                        "configOptions": [model_option(selected_model.as_deref().unwrap_or("default"))],
                    }),
                );
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
                let may_resolve = if mode == "question" {
                    request_question(&mut out);
                    read_question_response(&mut reader)
                } else {
                    request_permission(&mut out);
                    read_permission_response(&mut reader)
                };
                let expected_model = std::env::var("MOCK_ACP_EXPECT_MODEL").ok();
                let model_matches = expected_model.as_deref() == selected_model.as_deref()
                    || expected_model.is_none();
                if may_resolve && model_matches && (mode == "succeed" || mode == "question") {
                    resolve_markers(Path::new(&cwd));
                    commit(Path::new(&cwd));
                }
                // Usage reporting, like the real adapter: a `usage_update`
                // notification (context + cost) and per-turn token counts on
                // the prompt response.
                notify(
                    &mut out,
                    json!({
                        "sessionId": "mock-session",
                        "update": {
                            "sessionUpdate": "usage_update",
                            "used": 45000,
                            "size": 200000,
                            "cost": { "amount": 0.1234, "currency": "USD" }
                        }
                    }),
                );
                respond(
                    &mut out,
                    id,
                    json!({
                        "stopReason": "end_turn",
                        "usage": { "inputTokens": 1234, "outputTokens": 567, "totalTokens": 1801 }
                    }),
                );
            }
            // Any other request (authenticate, session/cancel, …) → empty ok.
            (Some(_), Some(id)) => respond(&mut out, id, json!({})),
            _ => {} // notification, or a response we don't need
        }
    }
}

fn model_option(current: &str) -> Value {
    json!({
        "id": "model",
        "name": "Model",
        "category": "model",
        "type": "select",
        "currentValue": current,
        "options": [
            { "value": "default", "name": "Default" },
            { "value": "sonnet", "name": "Sonnet" },
            { "value": "opus", "name": "Opus" },
            { "value": "haiku", "name": "Haiku" },
        ],
    })
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

fn request_question(out: &mut impl Write) {
    send(
        out,
        json!({
            "jsonrpc": "2.0",
            "id": 9998,
            "method": "elicitation/create",
            "params": {
                "sessionId": "mock-session",
                "mode": "form",
                "message": "Which spelling should I use?",
                "requestedSchema": {
                    "type": "object",
                    "properties": { "spelling": { "type": "string" } },
                    "required": ["spelling"]
                }
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

fn read_question_response(reader: &mut impl BufRead) -> bool {
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
        if msg.get("id").and_then(Value::as_i64) == Some(9998) {
            return msg["result"]["action"].as_str() == Some("accept")
                && msg["result"]["content"]["spelling"].as_str().is_some();
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
