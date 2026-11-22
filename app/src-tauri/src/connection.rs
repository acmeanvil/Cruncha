// SPDX-License-Identifier: GPL-3.0-or-later

//! The connection file for agents outside Cruncha (ADR-0009). While "Allow agents on this
//! computer" is on, the app writes the MCP server's URL and token to
//! `<app data>/agents/server.json`, readable only by this user (0600), so a client on this
//! computer (looking_glass, a script) can connect without copy and paste. Turning sharing off,
//! or quitting, removes it. After a crash a stale file can remain: its server is gone, so a
//! client that can't connect (or gets 401) should ignore it; `pid` helps it tell.
//!
//! looking_glass reads this file (format 1, transport "http"; it uses `name`, `url` and
//! `headers`). Changing the path, the app identifier or `FORMAT` breaks it: bump `FORMAT`
//! and change looking_glass in step.
//!
//! ```json
//! { "format": 1, "name": "cruncha", "transport": "http", "url": "http://127.0.0.1:PORT/mcp",
//!   "token": "…", "headers": { "Authorization": "Bearer …" }, "pid": 1234, "startedAt": "…" }
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub format: u32,
    pub name: String,
    pub transport: String,
    pub url: String,
    pub token: String,
    pub headers: std::collections::BTreeMap<String, String>,
    pub pid: u32,
    pub started_at: String,
}

impl Connection {
    pub fn new(url: &str, token: &str, started_at: String) -> Self {
        Connection {
            format: FORMAT,
            name: "cruncha".into(),
            transport: "http".into(),
            url: url.into(),
            token: token.into(),
            headers: [("Authorization".to_string(), format!("Bearer {token}"))].into(),
            pid: std::process::id(),
            started_at,
        }
    }
}

/// `<app data>/agents/server.json`
pub fn path(app_data: &Path) -> PathBuf {
    app_data.join("agents").join("server.json")
}

/// Write the file atomically (a 0600 temporary file renamed over it).
pub fn write(file: &Path, c: &Connection) -> std::io::Result<()> {
    let dir = file.parent().ok_or_else(|| std::io::Error::other("no parent directory"))?;
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(".server.{}.json", std::process::id()));
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let written = (|| {
        let mut f = opts.open(&tmp)?;
        f.write_all(serde_json::to_string_pretty(c).map_err(std::io::Error::other)?.as_bytes())?;
        f.sync_all()?;
        std::fs::rename(&tmp, file)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written
}

/// Remove the file if it is still this server's (another Cruncha may have replaced it).
pub fn remove(file: &Path, token: &str) {
    let ours = std::fs::read_to_string(file)
        .ok()
        .and_then(|s| serde_json::from_str::<Connection>(&s).ok())
        .is_some_and(|c| c.token == token);
    if ours {
        let _ = std::fs::remove_file(file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn written_private_and_removed_only_by_its_server() {
        let dir = tempfile::tempdir().unwrap();
        let file = path(dir.path());
        let c = Connection::new("http://127.0.0.1:4242/mcp", "secret", "2026-11-22T00:00:00Z".into());
        write(&file, &c).unwrap();
        let back: Connection = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(back, c);
        assert_eq!(back.headers["Authorization"], "Bearer secret");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o600);
        }
        // Another server's token leaves it; this one's removes it.
        remove(&file, "other");
        assert!(file.exists());
        remove(&file, "secret");
        assert!(!file.exists());
        // No temporary files left behind.
        assert_eq!(std::fs::read_dir(file.parent().unwrap()).unwrap().count(), 0);
    }
}
