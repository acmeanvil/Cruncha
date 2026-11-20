// SPDX-License-Identifier: GPL-3.0-or-later
//! Running a document's Python cells (ADR-0005): the worker processes and the run loop.
//!
//! Each open document has two workers: one **runs** cells, one only **scans** sources for the
//! names they define and read, so a commit never waits behind a long-running cell. Both start
//! on first use. The run loop takes the next due job from the [`Engine`], runs it, records the
//! outcome and reports fresh results, until nothing is due. A newer commit starts a newer
//! loop; the older one stops before its next job (the cell already running finishes, and its
//! outcome is memoized, so the newer loop reuses it when the inputs still match).

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
        })
    }

    /// The interpreter, if the environment is set up.
    pub fn interpreter(&self) -> Option<PathBuf> {
        env::ready(&self.base, LOCK)
    }

    async fn ensure<'a>(
        &self,
        slot: &'a mut Option<Kernel>,
    ) -> Result<&'a mut Kernel, KernelError> {
        if slot.as_mut().is_some_and(|k| !k.is_alive()) {
            *slot = None;
        }
        if slot.is_none() {
            let python = self
                .interpreter()
                .ok_or_else(|| KernelError::Start("Python isn't set up".into()))?;
            *slot = Some(Kernel::start(Config::new(python, &self.worker_dir)).await?);
        }
        Ok(slot.as_mut().expect("just started"))
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
                    eprintln!("cruncha: Python scan unavailable: {e}");
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
                    eprintln!("cruncha: Python scan failed: {e}");
                    *slot = None;
                }
            }
        }
        out
    }

    /// Stop: interrupt the running cell, and end the run loop.
    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(i) = *self.interrupter.lock().unwrap() {
            i.interrupt();
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
            let results = {
                let mut e = engine.lock().unwrap();
                e.set_running(Some(&job));
                e.run(&cells)
            };
            notify(results);
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
