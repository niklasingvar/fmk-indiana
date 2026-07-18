//! Per-repo Casablanca settings — a small JSON bag at
//! `<root>/.indiana/casablanca/settings.json`, shared by the editor and this
//! CLI (CASABLANCA_OVERVIEW.md). Repo-local user input, committable, not derived
//! from source and not part of the marker index — a separate store, like the
//! Chief of Staff tracker. Free-form so a new setting is a write, not a schema
//! change in two languages; the editor reads the keys it knows and ignores the
//! rest.

use serde_json::{Map, Value};
use std::io;
use std::path::{Path, PathBuf};

/// `<root>/.indiana/casablanca/settings.json`.
pub fn settings_path(root: &Path) -> PathBuf {
    root.join(".indiana")
        .join("casablanca")
        .join("settings.json")
}

/// Load the settings object; an empty map if the file is absent or unreadable
/// (degrade safely — a missing file just means "no per-repo overrides").
pub fn load(root: &Path) -> Map<String, Value> {
    match std::fs::read_to_string(settings_path(root)) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(map)) => map,
            _ => Map::new(),
        },
        Err(_) => Map::new(),
    }
}

/// Write the settings object, creating `.indiana/casablanca/` if needed.
pub fn save(root: &Path, settings: &Map<String, Value>) -> io::Result<()> {
    let path = settings_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(settings.clone()))?;
    text.push('\n');
    std::fs::write(path, text)
}

/// One key's value, if present.
pub fn get(root: &Path, key: &str) -> Option<Value> {
    load(root).get(key).cloned()
}

/// Set one key and persist. A `raw` string that parses as JSON is stored as
/// that JSON value (so `true`, `42`, `"x"` keep their types); otherwise it is
/// stored as a plain string — `set color "255 0 0"` stores the string.
pub fn set(root: &Path, key: &str, raw: &str) -> io::Result<Value> {
    let value =
        serde_json::from_str::<Value>(raw).unwrap_or_else(|_| Value::String(raw.to_string()));
    let mut settings = load(root);
    settings.insert(key.to_string(), value.clone());
    save(root, &settings)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tmp() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "indiana-cb-{n}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn test_load_missing_is_empty() {
        let d = tmp();
        assert!(load(&d).is_empty());
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_set_creates_file_and_get_round_trips() {
        let d = tmp();
        set(&d, "color", "255 0 0").unwrap();
        assert!(settings_path(&d).exists(), "set creates the file");
        assert_eq!(get(&d, "color"), Some(Value::String("255 0 0".into())));
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_set_parses_json_typed_values() {
        let d = tmp();
        set(&d, "wrap", "true").unwrap();
        set(&d, "tabWidth", "2").unwrap();
        assert_eq!(get(&d, "wrap"), Some(Value::Bool(true)));
        assert_eq!(get(&d, "tabWidth"), Some(Value::from(2)));
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_set_preserves_other_keys() {
        let d = tmp();
        set(&d, "color", "1 2 3").unwrap();
        set(&d, "theme", "dark").unwrap();
        let s = load(&d);
        assert_eq!(s.get("color"), Some(&Value::String("1 2 3".into())));
        assert_eq!(s.get("theme"), Some(&Value::String("dark".into())));
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_load_ignores_non_object() {
        let d = tmp();
        std::fs::create_dir_all(settings_path(&d).parent().unwrap()).unwrap();
        std::fs::write(settings_path(&d), "[1,2,3]").unwrap();
        assert!(load(&d).is_empty(), "a non-object file degrades to empty");
        std::fs::remove_dir_all(d).ok();
    }
}
