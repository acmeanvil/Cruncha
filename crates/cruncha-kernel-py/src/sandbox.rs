// SPDX-License-Identifier: GPL-3.0-or-later
//! The worker's sandbox (ADR-0005): on macOS, the worker runs under `sandbox-exec` with a
//! deny-by-default profile. It may read its Python environment, the worker package and the
//! system's libraries and fonts, and write only its own scratch directory. It has no network,
//! can't start other programs, and can't read the user's files (documents included: values
//! reach a cell through the protocol, never by path).
//!
//! `sandbox-exec` is deprecated by Apple but is what the OS itself, Bazel and others still use
//! for this. Elsewhere the worker runs unsandboxed for now (seccomp on Linux is a follow-up).

use std::path::{Path, PathBuf};

/// What a sandboxed worker may touch.
#[derive(Debug, Clone, PartialEq)]
pub struct Sandbox {
    /// Readable trees: the Python environments (interpreter and packages) and the worker
    /// package.
    pub read: Vec<PathBuf>,
    /// The only writable directory (matplotlib's cache, temp files). Created on start.
    pub scratch: PathBuf,
}

/// The sandbox program, if this system has one.
pub fn program() -> Option<&'static Path> {
    let p = Path::new("/usr/bin/sandbox-exec");
    (cfg!(target_os = "macos") && p.exists()).then_some(p)
}

/// A path as the sandbox sees it: resolved (`/var` is `/private/var`), quoted for SBPL.
fn quoted(p: &Path) -> String {
    let real = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = real.display().to_string();
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

impl Sandbox {
    /// The SBPL profile.
    pub fn profile(&self) -> String {
        let read: Vec<String> = self.read.iter().map(|p| format!("(subpath {})", quoted(p))).collect();
        let scratch = quoted(&self.scratch);
        format!(
            r#"(version 1)
(deny default)
(allow process-exec* (subpath {exec}))
(allow signal (target self))
(allow sysctl-read)
(allow file-read-metadata)
(allow file-read*
  (literal "/")
  {read}
  (subpath {scratch})
  (subpath "/System") (subpath "/usr/lib") (subpath "/usr/share") (subpath "/Library/Fonts")
  (subpath "/private/var/db/timezone") (literal "/private/etc/localtime")
  (literal "/dev/null") (literal "/dev/random") (literal "/dev/urandom"))
(allow file-write* (subpath {scratch}) (literal "/dev/null"))
(allow file-ioctl (literal "/dev/null"))
"#,
            exec = self.read.first().map(|p| quoted(p)).unwrap_or_else(|| "\"/nonexistent\"".into()),
            read = read.join(" "),
        )
    }
}
