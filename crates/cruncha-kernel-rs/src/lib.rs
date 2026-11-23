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
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

mod bindings {
    wasmtime::component::bindgen!({ world: "cell", path: "wit" });
}
use bindings::Cell as CellWorld;

/// The guest prelude crate, built as a path dependency of each cell.
pub fn prelude_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("guest/cruncha-prelude")
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

    /// The default place: `~/Library/Application Support/io.github.acmeanvil.cruncha/rust`.
    pub fn default_base() -> Option<PathBuf> {
        Some(PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support/io.github.acmeanvil.cruncha/rust"))
    }

    pub fn cargo(&self) -> PathBuf {
        self.cargo_home.join("bin/cargo")
    }

    /// Build the cell crate at `dir` (a `cdylib`) into `target`; returns the component path
    /// and how long the build took.
    pub fn build(&self, dir: &Path, target: &Path, release: bool) -> Result<(PathBuf, Duration)> {
        let t = Instant::now();
        let mut cmd = Command::new(self.cargo());
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

/// Write a cell crate: `source` is the body of `lib.rs` after the prelude import.
pub fn write_cell(dir: &Path, name: &str, source: &str) -> Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n\
             [dependencies]\ncruncha-prelude = {{ path = {:?} }}\n\n[profile.release]\nopt-level = 2\ndebug = false\n\n[workspace]\n",
            prelude_dir().display().to_string()
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
