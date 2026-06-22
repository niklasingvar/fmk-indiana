//! indiana — one binary, multi-mode (IN_PRD.md): serve | scan | copy | service.
//! Faces only render; all domain logic lives in indiana_core.

use clap::{Parser, Subcommand};
use indiana_core::index::Index;
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
    /// Walk markdown under a path and list every marker (one pass).
    Scan {
        /// Repo root to scan (default: current directory).
        path: Option<PathBuf>,
        /// Emit JSON instead of the human list.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        Cmd::Scan { path, json } => scan(path.unwrap_or_else(|| PathBuf::from(".")), json),
    }
}

fn scan(root: PathBuf, json: bool) -> ExitCode {
    let idx = Index::build(&root);

    if json {
        // Core computes; the face only serializes what it computed.
        println!("{}", serde_json::to_string_pretty(&idx).unwrap());
        return ExitCode::SUCCESS;
    }

    // Human view: grouped by file, in path order.
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
        if idx.warnings.is_empty() { String::new() } else { format!(", {} warning(s)", idx.warnings.len()) }
    );
    ExitCode::SUCCESS
}

fn kind_name(k: indiana_core::markers::Kind) -> &'static str {
    use indiana_core::markers::Kind::*;
    match k {
        Question => "question",
        Hate => "hate",
        Love => "love",
        Keep => "keep",
        Fix => "fix",
        Elaborate => "elaborate",
        Note => "note",
        Action => "action",
        Todo => "todo",
    }
}
