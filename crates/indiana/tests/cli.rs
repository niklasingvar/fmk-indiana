//! IN_TEST.md E10 — the CLI face. Runs the built binary against a fixture.

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_indiana");

fn fixture(body: &str) -> std::path::PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "indiana-cli-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
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

#[test]
fn test_cli_copy_uses_folder_template() {
    let dir = fixture("Fix this ::fix tighten\n");
    let prompt = dir.join(".indiana/fix/prompt.md");
    fs::create_dir_all(prompt.parent().unwrap()).unwrap();
    fs::write(
        prompt,
        "---\nstatus: draft\npurpose: test\napproval: pending\ncommand: fix\ncommand_type: test\n---\n\nRepair this. {message}\n",
    )
    .unwrap();
    let out = Command::new(BIN).arg("copy").arg(&dir).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Repair this. tighten"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_cli_add_scaffolds_folder_templates_idempotently() {
    let state = fixture("");
    let dir = fixture("Fix this ::fix tighten\n");
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &state)
        .arg("add")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let prompt = dir.join(".indiana/fix/prompt.md");
    assert!(prompt.exists());

    fs::write(&prompt, "custom").unwrap();
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &state)
        .arg("add")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(fs::read_to_string(&prompt).unwrap(), "custom");
    fs::remove_dir_all(&state).ok();
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_cli_add_then_user_template_edit_affects_copy() {
    let state = fixture("");
    let dir = fixture("Fix this ::fix tighten\n");
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &state)
        .arg("add")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());

    fs::write(
        dir.join(".indiana/fix/prompt.md"),
        "---\nstatus: draft\npurpose: test\napproval: pending\ncommand: fix\ncommand_type: test\n---\n\nRepair this. {message}\n",
    )
    .unwrap();
    let out = Command::new(BIN).arg("copy").arg(&dir).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Repair this. tighten"));
    fs::remove_dir_all(&state).ok();
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_cli_templates_refresh_restores_missing_without_overwrite() {
    let dir = fixture("Fix this ::fix tighten\n");
    indiana_core::templates::init_folder_indiana(&dir).unwrap();
    let fix = dir.join(".indiana/fix/prompt.md");
    let note = dir.join(".indiana/note/prompt.md");
    fs::write(&fix, "custom").unwrap();
    fs::remove_file(&note).unwrap();

    let out = Command::new(BIN)
        .arg("templates")
        .arg("refresh")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(fs::read_to_string(&fix).unwrap(), "custom");
    assert!(note.exists());
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

// E10 + CLI-first: `indiana help` must stay accurate — snapshot enforced.
#[test]
fn test_cli_help_snapshot() {
    let out = Command::new(BIN).arg("help").output().unwrap();
    assert!(out.status.success());
    let got = String::from_utf8(out.stdout).unwrap();
    let want = include_str!("cli_help.snap");
    // Normalize: trim trailing whitespace per line, then compare.
    let norm = |s: &str| -> String {
        s.lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let got_norm = norm(&got);
    let want_norm = norm(want);
    if got_norm != want_norm {
        eprintln!("=== snapshot mismatch ===");
        eprintln!("--- expected (snapshot) ---");
        eprintln!("{want_norm}");
        eprintln!("--- got ---");
        eprintln!("{got_norm}");
        panic!(
            "cli help snapshot mismatch — update tests/cli_help.snap if the change is intentional"
        );
    }
}

// ── copy --latest cursor tests ──

/// Run twice: first copies N>0, second copies 0 (nothing new).
#[test]
fn test_cli_copy_latest_twice() {
    let home = fixture("bad ::h\nimportant ::note remember\n");
    // Use temp INDIANA_HOME so the cursor file is isolated.
    let out1 = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    assert!(out1.status.success());
    let stdout1 = String::from_utf8(out1.stdout).unwrap();
    assert!(
        stdout1.contains("bad"),
        "first copy should include hate: {stdout1}"
    );
    assert!(
        stdout1.contains("remember"),
        "first copy should include note"
    );

    let out2 = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    assert!(out2.status.success());
    let stdout2 = String::from_utf8(out2.stdout).unwrap();
    assert!(
        !stdout2.contains("bad") && !stdout2.contains("remember"),
        "second copy should be empty: {stdout2}"
    );
    fs::remove_dir_all(&home).ok();
}

/// Composition: `--kind action --latest` copies only action, leaves hate/note uncopied.
#[test]
fn test_cli_copy_kind_action_latest() {
    let home = fixture("bad ::h\nFix ::action ship it\nRemember ::note log\n");
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--kind")
        .arg("action")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("ship it"),
        "should include action: {stdout}"
    );
    assert!(!stdout.contains("bad"), "should exclude hate: {stdout}");
    assert!(
        !stdout.contains("Remember"),
        "should exclude note: {stdout}"
    );

    // A follow-up `copy --latest` (no kind filter) should still surface hate and note.
    let out2 = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    assert!(out2.status.success());
    let stdout2 = String::from_utf8(out2.stdout).unwrap();
    assert!(
        stdout2.contains("bad"),
        "follow-up should include hate: {stdout2}"
    );
    assert!(
        stdout2.contains("Remember"),
        "follow-up should include note: {stdout2}"
    );
    assert!(
        !stdout2.contains("ship it"),
        "action already copied → excluded"
    );
    fs::remove_dir_all(&home).ok();
}

/// Add a marker between two `--latest` runs → second copies only the new one.
#[test]
fn test_cli_copy_latest_new_marker() {
    let home = fixture("old ::h\n");
    // First copy — picks up the hate.
    let _ = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    // Add a new marker.
    fs::write(home.join("doc.md"), "old ::h\nnew ::fix tighten\n").unwrap();
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Check for raw token `[h]` to avoid false positives in paths.
    assert!(
        !stdout.contains("[h]"),
        "old hate should be excluded: {stdout}"
    );
    assert!(
        stdout.contains("tighten"),
        "new fix should be included: {stdout}"
    );
    fs::remove_dir_all(&home).ok();
}
/// Delete copied.json → `--latest` falls back to copy-all.
#[test]
fn test_cli_copy_latest_missing_cursor() {
    let home = fixture("bad ::h\nimportant ::note log\n");
    // First copy — records the cursor.
    let _ = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    // Delete the cursor.
    let cursor_path = home.join("copied.json");
    assert!(
        cursor_path.exists(),
        "cursor file should exist after first copy"
    );
    fs::remove_file(&cursor_path).unwrap();
    // Second copy — should copy everything again.
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("bad"));
    assert!(stdout.contains("log"));
    fs::remove_dir_all(&home).ok();
}

/// Plain `indiana copy` (no --latest) still works and does not filter by cursor.
#[test]
fn test_cli_copy_plain_ignores_cursor() {
    let home = fixture("bad ::h\n");
    // First, copy with --latest to plant a cursor.
    let _ = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg("--latest")
        .arg(&home)
        .output()
        .unwrap();
    // Plain copy should still include everything.
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .arg("copy")
        .arg(&home)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("bad"),
        "plain copy should include hate: {stdout}"
    );
    fs::remove_dir_all(&home).ok();
}
