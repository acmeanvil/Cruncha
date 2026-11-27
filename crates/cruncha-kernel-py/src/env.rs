// SPDX-License-Identifier: GPL-3.0-or-later
//! The Python environment Python cells run in: a standalone CPython (python-build-standalone,
//! installed by `uv`) and the packages pinned in `python/requirements.lock`, with hashes.
//!
//! Everything lives under one base directory (the app's data directory):
//!
//! ```text
//! <base>/tools/bin/uv          uv, if the app installed it (from PyPI, into a venv)
//! <base>/interpreters/         standalone Pythons (UV_PYTHON_INSTALL_DIR)
//! <base>/cache/                uv's download cache
//! <base>/env-<lock hash>/      one environment per lockfile; a new lock gets a new one
//! ```
//!
//! An environment is complete when `.cruncha-ready` exists in it (written last), so an
//! interrupted setup is simply redone.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// The app's bundle identifier (`tauri.conf.json`), the name of its data directory.
pub const APP_ID: &str = "io.github.acmeanvil.cruncha";

/// The user's home directory (`HOME`, or `USERPROFILE` on Windows).
pub fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| if cfg!(windows) { std::env::var_os("USERPROFILE") } else { None })
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
}

/// The app's data directory, where Tauri's `app_data_dir()` puts it: `~/Library/Application
/// Support/<id>` on macOS, `%APPDATA%\<id>` on Windows, `$XDG_DATA_HOME/<id>` (or
/// `~/.local/share/<id>`) elsewhere. For tests and tools; the app asks Tauri.
pub fn app_data_dir() -> Option<PathBuf> {
    let base = if cfg!(target_os = "macos") {
        home()?.join("Library/Application Support")
    } else if cfg!(windows) {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else {
        match std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).filter(|p| p.is_absolute()) {
            Some(p) => p,
            None => home()?.join(".local/share"),
        }
    };
    Some(base.join(APP_ID))
}

/// Where the app keeps its Python (`<app data>/python`).
pub fn default_base() -> Option<PathBuf> {
    Some(app_data_dir()?.join("python"))
}

/// Start `cmd` without a console window (Windows: a GUI app's console children each get
/// one otherwise). Does nothing elsewhere.
pub fn no_window(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    cmd
}

pub const PYTHON_VERSION: &str = "3.13";
pub const UV_VERSION: &str = "0.13.9";
const READY: &str = ".cruncha-ready";

/// Where an environment for `lock` (the lockfile's text) lives under `base`.
pub fn env_dir(base: &Path, lock: &str) -> PathBuf {
    let hash = Sha256::digest(lock.as_bytes());
    let short: String = hash.iter().take(6).map(|b| format!("{b:02x}")).collect();
    base.join(format!("env-{short}"))
}

/// The interpreter of an environment directory.
pub fn python_in(env: &Path) -> PathBuf {
    if cfg!(windows) {
        env.join("Scripts").join("python.exe")
    } else {
        env.join("bin").join("python")
    }
}

/// The ready environment for `lock`, if it has been set up.
pub fn ready(base: &Path, lock: &str) -> Option<PathBuf> {
    let dir = env_dir(base, lock);
    (dir.join(READY).exists() && python_in(&dir).exists()).then(|| python_in(&dir))
}

