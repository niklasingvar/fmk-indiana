//! indiana — one binary, multi-mode (IN_PRD.md): serve | scan | add.
//! Faces only render; all domain logic lives in indiana_core.

mod acp;
mod casablanca;
mod config;
mod copied;
mod daemon;
mod dispatch;
mod mcp;
mod paths;
mod protocol;
mod run_record;
mod service;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use config::Config;
use indiana_core::compile::{
    compile_with_options, render_text, system_prompt_for_roots, CompileOptions,
};
use indiana_core::cos;
use indiana_core::frontmatter;
use indiana_core::index::Index;
use indiana_core::markers::{long_name, parse_kind, Queue};
use indiana_core::templates::{init_folder_indiana, replace_folder_indiana};
use indiana_core::write::WriteResult;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "indiana", about = "Scan markdown for :: markers", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the daemon: bind the socket and hold the index in memory.
    Serve {
        /// Folder to monitor for this run (default: configured folders, else cwd).
        root: Option<PathBuf>,
    },
    /// Walk markdown and list every marker. With no path, asks a running daemon.
    Scan {
        /// Repo root to scan (forces a standalone scan of this path).
        path: Option<PathBuf>,
        /// Emit JSON instead of the human list.
        #[arg(long)]
        json: bool,
        /// Scan without injecting or repairing tracked IDs.
        #[arg(long)]
        read_only: bool,
    },
    /// Add a folder to the monitored-folders config.
    Add {
        /// Folder to monitor across daemon restarts.
        path: PathBuf,
    },
    /// Remove a folder from the monitored-folders config.
    Remove {
        /// Folder to stop monitoring.
        path: PathBuf,
    },
    /// Show the daemon's monitored folders and marker counts.
    Status,
    /// Compile markers and copy the bundle to the clipboard.
    Copy {
        /// Repo root to scan (forces a standalone scan of this path).
        path: Option<PathBuf>,
        /// Copy only markers of this kind (action, note, fix, …).
        #[arg(long, value_name = "KIND")]
        kind: Option<String>,
        /// Copy only markers carrying this numeric batch label (`-1`, `-2`, …).
        #[arg(long, value_name = "N")]
        group: Option<u64>,
        /// Copy only markers tagged for this agent persona (`-m` / `-mike`),
        /// rendered with the agent's own system prompt.
        #[arg(long, value_name = "NAME")]
        agent: Option<String>,
        /// Copy only markers that have not been copied before.
        #[arg(long)]
        latest: bool,
    },
    /// Run the MCP stdio server.
    Mcp,
    /// Manage repo-local `.indiana` command templates.
    Templates {
        #[command(subcommand)]
        cmd: TemplatesCmd,
    },
    /// Manage the launchd service.
    Service {
        #[command(subcommand)]
        cmd: ServiceCmd,
    },
    /// Manage the Chief of Staff task tracker (`.indiana/chief-of-staff/tasks.md`).
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    /// Tail the Chief of Staff action log (`.indiana/chief-of-staff/log.md`).
    Log {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Number of entries to show, newest last.
        #[arg(short = 'n', default_value_t = 20)]
        lines: usize,
        /// Emit JSON for agents.
        #[arg(long)]
        json: bool,
    },
    /// List agent-run audit records (`.indiana/chief-of-staff/runs/`).
    Runs {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Number of records to show, newest first.
        #[arg(short = 'n', default_value_t = 20)]
        lines: usize,
        /// Emit JSON for faces and agents.
        #[arg(long)]
        json: bool,
    },
    /// Read or edit per-repo Casablanca settings (`.indiana/casablanca/settings.json`).
    Casablanca {
        #[command(subcommand)]
        cmd: CasablancaCmd,
    },
    /// Add default frontmatter to markdown files missing it.
    Frontmatter {
        /// Folder to lint (default: current directory).
        path: Option<PathBuf>,
        /// Prepend the default frontmatter block. Without this, only report.
        #[arg(long)]
        write: bool,
    },
}

#[derive(Subcommand)]
enum ServiceCmd {
    /// Install the launchd plist for `indiana serve`.
    Install,
}

