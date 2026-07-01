//! Minimal stdio JSON-RPC MCP face. The daemon remains the data plane.

use crate::daemon;
use indiana_core::compile::CompiledPayload;
use indiana_core::markers::{kind_matches_filter, parse_kind, TABLE};
use indiana_core::parser::Status;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                writeln!(stdout, "{}", error(json!(null), -32700, &e.to_string()))?;
                continue;
            }
        };
        let Some(id) = req.get("id").cloned() else {
            continue;
        };
        let method = req
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let response = match method {
            "initialize" => ok(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "indiana", "version": env!("CARGO_PKG_VERSION") }
                }),
            ),
            "tools/list" => ok(id, tools()),
            "tools/call" => call_tool(id, req.get("params").cloned().unwrap_or_default()),
            _ => error(id, -32601, "method not found"),
        };
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn call_tool(id: Value, params: Value) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or_default();
    match name {
        "list_pending_indianas" => match current_payload() {
            Some(mut payload) => {
                payload
                    .markers
                    .retain(|m| !matches!(m.status, Some(Status::Done | Status::Failed)));
                tool_text(id, serde_json::to_string_pretty(&payload).unwrap())
            }
            None => daemon_unavailable(id),
        },
        "read_indiana" => {
            let needle = args.get("id").and_then(Value::as_str).unwrap_or_default();
            match current_payload() {
                Some(payload) => match payload
                    .markers
                    .into_iter()
                    .find(|m| m.id.as_deref() == Some(needle))
                {
                    Some(marker) => tool_text(id, serde_json::to_string_pretty(&marker).unwrap()),
                    None => error(id, -32602, "unknown indiana id"),
                },
                None => daemon_unavailable(id),
            }
        }
        "read_payload" => {
            let filter = match args.get("kind").and_then(Value::as_str) {
                Some(token) => match parse_kind(token) {
                    Some(k) => Some(k),
                    None => return error(id, -32602, &format!("unknown marker kind '{token}'")),
                },
                None => None,
            };
            match current_payload() {
                Some(mut payload) => {
                    if let Some(k) = filter {
                        payload.markers.retain(|m| kind_matches_filter(k, m.kind));
                    }
                    tool_text(id, serde_json::to_string_pretty(&payload).unwrap())
                }
                None => daemon_unavailable(id),
            }
        }
        "marker_grammar" => tool_text(id, marker_grammar()),
        _ => error(id, -32602, "unknown tool"),
    }
}

/// Fetch the compiled payload from the daemon — the single data plane.
/// No local fallback by design: the MCP face must not compute (IN_MCP.md,
/// IN_PRINCIPLES.md). `None` when no daemon answers; callers return an error
/// rather than scanning the process cwd, which is not a monitored repo.
fn current_payload() -> Option<CompiledPayload> {
    daemon::client_payload()
}

fn daemon_unavailable(id: Value) -> Value {
    error(
        id,
        -32603,
        "indiana daemon unavailable: start it with `indiana serve`",
    )
}

fn tools() -> Value {
    json!({
        "tools": [
            tool("list_pending_indianas", "List pending compiled Indiana markers."),
            tool("read_indiana", "Read one compiled Indiana marker by tracked id."),
            {
                "name": "read_payload",
                "description": "Read the full compiled Indiana payload. Optionally filter by marker kind.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "description": "Filter to this marker kind (e.g. action, note, fix)" }
                    }
                }
            },
            tool("marker_grammar", "Read marker grammar and prompt meanings.")
        ]
    })
}

fn tool(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": {} }
    })
}

fn marker_grammar() -> String {
    TABLE
        .iter()
        .map(|spec| {
            format!(
                "::{} ({:?}) shorts={:?} tracked={} command_type={}",
                spec.long, spec.kind, spec.shorts, spec.tracked, spec.command_type
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn tool_text(id: Value, text: String) -> Value {
    ok(
        id,
        json!({
            "content": [{ "type": "text", "text": text }],
            "isError": false
        }),
    )
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}
