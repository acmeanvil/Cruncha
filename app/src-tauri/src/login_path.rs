// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from looking_glass (crates/looking-glass/src/terminal_apps.rs and main.rs), Copyright (c)
// 2026 acmeanvil, MIT License; see app/src-tauri/LICENSE-LOOKING-GLASS.

//! `PATH` for an app opened from Finder (or a desktop launcher): it gets only the system
//! minimum, so agents installed with Homebrew, npm or cargo (`npx`, `claude`, …) and Python's
//! `uv` wouldn't be found. When the inherited `PATH` looks like that, take the login shell's.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Whether `PATH` looks like a launcher's bare one (none of the usual tool directories).
fn looks_bare(path: Option<&str>, home: &std::path::Path) -> bool {
    let Some(p) = path else { return true };
    let wanted: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec!["/usr/local/bin".into(), "/opt/homebrew/bin".into()]
    } else {
        vec![home.join(".local/bin"), home.join(".cargo/bin")]
    };
    !std::env::split_paths(p).any(|d| wanted.contains(&d))
}

/// The shell in the account record (`$SHELL` can be stale): Directory Services on macOS, the
/// passwd database elsewhere.
fn account_shell() -> Option<PathBuf> {
    let user = std::env::var("USER").ok()?;
    let out = if cfg!(target_os = "macos") {
        Command::new("dscl").args([".", "-read", &format!("/Users/{user}"), "UserShell"]).output().ok()?
    } else {
        Command::new("getent").args(["passwd", &user]).output().ok()?
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let shell = if cfg!(target_os = "macos") {
        text.split_whitespace().nth(1)?.to_owned()
    } else {
        text.trim().rsplit(':').next().filter(|s| !s.is_empty())?.to_owned()
    };
    Some(PathBuf::from(shell))
}

fn login_shell() -> PathBuf {
    account_shell()
        .filter(|p| p.exists())
        .or_else(|| std::env::var_os("SHELL").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(if cfg!(target_os = "macos") { "/bin/zsh" } else { "/bin/bash" }))
}

/// How a shell prints its `PATH` with `:` separators.
fn print_path_script(shell: &std::path::Path) -> &'static str {
    match shell.file_name().and_then(|n| n.to_str()).unwrap_or_default() {
        "nu" => "$env.PATH | str join (char esep)",
        "fish" => "string join : $PATH",
        _ => "printf %s \"$PATH\"",
    }
}

/// `PATH` as the login shell sets it up; gives up after a few seconds.
fn login_path() -> Option<String> {
    let shell = login_shell();
    let script = print_path_script(&shell);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = Command::new(&shell).args(["-l", "-c", script]).stdin(Stdio::null()).stderr(Stdio::null()).output();
        let _ = tx.send(out);
    });
    let out = rx.recv_timeout(std::time::Duration::from_secs(4)).ok()?.ok()?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && path.contains('/')).then_some(path)
}

/// Call first thing in `main`, before any thread reads the environment.
pub fn adopt_login_path() {
    if cfg!(windows) {
        return; // GUI apps get the user's full PATH there
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    if looks_bare(std::env::var("PATH").ok().as_deref(), &home)
        && let Some(path) = login_path()
    {
        // Safe: called before anything else runs or reads the environment.
        unsafe { std::env::set_var("PATH", path) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_launchers_path_is_bare_a_shells_is_not() {
        let home = std::path::Path::new("/Users/x");
        assert!(looks_bare(Some("/usr/bin:/bin:/usr/sbin:/sbin"), home));
        assert!(looks_bare(None, home));
        let full = if cfg!(target_os = "macos") { "/usr/local/bin:/usr/bin" } else { "/Users/x/.cargo/bin:/usr/bin" };
        assert!(!looks_bare(Some(full), home));
        assert_eq!(print_path_script(std::path::Path::new("/Users/x/.cargo/bin/nu")), "$env.PATH | str join (char esep)");
    }

    #[test]
    fn the_login_shell_gives_a_path() {
        if cfg!(windows) {
            return;
        }
        let p = login_path().expect("the login shell prints its PATH");
        assert!(p.contains("/usr/bin"), "{p}");
    }
}
