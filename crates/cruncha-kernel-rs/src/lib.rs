// SPDX-License-Identifier: GPL-3.0-or-later
//! Rust cells (ADR-0006; spike S5): a cell is a small crate built with Cruncha's own Rust
//! toolchain for `wasm32-wasip2`, and run as a WebAssembly component in wasmtime with an
//! empty WASI context: no files, no network, no environment, no clock beyond the defaults.
//!
//! - [`Toolchain`] finds the toolchain (rustup home under the app's data, with the
//!   `wasm32-wasip2` target) and builds a cell crate with a shared target directory, so the
//!   prelude and its dependencies compile once.
//! - [`Runtime`] compiles components (Cranelift) and instantiates them; [`Cell`] calls the
//!   `cell` world's exports (`wit/cell.wit`).

pub mod cells;
pub mod env;
pub mod rename;
pub mod scan;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Error, Result, Store};
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;

/// Windows: start a console program (cargo, rustup-init, curl) without a console window.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `cmd` without a console window on Windows; nothing elsewhere.
pub(crate) fn no_window(cmd: &mut tokio::process::Command) -> &mut tokio::process::Command {
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

mod bindings {
    wasmtime::component::bindgen!({ world: "cell", path: "wit" });
}
use bindings::Cell as CellWorld;

/// The guest crates' sources, built into the binary so an installed app needs no source tree:
/// the prelude and the `cruncha-units` it uses (paths relative to the guest directory).
const GUEST: &[(&str, &str)] = &[
    ("cruncha-prelude/Cargo.toml", include_str!("../guest/cruncha-prelude/Cargo.toml")),
    ("cruncha-prelude/src/lib.rs", include_str!("../guest/cruncha-prelude/src/lib.rs")),
    ("wit/cell.wit", include_str!("../wit/cell.wit")),
    ("cruncha-units/Cargo.toml", include_str!("../../cruncha-units/Cargo.toml")),
    ("cruncha-units/LICENSE-HURMET", include_str!("../../cruncha-units/LICENSE-HURMET")),
    ("cruncha-units/src/lib.rs", include_str!("../../cruncha-units/src/lib.rs")),
    ("cruncha-units/src/catalog.rs", include_str!("../../cruncha-units/src/catalog.rs")),
    ("cruncha-units/src/jsmath.rs", include_str!("../../cruncha-units/src/jsmath.rs")),
    ("cruncha-units/src/jsnum.rs", include_str!("../../cruncha-units/src/jsnum.rs")),
    ("cruncha-units/src/rational.rs", include_str!("../../cruncha-units/src/rational.rs")),
    ("cruncha-units/src/table.rs", include_str!("../../cruncha-units/src/table.rs")),
    ("cruncha-units/src/units.rs", include_str!("../../cruncha-units/src/units.rs")),
];

/// A guest file as written out: the prelude finds `cruncha-units` and the cell world beside it, and
/// `cruncha-units` states what it otherwise inherits from this repository's workspace.
fn guest_text(path: &str, text: &'static str) -> std::borrow::Cow<'static, str> {
    match path {
        "cruncha-prelude/Cargo.toml" => text.replace("path = \"../../../cruncha-units\"", "path = \"../cruncha-units\"").into(),
        "cruncha-units/Cargo.toml" => text
            .replace("version.workspace = true", concat!("version = \"", env!("CARGO_PKG_VERSION"), "\""))
            .replace("edition.workspace = true", "edition = \"2024\"")
            .replace("license.workspace = true", concat!("license = \"", env!("CARGO_PKG_LICENSE"), "\""))
            .into(),
        "cruncha-prelude/src/lib.rs" => text.replace("path: \"../../wit\"", "path: \"../wit\"").into(),
        _ => text.into(),
    }
}

/// The prelude crate each cell depends on, written under `work` (`<work>/guest-<hash>/`, named
/// by its contents, so a new app version writes a new one) the first time it's needed.
pub fn prelude_dir(work: &Path) -> std::io::Result<PathBuf> {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for (path, text) in GUEST {
        let t = guest_text(path, text);
        h.update((path.len() as u64).to_le_bytes());
        h.update(path);
        h.update((t.len() as u64).to_le_bytes());
        h.update(t.as_bytes());
    }
    let hash: String = h.finalize()[..6].iter().map(|b| format!("{b:02x}")).collect();
    let dir = work.join(format!("guest-{hash}"));
    if !dir.join("cruncha-prelude/Cargo.toml").is_file() {
        // Written aside and renamed into place, so a half-written copy is never used.
        let tmp = work.join(format!(".guest-{hash}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        for (path, text) in GUEST {
            let f = tmp.join(path);
            std::fs::create_dir_all(f.parent().expect("a guest file is in a directory"))?;
            std::fs::write(&f, guest_text(path, text).as_bytes())?;
        }
        if std::fs::rename(&tmp, &dir).is_err() {
            let _ = std::fs::remove_dir_all(&tmp);
            if !dir.join("cruncha-prelude/Cargo.toml").is_file() {
                return Err(std::io::Error::other(format!("couldn't write {}", dir.display())));
            }
        }
    }
    Ok(dir.join("cruncha-prelude"))
}

/// Cruncha's Rust toolchain: a rustup home with `wasm32-wasip2`.
#[derive(Debug, Clone)]
pub struct Toolchain {
    pub rustup_home: PathBuf,
    pub cargo_home: PathBuf,
}

impl Toolchain {
    /// `<base>/rustup` and `<base>/cargo`, if the cargo there exists.
    pub fn at(base: &Path) -> Option<Toolchain> {
        let t = Toolchain { rustup_home: base.join("rustup"), cargo_home: base.join("cargo") };
        t.cargo().exists().then_some(t)
    }

    /// The default place, `<app data>/rust`: the app's data directory as Tauri's
    /// `app_data_dir()` has it (`~/Library/Application Support/<id>` on macOS,
    /// `%APPDATA%\<id>` on Windows, `$XDG_DATA_HOME/<id>` or `~/.local/share/<id>` elsewhere).
    /// For tests and tools; the app asks Tauri.
    pub fn default_base() -> Option<PathBuf> {
        let var = |k| std::env::var_os(k).map(PathBuf::from).filter(|p: &PathBuf| p.is_absolute());
        let home = || var("HOME").or_else(|| if cfg!(windows) { var("USERPROFILE") } else { None });
        let data = if cfg!(target_os = "macos") {
            home()?.join("Library/Application Support")
        } else if cfg!(windows) {
            var("APPDATA")?
        } else {
            var("XDG_DATA_HOME").or_else(|| Some(home()?.join(".local/share")))?
        };
        Some(data.join("io.github.acmeanvil.cruncha").join("rust"))
    }

    pub fn cargo(&self) -> PathBuf {
        self.cargo_home.join("bin").join(if cfg!(windows) { "cargo.exe" } else { "cargo" })
    }

    /// Build the cell crate at `dir` (a `cdylib`) into `target`; returns the component path
    /// and how long the build took.
    pub fn build(&self, dir: &Path, target: &Path, release: bool) -> Result<(PathBuf, Duration)> {
        let t = Instant::now();
        let mut cmd = Command::new(self.cargo());
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut cmd, CREATE_NO_WINDOW);
        cmd.arg("build").arg("--quiet").arg("--target").arg("wasm32-wasip2").arg("--manifest-path").arg(dir.join("Cargo.toml"));
        if release {
            cmd.arg("--release");
        }
        let out = cmd
            .env("RUSTUP_HOME", &self.rustup_home)
            .env("CARGO_HOME", &self.cargo_home)
            .env("CARGO_TARGET_DIR", target)
            .env_remove("RUSTC")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_BUILD_TARGET")
            .output()
            .map_err(|e| Error::msg(format!("couldn't start cargo: {e}")))?;
        if !out.status.success() {
            return Err(Error::msg(format!("the cell didn't compile:\n{}", String::from_utf8_lossy(&out.stderr))));
        }
        let name = cell_crate_name(dir)?;
        let wasm = target.join("wasm32-wasip2").join(if release { "release" } else { "debug" }).join(format!("{name}.wasm"));
        Ok((wasm, t.elapsed()))
    }
}

fn cell_crate_name(dir: &Path) -> Result<String> {
    let toml = std::fs::read_to_string(dir.join("Cargo.toml"))?;
    let name = toml
        .lines()
        .find_map(|l| l.trim().strip_prefix("name = \"")?.strip_suffix('"').map(str::to_owned))
        .ok_or_else(|| Error::msg("no package name"))?;
    Ok(name.replace('-', "_"))
}

/// Write a cell crate: `source` is the body of `lib.rs` after the prelude import. The prelude
/// goes beside it, in `dir`'s parent.
pub fn write_cell(dir: &Path, name: &str, source: &str) -> Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    let prelude = prelude_dir(dir.parent().unwrap_or(dir))?;
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n\
             [dependencies]\ncruncha-prelude = {{ path = {:?} }}\n\n[profile.release]\nopt-level = 2\ndebug = false\n\n[workspace]\n",
            prelude.display().to_string()
        ),
    )?;
    std::fs::write(dir.join("src/lib.rs"), format!("use cruncha_prelude::*;\n\n{source}\n"))?;
    Ok(())
}

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.ctx, table: &mut self.table }
    }
}

