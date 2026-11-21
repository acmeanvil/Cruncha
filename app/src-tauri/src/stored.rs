// SPDX-License-Identifier: GPL-3.0-or-later
//! Stored code-cell results (ADR-0018) between the engine and the document's `results/`.
//!
//! Before a Python cell runs, its `(cell id, input hash)` is looked up; a stored run with the
//! same key is used as its outcome, so a cell whose inputs nobody changed doesn't run again on
//! reopen, and past versions, snapshots and their reports show what was computed. Finished
//! runs are written back in the background. Results are a cache: they never add a document
//! version.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use cruncha_store::{Cell, Result, Store};

use crate::engine::{Engine, Run};

/// Lookups per evaluation: each round can unlock the next level of a Python → calc → Python
/// chain (a calc's value is known only once the cell it reads has an outcome).
const MAX_ROUNDS: usize = 16;

/// This computer's name, as `hostname` prints it.
fn host() -> &'static str {
    static HOST: OnceLock<String> = OnceLock::new();
    HOST.get_or_init(|| {
        std::process::Command::new("hostname")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown".into())
    })
}

/// Who this computer's runs are recorded as: `user:<name>@<host>`.
pub fn this_computer(actor: &str) -> String {
    format!("{actor}@{}", host())
}

/// Evaluate `cells`, using stored runs for every Python cell whose inputs match one. Nothing
/// runs here; the returned jobs are the cells still due.
pub async fn evaluate(store: &Store, engine: &Mutex<Engine>, cells: &[Cell]) -> Result<Run> {
    let mut tried: HashSet<(String, String)> = HashSet::new();
    let mut run = engine.lock().unwrap().evaluate(cells);
    for _ in 0..MAX_ROUNDS {
        let want: HashSet<(String, String)> = run.missing.iter().filter(|k| !tried.contains(*k)).cloned().collect();
        if want.is_empty() {
            break;
        }
        let found = store.results_for(&want).await?;
        tried.extend(want);
        if found.is_empty() {
            break;
        }
        let mut e = engine.lock().unwrap();
        e.use_stored(cells, found.into_values().collect());
        run = e.evaluate(cells);
    }
    Ok(run)
}

/// Write the runs the engine finished since the last call. Returns how many were stored.
pub async fn flush(store: &mut Store, engine: &Mutex<Engine>, actor: &str) -> Result<usize> {
    let runs = engine.lock().unwrap().take_unstored();
    if runs.is_empty() {
        return Ok(0);
    }
    let r = store.put_results(runs, actor).await?;
    for id in &r.too_large {
        eprintln!("cruncha: the output of cell {id} is over 64 MB; it isn't stored");
    }
    Ok(r.written)
}

/// The keys `cells` look up: their Python cells' input hashes.
async fn keys_of(store: &Store, engine: &Engine, cells: &[Cell]) -> Result<HashSet<(String, String)>> {
    let e = Mutex::new(engine.for_past());
    Ok(evaluate(store, &e, cells).await?.keys.into_iter().collect())
}

/// After a prune: keep the stored runs that what's left of the history can use, and drop the
/// rest (ADR-0018 §5). Kept: runs computed at a version still available, and every run the
/// latest version, the earliest available version or any snapshot (pinned or not) looks up.
/// Returns how many were dropped.
pub async fn retain_after_prune(store: &mut Store, engine: &Mutex<Engine>, actor: &str) -> Result<usize> {
    if store.result_entries().await?.is_empty() {
        return Ok(0);
    }
    let past = engine.lock().unwrap().for_past();
    let mut keep = keys_of(store, &past, &store.cells().await?).await?;
    keep.extend(keys_of(store, &past, &store.cells_at(store.earliest_version()).await?).await?);
    for r in store.refs().await? {
        keep.extend(keys_of(store, &past, &store.cells_at_ref(&r.name).await?).await?);
    }
    let earliest = store.earliest_version();
    let recent: HashSet<(String, String)> = store
        .result_entries()
        .await?
        .into_iter()
        .filter(|e| e.doc_version >= earliest)
        .map(|e| (e.cell_id, e.input_hash))
        .collect();
    store
        .retain_results(
            |cell, hash| {
                let k = (cell.to_owned(), hash.to_owned());
                keep.contains(&k) || recent.contains(&k)
            },
            actor,
        )
        .await
}
