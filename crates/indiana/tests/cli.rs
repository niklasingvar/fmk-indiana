//! IN_TEST.md E10 — the CLI face. Runs the built binary against a fixture.

use std::fs;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_indiana");

fn fixture(body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "indiana-cli-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("doc.md"), body).unwrap();
    dir
}

// E10: `indiana scan` lists every marker.
#[test]
fn test_cli_scan() {
    let dir = fixture("::h\n::fix tighten this\n`::l quoted` is ignored\n");
    let out = Command::new(BIN).arg("scan").arg(&dir).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("hate"));
    assert!(stdout.contains("fix"));
    assert!(stdout.contains("tighten this"));
    assert!(!stdout.contains("love"), "inline-code marker should be ignored");
    fs::remove_dir_all(&dir).ok();
}

// E10 + D4: `--json` emits structured markers.
#[test]
fn test_cli_scan_json() {
    let dir = fixture("::action[bear-mouse:done] ship it\n");
    let out = Command::new(BIN).arg("scan").arg(&dir).arg("--json").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\"kind\": \"action\""));
    assert!(stdout.contains("\"id\": \"bear-mouse\""));
    assert!(stdout.contains("\"status\": \"done\""));
    fs::remove_dir_all(&dir).ok();
}
