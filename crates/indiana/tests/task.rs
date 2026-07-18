//! `indiana task` + `indiana log` over the Chief of Staff tracker
//! (`.indiana/chief-of-staff/tasks.md` + `log.md`, COS_PRD.md).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_indiana");

fn tmp_root() -> PathBuf {
    let dir = tempfile::tempdir().unwrap().keep();
    dir
}

fn run(root: &Path, args: &[&str]) -> (String, String, bool) {
    let out = Command::new(BIN)
        .args(args)
        .args(["--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.success(),
    )
}

#[test]
fn test_add_list_done_round_trip() {
    let root = tmp_root();
    let (id, _, ok) = run(&root, &["task", "add", "review the notes"]);
    assert!(ok);
    let id = id.trim().to_string();
    assert!(!id.is_empty());

    // Default queue is human; default list shows open tasks.
    let (out, _, ok) = run(&root, &["task", "list"]);
    assert!(ok);
    assert!(out.contains("Human"), "got: {out}");
    assert!(out.contains(&id));
    assert!(out.contains("review the notes"));

    let (out, _, ok) = run(&root, &["task", "done", &id]);
    assert!(ok);
    assert_eq!(out.trim(), id);

    // Done tasks leave the default view but survive in --state all.
    let (out, _, _) = run(&root, &["task", "list"]);
    assert!(!out.contains(&id));
    let (out, _, _) = run(&root, &["task", "list", "--state", "all"]);
    assert!(out.contains(&id));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn test_queue_filter_and_json_shape() {
    let root = tmp_root();
    run(&root, &["task", "add", "--queue", "agent", "agent job"]);
    run(&root, &["task", "add", "--queue", "human", "human job"]);

    let (out, _, ok) = run(&root, &["task", "list", "--queue", "agent", "--json"]);
    assert!(ok);
    let tasks: serde_json::Value = serde_json::from_str(&out).unwrap();
    let tasks = tasks.as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["queue"], "agent");
    assert_eq!(tasks[0]["state"], "open");
    assert_eq!(tasks[0]["text"], "agent job");
    assert!(tasks[0]["id"].as_str().unwrap().len() > 3);
    assert!(tasks[0].get("origin").is_none(), "hand-added has no origin");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn test_done_unknown_id_fails() {
    let root = tmp_root();
    let (_, err, ok) = run(&root, &["task", "done", "no-such-id"]);
    assert!(!ok);
    assert!(err.contains("no task"), "got: {err}");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn test_hand_edited_file_survives() {
    let root = tmp_root();
    run(&root, &["task", "add", "first"]);
    let tracker = root.join(".indiana/chief-of-staff/tasks.md");
    let mut text = fs::read_to_string(&tracker).unwrap();
    text.push_str("\nfree-form operator note\n");
    fs::write(&tracker, &text).unwrap();

    run(&root, &["task", "add", "second"]);
    let after = fs::read_to_string(&tracker).unwrap();
    assert!(after.contains("free-form operator note"));
    assert!(after.contains("first"));
    assert!(after.contains("second"));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn test_scan_capture_visible_to_task_list() {
    let root = tmp_root();
    fs::write(root.join("doc.md"), "::task wire the panel\n::action review it\n").unwrap();
    let ok = Command::new(BIN)
        .args(["scan", root.to_str().unwrap()])
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok);

    let (out, _, _) = run(&root, &["task", "list", "--json"]);
    let tasks: serde_json::Value = serde_json::from_str(&out).unwrap();
    let tasks = tasks.as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    let by_text = |s: &str| {
        tasks
            .iter()
            .find(|t| t["text"] == s)
            .unwrap_or_else(|| panic!("missing task {s}"))
    };
    assert_eq!(by_text("wire the panel")["queue"], "agent");
    assert_eq!(by_text("review it")["queue"], "human");
    assert_eq!(by_text("wire the panel")["origin"]["path"], "doc.md");
    // The injected source id is the tracker id (join key).
    let source = fs::read_to_string(root.join("doc.md")).unwrap();
    let id = by_text("wire the panel")["id"].as_str().unwrap();
    assert!(source.contains(&format!("::task[{id}]")), "got: {source}");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn test_log_tail_order_and_json() {
    let root = tmp_root();
    let (id1, _, _) = run(&root, &["task", "add", "one"]);
    let (id2, _, _) = run(&root, &["task", "add", "two"]);
    run(&root, &["task", "done", id2.trim()]);

    let (out, _, ok) = run(&root, &["log"]);
    assert!(ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3, "got: {out}");
    assert!(lines[0].contains("task-add") && lines[0].contains(id1.trim()));
    assert!(lines[2].contains("task-done") && lines[2].contains(id2.trim()));

    // -n limits from the tail; --json is machine-shaped.
    let (out, _, _) = run(&root, &["log", "-n", "1", "--json"]);
    let entries: serde_json::Value = serde_json::from_str(&out).unwrap();
    let entries = entries.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["event"], "task-done");
    assert_eq!(entries[0]["id"], id2.trim());
    assert!(entries[0]["ts"].as_str().unwrap().len() >= 16);
    fs::remove_dir_all(&root).ok();
}