#[derive(Subcommand)]
enum TaskCmd {
    /// Add a task by hand. Prints the new id; `--json` prints the full row.
    Add {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Queue for the task (a typed add is operator intent, so human).
        #[arg(long, default_value = "human")]
        queue: String,
        /// Emit JSON for agents.
        #[arg(long)]
        json: bool,
        /// The task text.
        text: String,
    },
    /// List tasks (default: open and working only).
    List {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Only this queue (agent | human).
        #[arg(long)]
        queue: Option<String>,
        /// Only this state (open | working | done | failed | all).
        #[arg(long)]
        state: Option<String>,
        /// Emit JSON for agents.
        #[arg(long)]
        json: bool,
    },
    /// Mark a task done by id.
    Done {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Emit JSON for agents.
        #[arg(long)]
        json: bool,
        /// Id of the task to mark done.
        id: String,
    },
}

#[derive(Subcommand)]
enum CasablancaCmd {
    /// Print the settings file path.
    Path {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Print all settings as JSON.
    Settings {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Print one setting's value. Exits non-zero if the key is unset.
    Get {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Setting key.
        key: String,
    },
    /// Set one setting. A value that parses as JSON keeps its type; else it is a string.
    Set {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Setting key.
        key: String,
        /// Setting value (JSON or plain string).
        value: String,
    },
}

#[derive(Subcommand)]
enum TemplatesCmd {
    /// Create any missing `.indiana/<command>/prompt.md` files.
    Refresh {
        /// Folder whose local command templates should be refreshed.
        path: PathBuf,
    },
    /// Overwrite every `.indiana/indianas/<command>/prompt.md` with the embedded default.
    Replace {
        /// Folder whose local command templates should be replaced.
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let matches = <Cli as CommandFactory>::command()
        .after_long_help(indiana_core::markers::kind_help_string())
        .get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    match cli.cmd {
        Cmd::Serve { root } => match daemon::serve(root) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("indiana: {e}");
                ExitCode::FAILURE
            }
        },
        Cmd::Scan {
            path,
            json,
            read_only,
        } => scan(path, json, read_only),
        Cmd::Add { path } => add(path),
        Cmd::Remove { path } => remove(path),
        Cmd::Status => status(),
        Cmd::Copy {
            path,
            kind,
            group,
            agent,
            latest,
        } => copy(path, kind, group, agent, latest),
        Cmd::Mcp => match mcp::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("indiana: mcp error: {e}");
                ExitCode::FAILURE
            }
        },
        Cmd::Templates {
            cmd: TemplatesCmd::Refresh { path },
        } => templates_refresh(path),
        Cmd::Templates {
            cmd: TemplatesCmd::Replace { path },
        } => templates_replace(path),
        Cmd::Service {
            cmd: ServiceCmd::Install,
        } => service_install(),
        Cmd::Task { cmd } => task_cmd(cmd),
        Cmd::Log { root, lines, json } => log_cmd(root, lines, json),
        Cmd::Runs { root, lines, json } => runs_cmd(root, lines, json),
        Cmd::Casablanca { cmd } => casablanca_cmd(cmd),
        Cmd::Frontmatter { path, write } => frontmatter_cmd(path, write),
    }
}

fn scan(path: Option<PathBuf>, json: bool, read_only: bool) -> ExitCode {
    // Explicit path → standalone. No path → daemon view, else cwd standalone.
    let idx = match &path {
        Some(p) if read_only => Index::build_read_only(p),
        // An explicit scan is a deliberate act on that root: inject ids AND
        // capture tasks (COS_PRD.md). Implicit cwd/copy scans only inject.
        Some(p) => {
            Index::build_with_options(p, indiana_core::index::ScanOptions::write_ids().with_capture())
                .index
        }
        None if read_only => Index::build_read_only(&PathBuf::from(".")),
        None => daemon::client_scan().unwrap_or_else(|| Index::build(&PathBuf::from("."))),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&idx).unwrap());
        return ExitCode::SUCCESS;
    }

    let mut last: Option<&PathBuf> = None;
    for m in &idx.markers {
        if last != Some(&m.path) {
            println!("{}", m.path.display());
            last = Some(&m.path);
        }
        let msg = m.message.as_deref().unwrap_or("");
        let id =
            m.id.as_deref()
                .map(|i| format!(" [{i}]"))
                .unwrap_or_default();
        println!("  {:>4} · {:<10}{id} {msg}", m.line, long_name(m.kind));
    }
    for w in &idx.warnings {
        eprintln!("warning: {w}");
    }
    // Core computes the tallies; the face only prints them.
    let c = idx.counts();
    let mut parts = Vec::new();
    for (label, n) in [
        ("hate", c.hate),
        ("love", c.love),
        ("keep", c.keep),
        ("fix", c.fix),
        ("elaborate", c.elaborate),
        ("question", c.question),
        ("note", c.note),
        ("action", c.action),
        ("todo", c.todo),
        ("delete", c.delete),
        ("prompt", c.prompt),
    ] {
        if n > 0 {
            parts.push(format!("{label}:{n}"));
        }
    }
    eprintln!(
        "{} marker(s){}{}",
        c.total(),
        if parts.is_empty() {
            String::new()
        } else {
            format!(" ({})", parts.join(" "))
        },
        if idx.warnings.is_empty() {
            String::new()
        } else {
            format!(", {} warning(s)", idx.warnings.len())
        },
    );
    ExitCode::SUCCESS
}