/// Find `uv`: the app's own copy, then the usual install locations (a GUI app's `PATH` is
/// minimal on macOS), then `PATH`.
pub fn find_uv(base: &Path) -> Option<PathBuf> {
    let exe = if cfg!(windows) { "uv.exe" } else { "uv" };
    let mut candidates = vec![
        base.join("tools")
            .join(if cfg!(windows) { "Scripts" } else { "bin" })
            .join(exe),
    ];
    if let Some(home) = home() {
        candidates.push(home.join(".local/bin").join(exe));
        candidates.push(home.join(".cargo/bin").join(exe));
    }
    if cfg!(unix) {
        candidates.push(PathBuf::from("/opt/homebrew/bin").join(exe));
        candidates.push(PathBuf::from("/usr/local/bin").join(exe));
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|p| p.join(exe)));
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// A system Python able to bootstrap `uv` (`python -m venv` + pip).
fn find_bootstrap_python() -> Option<PathBuf> {
    let mut names = vec![];
    let mut dirs = vec![];
    if cfg!(windows) {
        // python.org's installer: per user under %LOCALAPPDATA%\Programs\Python\Python3NN.
        names.extend(["python.exe".to_string(), "python3.exe".into()]);
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            for minor in (9..=20).rev() {
                dirs.push(PathBuf::from(&local).join(format!("Programs/Python/Python3{minor}")));
            }
        }
    } else {
        for minor in (9..=20).rev() {
            names.push(format!("python3.{minor}"));
        }
        names.push("python3".into());
        dirs.extend([PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/usr/local/bin"), PathBuf::from("/usr/bin")]);
    }
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    dirs.iter()
        .flat_map(|d| names.iter().map(move |n| d.join(n)))
        // Windows' `WindowsApps\python.exe` is a stub that opens the Microsoft Store.
        .filter(|p| !p.components().any(|c| c.as_os_str().eq_ignore_ascii_case("WindowsApps")))
        .find(|p| p.is_file())
}

/// Progress of [`setup`], for the UI.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "step")]
pub enum Progress {
    InstallingUv,
    InstallingPython,
    InstallingPackages,
    Done { python: PathBuf },
}

async fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    let out = no_window(cmd)
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|e| format!("{what}: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let tail: Vec<&str> = err.lines().rev().take(12).collect();
        Err(format!(
            "{what} failed:\n{}",
            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
        ))
    }
}

/// Set up the environment for `lock` under `base` (downloads from PyPI and GitHub on first
/// use: about 70 MB of Python and 450 MB of packages). Returns the interpreter.
pub async fn setup(
    base: &Path,
    lock: &str,
    progress: impl Fn(Progress),
) -> Result<PathBuf, String> {
    if let Some(p) = ready(base, lock) {
        progress(Progress::Done { python: p.clone() });
        return Ok(p);
    }
    tokio::fs::create_dir_all(base)
        .await
        .map_err(|e| e.to_string())?;
    let uv = match find_uv(base) {
        Some(uv) => uv,
        None => {
            progress(Progress::InstallingUv);
            let py = find_bootstrap_python()
                .ok_or("uv isn't installed and no Python 3 was found to install it with")?;
            let tools = base.join("tools");
            run(
                Command::new(&py).arg("-m").arg("venv").arg(&tools),
                "creating a venv for uv",
            )
            .await?;
            run(
                Command::new(python_in(&tools))
                    .args(["-m", "pip", "install", "-q", "--disable-pip-version-check"])
                    .arg(format!("uv=={UV_VERSION}")),
                "installing uv",
            )
            .await?;
            find_uv(base).ok_or("uv was installed but can't be found")?
        }
    };
    let dir = env_dir(base, lock);
    let uv_env = |c: &mut Command| {
        c.env("UV_PYTHON_INSTALL_DIR", base.join("interpreters"))
            .env("UV_CACHE_DIR", base.join("cache"))
            .env("UV_NO_PROGRESS", "1");
    };
    progress(Progress::InstallingPython);
    let mut venv = Command::new(&uv);
    venv.args([
        "venv",
        "-q",
        "--clear",
        "--managed-python",
        "--python",
        PYTHON_VERSION,
    ])
    .arg(&dir);
    uv_env(&mut venv);
    run(&mut venv, "creating the Python environment").await?;
    progress(Progress::InstallingPackages);
    let lock_file = dir.join("requirements.lock");
    tokio::fs::write(&lock_file, lock)
        .await
        .map_err(|e| e.to_string())?;
    let mut pip = Command::new(&uv);
    // Precompiled bytecode: the worker runs with PYTHONDONTWRITEBYTECODE, so without it every
    // worker would recompile SciPy and friends on import (20 s for `scipy.optimize`; spike S4).
    pip.args([
        "pip",
        "install",
        "-q",
        "--require-hashes",
        "--compile-bytecode",
        "--python",
    ])
    .arg(python_in(&dir))
    .arg("-r")
    .arg(&lock_file);
    uv_env(&mut pip);
    run(&mut pip, "installing packages").await?;
    tokio::fs::write(dir.join(READY), lock)
        .await
        .map_err(|e| e.to_string())?;
    let python = python_in(&dir);
    progress(Progress::Done {
        python: python.clone(),
    });
    Ok(python)
}
