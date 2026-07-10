//! IN_TEST.md E13 — auto-run dispatch. Black-box: drives the built daemon with
//! INDIANA_HOME set to a temp dir and `config.agent` pointed at the test-only
//! mock ACP agent, so no real Claude Code adapter (or network) is involved.
//!
//! Built only under `--features test-support` (the mock bin lives behind it).
#![cfg(feature = "test-support")]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_indiana");
const MOCK: &str = env!("CARGO_BIN_EXE_mock-acp-agent");

struct Daemon(Child);
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn unique(tag: &str) -> PathBuf {
    // Keep the path short: a Unix socket path under `home` must fit SUN_LEN
    // (~104 bytes on macOS), and the macOS temp dir prefix is already long.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let _ = tag;
    let d = std::env::temp_dir().join(format!(
        "iar-{n}-{}",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A git repo with `doc.md`, so the mock agent can commit its resolution.
fn git_repo_with(body: &str) -> PathBuf {
    let d = unique("repo");
    std::fs::write(d.join("doc.md"), body).unwrap();
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(&d)
            .args(args)
            .output()
            .unwrap();
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&[
        "-c",
        "user.email=t@t",
        "-c",
        "user.name=T",
        "commit",
        "-q",
        "-m",
        "init",
    ]);
    d
}

/// Write a config that monitors `repo`, turns auto-run on, and points the agent
/// at the mock adapter in the given mode.
fn write_config(home: &Path, repo: &Path, mode: &str, auto_run: bool) {
    let cfg = serde_json::json!({
        "folders": [repo],
        "auto_run": auto_run,
        "agent": {
            "command": MOCK,
            "args": [],
            "env": { "MOCK_ACP_MODE": mode }
        }
    });
    std::fs::write(
        home.join("config.json"),
        serde_json::to_string_pretty(&cfg).unwrap(),
    )
    .unwrap();
}

fn spawn_serve(home: &Path) -> Daemon {
    let log = std::fs::File::create(home.join("serve.log")).unwrap();
    let c = Command::new(BIN)
        .env("INDIANA_HOME", home)
        .arg("serve")
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    Daemon(c)
}

fn request(home: &Path, body: serde_json::Value) -> serde_json::Value {
    let stream = UnixStream::connect(home.join("indiana.sock")).unwrap();
    let mut writer = stream.try_clone().unwrap();
    writer.write_all(body.to_string().as_bytes()).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.flush().unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(line.trim()).unwrap()
}

fn wait_ready(home: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let ok = Command::new(BIN)
            .env("INDIANA_HOME", home)
            .arg("status")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

fn wait_until_file<F: Fn(&str) -> bool>(path: &Path, pred: F) -> bool {
    wait_until(|| {
        std::fs::read_to_string(path)
            .map(|t| pred(&t))
            .unwrap_or(false)
    })
}

fn wait_until<F: Fn() -> bool>(pred: F) -> bool {
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if pred() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn git_log_count(repo: &Path) -> usize {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap_or(0)
}

// E13: `::fix -a` is claimed to `:working`, dispatched, and the mock agent
// resolves it — the marker line is removed and a commit lands.
#[test]
fn test_autorun_success_resolves_and_commits() {
    let home = unique("home");
    let repo = git_repo_with("intro paragraph\n::fix -a fix the typo\ntrailer\n");
    write_config(&home, &repo, "succeed", true);
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(
        wait_ready(&home),
        "daemon never came up; serve.log = {:?}",
        std::fs::read_to_string(home.join("serve.log"))
    );

    // Wait on the commit, which the agent makes *after* removing the marker
    // line — so this also proves the resolution landed (no read-before-commit
    // race on the assertions below).
    assert!(
        wait_until(|| git_log_count(&repo) > commits_before),
        "agent never committed; doc.md = {:?}, serve.log = {:?}",
        std::fs::read_to_string(&doc),
        std::fs::read_to_string(home.join("serve.log"))
    );
    let text = std::fs::read_to_string(&doc).unwrap();
    assert!(
        !text.contains("::fix"),
        "marker line should be gone: {text:?}"
    );
    // Surrounding content is intact — only the marker line was removed.
    assert!(text.contains("intro paragraph"));
    assert!(text.contains("trailer"));
    assert!(
        !text.contains(":working]"),
        "no in-flight bracket should remain"
    );
}

// E13: an ACP form question surfaces as a live daemon job, accepts a human
// answer over the socket, and resumes the same turn.
#[test]
fn test_autorun_question_pauses_and_resumes() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a choose spelling\n");
    write_config(&home, &repo, "question", true);
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));

    let deadline = Instant::now() + Duration::from_secs(15);
    let job = loop {
        let jobs = request(&home, serde_json::json!({ "cmd": "jobs" }));
        if let Some(job) = jobs["jobs"].as_array().and_then(|jobs| jobs.first()) {
            if job["state"].as_str() == Some("awaiting_input")
                && job["question"]["message"].as_str() == Some("Which spelling should I use?")
            {
                break job.clone();
            }
        }
        assert!(Instant::now() < deadline, "agent never asked a question");
        std::thread::sleep(Duration::from_millis(100));
    };
    let job_id = job["id"].as_str().expect("awaiting job supplies id");

    let answer = request(
        &home,
        serde_json::json!({
            "cmd": "answerjob",
            "job_id": job_id,
            "action": "accept",
            "answer": "colour",
        }),
    );
    assert_eq!(answer["accepted"], true);
    assert!(wait_until(|| git_log_count(&repo) > commits_before));
    assert!(!std::fs::read_to_string(doc).unwrap().contains("::fix"));
}

// E13: when the agent fails to resolve, the marker is left `:failed`, not
// re-dispatched, and the surrounding text is preserved.
#[test]
fn test_autorun_failure_marks_failed() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::elaborate -a expand this\n");
    write_config(&home, &repo, "fail", true);
    let doc = repo.join("doc.md");

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));

    assert!(
        wait_until_file(&doc, |t| t.contains(":failed]")),
        "marker was never marked failed; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
    let text = std::fs::read_to_string(&doc).unwrap();
    assert!(
        text.contains("::elaborate["),
        "the marker survives, now failed"
    );
    assert!(text.contains("expand this"), "message preserved");
    assert!(!text.contains("-a"), "the -a flag was consumed on claim");
}