fn add(path: PathBuf) -> ExitCode {
    // A running daemon owns the live add: it persists config, watches the
    // folder, and rescans now. Without one, fall back to config-only so the
    // next `serve` picks it up (IN_DAEMON.md).
    if let Some(resp) = daemon::client_add(&path) {
        if resp.added {
            eprintln!(
                "indiana: monitoring {} ({} marker(s))",
                path.display(),
                resp.index.markers.len()
            );
        } else {
            eprintln!("indiana: already monitoring {}", path.display());
        }
        return ExitCode::SUCCESS;
    }

    let mut cfg = Config::load();
    if cfg.add_folder(&path) {
        if let Err(e) = cfg.save() {
            eprintln!("indiana: could not save config: {e}");
            return ExitCode::FAILURE;
        }
        let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
        if let Err(e) = init_folder_indiana(&abs) {
            eprintln!(
                "indiana: warning: could not scaffold templates in {}: {e}",
                abs.display()
            );
        }
        eprintln!(
            "indiana: monitoring {} (daemon not running; scans on next serve)",
            path.display()
        );
    } else {
        eprintln!("indiana: already monitoring {}", path.display());
    }
    ExitCode::SUCCESS
}

fn remove(path: PathBuf) -> ExitCode {
    // A running daemon owns the live remove: it persists config, un-watches,
    // and rescans now. Without one, fall back to config-only.
    if let Some(resp) = daemon::client_remove(&path) {
        if resp.removed {
            eprintln!(
                "indiana: stopped monitoring {} ({} marker(s) remaining)",
                path.display(),
                resp.index.markers.len()
            );
        } else {
            eprintln!("indiana: not monitoring {}", path.display());
        }
        return ExitCode::SUCCESS;
    }

    let mut cfg = Config::load();
    if cfg.remove_folder(&path) {
        if let Err(e) = cfg.save() {
            eprintln!("indiana: could not save config: {e}");
            return ExitCode::FAILURE;
        }
        eprintln!(
            "indiana: stopped monitoring {} (daemon not running; scans on next serve)",
            path.display()
        );
    } else {
        eprintln!("indiana: not monitoring {}", path.display());
    }
    ExitCode::SUCCESS
}

fn status() -> ExitCode {
    match daemon::client_status() {
        Some(s) => {
            if s.folders.is_empty() {
                println!("indiana: running, no folders monitored");
            } else {
                for f in &s.folders {
                    println!("{}  ({})", f.path, f.count);
                }
                println!("{} folder(s)", s.folders.len());
            }
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("indiana: daemon not running");
            ExitCode::FAILURE
        }
    }
}

