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

fn expect_agent_model(home: &Path, model: &str) {
    let path = home.join("config.json");
    let mut config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    config["agent"]["env"]["MOCK_ACP_EXPECT_MODEL"] = model.into();
    std::fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
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
    // The dispatch lifecycle lands in the repo's chief-of-staff action log
    // (COS_PRD.md): claimed on dispatch, done on resolution.
    assert!(
        wait_until(|| {
            std::fs::read_to_string(repo.join(".indiana/chief-of-staff/log.md"))
                .map(|log| log.contains(" claimed [") && log.contains(" done ["))
                .unwrap_or(false)
        }),
        "lifecycle missing from action log: {:?}",
        std::fs::read_to_string(repo.join(".indiana/chief-of-staff/log.md"))
    );
}

// E13: every turn leaves a durable audit record under
// `.indiana/chief-of-staff/runs/` — structured frontmatter (outcome, tokens,
// cost) plus the transcript body — indexed by one `run` line in the action
// log and served by `indiana runs --json` (the one grammar).
#[test]
fn test_run_leaves_audit_record_with_usage() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a fix the typo\n");
    write_config(&home, &repo, "succeed", true);
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    assert!(wait_until(|| git_log_count(&repo) > commits_before));

    let runs = repo.join(".indiana/chief-of-staff/runs");
    assert!(
        wait_until(|| std::fs::read_dir(&runs)
            .map(|d| d.count() > 0)
            .unwrap_or(false)),
        "no run record was written"
    );
    let entry = std::fs::read_dir(&runs).unwrap().next().unwrap().unwrap();
    let record = std::fs::read_to_string(entry.path()).unwrap();
    assert!(record.contains("outcome: done"), "record: {record}");
    assert!(
        record.contains("tokensIn: 1234") && record.contains("tokensOut: 567"),
        "per-turn tokens from the prompt response: {record}"
    );
    assert!(
        record.contains("contextUsed: 45000") && record.contains("contextSize: 200000"),
        "context window from usage_update: {record}"
    );
    assert!(record.contains("cost: 0.1234"), "record: {record}");
    assert!(
        record.contains("mock agent working"),
        "transcript survives the job: {record}"
    );

    // `indiana runs --json` is the face surface — one grammar, in Rust.
    let out = Command::new(BIN)
        .env("INDIANA_HOME", &home)
        .args(["runs", "--root"])
        .arg(&repo)
        .arg("--json")
        .output()
        .unwrap();
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let run = &listed.as_array().expect("json array")[0];
    assert_eq!(run["outcome"], "done", "runs --json: {listed}");
    assert_eq!(run["tokensIn"], 1234);
    assert_eq!(run["cost"], 0.1234);
    assert!(run["file"].as_str().unwrap().ends_with(".md"));

    // The action log indexes the record with one `run` line carrying usage.
    assert!(
        wait_until_file(&repo.join(".indiana/chief-of-staff/log.md"), |log| {
            log.lines().any(|l| {
                l.contains(" run [")
                    && l.contains("in 1234 out 567 tok")
                    && l.contains(".indiana/chief-of-staff/runs/")
            })
        }),
        "run line missing from action log: {:?}",
        std::fs::read_to_string(repo.join(".indiana/chief-of-staff/log.md"))
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

// E13: a live turn's streamed work is served over the socket as a transcript
// (`jobtranscript`), reads as agent → question → answer, and vanishes with
// the job once the turn resolves.
#[test]
fn test_job_transcript_follows_live_turn() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a choose spelling\n");
    write_config(&home, &repo, "question", true);
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));

    // The question pause holds the job open so the transcript is observable.
    let deadline = Instant::now() + Duration::from_secs(15);
    let job_id = loop {
        let jobs = request(&home, serde_json::json!({ "cmd": "jobs" }));
        if let Some(job) = jobs["jobs"].as_array().and_then(|jobs| jobs.first()) {
            if job["state"].as_str() == Some("awaiting_input") {
                break job["id"].as_str().expect("job id").to_string();
            }
        }
        assert!(Instant::now() < deadline, "agent never asked a question");
        std::thread::sleep(Duration::from_millis(100));
    };

    let transcript = request(
        &home,
        serde_json::json!({ "cmd": "jobtranscript", "job_id": job_id, "since_seq": 0 }),
    );
    assert_eq!(transcript["found"], true, "transcript {transcript:?}");
    let events = transcript["events"].as_array().unwrap();
    assert!(
        events.iter().any(|e| e["kind"] == "agent"
            && e["text"]
                .as_str()
                .unwrap_or("")
                .contains("mock agent working")),
        "expected the streamed agent chunk: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| e["kind"] == "question" && e["text"] == "Which spelling should I use?"),
        "expected the question event: {events:?}"
    );
    let next_seq = transcript["next_seq"].as_u64().unwrap();

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

    // The answer lands in the transcript (unless the turn resolves first and
    // takes the whole transcript with it — both are valid outcomes here, so
    // only assert on it while the job is still alive).
    let after = request(
        &home,
        serde_json::json!({ "cmd": "jobtranscript", "job_id": job_id, "since_seq": next_seq }),
    );
    if after["found"] == true {
        let fresh = after["events"].as_array().unwrap();
        assert!(
            fresh.iter().all(|e| e["seq"].as_u64().unwrap() >= next_seq),
            "since_seq must filter already-seen events: {fresh:?}"
        );
    }

    assert!(wait_until(|| git_log_count(&repo) > commits_before));
    // Turn ended → job gone → transcript gone.
    assert!(wait_until(|| {
        let gone = request(
            &home,
            serde_json::json!({ "cmd": "jobtranscript", "job_id": job_id, "since_seq": 0 }),
        );
        gone["found"] == false
    }));
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
    assert!(
        text.contains(":failed] expand this"),
        "the -a flag is consumed by the claim; failure keeps the message only: {text:?}"
    );
}