// E13: with auto-run off (the pausable switch), a `-a` marker is left untouched
// — no claim, no dispatch.
#[test]
fn test_autorun_disabled_leaves_marker() {
    let home = unique("home");
    let repo = git_repo_with("::fix -a do nothing yet\n");
    write_config(&home, &repo, "succeed", false);
    let doc = repo.join("doc.md");

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    // Give the daemon time to (not) act.
    std::thread::sleep(Duration::from_millis(800));

    let text = std::fs::read_to_string(&doc).unwrap();
    assert_eq!(
        text, "::fix -a do nothing yet\n",
        "marker must be untouched when auto-run is off"
    );
}

/// Write the repo's per-repo auto-run opt-in (`.indiana/casablanca/settings.json`).
fn set_repo_autorun(repo: &Path, enabled: bool) {
    let dir = repo.join(".indiana").join("casablanca");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("settings.json"),
        serde_json::json!({ "autoRun": enabled }).to_string(),
    )
    .unwrap();
}

// E13: per-repo opt-in dispatches even when the global default is off.
#[test]
fn test_autorun_per_repo_enables_over_global_off() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a fix the typo\n");
    write_config(&home, &repo, "succeed", false); // global off
    set_repo_autorun(&repo, true); // repo opts in
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    assert!(
        wait_until(|| git_log_count(&repo) > commits_before),
        "per-repo autoRun did not dispatch; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
    assert!(!std::fs::read_to_string(&doc).unwrap().contains("::fix"));
}

// E13: a per-repo opt-OUT overrides a global default of on.
#[test]
fn test_autorun_per_repo_disables_over_global_on() {
    let home = unique("home");
    let repo = git_repo_with("::fix -a do nothing yet\n");
    write_config(&home, &repo, "succeed", true); // global on
    set_repo_autorun(&repo, false); // repo opts out
    let doc = repo.join("doc.md");

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(
        std::fs::read_to_string(&doc).unwrap(),
        "::fix -a do nothing yet\n",
        "per-repo autoRun:false must veto the global default"
    );
}

#[test]
fn test_group_summary_copy_and_run_one_turn() {
    let home = unique("home");
    let repo = git_repo_with(
        "::fix -1 first task\n\n::elaborate -1 second task\n\n::fix -2 leave this batch\n",
    );
    write_config(&home, &repo, "succeed", false);
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));

    let status = request(&home, serde_json::json!({ "cmd": "status" }));
    assert_eq!(status["folders"][0]["groups"][0]["group"], 1);
    assert_eq!(status["folders"][0]["groups"][0]["count"], 2);
    assert_eq!(status["folders"][0]["groups"][1]["group"], 2);
    assert_eq!(status["folders"][0]["groups"][1]["count"], 1);

    let copied = request(
        &home,
        serde_json::json!({ "cmd": "copy", "path": repo, "group": 1 }),
    );
    let text = copied["text"].as_str().unwrap();
    assert!(
        text.contains("first task"),
        "group copy response was {copied:?}"
    );
    assert!(text.contains("second task"));
    assert_eq!(text.matches("\nFix this.").count(), 1);
    assert_eq!(text.matches("\nTake action on this").count(), 1);

    let run = request(
        &home,
        serde_json::json!({ "cmd": "rungroup", "path": repo, "group": 1 }),
    );
    assert_eq!(
        run["accepted"],
        true,
        "run response {run:?}; doc {:?}",
        std::fs::read_to_string(&doc)
    );
    assert_eq!(run["count"], 2);
    assert!(
        wait_until(|| git_log_count(&repo) > commits_before),
        "group agent never committed; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
    let text = std::fs::read_to_string(&doc).unwrap();
    assert!(
        !text.contains("-1"),
        "group -1 should be resolved: {text:?}"
    );
    assert!(
        text.contains("::fix -2 leave this batch"),
        "other groups must remain: {text:?}"
    );
}

#[test]
fn test_group_failure_marks_all_survivors_failed() {
    let home = unique("home");
    let repo = git_repo_with("::fix -4 first\n\n::elaborate -4 second\n");
    write_config(&home, &repo, "fail", false);
    let doc = repo.join("doc.md");

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    let run = request(
        &home,
        serde_json::json!({ "cmd": "rungroup", "path": repo, "group": 4 }),
    );
    assert_eq!(
        run["accepted"],
        true,
        "run response {run:?}; doc {:?}",
        std::fs::read_to_string(&doc)
    );
    assert_eq!(run["count"], 2);
    assert!(
        wait_until_file(&doc, |text| text.matches(":failed]").count() == 2),
        "group survivors were not all marked failed: {:?}",
        std::fs::read_to_string(&doc)
    );
}
