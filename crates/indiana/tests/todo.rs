//! IN_TEST.md E12 — the Montmartre todo CLI (SQLite, repo-local).

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_indiana");

/// A fresh temp repo root per test. The todo db lands at `<root>/.indiana/montmartre/todos.db`.
fn root() -> std::path::PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "indiana-todo-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run `indiana todo add` in human mode and return the new id from stdout.
fn add(root: &std::path::Path, domain: &str, todo: &str, deps: &[&str]) -> String {
    let mut cmd = Command::new(BIN);
    cmd.arg("todo")
        .arg("add")
        .arg("--root")
        .arg(root)
        .arg("--domain")
        .arg(domain);
    for d in deps {
        cmd.arg("--dependency").arg(d);
    }
    cmd.arg(todo);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "todo add failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

// E12: add then list then delete round-trips through SQLite.
#[test]
fn test_todo_add_list_delete() {
    let root = root();
    let id = add(&root, "core", "Keep Montmartre todos in SQLite", &[]);

    let out = Command::new(BIN)
        .arg("todo")
        .arg("list")
        .arg("--root")
        .arg(&root)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("core"));
    assert!(stdout.contains(&id));
    assert!(stdout.contains("Keep Montmartre todos in SQLite"));

    let out = Command::new(BIN)
        .arg("todo")
        .arg("delete")
        .arg("--root")
        .arg(&root)
        .arg(&id)
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = Command::new(BIN)
        .arg("todo")
        .arg("list")
        .arg("--root")
        .arg(&root)
        .arg("--json")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.trim(), "[]", "list should be empty after delete");
    fs::remove_dir_all(&root).ok();
}

// E12: `--json` emits a stable, flat array for agents.
#[test]
fn test_todo_list_json() {
    let root = root();
    let a = add(&root, "core", "first task", &[]);
    let b = add(&root, "ui", "second task", &[]);

    let out = Command::new(BIN)
        .arg("todo")
        .arg("list")
        .arg("--root")
        .arg(&root)
        .arg("--json")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let arr = v.as_array().expect("list --json is an array");
    assert_eq!(arr.len(), 2);
    let ids: Vec<&str> = arr
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&a.as_str()));
    assert!(ids.contains(&b.as_str()));
    for t in arr {
        assert!(t["todo"].is_string());
        assert!(t["domain"].is_string());
        assert!(t["dependencies"].is_array());
    }
    fs::remove_dir_all(&root).ok();
}

// E12: `add --json` returns the full row including dependencies.
#[test]
fn test_todo_add_json() {
    let root = root();
    let first = add(&root, "core", "do the first thing", &[]);

    let out = Command::new(BIN)
        .arg("todo")
        .arg("add")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("core")
        .arg("--dependency")
        .arg(&first)
        .arg("--json")
        .arg("do the second thing")
        .output()
        .unwrap();
    assert!(out.status.success());
    let t: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    assert!(t["id"].as_str().unwrap().len() > 4);
    assert_eq!(t["todo"].as_str().unwrap(), "do the second thing");
    assert_eq!(t["domain"].as_str().unwrap(), "core");
    let deps = t["dependencies"].as_array().unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].as_str().unwrap(), first);
    fs::remove_dir_all(&root).ok();
}

// E12: a todo is at most 29 whitespace-delimited words.
#[test]
fn test_todo_max_29_words() {
    let root = root();
    let twenty_nine = (0..29)
        .map(|n| format!("word{n}"))
        .collect::<Vec<_>>()
        .join(" ");
    let out = Command::new(BIN)
        .arg("todo")
        .arg("add")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("core")
        .arg(&twenty_nine)
        .output()
        .unwrap();
    assert!(out.status.success(), "29 words should be accepted");

    let thirty = format!("{twenty_nine} extra");
    let out = Command::new(BIN)
        .arg("todo")
        .arg("add")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("core")
        .arg(&thirty)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("29 words"), "stderr should mention 29: {stderr}");
    fs::remove_dir_all(&root).ok();
}

// E12: empty todo and empty domain are rejected.
#[test]
fn test_todo_empty_todo_and_domain() {
    let root = root();
    let out = Command::new(BIN)
        .arg("todo")
        .arg("add")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("core")
        .arg("")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("invalid todo"));

    let out = Command::new(BIN)
        .arg("todo")
        .arg("add")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("")
        .arg("do a thing")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("invalid domain"));
    fs::remove_dir_all(&root).ok();
}

// E12: a dependency must reference an existing todo.
#[test]
fn test_todo_unknown_dependency() {
    let root = root();
    let out = Command::new(BIN)
        .arg("todo")
        .arg("add")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("core")
        .arg("--dependency")
        .arg("nope-nada")
        .arg("depends on a missing todo")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("unknown dependency"), "stderr: {stderr}");
    assert!(stderr.contains("nope-nada"));
    fs::remove_dir_all(&root).ok();
}

// E12: deleting a todo cascades dependency edges to and from it.
#[test]
fn test_todo_dependency_cascade() {
    let root = root();
    let a = add(&root, "core", "the prerequisite", &[]);
    let b = add(&root, "core", "the dependent", &[&a]);

    // B depends on A.
    let out = Command::new(BIN)
        .arg("todo")
        .arg("list")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("core")
        .arg("--json")
        .output()
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let b_row = v
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"].as_str() == Some(b.as_str()))
        .unwrap();
    assert_eq!(b_row["dependencies"].as_array().unwrap().len(), 1);

    // Delete A → the edge from B to A is removed (cascade), B survives.
    let out = Command::new(BIN)
        .arg("todo")
        .arg("delete")
        .arg("--root")
        .arg(&root)
        .arg(&a)
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = Command::new(BIN)
        .arg("todo")
        .arg("list")
        .arg("--root")
        .arg(&root)
        .arg("--json")
        .output()
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1, "only B should remain");
    let b_row = arr
        .iter()
        .find(|t| t["id"].as_str() == Some(b.as_str()))
        .unwrap();
    assert!(
        b_row["dependencies"].as_array().unwrap().is_empty(),
        "B's dependency on A should be cascaded away"
    );
    fs::remove_dir_all(&root).ok();
}

// E12: deleting a missing id is a clean failure, not a silent success.
#[test]
fn test_todo_delete_not_found() {
    let root = root();
    let out = Command::new(BIN)
        .arg("todo")
        .arg("delete")
        .arg("--root")
        .arg(&root)
        .arg("ghost-id")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("no todo 'ghost-id'"));
    fs::remove_dir_all(&root).ok();
}

// E12: `--domain` filters the list.
#[test]
fn test_todo_list_domain_filter() {
    let root = root();
    let _a = add(&root, "core", "core task", &[]);
    let b = add(&root, "ui", "ui task", &[]);

    let out = Command::new(BIN)
        .arg("todo")
        .arg("list")
        .arg("--root")
        .arg(&root)
        .arg("--domain")
        .arg("ui")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("ui"));
    assert!(stdout.contains(&b));
    assert!(!stdout.contains("core task"));
    fs::remove_dir_all(&root).ok();
}

// E12: the db lands at the documented Montmartre path.
#[test]
fn test_todo_db_path() {
    let root = root();
    add(&root, "core", "place the db", &[]);
    let db = root.join(".indiana").join("montmartre").join("todos.db");
    assert!(db.exists(), "todos.db should exist at {db:?}");
    fs::remove_dir_all(&root).ok();
}
