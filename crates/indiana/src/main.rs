//! indiana — one binary, multi-mode (IN_PRD.md): serve | scan | add.
//! Faces only render; all domain logic lives in indiana_core.

mod config;
mod copied;
mod daemon;
mod mcp;
mod paths;
mod protocol;
mod service;
mod todos;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use config::Config;
use indiana_core::compile::{compile_with_options, render_text, CompileOptions};
use indiana_core::frontmatter;
use indiana_core::index::Index;
use indiana_core::markers::{long_name, parse_kind};
use indiana_core::templates::{init_folder_indiana, replace_folder_indiana};
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
    /// Manage the Montmartre todo list (SQLite, repo-local `.indiana/montmartre/todos.db`).
    Todo {
        #[command(subcommand)]
        cmd: TodoCmd,
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
enum TodoCmd {
    /// Add a todo (max 29 words). Prints the new id; `--json` prints the full row.
    Add {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Domain the todo belongs to.
        #[arg(long)]
        domain: String,
        /// Id of an existing todo this one depends on. Repeatable.
        #[arg(long = "dependency", value_name = "ID")]
        dependencies: Vec<String>,
        /// Emit JSON for agents.
        #[arg(long)]
        json: bool,
        /// The todo text (max 29 words).
        todo: String,
    },
    /// List todos, grouped by domain.
    List {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Only list todos in this domain.
        #[arg(long)]
        domain: Option<String>,
        /// Emit JSON for agents.
        #[arg(long)]
        json: bool,
    },
    /// Delete a todo by id. Cascade-removes dependency edges to and from it.
    Delete {
        /// Repo root (default: current directory).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Emit JSON for agents.
        #[arg(long)]
        json: bool,
        /// Id of the todo to delete.
        id: String,
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
        Cmd::Copy { path, kind, latest } => copy(path, kind, latest),
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
        Cmd::Todo { cmd } => todo_cmd(cmd),
        Cmd::Frontmatter { path, write } => frontmatter_cmd(path, write),
    }
}

fn scan(path: Option<PathBuf>, json: bool, read_only: bool) -> ExitCode {
    // Explicit path → standalone. No path → daemon view, else cwd standalone.
    let idx = match &path {
        Some(p) if read_only => Index::build_read_only(p),
        Some(p) => Index::build(p),
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
            eprintln!("indiana: warning: could not scaffold templates in {}: {e}", abs.display());
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

fn copy(path: Option<PathBuf>, kind: Option<String>, latest: bool) -> ExitCode {
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
    let rendered = render_text(&payload);
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
            eprintln!("indiana: added frontmatter to {} file(s)", report.written.len());
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
            eprintln!("indiana: {} file(s) missing frontmatter", report.missing.len());
            ExitCode::FAILURE
        }
    }
}

fn todo_cmd(cmd: TodoCmd) -> ExitCode {
    match cmd {
        TodoCmd::Add {
            root,
            domain,
            dependencies,
            json,
            todo,
        } => todo_add(root, domain, dependencies, json, todo),
        TodoCmd::List {
            root,
            domain,
            json,
        } => todo_list(root, domain, json),
        TodoCmd::Delete { root, json, id } => todo_delete(root, json, id),
    }
}

/// Resolve `--root` to an absolute path, defaulting to the current directory.
fn todo_root(root: Option<PathBuf>) -> PathBuf {
    let p = root.unwrap_or_else(|| PathBuf::from("."));
    p.canonicalize().unwrap_or(p)
}

fn todo_add(
    root: Option<PathBuf>,
    domain: String,
    dependencies: Vec<String>,
    json: bool,
    todo: String,
) -> ExitCode {
    let r = todo_root(root);
    match todos::add(&r, &todo, &domain, &dependencies) {
        Ok(t) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&t).unwrap());
            } else {
                println!("{}", t.id);
                eprintln!("indiana: added {} ({})", t.id, t.domain);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("indiana: {e}");
            ExitCode::FAILURE
        }
    }
}

fn todo_list(root: Option<PathBuf>, domain: Option<String>, json: bool) -> ExitCode {
    let r = todo_root(root);
    match todos::list(&r, domain.as_deref()) {
        Ok(list) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&list).unwrap());
            } else if list.is_empty() {
                eprintln!("indiana: no todos");
            } else {
                let mut current: Option<&str> = None;
                for t in &list {
                    if current != Some(t.domain.as_str()) {
                        println!("{}", t.domain);
                        current = Some(t.domain.as_str());
                    }
                    let deps = if t.dependencies.is_empty() {
                        String::new()
                    } else {
                        format!("  [deps: {}]", t.dependencies.join(", "))
                    };
                    println!("  {}  {}{}", t.id, t.todo, deps);
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("indiana: {e}");
            ExitCode::FAILURE
        }
    }
}

fn todo_delete(root: Option<PathBuf>, json: bool, id: String) -> ExitCode {
    let r = todo_root(root);
    match todos::delete(&r, &id) {
        Ok(()) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"id": id, "deleted": true})
                );
            } else {
                println!("{}", id);
                eprintln!("indiana: deleted {}", id);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("indiana: {e}");
            ExitCode::FAILURE
        }
    }
}
