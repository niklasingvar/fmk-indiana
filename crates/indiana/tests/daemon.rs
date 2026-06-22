//! IN_TEST.md E8 — daemon lifecycle. Black-box: drives the built binary with
//! INDIANA_HOME set to a temp dir, so tests never touch the real daemon.

use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_indiana");

/// Kills the spawned daemon when the test ends, pass or panic.
struct Daemon(Child);
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn unique(tag: &str) -> PathBuf {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("indiana-{tag}-{n}"));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn repo_with(body: &str) -> PathBuf {
    let d = unique("repo");
    std::fs::write(d.join("doc.md"), body).unwrap();
    d
}

fn spawn_serve(home: &Path, root: Option<&Path>) -> Daemon {
    let mut c = Command::new(BIN);
    c.env("INDIANA_HOME", home).arg("serve");
    if let Some(r) = root {
        c.arg(r);
    }
    c.stdout(Stdio::null()).stderr(Stdio::null());
    Daemon(c.spawn().unwrap())
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

fn scan_json(home: &Path) -> String {
    let out = Command::new(BIN)
        .env("INDIANA_HOME", home)
        .arg("scan")
        .arg("--json")
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

// E8: one daemon binds; a second fails cleanly.
#[test]
fn test_socket_single_bind() {
    let home = unique("home");
    let repo = repo_with("::h\n");
    let _a = spawn_serve(&home, Some(&repo));
    assert!(wait_socket(&home), "daemon A never bound the socket");

    let b = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("serve")
        .arg(&repo)
        .output()
        .unwrap();
    assert!(!b.status.success(), "second daemon should fail to bind");
    let err = String::from_utf8(b.stderr).unwrap();
    assert!(err.contains("already running"), "stderr was: {err}");
}

// E8: a stale socket file (no daemon behind it) is cleaned and rebound.
#[test]
fn test_stale_socket() {
    let home = unique("home");
    std::fs::write(home.join("indiana.sock"), b"stale").unwrap();
    let repo = repo_with("::l\n");
    let _a = spawn_serve(&home, Some(&repo));
    assert!(wait_socket(&home), "daemon did not recover the stale socket");
    assert!(scan_json(&home).contains("\"love\""));
}

// E8: config persists across daemon restarts (add via CLI, serve picks it up).
#[test]
fn test_config_persists() {
    let home = unique("home");
    let repo = repo_with("::k\n");
    let add = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("add")
        .arg(&repo)
        .output()
        .unwrap();
    assert!(add.status.success());

    let cfg = std::fs::read_to_string(home.join("config.json")).unwrap();
    let repo_name = repo.file_name().unwrap().to_str().unwrap();
    assert!(cfg.contains(repo_name), "config.json missing folder: {cfg}");

    // Fresh daemon with no root arg → must read folders from config.
    let _a = spawn_serve(&home, None);
    assert!(wait_socket(&home));
    assert!(scan_json(&home).contains("\"keep\""));
}

fn wait_until<F: Fn(&str) -> bool>(home: &Path, pred: F) -> bool {
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if pred(&scan_json(home)) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn count_markers(json: &str) -> usize {
    json.matches("\"kind\"").count()
}

// E11: a new file's markers are detected.
#[test]
fn test_watch_new_file() {
    let home = unique("home");
    let repo = repo_with(""); // doc.md with no markers
    let _a = spawn_serve(&home, Some(&repo));
    assert!(wait_socket(&home));
    std::fs::write(repo.join("new.md"), "::h\n").unwrap();
    assert!(wait_until(&home, |j| j.contains("\"hate\"")), "new file not picked up");
}

// E11: a modified file is re-scanned.
#[test]
fn test_watch_modify() {
    let home = unique("home");
    let repo = repo_with("::h\n");
    let _a = spawn_serve(&home, Some(&repo));
    assert!(wait_socket(&home));
    assert!(wait_until(&home, |j| j.contains("\"hate\"")));
    std::fs::write(repo.join("doc.md"), "::h\n::l\n").unwrap();
    assert!(wait_until(&home, |j| j.contains("\"love\"")), "modification not picked up");
}

// E11: a deleted file's markers leave the index.
#[test]
fn test_watch_delete() {
    let home = unique("home");
    let repo = repo_with("::h\n");
    let _a = spawn_serve(&home, Some(&repo));
    assert!(wait_socket(&home));
    assert!(wait_until(&home, |j| j.contains("\"hate\"")));
    std::fs::remove_file(repo.join("doc.md")).unwrap();
    assert!(wait_until(&home, |j| !j.contains("\"hate\"")), "deletion not picked up");
}

// E11: a burst of writes coalesces; final state has all markers.
#[test]
fn test_watch_debounce() {
    let home = unique("home");
    let repo = repo_with("");
    let _a = spawn_serve(&home, Some(&repo));
    assert!(wait_socket(&home));
    for i in 0..10 {
        std::fs::write(repo.join(format!("f{i}.md")), "::h\n").unwrap();
    }
    assert!(wait_until(&home, |j| count_markers(j) == 10), "burst not fully indexed");
}

// E8: a client that disconnects and reconnects sees the same state.
#[test]
fn test_client_reconnect() {
    let home = unique("home");
    let repo = repo_with("::h\n::fix yo\n");
    let _a = spawn_serve(&home, Some(&repo));
    assert!(wait_socket(&home));

    let one = scan_json(&home);
    let two = scan_json(&home);
    assert_eq!(one, two, "reconnect returned different state");
    assert!(one.contains("\"hate\"") && one.contains("\"fix\""));
}