// E13: several `-a` markers in one repo run one turn at a time — never as
// concurrent agents racing the same working tree. The mock resolves every
// `:working` line it finds and commits once per turn, so serialized dispatch
// yields exactly one commit per marker; overlapping turns would collapse them.
#[test]
fn test_autorun_serializes_turns_per_repo() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a first thing\nmiddle\n::fix -a second thing\n");
    write_config(&home, &repo, "succeed", true);
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));

    assert!(
        wait_until(|| git_log_count(&repo) >= commits_before + 2),
        "expected two serialized turns (one commit each); commits = {}, doc.md = {:?}",
        git_log_count(&repo) - commits_before,
        std::fs::read_to_string(&doc)
    );
    let text = std::fs::read_to_string(&doc).unwrap();
    assert!(!text.contains("::fix"), "both markers resolved: {text:?}");
    assert!(
        text.contains("intro") && text.contains("middle"),
        "content intact"
    );
    assert_eq!(
        git_log_count(&repo),
        commits_before + 2,
        "exactly one commit per marker — turns did not overlap"
    );
}

// E13: one editor save burst becomes one watcher rebuild and therefore one
// agent turn for the marker, not one turn per filesystem event.
#[test]
fn test_autorun_debounces_save_burst_to_one_turn() {
    let home = unique("home");
    let repo = git_repo_with("intro\n");
    write_config(&home, &repo, "succeed", true);
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));

    for _ in 0..40 {
        std::fs::write(&doc, "intro\n::fix -a save once\n").unwrap();
        std::thread::sleep(Duration::from_millis(5));
    }

    assert!(
        wait_until(|| git_log_count(&repo) > commits_before),
        "debounced marker never dispatched; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(
        git_log_count(&repo),
        commits_before + 1,
        "one save burst must launch exactly one agent turn"
    );
    assert!(!std::fs::read_to_string(&doc).unwrap().contains("::fix"));
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

