//! M10 — MCP stdio face. Black-box JSON-RPC over stdin/stdout.

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_indiana");

fn fixture(body: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "indiana-mcp-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("doc.md"), body).unwrap();
    dir
}

struct Daemon(Child);
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_serve(home: &Path, root: &Path) -> Daemon {
    let child = Command::new(BIN)
        .env("INDIANA_HOME", home)
        .arg("serve")
        .arg(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    Daemon(child)
}

fn wait_socket(home: &Path) -> bool {
    let sock = home.join("indiana.sock");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if UnixStream::connect(&sock).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

fn rpc(cwd: &PathBuf, home: Option<&Path>, request: &str) -> String {
    let mut command = Command::new(BIN);
    command
        .current_dir(cwd)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    if let Some(home) = home {
        command.env("INDIANA_HOME", home);
    }
    let mut child = command.spawn().unwrap();
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
    let out = rpc(
        &dir,
        None,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
    );
    assert!(out.contains("list_pending_indianas"));
    assert!(out.contains("read_payload"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn test_mcp_read_payload() {
    let home = fixture("");
    let dir = fixture("Fix this ::fix tighten\n");
    let prompt = dir.join(".indiana/indianas/fix/prompt.md");
    std::fs::create_dir_all(prompt.parent().unwrap()).unwrap();
    std::fs::write(
        prompt,
        "---\nstatus: draft\npurpose: test\napproval: pending\ncommand: fix\ncommand_type: test\n---\n\nRepair this. {message}\n",
    )
    .unwrap();
    let _daemon = spawn_serve(&home, &dir);
    assert!(wait_socket(&home));
    let out = rpc(
        &dir,
        Some(&home),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"read_payload","arguments":{}}}"#,
    );
    assert!(out.contains("Repair this. tighten"));
    assert!(out.contains("scope_content"));
    std::fs::remove_dir_all(home).ok();
    std::fs::remove_dir_all(dir).ok();
}