/// Compiles and instantiates cell components.
pub struct Runtime {
    pub(crate) engine: Engine,
    linker: Linker<State>,
}

impl Runtime {
    pub fn new() -> Result<Runtime> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        // Timeouts: a call gets a deadline in epochs, and a timer advances the epoch.
        config.epoch_interruption(true);
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        Ok(Runtime { engine, linker })
    }

    /// Compile a component to machine code (Cranelift).
    pub fn compile(&self, wasm: &Path) -> Result<Component> {
        Component::from_file(&self.engine, wasm)
    }

    /// The compiled code, to cache next to the component.
    pub fn precompile(&self, wasm: &Path) -> Result<Vec<u8>> {
        self.engine.precompile_component(&std::fs::read(wasm)?)
    }

    /// A component from [`Runtime::precompile`]'s output.
    ///
    /// # Safety
    /// `bytes` must come from `precompile` of this same wasmtime build (wasmtime checks the
    /// version and configuration, but not that the code wasn't tampered with).
    pub unsafe fn load_precompiled(&self, bytes: &[u8]) -> Result<Component> {
        unsafe { Component::deserialize(&self.engine, bytes) }
    }

    /// A fresh instance with an empty WASI context: no files, environment or network; its
    /// stdout and stderr are kept (up to 1 MB each) for the cell's output.
    pub fn instantiate(&self, component: &Component) -> Result<Cell> {
        let stdout = MemoryOutputPipe::new(1 << 20);
        let stderr = MemoryOutputPipe::new(1 << 20);
        let ctx = WasiCtx::builder().stdout(stdout.clone()).stderr(stderr.clone()).build();
        let mut store = Store::new(&self.engine, State { ctx, table: ResourceTable::new() });
        store.set_epoch_deadline(u64::MAX / 2);
        let bindings = CellWorld::instantiate(&mut store, component, &self.linker)?;
        Ok(Cell { store, bindings, engine: self.engine.clone(), stdout, stderr })
    }
}

