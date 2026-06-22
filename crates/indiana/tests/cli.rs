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
    assert!(
        !stdout.contains("love"),
        "inline-code marker should be ignored"
    );
    fs::remove_dir_all(&dir).ok();
}

// E10 + D4: `--json` emits structured markers.
#[test]
fn test_cli_scan_json() {
    let dir = fixture("::action[bear-mouse:done] ship it\n");
    let out = Command::new(BIN)
        .arg("scan")
        .arg(&dir)
        .arg("--json")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\"kind\": \"action\""));
    assert!(stdout.contains("\"id\": \"bear-mouse\""));
    assert!(stdout.contains("\"status\": \"done\""));
    fs::remove_dir_all(&dir).ok();
}

// E10: `indiana copy` renders the compiled bundle; clipboard is best-effort.
#[test]
fn test_cli_copy() {
    let dir = fixture("Fix this ::fix tighten\n");
    let out = Command::new(BIN).arg("copy").arg(&dir).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Fix this. tighten"));
    assert!(stdout.contains("Fix this"));
    fs::remove_dir_all(&dir).ok();
}

// E10: `indiana service install` writes a launchd plist under HOME.
#[test]
fn test_cli_service_install() {
    let home = fixture("");
    let out = Command::new(BIN)
        .env("HOME", &home)
        .arg("service")
        .arg("install")
        .output()
        .unwrap();
    assert!(out.status.success());
    let plist = home
        .join("Library")
        .join("LaunchAgents")
        .join("com.niklas.indiana.plist");
    let body = fs::read_to_string(&plist).unwrap();
    assert!(body.contains("<string>com.niklas.indiana</string>"));
    assert!(body.contains("<string>serve</string>"));
    assert!(body.contains("<key>RunAtLoad</key>"));
    assert!(body.contains("<key>KeepAlive</key>"));
    fs::remove_dir_all(&home).ok();
}
