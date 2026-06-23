//! Integration test: boots a real daemon, drives the full socket protocol roundtrip.
//! Uses the shared `indiana-protocol` types — any field mismatch fails at compile time.
//!
//! Run: `INDIANA_HOME=$(mktemp -d) cargo test --test protocol_roundtrip`

use std::process::{Command, Stdio};
use std::time::Duration;

/// Path to the compiled `indiana` binary — set by cargo at compile time.
const DAEMON_BIN: &str = env!("CARGO_BIN_EXE_indiana");

/// Wait for the daemon socket to appear (poll up to 10 s).
fn wait_for_socket(home: &std::path::Path) -> bool {
    let sock = home.join("indiana.sock");
    for _ in 0..40 {
        if sock.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    false
}

#[test]
fn protocol_roundtrip() {
    // ----- setup: temp directories -----
    let temp_home = tempfile::TempDir::new().expect("temp home dir");
    let temp_repo = tempfile::TempDir::new().expect("temp repo dir");

    // Canonicalize to resolve /var → /private/var symlinks on macOS.
    let home_path = temp_home.path().canonicalize().expect("canonicalize home");
    let repo_path = temp_repo.path().canonicalize().expect("canonicalize repo");

    // Write a markdown file with markers the daemon can scan.
    let md = repo_path.join("notes.md");
    std::fs::write(
        &md,
        "::action fix the thing\nSome context here.\n::note remember this\n",
    )
    .expect("write test markdown");
    // ----- start daemon -----
    let mut child = Command::new(DAEMON_BIN)
        .arg("serve")
        .env("INDIANA_HOME", &home_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn daemon");

    assert!(
        wait_for_socket(&home_path),
        "daemon socket never appeared"
    );

    // Point the client functions at the same INDIANA_HOME.
    std::env::set_var("INDIANA_HOME", &home_path);

    // ----- add folder -----
    let add_resp =
        indiana::daemon::client_add(&repo_path).expect("client_add should succeed");
    assert!(add_resp.added, "folder should be newly added");
    assert!(!add_resp.index.markers.is_empty(), "index should have markers");

    // ----- status -----
    let status_resp = indiana::daemon::client_status().expect("client_status should succeed");
    assert_eq!(status_resp.folders.len(), 1, "should have 1 monitored folder");
    assert_eq!(
        status_resp.folders[0].path,
        repo_path.to_string_lossy(),
        "folder path should match"
    );
    assert!(status_resp.folders[0].count > 0, "should have markers");

    // ----- copy -----
    let copy_resp = indiana::daemon::client_copy(&repo_path)
        .expect("client_copy should succeed");
    assert!(!copy_resp.text.is_empty(), "copy text should not be empty");
    assert!(
        copy_resp.text.contains("fix the thing"),
        "copy text should contain marker prompt"
    );

    // ----- remove -----
    let remove_resp =
        indiana::daemon::client_remove(&repo_path).expect("client_remove should succeed");
    assert!(remove_resp.removed, "folder should be removed");
    assert!(
        remove_resp.index.markers.is_empty(),
        "index should be empty after remove"
    );

    // ----- shutdown -----
    indiana::daemon::client_shutdown().expect("client_shutdown should succeed");

    // Wait for daemon to exit.
    let status = child.wait().expect("daemon should exit cleanly");
    assert!(status.success(), "daemon exit status should be 0");
}
