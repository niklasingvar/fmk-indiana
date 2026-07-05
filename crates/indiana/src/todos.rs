//! Chief of Staff todo store — a repo-local SQLite list of agent/human todos.
//!
//! Lives at `<root>/.indiana/chief-of-staff/todos.db`. Separate from markdown
//! `::todo` markers: this is authoritative state, not derived from source
//! (IN_PRINCIPLES.md Chief of Staff carve-out). TODO is state; finished work is
//! deleted (rules/template_todo.md).

use indiana_core::id::IdGenerator;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};

/// Maximum words in a todo text.
const MAX_WORDS: usize = 29;

/// One todo row.
#[derive(Debug, Clone, Serialize)]
pub struct Todo {
    pub id: String,
    pub todo: String,
    pub domain: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub enum TodoError {
    InvalidTodo(String),
    InvalidDomain(String),
    InvalidDependency(String),
    UnknownDependency(String),
    NotFound(String),
    Db(String),
    Io(std::io::Error),
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TodoError::InvalidTodo(m) => write!(f, "invalid todo: {m}"),
            TodoError::InvalidDomain(m) => write!(f, "invalid domain: {m}"),
            TodoError::InvalidDependency(m) => write!(f, "invalid dependency: {m}"),
            TodoError::UnknownDependency(id) => write!(f, "unknown dependency '{id}'"),
            TodoError::NotFound(id) => write!(f, "no todo '{id}'"),
            TodoError::Db(m) => write!(f, "database error: {m}"),
            TodoError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TodoError {}

impl From<rusqlite::Error> for TodoError {
    fn from(e: rusqlite::Error) -> Self {
        TodoError::Db(e.to_string())
    }
}

impl From<std::io::Error> for TodoError {
    fn from(e: std::io::Error) -> Self {
        TodoError::Io(e)
    }
}

fn db_path(root: &Path) -> PathBuf {
    root.join(".indiana").join("chief-of-staff").join("todos.db")
}

/// Open (creating the dir + db if missing) and ensure the schema is present.
fn open(root: &Path) -> Result<Connection, TodoError> {
    let path = db_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path)?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS todos (
             id     TEXT PRIMARY KEY,
             todo   TEXT NOT NULL,
             domain TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS todo_dependencies (
             todo_id       TEXT NOT NULL,
             depends_on_id TEXT NOT NULL,
             PRIMARY KEY (todo_id, depends_on_id),
             FOREIGN KEY (todo_id)       REFERENCES todos(id) ON DELETE CASCADE,
             FOREIGN KEY (depends_on_id) REFERENCES todos(id) ON DELETE CASCADE
         );",
    )?;
    Ok(conn)
}

fn normalize_deps(dependencies: &[String]) -> Result<Vec<String>, TodoError> {
    let mut out = Vec::new();
    for d in dependencies {
        let t = d.trim();
        if t.is_empty() {
            return Err(TodoError::InvalidDependency(
                "dependency must not be empty".into(),
            ));
        }
        out.push(t.to_string());
    }
    Ok(out)
}

/// Add a todo. Generates an Indiana-style id; validates text (≤29 words),
/// domain, and that every dependency id already exists.
pub fn add(
    root: &Path,
    todo: &str,
    domain: &str,
    dependencies: &[String],
) -> Result<Todo, TodoError> {
    let todo_text = todo.trim();
    if todo_text.is_empty() {
        return Err(TodoError::InvalidTodo("todo must not be empty".into()));
    }
    let words = todo_text.split_whitespace().count();
    if words > MAX_WORDS {
        return Err(TodoError::InvalidTodo(format!(
            "todo must be at most {MAX_WORDS} words, got {words}"
        )));
    }
    let domain_text = domain.trim();
    if domain_text.is_empty() {
        return Err(TodoError::InvalidDomain("domain must not be empty".into()));
    }
    let deps = normalize_deps(dependencies)?;

    let mut conn = open(root)?;

    for dep in &deps {
        let exists: bool = conn
            .query_row("SELECT 1 FROM todos WHERE id = ?1", params![dep], |_| Ok(true))
            .optional()?
            .unwrap_or(false);
        if !exists {
            return Err(TodoError::UnknownDependency(dep.clone()));
        }
    }

    // Generate an id not already in the table.
    let mut gen = IdGenerator::new();
    let existing: Vec<String> = conn
        .prepare("SELECT id FROM todos")?
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    for id in existing {
        gen.register(&id);
    }
    let new_id = gen.next();

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO todos (id, todo, domain) VALUES (?1, ?2, ?3)",
        params![new_id, todo_text, domain_text],
    )?;
    for dep in &deps {
        tx.execute(
            "INSERT INTO todo_dependencies (todo_id, depends_on_id) VALUES (?1, ?2)",
            params![new_id, dep],
        )?;
    }
    tx.commit()?;

    Ok(Todo {
        id: new_id,
        todo: todo_text.to_string(),
        domain: domain_text.to_string(),
        dependencies: deps,
    })
}

/// List todos, optionally filtered by domain. Ordered by domain, then id.
pub fn list(root: &Path, domain: Option<&str>) -> Result<Vec<Todo>, TodoError> {
    let conn = open(root)?;
    let mut todos: Vec<Todo> = match domain {
        Some(d) => conn
            .prepare("SELECT id, todo, domain FROM todos WHERE domain = ?1 ORDER BY domain, id")?
            .query_map(params![d], row_to_todo)?
            .filter_map(|r| r.ok())
            .collect(),
        None => conn
            .prepare("SELECT id, todo, domain FROM todos ORDER BY domain, id")?
            .query_map([], row_to_todo)?
            .filter_map(|r| r.ok())
            .collect(),
    };

    for t in &mut todos {
        let deps: Vec<String> = conn
            .prepare(
                "SELECT depends_on_id FROM todo_dependencies WHERE todo_id = ?1 ORDER BY depends_on_id",
            )?
            .query_map(params![t.id], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        t.dependencies = deps;
    }

    Ok(todos)
}

fn row_to_todo(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
    Ok(Todo {
        id: row.get(0)?,
        todo: row.get(1)?,
        domain: row.get(2)?,
        dependencies: Vec::new(),
    })
}

/// Delete a todo by id. Cascade-removes dependency edges to and from it.
pub fn delete(root: &Path, id: &str) -> Result<(), TodoError> {
    let conn = open(root)?;
    let n = conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(TodoError::NotFound(id.to_string()));
    }
    Ok(())
}
