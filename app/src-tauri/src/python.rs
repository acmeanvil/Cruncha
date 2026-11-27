// SPDX-License-Identifier: GPL-3.0-or-later
//! Running a document's Python cells (ADR-0005): the worker processes and the run loop.
//!
//! Each open document has two workers: one **runs** cells, one only **scans** sources for the
//! names they define and read, so a commit never waits behind a long-running cell. Both start
//! on first use. The run loop takes the next due job from the [`Engine`], runs it, records the
//! outcome and reports fresh results, until nothing is due. A newer commit starts a newer
//! loop; the older one stops before its next job (the cell already running finishes, and its
//! outcome is memoized, so the newer loop reuses it when the inputs still match).
//!
//! Rust cells (ADR-0006) are jobs of the same loop: they compile and run in wasmtime
//! ([`cruncha_kernel_rs::cells::Cells`]) instead of the worker, and come back as the same
//! `RunResult`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cruncha_kernel_py::{Config, Error as KernelError, Interrupter, Kernel, env};
use cruncha_store::Cell;
use tokio::sync::Mutex as AsyncMutex;

use crate::engine::{CellResult, Engine, PyNames};

/// The packages Python cells get.
pub const LOCK: &str = include_str!("../../../python/requirements.lock");

/// The environment's name (`env-9a9710dddbc2`): it identifies the lockfile Python ran with.
pub fn env_label(base: &std::path::Path) -> String {
    env::env_dir(base, LOCK)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Called with fresh results of the latest version.
pub type Notify = Arc<dyn Fn(HashMap<String, CellResult>) + Send + Sync>;

pub struct Python {
    base: PathBuf,
    worker_dir: PathBuf,
    runner: AsyncMutex<Option<Kernel>>,
    scanner: AsyncMutex<Option<Kernel>>,
    interrupter: Mutex<Option<Interrupter>>,
    generation: AtomicU64,
    /// A cell is interrupted after this long (none: never).
    pub timeout: Option<Duration>,
    /// Rust cells, once Rust is set up.
    rust: Mutex<Option<Arc<cruncha_kernel_rs::cells::Cells>>>,
    /// When the workers were last used (a scan or a run), for stopping idle ones (ADR-0024).
    last_used: Mutex<std::time::Instant>,
}

impl Python {
    /// Python for one document. `base` holds the environments (see `cruncha_kernel_py::env`);
    /// `worker_dir` the `cruncha_worker` package.
    pub fn new(base: &Path, worker_dir: &Path) -> Arc<Python> {
        Arc::new(Python {
            base: base.to_owned(),
            worker_dir: worker_dir.to_owned(),
            runner: AsyncMutex::new(None),
            scanner: AsyncMutex::new(None),
            interrupter: Mutex::new(None),
            generation: AtomicU64::new(0),
            timeout: None,
            rust: Mutex::new(None),
            last_used: Mutex::new(std::time::Instant::now()),
        })
    }

    /// Run Rust cells with `cells` (Rust is set up).
    pub fn set_rust(&self, cells: Option<Arc<cruncha_kernel_rs::cells::Cells>>) {
        *self.rust.lock().unwrap() = cells;
    }

    pub fn rust(&self) -> Option<Arc<cruncha_kernel_rs::cells::Cells>> {
        self.rust.lock().unwrap().clone()
    }

    /// The interpreter, if the environment is set up.
    pub fn interpreter(&self) -> Option<PathBuf> {
        env::ready(&self.base, LOCK)
    }

    fn touch(&self) {
        *self.last_used.lock().unwrap() = std::time::Instant::now();
    }

    /// Is a worker process running (a runner or a scanner)?
    #[cfg(test)]
    pub fn has_workers(&self) -> bool {
        let live = |m: &AsyncMutex<Option<Kernel>>| m.try_lock().map_or(true, |s| s.is_some());
        live(&self.runner) || live(&self.scanner)
    }

    /// Stop the workers if they have been idle for `idle` and nothing is running (ADR-0024: a
    /// background tab's Python stops after 10 minutes). They start again on the next scan or
    /// run. Returns whether they were stopped; the caller tells the engine its values are gone.
    pub async fn sleep_if_idle(&self, idle: Duration) -> bool {
        if self.last_used.lock().unwrap().elapsed() < idle {
            return false;
        }
        // A run in progress holds the runner; a scan, the scanner. Never wait for either.
        let (Ok(mut runner), Ok(mut scanner)) = (self.runner.try_lock(), self.scanner.try_lock()) else {
            return false;
        };
        if runner.is_none() && scanner.is_none() {
            return false;
        }
        if let Some(k) = runner.take() {
            k.shutdown().await;
        }
        if let Some(k) = scanner.take() {
            k.shutdown().await;
        }
        true
    }

    async fn ensure<'a>(
        &self,
        slot: &'a mut Option<Kernel>,
    ) -> Result<&'a mut Kernel, KernelError> {
        self.touch();
        if slot.as_mut().is_some_and(|k| !k.is_alive()) {
            *slot = None;
        }
        if slot.is_none() {
            let python = self
                .interpreter()
                .ok_or_else(|| KernelError::Start("Python isn't set up".into()))?;
            let mut config = Config::new(python, &self.worker_dir);
            // ADR-0005, ADR-0030: on macOS and Windows the worker reads only its environment
            // and package, writes only its scratch directory, and has no network or
            // subprocesses. On Windows this program starts it in its AppContainer
            // (`--py-sandbox`, main.rs); without that it doesn't start at all.
            config.sandbox = Some(cruncha_kernel_py::sandbox::Sandbox {
                read: vec![self.base.clone(), self.worker_dir.clone()],
                scratch: std::env::temp_dir().join(format!("cruncha-worker-{}", std::process::id())),
                launcher: std::env::current_exe().ok().map(|program| cruncha_kernel_py::sandbox::Launcher {
                    program,
                    args: vec!["--py-sandbox".into()],
                }),
            });
            *slot = Some(Kernel::start(config).await?);
        }
        Ok(slot.as_mut().expect("just started"))
    }

    /// `source` with these names renamed, by the worker (Python's tokenizer; ADR-0024).
    pub async fn rename(&self, source: &str, names: &std::collections::BTreeMap<String, String>) -> Result<String, String> {
        if self.interpreter().is_none() {
            return Err("renaming in a Python cell needs Python set up".into());
        }
        let mut slot = self.scanner.lock().await;
        let k = self.ensure(&mut slot).await.map_err(|e| e.to_string())?;
        k.rename(source, names).await.map_err(|e| e.to_string())
    }

    /// Scan these sources (empty when Python isn't set up).
    pub async fn scan(&self, sources: &[&str]) -> Vec<(String, PyNames)> {
        if sources.is_empty() || self.interpreter().is_none() {
            return Vec::new();
        }
        let mut slot = self.scanner.lock().await;
        let mut out = Vec::new();
        for s in sources {
            let k = match self.ensure(&mut slot).await {
                Ok(k) => k,
                Err(e) => {
                    crate::applog::warn("python", format!("Python scan unavailable: {e}"));
                    return out;
                }
            };
            match k.scan(s).await {
                Ok(r) => out.push((
                    (*s).to_owned(),
                    PyNames {
                        defines: r.defines,
                        reads: r.reads,
                        warnings: r.warnings,
                        syntax_error: r.error,
                    },
                )),
                Err(e) => {
                    crate::applog::warn("python", format!("Python scan failed: {e}"));
                    *slot = None;
                }
            }
        }
        out
    }

    /// Stop: interrupt the running cell, and end the run loop.
    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(i) = self.interrupter.lock().unwrap().clone() {
            i.interrupt();
        }
        if let Some(r) = self.rust() {
            r.interrupter().increment_epoch();
        }
    }

    /// Start a run loop over `cells` (the latest version), replacing any older one.
    pub fn start(
        self: &Arc<Self>,
        engine: Arc<Mutex<Engine>>,
        cells: Arc<Vec<Cell>>,
        notify: Notify,
    ) {
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let py = self.clone();
        tokio::spawn(async move { py.run_loop(generation, engine, cells, notify).await });
    }

    async fn run_loop(
        &self,
        generation: u64,
        engine: Arc<Mutex<Engine>>,
        cells: Arc<Vec<Cell>>,
        notify: Notify,
    ) {
        let current = || self.generation.load(Ordering::SeqCst) == generation;
        loop {
            let mut slot = self.runner.lock().await;
            if !current() {
                return;
            }
            let run = engine.lock().unwrap().evaluate(&cells);
            let Some(job) = run.jobs.into_iter().next() else {
                notify(run.results);
                return;
            };
            self.touch();
            let results = {
                let mut e = engine.lock().unwrap();
                e.set_running(Some(&job));
                e.run(&cells)
            };
            notify(results);
            if job.kind == cruncha_store::CellKind::Rust {
                let outcome = match self.rust() {
                    Some(cells) => Ok(rust_result(cells.run(&rust_request(&job.request), self.timeout).await)),
                    None => Err("Rust isn't set up".to_string()),
                };
                let results = {
                    let mut e = engine.lock().unwrap();
                    e.record(&job, outcome);
                    e.run(&cells)
                };
                notify(results);
                if !current() {
                    return;
                }
                continue;
            }
            let outcome = match self.ensure(&mut slot).await {
                Ok(k) => {
                    *self.interrupter.lock().unwrap() = Some(k.interrupter());
                    k.run(&job.request, self.timeout).await
                }
                Err(e) => Err(e),
            };
            *self.interrupter.lock().unwrap() = None;
            let outcome = outcome.map_err(|e| {
                if !matches!(e, KernelError::Worker(_)) {
                    // The worker is gone, and the values it held with it.
                    *slot = None;
                    engine.lock().unwrap().python_restarted();
                }
                e.to_string()
            });
            let results = {
                let mut e = engine.lock().unwrap();
                e.record(&job, outcome);
                e.run(&cells)
            };
            // Report even when stopped or replaced: results carry their document version, so
            // the window ignores them if a newer version's results have arrived.
            notify(results);
            if !current() {
                return;
            }
        }
    }

    /// Restart the worker: every Python value is recomputed.
    pub async fn restart(&self, engine: &Mutex<Engine>) {
        self.stop();
        if let Some(k) = self.runner.lock().await.take() {
            k.shutdown().await;
        }
        engine.lock().unwrap().python_restarted();
    }

    pub async fn shutdown(&self) {
        self.stop();
        if let Some(k) = self.runner.lock().await.take() {
            k.shutdown().await;
        }
        if let Some(k) = self.scanner.lock().await.take() {
            k.shutdown().await;
        }
    }
}

/// A Rust job's request (its inputs are all `Plain` values).
fn rust_request(r: &cruncha_kernel_py::RunRequest) -> cruncha_kernel_rs::cells::Request {
    cruncha_kernel_rs::cells::Request {
        cell: r.cell.clone(),
        source: r.source.clone(),
        defines: r.defines.clone(),
        inputs: r.inputs.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
    }
}

/// A Rust cell's outcome as a run result (the shape the engine records for code cells).
fn rust_result(o: cruncha_kernel_rs::cells::Outcome) -> cruncha_kernel_py::RunResult {
    use cruncha_kernel_py::{Defined, PyError, RunResult, Status};
    RunResult {
        status: if o.ok {
            Status::Done
        } else if o.interrupted {
            Status::Interrupted
        } else {
            Status::Error
        },
        defines: o
            .defines
            .into_iter()
            .map(|(name, summary, plain)| Defined { name, summary, plain })
            .collect(),
        stdout: o.stdout,
        stderr: o.stderr,
        outputs: vec![],
        error: o.error.map(|e| PyError { kind: e.kind, message: e.message, line: e.line, traceback: e.detail }),
        ms: o.compile_ms + o.run_ms,
    }
}