/// An instance of a cell.
pub struct Cell {
    store: Store<State>,
    bindings: CellWorld,
    engine: Engine,
    stdout: MemoryOutputPipe,
    stderr: MemoryOutputPipe,
}

impl Cell {
    /// What the cell printed so far.
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout.contents()).into_owned()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr.contents()).into_owned()
    }

    pub fn run(&mut self, inputs: &str) -> Result<std::result::Result<String, String>> {
        self.bindings.call_run(&mut self.store, inputs)
    }

    /// `run`, trapped if it takes longer than `timeout` (the instance is unusable after a trap).
    pub fn run_with_timeout(&mut self, inputs: &str, timeout: Duration) -> Result<std::result::Result<String, String>> {
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.store.set_epoch_deadline(1);
        let (engine, flag) = (self.engine.clone(), done.clone());
        let timer = std::thread::spawn(move || {
            let start = Instant::now();
            while !flag.load(std::sync::atomic::Ordering::Relaxed) {
                if start.elapsed() >= timeout {
                    engine.increment_epoch();
                    return;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let r = self.bindings.call_run(&mut self.store, inputs);
        done.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = timer.join();
        self.store.set_epoch_deadline(u64::MAX / 2);
        r
    }

    pub fn scalar(&mut self, x: f64) -> Result<f64> {
        self.bindings.call_scalar(&mut self.store, x)
    }

    pub fn sum(&mut self, xs: &[f64]) -> Result<f64> {
        self.bindings.call_sum(&mut self.store, xs)
    }
}

#[cfg(test)]
mod guest_tests {
    use super::*;

    fn files(root: &Path, under: &Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(under).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != "target") {
                    files(root, &p, out);
                }
            } else if p.extension().is_some_and(|x| x == "rs" || x == "toml" || x == "wit") || p.ends_with("LICENSE-HURMET") {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }

    #[test]
    fn every_guest_source_is_embedded() {
        let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut on_disk = Vec::new();
        let prelude = Path::new(env!("CARGO_MANIFEST_DIR")).join("guest");
        files(&prelude, &prelude.join("cruncha-prelude"), &mut on_disk);
        let units = crates.join("cruncha-units");
        files(crates, &units.join("src"), &mut on_disk);
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        files(here, &here.join("wit"), &mut on_disk);
        on_disk.push("cruncha-units/Cargo.toml".into());
        on_disk.push("cruncha-units/LICENSE-HURMET".into());
        on_disk.sort();
        let mut embedded: Vec<String> = GUEST.iter().map(|(p, _)| p.to_string()).collect();
        embedded.sort();
        assert_eq!(embedded, on_disk, "list new guest files in GUEST");
    }

    #[test]
    fn the_written_prelude_stands_alone() {
        let work = tempfile::tempdir().unwrap();
        let p = prelude_dir(work.path()).unwrap();
        assert_eq!(prelude_dir(work.path()).unwrap(), p, "written once, then reused");
        let prelude = std::fs::read_to_string(p.join("Cargo.toml")).unwrap();
        assert!(prelude.contains("cruncha-units = { path = \"../cruncha-units\" }"), "{prelude}");
        let lib = std::fs::read_to_string(p.join("src/lib.rs")).unwrap();
        assert!(lib.contains("path: \"../wit\"") && p.parent().unwrap().join("wit/cell.wit").is_file());
        let units = std::fs::read_to_string(p.parent().unwrap().join("cruncha-units/Cargo.toml")).unwrap();
        assert!(!units.contains(".workspace"), "{units}");
        assert!(units.contains("edition = \"2024\"") && units.contains("license = \"GPL-3.0-or-later\""), "{units}");
        // Nothing left over from writing it.
        assert_eq!(std::fs::read_dir(work.path()).unwrap().count(), 1);
    }
}
