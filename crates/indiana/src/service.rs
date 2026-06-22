//! launchd service installation for dogfood distribution.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const LABEL: &str = "com.niklas.indiana";

pub fn install() -> io::Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
    let dir = home.join("Library/LaunchAgents");
    fs::create_dir_all(&dir)?;
    let plist = dir.join(format!("{LABEL}.plist"));
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
    fn test_plist_body() {
        let body = plist_body(Path::new("/tmp/indiana"));
        assert!(body.contains("<string>com.niklas.indiana</string>"));
        assert!(body.contains("<string>/tmp/indiana</string>"));
        assert!(body.contains("<string>serve</string>"));
        assert!(body.contains("<key>RunAtLoad</key>"));
        assert!(body.contains("<key>KeepAlive</key>"));
    }
}
