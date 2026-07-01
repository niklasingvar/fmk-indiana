//! launchd service installation for dogfood distribution.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const LABEL: &str = "com.niklas.indiana";

/// Path to our launchd agent plist (`~/Library/LaunchAgents/<label>.plist`).
/// `None` only when `HOME` is unset.
pub fn plist_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join("Library/LaunchAgents")
            .join(format!("{LABEL}.plist"))
    })
}

/// Whether the launchd service is installed. When true, the daemon is
/// supervised with `KeepAlive=true`, so a `Shutdown` would be restarted — faces
/// use this (via `StatusResponse::stoppable`) to decide whether to offer stop.
pub fn is_installed() -> bool {
    plist_path().is_some_and(|p| p.exists())
}

pub fn install() -> io::Result<PathBuf> {
    let plist =
        plist_path().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
    fs::create_dir_all(plist.parent().expect("plist path has a parent"))?;
    let exe = std::env::current_exe()?;
    fs::write(&plist, plist_body(&exe))?;
    Ok(plist)
}

fn plist_body(exe: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
    <string>serve</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
</dict>
</plist>
"#,
        escape_xml(&exe.display().to_string())
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_path_uses_label_under_launchagents() {
        // Independent of whether the service is actually installed.
        if let Some(p) = plist_path() {
            assert!(p.ends_with("Library/LaunchAgents/com.niklas.indiana.plist"));
        }
    }

    #[test]
    fn test_plist_body() {
        let body = plist_body(Path::new("/tmp/indiana"));
        assert!(body.contains("<string>com.niklas.indiana</string>"));
        assert!(body.contains("<string>/tmp/indiana</string>"));
        assert!(body.contains("<string>serve</string>"));
        assert!(body.contains("<key>RunAtLoad</key>"));
        assert!(body.contains("<key>KeepAlive</key>"));
    }
}
