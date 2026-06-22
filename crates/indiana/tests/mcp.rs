//! M10 — MCP stdio face. Black-box JSON-RPC over stdin/stdout.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_indiana");

fn fixture(body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "indiana-mcp-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("doc.md"), body).unwrap();
    dir
}

fn rpc(cwd: &PathBuf, request: &str) -> String {
    let mut child = Command::new(BIN)
        .current_dir(cwd)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(request.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn test_mcp_tools_list() {
    let dir = fixture("");
    let out = rpc(&dir, r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
    assert!(out.contains("list_pending_indianas"));
    assert!(out.contains("read_payload"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn test_mcp_read_payload() {
    let dir = fixture("Fix this ::fix tighten\n");
    let out = rpc(
        &dir,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"read_payload","arguments":{}}}"#,
    );
    assert!(out.contains("Fix this. tighten"));
    assert!(out.contains("scope_content"));
    std::fs::remove_dir_all(dir).ok();
}
