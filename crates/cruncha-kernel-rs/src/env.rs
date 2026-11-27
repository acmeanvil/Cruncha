// SPDX-License-Identifier: GPL-3.0-or-later
//! Setting up Rust for cells (ADR-0006), as the app sets up Python: Cruncha's own rustup home
//! under the app's data, never the user's (Homebrew, rustup, PATH stay untouched).
//!
//! `setup` downloads `rustup-init` for this computer from static.rust-lang.org (with the system
//! `curl`, TLS only), checks it against the published SHA-256, installs the pinned toolchain
//! with the minimal profile and the `wasm32-wasip2` target, and builds a first cell so the
//! prelude's one-time compile (about 30 s) happens during setup.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::Toolchain;
use crate::cells::{Cells, Request};

/// The toolchain cells are built with. Results record it (it is part of their input hash).
pub const RUST_VERSION: &str = "1.99.1";
pub const TARGET: &str = "wasm32-wasip2";

/// Where setup is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Downloading,
    Installing,
    Warming,
    Done,
}

/// This computer's rustup host triple.
fn host() -> Option<&'static str> {
    Some(match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        _ => return None,
    })
}

/// The toolchain under `base`, if it is set up with the cell target.
pub fn ready(base: &Path) -> Option<Toolchain> {
    let tc = Toolchain::at(base)?;
    let has_target = std::fs::read_dir(tc.rustup_home.join("toolchains"))
        .ok()?
        .flatten()
        .any(|t| t.path().join("lib/rustlib").join(TARGET).is_dir());
    has_target.then_some(tc)
}

async fn curl(url: &str, to: &Path) -> Result<(), String> {
    let out = crate::no_window(&mut tokio::process::Command::new("curl"))
        .args(["--proto", "=https", "--tlsv1.2", "--silent", "--show-error", "--fail", "--location", "--output"])
        .arg(to)
        .arg(url)
        .output()
        .await
        .map_err(|e| format!("couldn't run curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("download of {url} failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(())
}

/// Set up Rust under `base` (idempotent). `progress` is told each step.
pub async fn setup(base: &Path, progress: impl Fn(Step)) -> Result<Toolchain, String> {
    if let Some(tc) = ready(base) {
        progress(Step::Done);
        return Ok(tc);
    }
    let host = host().ok_or("Rust cells aren't supported on this platform yet")?;
    std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
    progress(Step::Downloading);
    let exe = if cfg!(windows) { "rustup-init.exe" } else { "rustup-init" };
    let url = format!("https://static.rust-lang.org/rustup/dist/{host}/{exe}");
    let init = base.join(exe);
    let sums = base.join(format!("{exe}.sha256"));
    curl(&url, &init).await?;
    curl(&format!("{url}.sha256"), &sums).await?;
    let want = std::fs::read_to_string(&sums).map_err(|e| e.to_string())?;
    let want = want.split_whitespace().next().unwrap_or_default().to_lowercase();
    let got: String = Sha256::digest(std::fs::read(&init).map_err(|e| e.to_string())?).iter().map(|b| format!("{b:02x}")).collect();
    if want.len() != 64 || got != want {
        let _ = std::fs::remove_file(&init);
        return Err("the downloaded rustup-init didn't match its published checksum".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&init, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    }
    progress(Step::Installing);
    let out = crate::no_window(&mut tokio::process::Command::new(&init))
        .args(["-y", "--no-modify-path", "--profile", "minimal", "--default-toolchain", RUST_VERSION, "--target", TARGET])
        .env("RUSTUP_HOME", base.join("rustup"))
        .env("CARGO_HOME", base.join("cargo"))
        .output()
        .await
        .map_err(|e| format!("couldn't run rustup-init: {e}"))?;
    let _ = std::fs::remove_file(&init);
    let _ = std::fs::remove_file(&sums);
    if !out.status.success() {
        return Err(format!("installing Rust failed: {}", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or_default()));
    }
    let tc = ready(base).ok_or("Rust was installed, but the cell target is missing")?;
    progress(Step::Warming);
    warm(&tc, &work_dir(base)).await?;
    progress(Step::Done);
    Ok(tc)
}

/// Where cells are built.
pub fn work_dir(base: &Path) -> PathBuf {
    base.join("work")
}

/// Build and run a trivial cell, so the prelude is compiled before the first real one.
pub async fn warm(tc: &Toolchain, work: &Path) -> Result<(), String> {
    let cells = Cells::new(tc.clone(), work.to_path_buf()).map_err(|e| e.to_string())?;
    let r = cells
        .run(&Request { cell: "warm".into(), source: "let x = 1.0;".into(), defines: vec!["x".into()], ..Default::default() }, None)
        .await;
    match r.error {
        None => Ok(()),
        Some(e) => Err(format!("the first Rust cell didn't build: {}", e.message)),
    }
}
