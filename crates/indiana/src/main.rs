//! indiana — one binary, multi-mode (IN_PRD.md): serve | scan | add.
//! Faces only render; all domain logic lives in indiana_core.

mod config;
mod daemon;
mod mcp;
mod paths;
mod protocol;
mod service;

use clap::{Parser, Subcommand};
use config::Config;
use indiana_core::compile::{compile, render_text};
use indiana_core::index::Index;
use indiana_core::markers::Kind;
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
    /// Compile markers and copy the bundle to the clipboard.
    Copy {
        /// Repo root to scan (forces a standalone scan of this path).
        path: Option<PathBuf>,
    },
    /// Run the MCP stdio server.
    Mcp,
    /// Manage the launchd service.
    Service {
        #[command(subcommand)]
        cmd: ServiceCmd,
    },
}

#[derive(Subcommand)]
enum ServiceCmd {
    /// Install the launchd plist for `indiana serve`.
    Install,
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
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
        Cmd::Copy { path } => copy(path),
        Cmd::Mcp => match mcp::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("indiana: mcp error: {e}");
                ExitCode::FAILURE
            }
        },
        Cmd::Service {
            cmd: ServiceCmd::Install,
        } => service_install(),
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
        println!("  {:>4} · {:<10}{id} {msg}", m.line, kind_name(m.kind));
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
        eprintln!(
            "indiana: monitoring {} (daemon not running; scans on next serve)",
            path.display()
        );
    } else {
        eprintln!("indiana: already monitoring {}", path.display());
    }
    ExitCode::SUCCESS
}

fn copy(path: Option<PathBuf>) -> ExitCode {
    let idx = match &path {
        Some(p) => Index::build(p),
        None => daemon::client_scan().unwrap_or_else(|| Index::build(&PathBuf::from("."))),
    };
    let rendered = render_text(&compile(&idx));
    match arboard::Clipboard::new().and_then(|mut c| c.set_text(rendered.clone())) {
        Ok(()) => eprintln!("indiana: copied {} marker(s)", idx.markers.len()),
        Err(e) => eprintln!("indiana: clipboard unavailable: {e}"),
    }
    print!("{rendered}");
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

fn kind_name(k: Kind) -> &'static str {
    match k {
        Kind::Question => "question",
        Kind::Hate => "hate",
        Kind::Love => "love",
        Kind::Keep => "keep",
        Kind::Fix => "fix",
        Kind::Elaborate => "elaborate",
        Kind::Note => "note",
        Kind::Action => "action",
        Kind::Todo => "todo",
    }
}
