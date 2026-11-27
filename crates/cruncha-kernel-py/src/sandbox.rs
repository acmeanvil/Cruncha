// SPDX-License-Identifier: GPL-3.0-or-later
//! The worker's sandbox (ADR-0005): on macOS, the worker runs under `sandbox-exec` with a
//! deny-by-default profile. It may read its Python environment, the worker package and the
//! system's libraries and fonts, and write only its own scratch directory. It has no network,
//! can't start other programs, and can't read the user's files (documents included: values
//! reach a cell through the protocol, never by path).
//!
//! `sandbox-exec` is deprecated by Apple but is what the OS itself, Bazel and others still use
//! for this.
//!
//! On Windows (ADR-0030) the same rules come from an AppContainer with no capabilities (no
//! network; files only where its SID is granted) inside a Job Object (no other processes; the
//! worker dies with the launcher). A [`Launcher`] (the app's own executable with
//! `--py-sandbox`, or `cruncha-sandbox` in tests) starts the worker there, much as
//! `sandbox-exec` does; [`windows::prepare`] grants the readable trees and the scratch
//! directory first. On Windows a sandbox without a launcher is an error, never a fallback.
//!
//! Elsewhere the worker runs unsandboxed for now (seccomp on Linux is a follow-up).

#[cfg(windows)]
pub mod windows;

use std::path::{Path, PathBuf};
use tokio::process::Command;

/// What a sandboxed worker may touch.
#[derive(Debug, Clone, PartialEq)]
pub struct Sandbox {
    /// Readable trees: the Python environments (interpreter and packages) and the worker
    /// package.
    pub read: Vec<PathBuf>,
    /// The only writable directory (matplotlib's cache, temp files). Created on start.
    pub scratch: PathBuf,
    /// Windows: what starts the worker in its AppContainer. Unused elsewhere.
    pub launcher: Option<Launcher>,
}

/// A program that runs `<program> <args…> <python> <python's args…>` in the Windows sandbox.
#[derive(Debug, Clone, PartialEq)]
pub struct Launcher {
    pub program: PathBuf,
    pub args: Vec<String>,
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

impl Sandbox {
    /// The command that starts `python` in this sandbox, or `None` where this system has none
    /// (the worker then runs unsandboxed; never on Windows).
    pub(crate) fn command(&self, python: &Path) -> Result<Option<Command>, String> {
        let scratch = &self.scratch;
        if cfg!(windows) {
            let l = self
                .launcher
                .as_ref()
                .ok_or("the worker's sandbox has no launcher (ADR-0030)")?;
            std::fs::create_dir_all(scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
            let mut c = Command::new(&l.program);
            c.args(&l.args).arg(python);
            c.env("HOME", scratch)
                .env("USERPROFILE", scratch)
                .env("TEMP", scratch)
                .env("TMP", scratch)
                .env("MPLCONFIGDIR", scratch.join("matplotlib"))
                .current_dir(scratch);
            return Ok(Some(c));
        }
        let Some(exec) = program() else { return Ok(None) };
        std::fs::create_dir_all(scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
        let mut c = Command::new(exec);
        c.arg("-p").arg(self.profile()).arg(python);
        // Everything the worker writes goes to its scratch directory.
        c.env("HOME", scratch).env("TMPDIR", scratch).env("MPLCONFIGDIR", scratch.join("matplotlib"));
        c.current_dir(scratch);
        Ok(Some(c))
    }
}

/// The base interpreter's directory when `python` is a virtual environment's (`home` in the
/// `pyvenv.cfg` beside its `bin` or `Scripts` directory). On Windows the venv's `python.exe`
/// is a redirector that starts that interpreter as a child process.
pub fn venv_home(python: &Path) -> Option<PathBuf> {
    let cfg = std::fs::read_to_string(python.parent()?.parent()?.join("pyvenv.cfg")).ok()?;
    cfg.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == "home").then(|| PathBuf::from(v.trim()))
    })
}

/// Append one argument to a Windows command line, quoted the way the C runtime (and so
/// Python) parses it back: backslashes are literal except before a quote.
pub fn push_arg(line: &mut Vec<u16>, arg: &[u16]) {
    let (quote, backslash, space, tab) = (b'"' as u16, b'\\' as u16, b' ' as u16, b'\t' as u16);
    if !line.is_empty() {
        line.push(space);
    }
    if !arg.is_empty() && !arg.iter().any(|&c| c == space || c == tab || c == quote) {
        line.extend_from_slice(arg);
        return;
    }
    line.push(quote);
    let mut run = 0;
    for &c in arg {
        if c == backslash {
            run += 1;
        } else {
            if c == quote {
                line.extend(std::iter::repeat_n(backslash, run + 1));
            }
            run = 0;
        }
        line.push(c);
    }
    line.extend(std::iter::repeat_n(backslash, run));
    line.push(quote);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(args: &[&str]) -> String {
        let mut l = Vec::new();
        for a in args {
            push_arg(&mut l, &a.encode_utf16().collect::<Vec<_>>());
        }
        String::from_utf16(&l).unwrap()
    }

    #[test]
    fn windows_arguments_are_quoted_as_the_c_runtime_reads_them() {
        assert_eq!(line(&["C:\\py\\python.exe", "-I"]), "C:\\py\\python.exe -I");
        assert_eq!(line(&["", "a b", "tab\there"]), "\"\" \"a b\" \"tab\there\"");
        assert_eq!(line(&["say \"hi\""]), "\"say \\\"hi\\\"\"");
        // Trailing backslashes are doubled before the closing quote, not elsewhere.
        assert_eq!(line(&["C:\\My Dir\\"]), "\"C:\\My Dir\\\\\"");
        assert_eq!(line(&["a\\\\\"b c"]), "\"a\\\\\\\\\\\"b c\"");
    }

    #[test]
    fn a_venvs_base_interpreter_is_found_from_pyvenv_cfg() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join("Scripts")).unwrap();
        let py = d.path().join("Scripts").join("python.exe");
        assert_eq!(venv_home(&py), None);
        std::fs::write(d.path().join("pyvenv.cfg"), "home = C:\\uv\\cpython-3.13\nversion_info = 3.13.1\n").unwrap();
        assert_eq!(venv_home(&py), Some(PathBuf::from("C:\\uv\\cpython-3.13")));
    }
}
