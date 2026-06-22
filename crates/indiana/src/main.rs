//! indiana — one binary, multi-mode (IN_PRD.md): serve | scan | add.
//! Faces only render; all domain logic lives in indiana_core.

mod config;
mod daemon;
mod paths;
mod protocol;

use clap::{Parser, Subcommand};
use config::Config;
use indiana_core::index::Index;
use indiana_core::markers::Kind;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "indiana", about = "Scan markdown for :: markers")]
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
    },
    /// Add a folder to the monitored-folders config.
    Add {
        /// Folder to monitor across daemon restarts.
        path: PathBuf,
    },
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
        Cmd::Scan { path, json } => scan(path, json),
        Cmd::Add { path } => add(path),
    }
}

fn scan(path: Option<PathBuf>, json: bool) -> ExitCode {
    // Explicit path → standalone. No path → daemon view, else cwd standalone.
    let idx = match &path {
        Some(p) => Index::build(p),
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
        let id = m.id.as_deref().map(|i| format!(" [{i}]")).unwrap_or_default();
        println!("  {:>4} · {:<10}{id} {msg}", m.line, kind_name(m.kind));
    }
    for w in &idx.warnings {
        eprintln!("warning: {w}");
    }
    eprintln!(
        "{} marker(s){}",
        idx.markers.len(),
        if idx.warnings.is_empty() {
            String::new()
        } else {
            format!(", {} warning(s)", idx.warnings.len())
        }
    );
    ExitCode::SUCCESS
}

fn add(path: PathBuf) -> ExitCode {
    let mut cfg = Config::load();
    if cfg.add_folder(&path) {
        if let Err(e) = cfg.save() {
            eprintln!("indiana: could not save config: {e}");
            return ExitCode::FAILURE;
        }
        eprintln!("indiana: monitoring {}", path.display());
    } else {
        eprintln!("indiana: already monitoring {}", path.display());
    }
    ExitCode::SUCCESS
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