fn templates_refresh(path: PathBuf) -> ExitCode {
    let abs = path.canonicalize().unwrap_or(path);
    match init_folder_indiana(&abs) {
        Ok(()) => {
            eprintln!("indiana: refreshed templates in {}", abs.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!(
                "indiana: could not refresh templates in {}: {e}",
                abs.display()
            );
            ExitCode::FAILURE
        }
    }
}

fn templates_replace(path: PathBuf) -> ExitCode {
    let abs = path.canonicalize().unwrap_or(path);
    match replace_folder_indiana(&abs) {
        Ok(()) => {
            eprintln!(
                "indiana: replaced command templates in {} (user edits discarded)",
                abs.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!(
                "indiana: could not replace templates in {}: {e}",
                abs.display()
            );
            ExitCode::FAILURE
        }
    }
}

fn copy(
    path: Option<PathBuf>,
    kind: Option<String>,
    group: Option<u64>,
    agent: Option<String>,
    latest: bool,
) -> ExitCode {
    if group.is_some() && agent.is_some() {
        eprintln!("indiana: --group and --agent are mutually exclusive");
        return ExitCode::FAILURE;
    }
    // ── kind filter ──
    let opts = match kind.as_deref() {
        Some(token) => match parse_kind(token) {
            Some(k) => CompileOptions {
                kind: Some(k),
                ..CompileOptions::default()
            },
            None => {
                eprintln!("indiana: unknown marker kind '{}'", token);
                return ExitCode::FAILURE;
            }
        },
        None => CompileOptions::default(),
    };
    let opts = CompileOptions {
        group,
        agent: agent.clone(),
        ..opts
    };

    // ── scan ──
    let (idx, roots) = match &path {
        Some(p) => {
            let abs = p.canonicalize().unwrap_or_else(|_| p.clone());
            (Index::build(&abs), vec![abs])
        }
        None => match daemon::client_scan() {
            Some(idx) => {
                let roots = daemon::client_status()
                    .map(|status| {
                        status
                            .folders
                            .into_iter()
                            .map(|folder| PathBuf::from(folder.path))
                            .collect()
                    })
                    .unwrap_or_default();
                (idx, roots)
            }
            None => {
                let root = PathBuf::from(".");
                (Index::build(&root), vec![root])
            }
        },
    };

    // ── cursor (--latest) ──
    let stored: std::collections::HashSet<String> = if latest {
        copied::load()
    } else {
        Default::default()
    };
    let opts = CompileOptions {
        copied: if latest { Some(stored) } else { None },
        roots: Some(roots),
        ..opts
    };

    // ── compile & copy ──
    let payload = compile_with_options(&idx, &opts);
    for w in &payload.warnings {
        eprintln!("warning: {w}");
    }
    let roots_for_prompt = opts.roots.clone().unwrap_or_default();
    // An agent copy speaks with that persona's prompt (single-root copies;
    // a multi-root daemon copy falls back to the shared default).
    let system_prompt = match (&agent, roots_for_prompt.as_slice()) {
        (Some(name), [root]) => indiana_core::agents::system_prompt_for_agent(root, name),
        _ => system_prompt_for_roots(&roots_for_prompt),
    };
    for w in &system_prompt.warnings {
        eprintln!("warning: {w}");
    }
    let rendered = render_text(&payload, &system_prompt);
    match arboard::Clipboard::new().and_then(|mut c| c.set_text(rendered.clone())) {
        Ok(()) => eprintln!("indiana: copied {} marker(s)", payload.markers.len()),
        Err(e) => eprintln!("indiana: clipboard unavailable: {e}"),
    }
    print!("{rendered}");

    // ── advance cursor (only --latest records what it delivered) ──
    // A plain copy must not consume markers from the --latest cursor, or the
    // user's first intentional `copy --latest` would return nothing.
    if latest && !payload.markers.is_empty() {
        let copied_ids: std::collections::HashSet<String> = payload
            .markers
            .iter()
            .map(indiana_core::cursor::identity_compiled)
            .collect();
        if let Err(e) = copied::save(&copied_ids) {
            eprintln!("indiana: failed to save copy cursor: {e}");
        }
    }

    ExitCode::SUCCESS
}
fn service_install() -> ExitCode {
    match service::install() {
        Ok(path) => {
            eprintln!("indiana: installed {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("indiana: service install failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn frontmatter_cmd(path: Option<PathBuf>, write: bool) -> ExitCode {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    let abs = root.canonicalize().unwrap_or(root);
    let report = match frontmatter::lint(&abs, write) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("indiana: frontmatter lint failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    if write {
        for p in &report.written {
            println!("added frontmatter: {}", p.display());
        }
        let leftover = report.missing.len() - report.written.len();
        if leftover == 0 {
            eprintln!(
                "indiana: added frontmatter to {} file(s)",
                report.written.len()
            );
            ExitCode::SUCCESS
        } else {
            eprintln!("indiana: {} file(s) still missing frontmatter", leftover);
            ExitCode::FAILURE
        }
    } else {
        for p in &report.missing {
            println!("missing frontmatter: {}", p.display());
        }
        if report.missing.is_empty() {
            eprintln!("indiana: all markdown files have frontmatter");
            ExitCode::SUCCESS
        } else {
            eprintln!(
                "indiana: {} file(s) missing frontmatter",
                report.missing.len()
            );
            ExitCode::FAILURE
        }
    }
}

fn task_cmd(cmd: TaskCmd) -> ExitCode {
    match cmd {
        TaskCmd::Add {
            root,
            queue,
            json,
            text,
        } => task_add(root, queue, json, text),
        TaskCmd::List {
            root,
            queue,
            state,
            json,
        } => task_list(root, queue, state, json),
        TaskCmd::Done { root, json, id } => task_done(root, json, id),
    }
}

fn casablanca_cmd(cmd: CasablancaCmd) -> ExitCode {
    // `--root` resolution mirrors `task` (default cwd, canonicalized).
    match cmd {
        CasablancaCmd::Path { root } => {
            println!("{}", casablanca::settings_path(&resolve_root(root)).display());
            ExitCode::SUCCESS
        }
        CasablancaCmd::Settings { root } => {
            let settings = casablanca::load(&resolve_root(root));
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::Value::Object(settings)).unwrap()
            );
            ExitCode::SUCCESS
        }
        CasablancaCmd::Get { root, key } => match casablanca::get(&resolve_root(root), &key) {
            Some(value) => {
                // Print bare string values unquoted; other JSON types as-is.
                match value {
                    serde_json::Value::String(s) => println!("{s}"),
                    other => println!("{other}"),
                }
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("indiana: no casablanca setting '{key}'");
                ExitCode::FAILURE
            }
        },
        CasablancaCmd::Set { root, key, value } => {
            let r = resolve_root(root);
            match casablanca::set(&r, &key, &value) {
                Ok(_) => {
                    eprintln!(
                        "indiana: set casablanca.{key} in {}",
                        casablanca::settings_path(&r).display()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("indiana: cannot write casablanca settings: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// Resolve `--root` to an absolute path, defaulting to the current directory.
fn resolve_root(root: Option<PathBuf>) -> PathBuf {
    let p = root.unwrap_or_else(|| PathBuf::from("."));
    p.canonicalize().unwrap_or(p)
}

fn parse_queue(token: &str) -> Option<Queue> {
    match token {
        "agent" => Some(Queue::Agent),
        "human" => Some(Queue::Human),
        _ => None,
    }
}

fn task_add(root: Option<PathBuf>, queue: String, json: bool, text: String) -> ExitCode {
    let r = resolve_root(root);
    let Some(queue) = parse_queue(&queue) else {
        eprintln!("indiana: --queue must be agent or human");
        return ExitCode::FAILURE;
    };
    if text.trim().is_empty() {
        eprintln!("indiana: task text is empty");
        return ExitCode::FAILURE;
    }
    // Register existing ids so a minted one can't collide.
    let mut ids = indiana_core::id::IdGenerator::new();
    for t in cos::load_tasks(&r) {
        ids.register(&t.id);
    }
    let task = cos::Task {
        id: ids.next(),
        text: text.trim().to_string(),
        queue,
        state: cos::TaskState::Open,
        origin: None,
        created: Some(cos::today()),
    };
    // Only a confirmed write is success: an append can only be Written or
    // Retry (the insert always changes the file), and reporting a Retry as
    // success would print an id that is in neither the tracker nor the log.
    match cos::append_task(&r, &task) {
        Ok(WriteResult::Written) => {
            let _ = cos::append_log(&r, "task-add", &task.id, "via cli");
            if json {
                println!("{}", serde_json::to_string_pretty(&task).unwrap());
            } else {
                println!("{}", task.id);
                eprintln!("indiana: added {} to the {queue:?} queue", task.id);
            }
            ExitCode::SUCCESS
        }
        Ok(_) => {
            eprintln!("indiana: tracker is busy — try again");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("indiana: cannot write tracker: {e}");
            ExitCode::FAILURE
        }
    }
}

fn task_list(
    root: Option<PathBuf>,
    queue: Option<String>,
    state: Option<String>,
    json: bool,
) -> ExitCode {
    let r = resolve_root(root);
    let queue = match queue.as_deref() {
        None => None,
        Some(q) => match parse_queue(q) {
            Some(q) => Some(q),
            None => {
                eprintln!("indiana: --queue must be agent or human");
                return ExitCode::FAILURE;
            }
        },
    };
    // Default view is the live queues: open + working.
    let state_ok: Box<dyn Fn(cos::TaskState) -> bool> = match state.as_deref() {
        None => Box::new(|s| matches!(s, cos::TaskState::Open | cos::TaskState::Working)),
        Some("all") => Box::new(|_| true),
        Some("open") => Box::new(|s| s == cos::TaskState::Open),
        Some("working") => Box::new(|s| s == cos::TaskState::Working),
        Some("done") => Box::new(|s| s == cos::TaskState::Done),
        Some("failed") => Box::new(|s| s == cos::TaskState::Failed),
        Some(other) => {
            eprintln!("indiana: unknown --state '{other}' (open|working|done|failed|all)");
            return ExitCode::FAILURE;
        }
    };
    let tasks: Vec<cos::Task> = cos::load_tasks(&r)
        .into_iter()
        .filter(|t| queue.map_or(true, |q| t.queue == q) && state_ok(t.state))
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&tasks).unwrap());
    } else if tasks.is_empty() {
        eprintln!("indiana: no tasks");
    } else {
        let mut current: Option<Queue> = None;
        for t in &tasks {
            if current != Some(t.queue) {
                println!("{:?}", t.queue);
                current = Some(t.queue);
            }
            let origin = t
                .origin
                .as_ref()
                .map(|o| format!("  ({}:{})", o.path, o.line))
                .unwrap_or_default();
            println!("  [{}] {}  {}{origin}", t.state.glyph(), t.id, t.text);
        }
    }
    ExitCode::SUCCESS
}

fn task_done(root: Option<PathBuf>, json: bool, id: String) -> ExitCode {
    let r = resolve_root(root);
    // One write decides everything: Written = done, Unchanged = no such task
    // (set_task_state no-ops on a missing id), Retry = lost the race twice.
    match cos::set_task_state(&r, &id, cos::TaskState::Done) {
        Ok(WriteResult::Written) => {
            let _ = cos::append_log(&r, "task-done", &id, "via cli");
            if json {
                println!("{}", serde_json::json!({"id": id, "state": "done"}));
            } else {
                println!("{id}");
                eprintln!("indiana: marked {id} done");
            }
            ExitCode::SUCCESS
        }
        Ok(WriteResult::Unchanged) => {
            eprintln!("indiana: no task '{id}'");
            ExitCode::FAILURE
        }
        Ok(WriteResult::Retry) => {
            eprintln!("indiana: tracker is busy — try again");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("indiana: cannot write tracker: {e}");
            ExitCode::FAILURE
        }
    }
}

/// List run records through the one Rust grammar (run_record.rs). `--json`
/// is the face surface: Casablanca and agents read this, never the files'
/// frontmatter directly.
fn runs_cmd(root: Option<PathBuf>, lines: usize, json: bool) -> ExitCode {
    let r = resolve_root(root);
    let listed = run_record::list(&r, lines);
    if json {
        println!("{}", serde_json::to_string_pretty(&listed).unwrap());
    } else if listed.is_empty() {
        eprintln!("indiana: no run records");
    } else {
        for run in listed {
            let s = &run.summary;
            let usage: Vec<String> = [s.token_summary(), s.cost_summary()]
                .into_iter()
                .flatten()
                .collect();
            let usage = if usage.is_empty() {
                String::new()
            } else {
                format!(" · {}", usage.join(" · "))
            };
            println!("{} {} [{}]{usage}", s.started, s.outcome, s.job);
        }
    }
    ExitCode::SUCCESS
}

fn log_cmd(root: Option<PathBuf>, lines: usize, json: bool) -> ExitCode {
    let r = resolve_root(root);
    let entries = cos::load_log(&r);
    let tail: Vec<&cos::LogEntry> = entries.iter().rev().take(lines).rev().collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&tail).unwrap());
    } else if tail.is_empty() {
        eprintln!("indiana: log is empty");
    } else {
        for e in tail {
            let detail = if e.detail.is_empty() {
                String::new()
            } else {
                format!(" {}", e.detail)
            };
            println!("{} {} [{}]{detail}", e.ts, e.event, e.id);
        }
    }
    ExitCode::SUCCESS
}