fn set_repo_model(repo: &Path, model: &str) {
    let dir = repo.join(".indiana").join("casablanca");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("settings.json"),
        serde_json::json!({ "model": model }).to_string(),
    )
    .unwrap();
}

fn set_repo_provider(repo: &Path, provider: &str) {
    let dir = repo.join(".indiana").join("casablanca");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("settings.json"),
        serde_json::json!({ "provider": provider }).to_string(),
    )
    .unwrap();
}

/// Config whose default agent cannot launch, with the mock registered as the
/// named provider `mock` — proving dispatch honors the repo's `provider`.
fn write_config_with_named_mock(home: &Path, repo: &Path) {
    let cfg = serde_json::json!({
        "folders": [repo],
        "auto_run": true,
        "agent": { "command": "/nonexistent/adapter", "args": [] },
        "agents": {
            "mock": { "command": MOCK, "args": [], "env": { "MOCK_ACP_MODE": "succeed" } }
        }
    });
    std::fs::write(
        home.join("config.json"),
        serde_json::to_string_pretty(&cfg).unwrap(),
    )
    .unwrap();
}

// E13: the repo-local `provider` picks a named agent from `config.agents`;
// the global `config.agent` (here unlaunchable) is only the unset fallback.
#[test]
fn test_autorun_repo_provider_selects_named_agent() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a use the named provider\n");
    write_config_with_named_mock(&home, &repo);
    set_repo_provider(&repo, "mock");
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    assert!(
        wait_until(|| git_log_count(&repo) > commits_before),
        "named provider never dispatched; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
    assert!(!std::fs::read_to_string(&doc).unwrap().contains("::fix"));
}

// E13: an unknown provider fails the turn (marker → failed) rather than
// silently falling back to another agent — same stance as unsupported models.
#[test]
fn test_autorun_unknown_provider_fails_turn() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a should fail\n");
    write_config(&home, &repo, "succeed", true); // global agent = working mock
    set_repo_provider(&repo, "no-such-provider");
    let doc = repo.join("doc.md");

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    assert!(
        wait_until_file(&doc, |t| t.contains(":failed]")),
        "unknown provider must fail the marker, not fall back; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
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

// E13: an adapter that gates sessions behind ACP `authenticate` (Cursor CLI's
// `agent acp`) works when `config.agent.auth_method` names its auth method —
// the client authenticates after `initialize`, before `session/new`.
#[test]
fn test_autorun_authenticates_when_configured() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a fix the typo\n");
    let cfg = serde_json::json!({
        "folders": [repo],
        "auto_run": true,
        "agent": {
            "command": MOCK,
            "args": [],
            "env": { "MOCK_ACP_MODE": "succeed", "MOCK_ACP_REQUIRE_AUTH": "1" },
            "auth_method": "mock_login",
        }
    });
    std::fs::write(
        home.join("config.json"),
        serde_json::to_string_pretty(&cfg).unwrap(),
    )
    .unwrap();
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    assert!(
        wait_until(|| git_log_count(&repo) > commits_before),
        "auth-gated agent never resolved; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
    assert!(!std::fs::read_to_string(&doc).unwrap().contains("::fix"));
}

// E13: the repo-local model is selected through ACP before the prompt runs.
#[test]
fn test_autorun_selects_repo_model() {
    let home = unique("home");
    let repo = git_repo_with("intro\n::fix -a use configured model\n");
    write_config(&home, &repo, "succeed", true);
    expect_agent_model(&home, "sonnet");
    set_repo_model(&repo, "sonnet");
    let doc = repo.join("doc.md");
    let commits_before = git_log_count(&repo);

    let _d = spawn_serve(&home);
    assert!(wait_ready(&home));
    assert!(
        wait_until(|| git_log_count(&repo) > commits_before),
        "configured model was not selected; doc.md = {:?}",
        std::fs::read_to_string(&doc)
    );
    assert!(!std::fs::read_to_string(&doc).unwrap().contains("::fix"));
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
