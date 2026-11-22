// SPDX-License-Identifier: GPL-3.0-or-later
//! Evaluating a document: calc and Python cells run in dependency order (`cruncha-graph`),
//! and results are memoized by their source and input values (design 02 §3.1, rule 6), so
//! an edit re-runs only what it affects.
//!
//! Calcs run here, synchronously, with `cruncha-eval`; they are always unit-aware (ADR-0007).
//! Python cells run in the document's worker (ADR-0005), which is slow and asynchronous, so
//! [`Engine::run`] never runs them: it returns the Python cells that are due as [`PyJob`]s,
//! the caller (`python.rs`) runs them one at a time and [`Engine::record`]s each outcome, then
//! calls `run` again: calcs downstream pick up the new values, and the next job is due.
//!
//! A Python cell runs on its own only when the document is trusted and the cell's source has
//! been approved: it was there when the document was opened or trusted, the user ran it, or
//! only its inputs changed since it last ran. An edited cell waits for Run, so half-typed code
//! never runs. Rust cells take part in the graph but don't run yet.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use base64::Engine as _;
use cruncha_eval::{Options, Outcome, Plain, Statement, UnitAware, Value, Vars};
use cruncha_graph::{Graph, Issue, Node};
use cruncha_kernel_py::{Output, PyError, RunRequest, RunResult, Status};
use cruncha_kernel_py::Defined;
use cruncha_store::{Cell, CellKind, RunRecord, RunStatus};
use serde::{Deserialize, Serialize};

/// What a cell shows.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellResult {
    /// The display line as TeX for Temml (with the echo and result), or empty when there is
    /// nothing to typeset (the statement could not be prepared or was blocked by the graph).
    pub tex: String,
    /// Why the cell has no (correct) result.
    pub error: Option<String>,
    /// Plain-text result line (for tooltips, the outline, agents).
    pub text: String,
    /// Names the cell defines.
    pub defines: Vec<String>,
    /// Names the cell reads (for the variable browser's "used by").
    pub reads: Vec<String>,
    /// The value of each name the cell defined, as evaluated (empty if it didn't run).
    pub values: Vec<ValueView>,
    /// Lints: shown, but the cell still runs (e.g. a name used above its definition).
    pub warnings: Vec<String>,
    /// Waiting for a Python cell upstream to finish.
    pub pending: bool,
    /// For Python cells: run state and output.
    pub python: Option<PyView>,
    /// For `@test` calcs that evaluated: whether the check holds.
    pub check: Option<bool>,
}

/// One variable's value, for the variable browser.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ValueView {
    pub name: String,
    /// The value as TeX, the same form a later statement's echo shows (e.g. `24\,\text{ft}`).
    /// Empty for a Python value calcs can't use (then `summary` says what it is).
    pub tex: String,
    /// `number`, `matrix`, `dataframe`, … (see [`cruncha_eval::Kind`]), or `python`.
    pub kind: String,
    /// For values from Python: what the object is (`ndarray float64 (50,)`, `12.5 kip`).
    pub summary: Option<String>,
    /// For vectors, matrices and data frames: their size, e.g. `3 × 4 matrix`, so the browser
    /// can show a one-line label while the value is collapsed.
    pub shape: Option<String>,
}

/// `vector of 51`, `3 × 4 matrix`, `12 rows × 3 columns`: the size of a multi-line value.
fn shape_of(v: &Value) -> Option<String> {
    use cruncha_eval::{Kind, Plain};
    if !matches!(v.kind(), Kind::Vector | Kind::Matrix | Kind::DataFrame) {
        return None;
    }
    let plural = |n: usize, one: &str| {
        if n == 1 {
            format!("1 {one}")
        } else {
            format!("{n} {one}s")
        }
    };
    match v.to_plain() {
        Plain::Array {
            shape, orientation, ..
        } => match shape.as_slice() {
            [n] => Some(format!("{orientation} vector of {n}")),
            [r, c] => Some(format!("{r} × {c} matrix")),
            _ => None,
        },
        Plain::Frame { columns, row_names } => {
            let rows = columns.first().map_or(0, |c| match &c.data {
                cruncha_eval::ColumnData::Number { values } => values.len(),
                cruncha_eval::ColumnData::Text { values } => values.len(),
            });
            let cols = columns.len() - usize::from(row_names && !columns.is_empty());
            Some(format!(
                "{} × {}",
                plural(rows, "row"),
                plural(cols, "column")
            ))
        }
        _ => None,
    }
}

fn value_views(values: &[(String, Value)]) -> Vec<ValueView> {
    values
        .iter()
        .map(|(name, v)| ValueView {
            name: name.clone(),
            tex: v.tex().to_owned(),
            kind: format!("{:?}", v.kind()).to_lowercase(),
            summary: None,
            shape: shape_of(v),
        })
        .collect()
}

/// Where a Python cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PyState {
    /// There's no Python environment yet.
    NoPython,
    /// Rust isn't set up yet (a Rust cell).
    NoRust,
    /// A past version with no stored result for this cell (nothing runs for history).
    Past,
    /// The document isn't trusted to run code.
    Untrusted,
    /// Edited since it last ran: press Run.
    Edited,
    /// An input isn't available (its cell failed or hasn't run).
    Blocked,
    Queued,
    Running,
    Done,
    Error,
    Interrupted,
}

/// A Python cell's state and what it showed.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PyView {
    pub state: PyState,
    /// Why it's blocked, or a worker failure.
    pub note: Option<String>,
    pub ms: Option<f64>,
    pub stdout: String,
    pub stderr: String,
    pub outputs: Vec<OutputView>,
    pub error: Option<PyError>,
    /// Set when the outcome was read from the document's stored results (ADR-0018) rather
    /// than run in this session.
    pub stored: Option<StoredFrom>,
}

/// Where a stored result came from.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredFrom {
    /// `user:<name>@<host>`.
    pub by: String,
    /// Cruncha's clock, µs.
    pub at_us: i64,
    pub env: String,
    /// Computed on another computer: its numbers weren't reproduced here.
    pub elsewhere: bool,
    /// Computed with another Python environment (ADR-0019): shown only because the cell
    /// can't run here; it would run again otherwise.
    pub other_env: bool,
}

impl PyView {
    fn state(state: PyState, note: Option<String>) -> PyView {
        PyView {
            state,
            note,
            ms: None,
            stdout: String::new(),
            stderr: String::new(),
            outputs: Vec::new(),
            error: None,
            stored: None,
        }
    }
}

/// Rich output, ready for the web view (images as data URLs). Also how outputs are stored
/// (ADR-0018), so a change here needs a new [`RESULT_FORMAT`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OutputView {
    Image {
        src: String,
        width: u32,
        height: u32,
    },
    #[serde(rename_all = "camelCase")]
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<Option<String>>>,
        total_rows: u64,
    },
    Text {
        text: String,
    },
    /// An interactive figure (ADR-0017): the spec (JSON text) of an allowlisted MIME type,
    /// drawn in a sandboxed frame; `staticSrc` is a PNG or SVG the cell supplied (data URL).
    #[serde(rename_all = "camelCase")]
    Figure {
        mime: String,
        spec: String,
        summary: String,
        #[serde(default)]
        ranges: serde_json::Value,
        #[serde(default)]
        static_src: Option<String>,
    },
}

/// `data:<mime>;base64,…`
fn data_url(mime: &str, data: &[u8]) -> String {
    format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(data))
}

impl From<&Output> for OutputView {
    fn from(o: &Output) -> OutputView {
        match o {
            Output::Image {
                mime,
                data,
                width,
                height,
            } => OutputView::Image {
                src: data_url(mime, data),
                width: *width,
                height: *height,
            },
            Output::Table {
                columns,
                rows,
                total_rows,
            } => OutputView::Table {
                columns: columns
                    .iter()
                    .map(|c| {
                        c.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_owned()
                    })
                    .collect(),
                rows: rows.clone(),
                total_rows: *total_rows,
            },
            Output::Text { text } => OutputView::Text { text: text.clone() },
            Output::Figure { mime, spec, summary, ranges, static_image } => OutputView::Figure {
                mime: mime.clone(),
                spec: spec.clone(),
                summary: summary.clone(),
                ranges: ranges.clone(),
                static_src: static_image.as_ref().map(|(m, d)| data_url(m, d)),
            },
        }
    }
}

/// A Python cell that is due to run.
#[derive(Debug, Clone)]
pub struct PyJob {
    pub id: String,
    /// [`py_input_hash`]: what the run's result depends on.
    pub key: String,
    /// [`py_base_hash`]: the same without the environment (ADR-0019).
    pub base: String,
    /// `Python` or `Rust`: which runner takes it.
    pub kind: CellKind,
    pub request: RunRequest,
}

/// What a run defined: values calcs can use, and every name with its summary.
type RunValues = (Vec<(String, Value)>, Vec<(String, String)>);

/// The outcome of a Python cell's last run.
#[derive(Clone)]
struct PyOutcome {
    key: String,
    source: String,
    /// Values calcs can use.
    values: Vec<(String, Value)>,
    /// Everything it defined, with a summary (including values calcs can't use).
    defined: Vec<(String, String)>,
    view: PyView,
}

/// What the worker's dependency scan said about a Python source.
#[derive(Debug, Clone, Default)]
pub struct PyNames {
    pub defines: Vec<String>,
    pub reads: Vec<String>,
    pub warnings: Vec<String>,
    pub syntax_error: Option<PyError>,
}

/// The name Hurmet's `format = "…"` statement assigns; every calc reads it implicitly.
const FORMAT_VAR: &str = "format";

fn options() -> Options {
    Options {
        unit_aware: UnitAware::Always,
        ..Options::default()
    }
}

/// A prepared calc: its statement, or why it couldn't be prepared.
type Prepared = Result<Statement, String>;

/// Per-document evaluation state: prepared statements and memoized outcomes.
pub struct Engine {
    opts: Options,
    /// Prepared statements by source (sources are cheap to key; most cells don't change).
    prepared: HashMap<String, Prepared>,
    /// Last outcome per calc cell, with the hash of what produced it.
    memo: HashMap<String, (u64, Outcome)>,
    /// The variables after the last run (for live previews).
    vars: Vars,
    /// Statements evaluated by the last [`Engine::run`] (the rest came from the memo).
    pub evaluated: usize,
    /// Python: the worker's scan of each source, last outcome per cell, approved sources.
    py_names: HashMap<String, PyNames>,
    py_memo: HashMap<String, PyOutcome>,
    approved: HashMap<String, String>,
    /// The job running now: cell id and key (a stale job of an edited cell isn't "running").
    running: Option<(String, String)>,
    /// Is there a Python environment? Is the document trusted to run code?
    pub python_available: bool,
    pub trusted: bool,
    /// The Python environment's label (`env-…`); part of every Python cell's input hash.
    pub env: String,
    /// Is Rust set up (ADR-0006)? Its toolchain and cell format label, for Rust input hashes.
    pub rust_available: bool,
    pub rust_env: String,
    /// The `syn` scan of each Rust source.
    rust_names: HashMap<String, PyNames>,
    /// Evaluating a past version: Python cells don't run.
    pub past: bool,
    /// `user:<name>@<host>`: who this computer's runs are recorded as.
    pub computed_by: String,
    /// The latest document version (recorded with each run).
    pub doc_version: u64,
    /// Finished runs not yet written to the document's `results/`.
    unstored: Vec<RunRecord>,
    /// Memo misses of the current evaluation.
    missed: Vec<PyKey>,
    /// Stored runs from another Python environment, by cell, keyed by their base hash: shown
    /// when the cell can't run here (ADR-0019).
    foreign: HashMap<String, PyOutcome>,
}

/// A Python cell's hashes in one evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyKey {
    pub id: String,
    /// [`py_input_hash`].
    pub key: String,
    /// [`py_base_hash`].
    pub base: String,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

/// One evaluation of the document: what every cell shows, and the Python cells due to run.
pub struct Run {
    pub results: HashMap<String, CellResult>,
    pub jobs: Vec<PyJob>,
    /// Python cells with no outcome for their current inputs, `(cell id, input hash)`: what
    /// to look up in the stored results before anything runs.
    pub missing: Vec<PyKey>,
    /// Every Python cell's input hash, by cell id (whether it had an outcome or not).
    pub keys: HashMap<String, PyKey>,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            opts: options(),
            prepared: HashMap::new(),
            memo: HashMap::new(),
            vars: Vars::new(),
            evaluated: 0,
            py_names: HashMap::new(),
            py_memo: HashMap::new(),
            approved: HashMap::new(),
            running: None,
            python_available: false,
            trusted: false,
            env: String::new(),
            rust_available: false,
            rust_env: String::new(),
            rust_names: HashMap::new(),
            past: false,
            computed_by: String::new(),
            doc_version: 0,
            unstored: Vec::new(),
            missed: Vec::new(),
            foreign: HashMap::new(),
        }
    }

    fn prepare(&mut self, source: &str) -> &Prepared {
        if self.prepared.len() > 20_000 {
            self.prepared.clear();
        }
        let opts = &self.opts;
        self.prepared
            .entry(source.to_owned())
            .or_insert_with(|| cruncha_eval::prepare(source, opts).map_err(|e| e.to_string()))
    }

    /// The names a calc source defines and reads (empty if it doesn't prepare).
    pub fn names(&mut self, source: &str) -> (Vec<String>, Vec<String>) {
        match self.prepare(source.trim()) {
            Ok(s) => (s.defines.clone(), s.reads.clone()),
            Err(_) => (Vec::new(), Vec::new()),
        }
    }

    /// Record the worker's scan of a Python source.
    pub fn set_py_names(&mut self, source: &str, names: PyNames) {
        if self.py_names.len() > 5_000 {
            self.py_names.clear();
        }
        self.py_names.insert(source.to_owned(), names);
    }

    /// The worker's scan of a Python source, if it has one.
    /// A Rust source's names (scanned here, with `syn`; cached by source).
    pub fn rust_names(&mut self, source: &str) -> &PyNames {
        if !self.rust_names.contains_key(source) {
            let n = cruncha_kernel_rs::scan::scan(source);
            let names = PyNames {
                defines: n.defines,
                reads: n.reads,
                syntax_error: n.syntax_error.map(|(message, line)| cruncha_kernel_py::PyError {
                    kind: "SyntaxError".into(),
                    message,
                    line,
                    traceback: String::new(),
                }),
                ..Default::default()
            };
            self.rust_names.insert(source.to_owned(), names);
        }
        &self.rust_names[source]
    }

    pub fn py_names(&self, source: &str) -> Option<&PyNames> {
        self.py_names.get(source)
    }

    /// Python sources that haven't been scanned yet.
    pub fn unscanned<'a>(&self, cells: &'a [Cell]) -> Vec<&'a str> {
        let mut v: Vec<&str> = cells
            .iter()
            .filter(|c| c.kind == CellKind::Python && !self.py_names.contains_key(&c.source))
            .map(|c| c.source.as_str())
            .collect();
        v.dedup();
        v
    }

    /// Allow these cells' current sources to run (Run, opening or trusting the document).
    pub fn approve<'a>(&mut self, cells: impl IntoIterator<Item = &'a Cell>) {
        for c in cells {
            if matches!(c.kind, CellKind::Python | CellKind::Rust) {
                self.approved.insert(c.id.clone(), c.source.clone());
            }
        }
    }

    /// Run these cells again even if nothing changed (Run on a cell that already ran).
    pub fn rerun(&mut self, ids: &[String]) {
        for id in ids {
            self.py_memo.remove(id);
        }
    }

    /// The worker restarted: every value it held is gone.
    pub fn python_restarted(&mut self) {
        self.py_memo.clear();
        self.running = None;
    }

    pub fn set_running(&mut self, job: Option<&PyJob>) {
        self.running = job.map(|j| (j.id.clone(), j.key.clone()));
    }

    /// Calc values and summaries from what a run defined.
    fn values_of(&self, defines: &[Defined]) -> RunValues {
        let mut values = Vec::new();
        let mut defined = Vec::new();
        for d in defines {
            defined.push((d.name.clone(), d.summary.clone()));
            let plain: Option<Plain> = serde_json::from_value(d.plain.clone()).ok();
            if let Some(v) = plain
                .as_ref()
                .and_then(|p| Value::from_plain_with(p, &self.opts).ok())
            {
                values.push((d.name.clone(), v));
            }
        }
        (values, defined)
    }

    /// Store the outcome of a job.
    pub fn record(&mut self, job: &PyJob, result: Result<RunResult, String>) {
        let (mut values, mut defined) = (Vec::new(), Vec::new());
        let view = match result {
            Ok(r) => {
                (values, defined) = self.values_of(&r.defines);
                let view = PyView {
                    state: match r.status {
                        Status::Done => PyState::Done,
                        Status::Error => PyState::Error,
                        Status::Interrupted => PyState::Interrupted,
                    },
                    note: None,
                    ms: Some(r.ms),
                    stdout: r.stdout,
                    stderr: r.stderr,
                    outputs: r.outputs.iter().map(OutputView::from).collect(),
                    error: r.error,
                    stored: None,
                };
                // Kept for the document (ADR-0018). An interrupted run says nothing about
                // its inputs, and a past version never runs.
                if r.status != Status::Interrupted && !self.past {
                    self.unstored.push(RunRecord {
                        cell_id: job.id.clone(),
                        input_hash: job.key.clone(),
                        base_hash: Some(job.base.clone()),
                        status: if r.status == Status::Done { RunStatus::Done } else { RunStatus::Error },
                        values: serde_json::to_value(&r.defines).unwrap_or_default(),
                        outputs: serde_json::to_value(&view.outputs).unwrap_or_default(),
                        stdout: view.stdout.clone(),
                        stderr: view.stderr.clone(),
                        error: view.error.as_ref().and_then(|e| serde_json::to_string(e).ok()),
                        duration_ms: r.ms.max(0.0).round() as u64,
                        env: self.env.clone(),
                        computed_at_us: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_or(0, |d| d.as_micros() as i64),
                        computed_by: self.computed_by.clone(),
                        doc_version: self.doc_version,
                    });
                }
                view
            }
            Err(e) => PyView::state(PyState::Error, Some(e)),
        };
        // It ran, so this source is approved (it re-runs on its own when its inputs change,
        // also after a worker restart).
        self.approved
            .insert(job.id.clone(), job.request.source.clone());
        self.py_memo.insert(
            job.id.clone(),
            PyOutcome {
                key: job.key.clone(),
                source: job.request.source.clone(),
                values,
                defined,
                view,
            },
        );
        if self.running.as_ref().is_some_and(|(id, _)| *id == job.id) {
            self.running = None;
        }
    }

    /// Finished runs to write to the document's stored results (and forget here).
    pub fn take_unstored(&mut self) -> Vec<RunRecord> {
        std::mem::take(&mut self.unstored)
    }

    /// A stored run as an outcome, keyed by `key`.
    fn outcome_of(&self, source: &str, r: RunRecord, key: String) -> PyOutcome {
        let host = |by: &str| by.rsplit_once('@').map(|(_, h)| h.to_owned());
        let defines: Vec<Defined> = serde_json::from_value(r.values).unwrap_or_default();
        let outputs: Vec<OutputView> = serde_json::from_value(r.outputs).unwrap_or_default();
        let (values, defined) = self.values_of(&defines);
        let elsewhere = host(&r.computed_by) != host(&self.computed_by);
        let other_env = r.env != self.env;
        let view = PyView {
            state: match r.status {
                RunStatus::Done => PyState::Done,
                RunStatus::Error => PyState::Error,
                RunStatus::Interrupted => PyState::Interrupted,
            },
            note: None,
            ms: Some(r.duration_ms as f64),
            stdout: r.stdout,
            stderr: r.stderr,
            outputs,
            error: r.error.and_then(|e| serde_json::from_str(&e).ok()),
            stored: Some(StoredFrom { by: r.computed_by, at_us: r.computed_at_us, env: r.env, elsewhere, other_env }),
        };
        PyOutcome { key, source: source.to_owned(), values, defined, view }
    }

    /// Use stored runs (ADR-0018) as the outcomes of cells whose input hash they match, as if
    /// they had run. `cells` gives each cell's source. Returns how many were used. A stored
    /// run's values are data: nothing runs, and trust isn't needed to show them.
    pub fn use_stored(&mut self, cells: &[Cell], runs: Vec<RunRecord>) -> usize {
        let source: HashMap<&str, &str> = cells.iter().map(|c| (c.id.as_str(), c.source.as_str())).collect();
        let mut used = 0;
        for r in runs {
            let Some(src) = source.get(r.cell_id.as_str()) else { continue };
            let (id, key) = (r.cell_id.clone(), r.input_hash.clone());
            let o = self.outcome_of(src, r, key);
            self.py_memo.insert(id, o);
            used += 1;
        }
        used
    }

    /// Keep stored runs from another Python environment (found by their base hash,
    /// ADR-0019). They are shown only while their cell can't run here. Returns how many.
    pub fn use_foreign(&mut self, cells: &[Cell], runs: Vec<RunRecord>) -> usize {
        let source: HashMap<&str, &str> = cells.iter().map(|c| (c.id.as_str(), c.source.as_str())).collect();
        let mut used = 0;
        for r in runs {
            let (Some(src), Some(base)) = (source.get(r.cell_id.as_str()), r.base_hash.clone()) else { continue };
            let id = r.cell_id.clone();
            let o = self.outcome_of(src, r, base);
            self.foreign.insert(id, o);
            used += 1;
        }
        used
    }

    /// A fresh engine for evaluating a past version: this one's Python scans, environment and
    /// identity, no memo, and nothing runs.
    pub fn for_past(&self) -> Engine {
        let mut e = Engine::new();
        e.py_names = self.py_names.clone();
        e.env = self.env.clone();
        e.computed_by = self.computed_by.clone();
        e.python_available = self.python_available;
        e.rust_available = self.rust_available;
        e.rust_env = self.rust_env.clone();
        e.trusted = self.trusted;
        e.past = true;
        e
    }

    /// Is a Python cell allowed to run on its own?
    fn may_run(&self, c: &Cell) -> bool {
        self.approved.get(&c.id) == Some(&c.source)
            || self
                .py_memo
                .get(&c.id)
                .is_some_and(|o| o.source == c.source)
    }

    /// Evaluate every calc cell of `cells` (document order) and return their results by id.
    pub fn run(&mut self, cells: &[Cell]) -> HashMap<String, CellResult> {
        self.evaluate(cells).results
    }

    /// Evaluate the document: calcs run, Python cells come from their last outcome, and the
    /// Python cells that are due are returned as jobs (in dependency order).
    /// The dependency graph of `cells`: calcs from their prepared statements, Python cells
    /// from the worker's scan, other cells from their stored names.
    pub fn graph(&mut self, cells: &[Cell]) -> Graph {
        // Graph nodes: calcs from their prepared statements, Python cells from the worker's
        // scan (when there is one), other cells from their stored names.
        let mut nodes = Vec::with_capacity(cells.len());
        for c in cells {
            let (defines, reads) = match c.kind {
                CellKind::Calc => self.names(&c.source),
                CellKind::Python => match self.py_names.get(&c.source) {
                    Some(n) => (n.defines.clone(), n.reads.clone()),
                    None => (c.defines.clone(), c.reads.clone()),
                },
                CellKind::Rust => {
                    let n = self.rust_names(&c.source);
                    (n.defines.clone(), n.reads.clone())
                }
                _ => (c.defines.clone(), c.reads.clone()),
            };
            nodes.push(Node {
                id: c.id.clone(),
                defines,
                reads,
            });
        }
        if nodes
            .iter()
            .any(|n| n.defines.iter().any(|d| d == FORMAT_VAR))
        {
            for (n, c) in nodes.iter_mut().zip(cells) {
                if c.kind == CellKind::Calc && !n.defines.iter().any(|d| d == FORMAT_VAR) {
                    n.reads.push(FORMAT_VAR.into());
                }
            }
        }
        Graph::new(nodes)
    }

    /// Evaluate `cells` as if they were the document (a proposal applied), without touching
    /// the memoized state the document's own results come from. Python cells aren't run:
    /// changed ones show as not calculated, unchanged ones keep their last outcome.
    pub fn what_if(&mut self, cells: &[Cell]) -> HashMap<String, CellResult> {
        let memo = std::mem::take(&mut self.memo);
        let vars = self.vars.clone();
        let py_memo = self.py_memo.clone();
        let evaluated = self.evaluated;
        let results = self.evaluate(cells).results;
        self.memo = memo;
        self.vars = vars;
        self.py_memo = py_memo;
        self.evaluated = evaluated;
        results
    }

    pub fn evaluate(&mut self, cells: &[Cell]) -> Run {
        self.evaluated = 0;
        self.missed.clear();
        let graph = self.graph(cells);
        let cell_of: HashMap<&str, &Cell> = cells.iter().map(|c| (c.id.as_str(), c)).collect();
        let kind_of: HashMap<&str, CellKind> = cells.iter().map(|c| (c.id.as_str(), c.kind)).collect();

        let mut vars = Vars::new();
        let mut out = HashMap::new();
        let mut jobs = Vec::new();
        let mut live: HashSet<String> = HashSet::new();
        // Python cells that will produce values later (queued or running) / their keys.
        let mut py_pending: HashSet<String> = HashSet::new();
        let mut py_keys: HashMap<String, PyKey> = HashMap::new();
        // Names defined in Python whose values calcs can't use, with what they are.
        let mut opaque: HashMap<String, String> = HashMap::new();
        let order: Vec<String> = graph.order().map(str::to_owned).collect();
        for id in &order {
            let node = graph
                .nodes()
                .iter()
                .find(|n| &n.id == id)
                .expect("graph node for every cell");
            let cell = cell_of[id.as_str()];
            let issues = graph.issues(id);
            let mut warnings: Vec<String> = issues
                .iter()
                .filter(|i| !i.is_error())
                .map(Issue::to_string)
                .collect();
            let mut result = CellResult {
                defines: node.defines.clone(),
                reads: node
                    .reads
                    .iter()
                    .filter(|r| *r != FORMAT_VAR)
                    .cloned()
                    .collect(),
                ..CellResult::default()
            };
            match cell.kind {
                CellKind::Calc => {}
                CellKind::Python | CellKind::Rust => {
                    live.insert(id.clone());
                    if let Some(n) = self.py_names.get(&cell.source).filter(|_| cell.kind == CellKind::Python) {
                        warnings.extend(n.warnings.iter().cloned());
                    }
                    result.warnings = warnings;
                    self.python_cell(
                        cell,
                        &graph,
                        issues,
                        &cell_of,
                        &mut vars,
                        &mut result,
                        &mut jobs,
                        &mut py_pending,
                        &mut py_keys,
                        &mut opaque,
                    );
                    out.insert(id.clone(), result);
                    continue;
                }
                _ => {
                    // Other cells take part in the graph but don't run yet.
                    if !node.defines.is_empty() {
                        result.error = Some("these cells can't run yet".to_string());
                        out.insert(id.clone(), result);
                    }
                    continue;
                }
            }
            live.insert(id.clone());
            result.warnings = warnings;
            let source = cell.source.trim();
            if source.is_empty() {
                out.insert(id.clone(), result);
                continue;
            }
            if let Some(e) = issues.iter().find(|i| i.is_error()) {
                result.error = Some(e.to_string());
                out.insert(id.clone(), result);
                continue;
            }
            // Inputs from cells that haven't produced a usable value.
            if let Some(blocked) =
                self.unavailable(node, &graph, &kind_of, &vars, &py_pending, &opaque)
            {
                match blocked {
                    Blocked::Pending => result.pending = true,
                    Blocked::Because(e) => result.error = Some(e),
                }
                out.insert(id.clone(), result);
                continue;
            }
            let stmt = match self.prepare(source) {
                Ok(s) => s.clone(),
                Err(e) => {
                    result.error = Some(e.clone());
                    out.insert(id.clone(), result);
                    continue;
                }
            };
            let key = input_hash(source, &stmt, &vars);
            let outcome = match self.memo.get(id) {
                Some((h, o)) if *h == key => o.clone(),
                _ => {
                    self.evaluated += 1;
                    let o = cruncha_eval::evaluate(&stmt, &vars, &self.opts);
                    self.memo.insert(id.clone(), (key, o.clone()));
                    o
                }
            };
            for (name, value) in &outcome.values {
                vars.insert(name.clone(), value.clone());
            }
            result.values = value_views(&outcome.values);
            result.tex = outcome.tex;
            result.text = outcome.result;
            result.check = outcome.check;
            result.error = outcome.error.map(|e| clean_message(&e));
            out.insert(id.clone(), result);
        }
        self.memo.retain(|id, _| live.contains(id));
        self.py_memo.retain(|id, _| live.contains(id));
        self.vars = vars;
        Run { results: out, jobs, missing: std::mem::take(&mut self.missed), keys: py_keys }
    }

    /// Why a cell can't use its inputs yet, if it can't.
    fn unavailable(
        &self,
        node: &Node,
        graph: &Graph,
        kind_of: &HashMap<&str, CellKind>,
        vars: &Vars,
        py_pending: &HashSet<String>,
        opaque: &HashMap<String, String>,
    ) -> Option<Blocked> {
        for name in &node.reads {
            let Some(definer) = graph.definer(name) else {
                continue;
            };
            match kind_of.get(definer) {
                Some(k @ (CellKind::Python | CellKind::Rust)) => {
                    if vars.contains(name) {
                        continue;
                    }
                    if py_pending.contains(definer) {
                        return Some(Blocked::Pending);
                    }
                    let lang = if *k == CellKind::Rust { "Rust" } else { "Python" };
                    if let Some(what) = opaque.get(name) {
                        return Some(Blocked::Because(format!(
                            "{name} is a {lang} {what}; calcs can use numbers, quantities, vectors, matrices and frames"
                        )));
                    }
                    let available = if *k == CellKind::Rust { self.rust_available } else { self.python_available };
                    let why = if self.past {
                        "its cell has no stored result for this version"
                    } else if !available {
                        if *k == CellKind::Rust { "Rust isn't set up yet" } else { "Python isn't set up yet" }
                    } else if !self.trusted {
                        "this document isn't trusted to run code"
                    } else {
                        "it hasn't run successfully"
                    };
                    return Some(Blocked::Because(format!(
                        "{name} comes from a {lang} cell; {why}"
                    )));
                }
                Some(CellKind::Calc) | None => {}
                Some(_) => {
                    return Some(Blocked::Because(format!(
                        "{name} comes from a cell that can't run yet"
                    )));
                }
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn python_cell(
        &mut self,
        cell: &Cell,
        graph: &Graph,
        issues: &[Issue],
        cell_of: &HashMap<&str, &Cell>,
        vars: &mut Vars,
        result: &mut CellResult,
        jobs: &mut Vec<PyJob>,
        py_pending: &mut HashSet<String>,
        py_keys: &mut HashMap<String, PyKey>,
        opaque: &mut HashMap<String, String>,
    ) {
        let id = cell.id.as_str();
        let set = |r: &mut CellResult, state, note: Option<String>| {
            r.python = Some(PyView::state(state, note))
        };
        if cell.source.trim().is_empty() {
            return;
        }
        let names = if cell.kind == CellKind::Rust { self.rust_names.get(&cell.source) } else { self.py_names.get(&cell.source) };
        if let Some(e) = names.and_then(|n| n.syntax_error.clone()) {
            result.error = Some(format!(
                "{}: {} (line {})",
                e.kind,
                e.message,
                e.line.unwrap_or(0)
            ));
            let mut v = PyView::state(PyState::Error, None);
            v.error = Some(e);
            result.python = Some(v);
            return;
        }
        if let Some(e) = issues.iter().find(|i| i.is_error()) {
            result.error = Some(e.to_string());
            return set(result, PyState::Blocked, Some(e.to_string()));
        }
        // Inputs: calc values (sent along) and Python values (already in the worker).
        let mut parts: Vec<(String, String)> = Vec::with_capacity(result.reads.len());
        // The same, with upstream Python cells' base hashes (ADR-0019).
        let mut base_parts: Vec<(String, String)> = Vec::with_capacity(result.reads.len());
        let mut request = RunRequest {
            cell: cell.id.clone(),
            source: cell.source.clone(),
            defines: result.defines.clone(),
            ..Default::default()
        };
        let mut waiting = false;
        let mut blocked: Option<String> = None;
        for name in &result.reads {
            let definer = graph.definer(name);
            let definer_kind = definer.and_then(|d| cell_of.get(d)).map(|c| c.kind);
            match definer_kind {
                // A Python cell reading another Python cell's value: it's in the worker.
                Some(CellKind::Python) if cell.kind == CellKind::Python => {
                    let d = definer.unwrap();
                    if py_pending.contains(d) {
                        waiting = true;
                    } else if self
                        .py_memo
                        .get(d)
                        .is_none_or(|o| o.view.state != PyState::Done)
                    {
                        blocked.get_or_insert(format!(
                            "{name} comes from a Python cell that hasn't run successfully"
                        ));
                    }
                    let (up, up_base) = py_keys.get(d).map_or(("?", "?"), |k| (k.key.as_str(), k.base.as_str()));
                    parts.push((name.clone(), format!("python:{up}")));
                    base_parts.push((name.clone(), format!("python:{up_base}")));
                    request.python_inputs.push(name.clone());
                }
                // Values between languages (and from Rust) travel as Plain, like calc values.
                Some(k @ (CellKind::Python | CellKind::Rust)) => {
                    let d = definer.unwrap();
                    let lang = if k == CellKind::Rust { "rust" } else { "python" };
                    let (up, up_base) = py_keys.get(d).map_or(("?", "?"), |k| (k.key.as_str(), k.base.as_str()));
                    parts.push((name.clone(), format!("{lang}:{up}")));
                    base_parts.push((name.clone(), format!("{lang}:{up_base}")));
                    if py_pending.contains(d) {
                        waiting = true;
                    } else if let Some(what) = opaque.get(name) {
                        blocked.get_or_insert(format!("{name} is a {what}, which can't be passed between languages"));
                    } else if let Some(v) = vars.get(name) {
                        request.inputs.insert(name.clone(), serde_json::to_value(v.to_plain()).expect("plain serializes"));
                    } else {
                        blocked.get_or_insert(format!("{name} comes from a cell that hasn't run successfully"));
                    }
                }
                Some(CellKind::Calc) => match vars.get(name) {
                    Some(v) => {
                        let plain = v.to_plain();
                        let json = serde_json::to_value(&plain).expect("plain serializes");
                        parts.push((name.clone(), format!("calc:{}", canonical_json(&json))));
                        base_parts.push(parts.last().cloned().expect("just pushed"));
                        match plain {
                            Plain::Unsupported { kind, .. } => {
                                blocked.get_or_insert(format!(
                                    "{name} ({kind}) can't be passed to a code cell yet"
                                ));
                            }
                            p => {
                                request.inputs.insert(
                                    name.clone(),
                                    serde_json::to_value(&p).expect("plain serializes"),
                                );
                            }
                        }
                    }
                    None => {
                        parts.push((name.clone(), "missing".into()));
                        base_parts.push((name.clone(), "missing".into()));
                        blocked
                            .get_or_insert(format!("{name} has no value (its calc has an error)"));
                    }
                },
                Some(_) => {
                    parts.push((name.clone(), "other".into()));
                    base_parts.push((name.clone(), "other".into()));
                    blocked.get_or_insert(format!("{name} comes from a cell that can't run yet"));
                }
                // Not defined anywhere: Python raises NameError, which says it best.
                None => {
                    parts.push((name.clone(), "undefined".into()));
                    base_parts.push((name.clone(), "undefined".into()));
                }
            }
        }
        let env = if cell.kind == CellKind::Rust { &self.rust_env } else { &self.env };
        let key = py_input_hash(cell.kind, &cell.source, env, parts);
        let base = py_base_hash(cell.kind, &cell.source, base_parts);
        let k = PyKey { id: id.to_owned(), key: key.clone(), base: base.clone() };
        py_keys.insert(id.to_owned(), k.clone());

        if let Some(o) = self.py_memo.get(id).filter(|o| o.key == key) {
            return apply_outcome(o, vars, opaque, result);
        }
        self.missed.push(k);
        if self.running.as_ref() == Some(&(id.to_owned(), key.clone())) {
            py_pending.insert(id.to_owned());
            return set(result, PyState::Running, None);
        }
        // A cell that can't run here shows a run from another environment, if there is one
        // (ADR-0019); one that can run, runs.
        let foreign = self.foreign.get(id).filter(|o| o.key == base);
        let cannot = if self.past {
            Some(PyState::Past)
        } else if cell.kind == CellKind::Python && !self.python_available {
            Some(PyState::NoPython)
        } else if cell.kind == CellKind::Rust && !self.rust_available {
            Some(PyState::NoRust)
        } else if !self.trusted {
            Some(PyState::Untrusted)
        } else if blocked.is_none() && !self.may_run(cell) {
            Some(PyState::Edited)
        } else {
            None
        };
        if let Some(state) = cannot {
            return match foreign {
                Some(o) => apply_outcome(o, vars, opaque, result),
                None => set(result, state, None),
            };
        }
        if let Some(why) = blocked {
            return set(result, PyState::Blocked, Some(why));
        }
        py_pending.insert(id.to_owned());
        if !waiting {
            jobs.push(PyJob {
                id: id.to_owned(),
                key,
                base,
                kind: cell.kind,
                request,
            });
        }
        set(result, PyState::Queued, None)
    }

    /// Evaluate `source` against the document's current values, without storing anything:
    /// the live preview while a calc is being typed.
    pub fn preview(&mut self, source: &str) -> CellResult {
        let source = source.trim();
        if source.is_empty() {
            return CellResult::default();
        }
        let stmt = match self.prepare(source) {
            Ok(s) => s.clone(),
            Err(e) => {
                return CellResult {
                    error: Some(e.clone()),
                    ..CellResult::default()
                };
            }
        };
        let o = cruncha_eval::evaluate(&stmt, &self.vars, &self.opts);
        CellResult {
            tex: o.tex,
            text: o.result,
            error: o.error.map(|e| clean_message(&e)),
            check: o.check,
            values: value_views(&o.values),
            defines: stmt.defines,
            reads: stmt.reads,
            ..CellResult::default()
        }
    }
}

/// Show `o` as the cell's outcome: its values for calcs, its output for the window.
fn apply_outcome(o: &PyOutcome, vars: &mut Vars, opaque: &mut HashMap<String, String>, result: &mut CellResult) {
    for (name, value) in &o.values {
        vars.insert(name.clone(), value.clone());
    }
    for (name, summary) in &o.defined {
        if !o.values.iter().any(|(n, _)| n == name) {
            opaque.insert(name.clone(), summary.clone());
        }
    }
    result.values = py_value_views(o);
    if o.view.state == PyState::Error {
        result.error = Some(
            o.view
                .error
                .as_ref()
                .map(|e| format!("{}: {}", e.kind, e.message))
                .or(o.view.note.clone())
                .unwrap_or_else(|| "failed".into()),
        );
    }
    result.python = Some(o.view.clone());
}

enum Blocked {
    Pending,
    Because(String),
}

fn py_value_views(o: &PyOutcome) -> Vec<ValueView> {
    o.defined
        .iter()
        .map(
            |(name, summary)| match o.values.iter().find(|(n, _)| n == name) {
                Some((_, v)) => ValueView {
                    name: name.clone(),
                    tex: v.tex().to_owned(),
                    kind: format!("{:?}", v.kind()).to_lowercase(),
                    summary: Some(summary.clone()),
                    shape: shape_of(v),
                },
                None => ValueView {
                    name: name.clone(),
                    tex: String::new(),
                    kind: "python".into(),
                    summary: Some(summary.clone()),
                    shape: None,
                },
            },
        )
        .collect()
}

/// Hurmet marks names in messages with `$…$`; the UI shows plain text.
fn clean_message(m: &str) -> String {
    m.replace('$', "")
}

/// Bump when a change to the engine or the worker would make an old stored result wrong for
/// the same inputs (how values or outputs are encoded, how inputs are passed): every code
/// cell's input hash changes, so nothing stored earlier is reused (ADR-0018).
pub const RESULT_FORMAT: u32 = 1;

/// What a code cell's run depends on, as hex SHA-256 (ADR-0018 §2): stable across Rust
/// versions, platforms and sessions, so it can key stored results. `inputs` are each name it
/// reads with what that name is (`calc:<canonical Plain JSON>`, `python:<upstream hash>`, …);
/// their order doesn't matter.
pub fn py_input_hash(kind: CellKind, source: &str, env: &str, inputs: Vec<(String, String)>) -> String {
    hash_inputs(&format!("cruncha-result/{RESULT_FORMAT}"), kind, source, Some(env), inputs)
}

/// [`py_input_hash`] without the environment (ADR-0019), under its own tag: finds a run with
/// the same inputs from any Python environment. Upstream Python inputs must be given by their
/// base hashes, so it is environment-free all the way up.
pub fn py_base_hash(kind: CellKind, source: &str, inputs: Vec<(String, String)>) -> String {
    hash_inputs(&format!("cruncha-result-base/{RESULT_FORMAT}"), kind, source, None, inputs)
}

fn hash_inputs(tag: &str, kind: CellKind, source: &str, env: Option<&str>, mut inputs: Vec<(String, String)>) -> String {
    use sha2::{Digest, Sha256};
    fn field(h: &mut Sha256, s: &str) {
        h.update((s.len() as u64).to_le_bytes());
        h.update(s.as_bytes());
    }
    inputs.sort();
    inputs.dedup();
    let mut h = Sha256::new();
    field(&mut h, tag);
    field(&mut h, kind.as_str());
    field(&mut h, source);
    if let Some(env) = env {
        field(&mut h, env);
    }
    h.update((inputs.len() as u64).to_le_bytes());
    for (name, what) in &inputs {
        field(&mut h, name);
        field(&mut h, what);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// JSON with object keys sorted and no whitespace, whatever map order serde_json uses.
pub fn canonical_json(v: &serde_json::Value) -> String {
    fn write(v: &serde_json::Value, out: &mut String) {
        match v {
            serde_json::Value::Object(o) => {
                let mut keys: Vec<&String> = o.keys().collect();
                keys.sort();
                out.push('{');
                for (i, k) in keys.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&serde_json::to_string(k).expect("string serializes"));
                    out.push(':');
                    write(&o[k], out);
                }
                out.push('}');
            }
            serde_json::Value::Array(a) => {
                out.push('[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(x, out);
                }
                out.push(']');
            }
            scalar => out.push_str(&scalar.to_string()),
        }
    }
    let mut out = String::new();
    write(v, &mut out);
    out
}

/// What determines a calc's outcome: its source and the values of what it reads.
fn input_hash(source: &str, stmt: &Statement, vars: &Vars) -> u64 {
    let mut h = DefaultHasher::new();
    source.hash(&mut h);
    for name in stmt
        .reads
        .iter()
        .chain(std::iter::once(&FORMAT_VAR.to_string()))
    {
        name.hash(&mut h);
        // `Value` has no Hash; its Debug form is complete and deterministic.
        format!("{:?}", vars.get(name)).hash(&mut h);
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(id: &str, key: &str, src: &str) -> Cell {
        let mut c = Cell::new(CellKind::Calc, key, src);
        c.id = id.into();
        c
    }

    fn beam() -> Vec<Cell> {
        vec![
            calc("W", "a1", "w = 1.5 kip/ft"),
            calc("L", "a2", "L = 24 ft"),
            calc("M", "a3", "M = (w L^2)/8 = ?? kip·ft"),
            calc("MA", "a4", "M_allow = 150 kip·ft"),
            calc("T", "a5", "@test M < M_allow"),
        ]
    }

    #[test]
    fn evaluates_in_dependency_order_with_units() {
        let mut e = Engine::new();
        let r = e.run(&beam());
        assert_eq!(r["M"].error, None, "{:?}", r["M"]);
        assert!(r["M"].text.ends_with("108 kip·ft"), "{}", r["M"].text);
        assert!(r["M"].tex.contains("108"), "{}", r["M"].tex);
        assert_eq!(r["M"].defines, ["M"]);
        assert_eq!(r["M"].reads, ["w", "L"]);
        assert_eq!(r["M"].values.len(), 1);
        assert_eq!(r["M"].values[0].name, "M");
        assert_eq!(r["M"].values[0].kind, "number");
        assert!(r["M"].values[0].tex.contains("108"), "{:?}", r["M"].values);
        assert_eq!(r["T"].error, None, "{:?}", r["T"]);
        // Order follows dependencies: move M above w and L and it still works, with a lint.
        let mut cells = beam();
        cells.swap(0, 2);
        let r = e.run(&cells);
        assert_eq!(r["M"].error, None);
        assert!(
            r["M"]
                .warnings
                .iter()
                .any(|w| w.contains("defined further down")),
            "{:?}",
            r["M"]
        );
    }

    #[test]
    fn always_unit_aware() {
        let mut e = Engine::new();
        let r = e.run(&[calc("L", "a", "L = 3 ft"), calc("X", "b", "2 L = ?")]);
        assert!(r["X"].text.contains("1.8288"), "{}", r["X"].text);
    }

    #[test]
    fn memoizes_unchanged_cells_and_reruns_downstream() {
        let mut e = Engine::new();
        let mut cells = beam();
        e.run(&cells);
        assert_eq!(e.evaluated, 5);
        e.run(&cells);
        assert_eq!(e.evaluated, 0, "nothing changed");
        cells[1].source = "L = 26 ft".into(); // L, then M and the test downstream
        let r = e.run(&cells);
        assert_eq!(e.evaluated, 3);
        assert!(r["M"].text.ends_with("126.75 kip·ft"), "{}", r["M"].text);
        // Renaming L away: M re-runs and reports the missing name.
        cells[1].source = "span = 26 ft".into();
        let r = e.run(&cells);
        assert!(
            r["M"].error.as_deref().is_some_and(|m| m.contains('L')),
            "{:?}",
            r["M"]
        );
        assert!(!r["M"].error.as_ref().unwrap().contains('$'));
    }

    #[test]
    fn graph_errors_block_evaluation() {
        let mut e = Engine::new();
        let r = e.run(&[
            calc("A", "a", "x = 1"),
            calc("B", "b", "x = 2"),
            calc("C", "c", "y = x + 1 = ?"),
            // Hurmet needs a selector for a calculation: `p = q + 1` alone is displayed math.
            calc("P", "d", "p = q + 1 = ?"),
            calc("Q", "e", "q = p + 1 = ?"),
        ]);
        assert!(
            r["A"]
                .error
                .as_deref()
                .unwrap()
                .contains("defined in 1 other cell")
        );
        assert!(r["C"].error.as_deref().unwrap().contains("x has an error"));
        assert!(r["P"].error.as_deref().unwrap().contains("circular"));
        assert!(r["A"].tex.is_empty());
    }

    #[test]
    fn python_names_and_previews() {
        let mut e = Engine::new();
        let mut py = Cell::new(CellKind::Python, "a", "x = 2");
        py.id = "PY".into();
        py.defines = vec!["x".into()];
        let r = e.run(&[py, calc("C", "b", "y = 2 x = ?"), calc("L", "c", "L = 3 m")]);
        assert_eq!(
            r["C"].error.as_deref(),
            Some("x comes from a Python cell; Python isn't set up yet")
        );
        // The Python cell's names are reported too (for the variable browser).
        assert_eq!(r["PY"].defines, ["x"]);
        assert!(r["PY"].values.is_empty());
        assert_eq!(r["PY"].python.as_ref().unwrap().state, PyState::NoPython);
        // A preview uses the document's values, without storing anything.
        let p = e.preview("A = L^2 = ?? m^2");
        assert!(
            p.text.ends_with("9 m^2") || p.text.ends_with("9 m²"),
            "{}",
            p.text
        );
        // An unknown name is reported, with Hurmet's `$…$` markers removed.
        let p = e.preview("z = nope + 1 = ?");
        assert!(
            p.error
                .as_deref()
                .is_some_and(|m| m.contains("nope") && !m.contains('$')),
            "{p:?}"
        );
        assert_eq!(
            e.names("M = w L^2 / 8 = ?? kip·ft"),
            (
                vec!["M".to_string()],
                vec!["w".to_string(), "L".to_string()]
            )
        );
    }

    #[test]
    fn format_statements_apply_everywhere() {
        let mut e = Engine::new();
        let r = e.run(&[
            calc("F", "a", "format = \"f2\""),
            calc("X", "b", "x = 1/3 = ?"),
        ]);
        assert!(r["X"].text.ends_with("0.33"), "{}", r["X"].text);
    }

    fn py_cell(id: &str, key: &str, src: &str) -> Cell {
        let mut c = Cell::new(CellKind::Python, key, src);
        c.id = id.into();
        c
    }

    fn names(e: &mut Engine, src: &str, defines: &[&str], reads: &[&str]) {
        let v = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect();
        e.set_py_names(
            src,
            PyNames {
                defines: v(defines),
                reads: v(reads),
                ..Default::default()
            },
        );
    }

    fn done(defines: Vec<cruncha_kernel_py::Defined>) -> RunResult {
        RunResult {
            status: Status::Done,
            defines,
            stdout: String::new(),
            stderr: String::new(),
            outputs: vec![],
            error: None,
            ms: 1.0,
        }
    }

    #[test]
    fn python_cells_queue_run_and_feed_calcs() {
        let mut e = Engine::new();
        e.python_available = true;
        e.trusted = true;
        names(&mut e, "y = 2 * x", &["y"], &["x"]);
        let mut cells = vec![
            calc("X", "a", "x = 3 m"),
            py_cell("PY", "b", "y = 2 * x"),
            calc("Z", "c", "z = y + 1m = ?? m"),
        ];

        // Not approved yet (typed here, never run): it waits for Run.
        let r = e.evaluate(&cells);
        assert!(r.jobs.is_empty());
        assert_eq!(
            r.results["PY"].python.as_ref().unwrap().state,
            PyState::Edited
        );
        assert!(
            r.results["Z"]
                .error
                .as_deref()
                .unwrap()
                .contains("hasn't run"),
            "{:?}",
            r.results["Z"]
        );

        // Approved: queued, with x sent as a quantity; z waits for it.
        e.approve(&cells);
        let r = e.evaluate(&cells);
        assert_eq!(r.jobs.len(), 1);
        let job = r.jobs[0].clone();
        assert_eq!(job.request.inputs["x"]["type"], "number");
        assert_eq!(job.request.inputs["x"]["unit"]["name"], "m");
        assert_eq!(
            r.results["PY"].python.as_ref().unwrap().state,
            PyState::Queued
        );
        assert!(r.results["Z"].pending);

        let six_m = serde_json::json!({"type": "number", "value": {"type": "float", "value": 6.0},
            "unit": {"name": "m", "factor": 1.0, "gauge": 0.0, "dims": [1, 0, 0, 0, 0, 0, 0, 0]}});
        e.record(
            &job,
            Ok(done(vec![cruncha_kernel_py::Defined {
                name: "y".into(),
                summary: "6 m".into(),
                plain: six_m,
            }])),
        );
        let r = e.evaluate(&cells);
        assert!(r.jobs.is_empty(), "memoized");
        assert!(r.results["Z"].text.ends_with("7 m"), "{:?}", r.results["Z"]);
        assert_eq!(
            r.results["PY"].python.as_ref().unwrap().state,
            PyState::Done
        );
        assert!(!r.results["PY"].values[0].tex.is_empty());

        // A changed input re-runs it on its own; an edit to its source waits for Run.
        cells[0].source = "x = 4 m".into();
        assert_eq!(e.evaluate(&cells).jobs.len(), 1);
        cells[1].source = "y = 3 * x".into();
        names(&mut e, "y = 3 * x", &["y"], &["x"]);
        let r = e.evaluate(&cells);
        assert!(r.jobs.is_empty());
        assert_eq!(
            r.results["PY"].python.as_ref().unwrap().state,
            PyState::Edited
        );

        // Untrusted documents and past versions don't run Python.
        e.trusted = false;
        e.approve(&cells);
        assert_eq!(
            e.evaluate(&cells).results["PY"]
                .python
                .as_ref()
                .unwrap()
                .state,
            PyState::Untrusted
        );
        let mut past = Engine::new();
        past.past = true;
        let r = past.evaluate(&cells);
        assert_eq!(
            r.results["PY"].python.as_ref().unwrap().state,
            PyState::Past
        );
    }

    #[test]
    fn python_values_calcs_cant_use_say_what_they_are() {
        let mut e = Engine::new();
        e.python_available = true;
        e.trusted = true;
        names(&mut e, "f = lambda t: t", &["f"], &[]);
        let cells = vec![
            py_cell("PY", "a", "f = lambda t: t"),
            calc("Z", "b", "z = f + 1 = ?"),
        ];
        e.approve(&cells);
        let job = e.evaluate(&cells).jobs.remove(0);
        let opaque = serde_json::json!({"type": "unsupported", "kind": "builtins.function", "summary": "function <lambda>"});
        e.record(
            &job,
            Ok(done(vec![cruncha_kernel_py::Defined {
                name: "f".into(),
                summary: "function <lambda>".into(),
                plain: opaque,
            }])),
        );
        let r = e.evaluate(&cells);
        assert!(
            r.results["Z"]
                .error
                .as_deref()
                .unwrap()
                .contains("f is a Python function <lambda>"),
            "{:?}",
            r.results["Z"]
        );
        assert_eq!(r.results["PY"].values[0].kind, "python");
    }

    #[test]
    fn python_input_hash_is_stable_and_order_independent() {
        let inputs = |v: &[(&str, &str)]| v.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect::<Vec<_>>();
        let a = py_input_hash(CellKind::Python, "y = L * 2", "env-1", inputs(&[("L", "calc:{\"v\":1}"), ("w", "python:ab")]));
        // A fixed encoding: this value must never change without bumping RESULT_FORMAT.
        assert_eq!(a, GOLDEN_HASH);
        let b = py_input_hash(CellKind::Python, "y = L * 2", "env-1", inputs(&[("w", "python:ab"), ("L", "calc:{\"v\":1}")]));
        assert_eq!(a, b, "the order of reads doesn't matter");
        for other in [
            py_input_hash(CellKind::Python, "y = L * 3", "env-1", inputs(&[("L", "calc:{\"v\":1}"), ("w", "python:ab")])),
            py_input_hash(CellKind::Python, "y = L * 2", "env-2", inputs(&[("L", "calc:{\"v\":1}"), ("w", "python:ab")])),
            py_input_hash(CellKind::Python, "y = L * 2", "env-1", inputs(&[("L", "calc:{\"v\":2}"), ("w", "python:ab")])),
            py_input_hash(CellKind::Python, "y = L * 2", "env-1", inputs(&[("L", "calc:{\"v\":1}")])),
            // Length-prefixed fields: moving text between fields changes the hash.
            py_input_hash(CellKind::Python, "y = L * 2env-1", "", inputs(&[("L", "calc:{\"v\":1}"), ("w", "python:ab")])),
        ] {
            assert_ne!(a, other);
        }
        assert_eq!(canonical_json(&serde_json::json!({"b": [1, {"d": 2, "c": "x"}], "a": null})), r#"{"a":null,"b":[1,{"c":"x","d":2}]}"#);
    }

    /// Also computed independently with Python's hashlib from the encoding in `py_input_hash`.
    #[test]
    fn base_hash_ignores_the_environment_and_is_distinct() {
        let inputs = vec![("L".to_string(), "calc:{\"v\":1}".to_string())];
        let b = py_base_hash(CellKind::Python, "y = L * 2", inputs.clone());
        assert_eq!(b, GOLDEN_BASE_HASH);
        assert_ne!(b, py_input_hash(CellKind::Python, "y = L * 2", "", inputs.clone()), "its own tag");
        assert_ne!(b, py_base_hash(CellKind::Python, "y = L * 3", inputs));
    }

    /// Also computed independently with Python's hashlib (no environment field).
    const GOLDEN_BASE_HASH: &str = "4cd0e7b863a1e970b21b0493a84998c095a81c8dede7e62aa9883c8c1eb5f949";

    #[test]
    fn runs_from_another_environment_show_only_where_the_cell_cant_run() {
        let cells = vec![calc("A", "a0", "L = 2"), py_cell("PY", "a1", "y = L")];
        let mut e = Engine::new();
        e.python_available = true;
        e.env = "env-new".into();
        e.computed_by = "user:me@here".into();
        e.set_py_names("y = L", PyNames { defines: vec!["y".into()], reads: vec!["L".into()], ..Default::default() });
        let k = e.evaluate(&cells).missing[0].clone();
        let record = RunRecord {
            cell_id: "PY".into(),
            input_hash: "computed-under-env-old".into(),
            base_hash: Some(k.base.clone()),
            status: RunStatus::Done,
            values: serde_json::json!([]),
            outputs: serde_json::json!([{ "kind": "text", "text": "old" }]),
            stdout: String::new(),
            stderr: String::new(),
            error: None,
            duration_ms: 1,
            env: "env-old".into(),
            computed_at_us: 1,
            computed_by: "user:me@here".into(),
            doc_version: 1,
        };
        assert_eq!(e.use_foreign(&cells, vec![record]), 1);
        // Untrusted: it can't run, so the old environment's output shows, labelled.
        let r = e.evaluate(&cells);
        let py = r.results["PY"].python.as_ref().unwrap();
        assert_eq!(py.state, PyState::Done);
        let from = py.stored.as_ref().unwrap();
        assert!(from.other_env && !from.elsewhere);
        assert!(r.jobs.is_empty());
        // Trusted and approved: it runs instead.
        e.trusted = true;
        e.approve(&cells);
        let r = e.evaluate(&cells);
        assert_eq!(r.results["PY"].python.as_ref().unwrap().state, PyState::Queued);
        assert_eq!(r.jobs[0].base, k.base);
    }

    const GOLDEN_HASH: &str = "02637db6f304c86dcac63b09556214fe48472eb2fcea079155fba0ea8a36a3a6";

    #[test]
    fn python_keys_follow_calc_values_and_the_environment() {
        let cells = vec![calc("A", "a0", "L = 2 m"), py_cell("PY", "a1", "y = L")];
        let mut e = Engine::new();
        e.python_available = true;
        e.trusted = true;
        e.approve(&cells);
        e.set_py_names("y = L", PyNames { defines: vec!["y".into()], reads: vec!["L".into()], ..Default::default() });
        let key = |e: &mut Engine, cells: &[Cell]| e.evaluate(cells).jobs[0].key.clone();
        let k1 = key(&mut e, &cells);
        assert_eq!(k1, key(&mut e, &cells), "same inputs, same key");
        assert_eq!(k1.len(), 64);
        let changed = vec![calc("A", "a0", "L = 3 m"), py_cell("PY", "a1", "y = L")];
        assert_ne!(k1, key(&mut e, &changed));
        e.env = "env-other".into();
        assert_ne!(k1, key(&mut e, &cells));
    }

    #[test]
    fn stored_runs_show_without_trust_and_feed_calcs() {
        let cells = vec![calc("A", "a0", "L = 2"), py_cell("PY", "a1", "y = L"), calc("B", "a2", "z = y + 1 = ?")];
        let mut e = Engine::new();
        e.python_available = true;
        e.computed_by = "user:me@here".into();
        e.set_py_names("y = L", PyNames { defines: vec!["y".into()], reads: vec!["L".into()], ..Default::default() });
        let run = e.evaluate(&cells);
        assert_eq!(run.results["PY"].python.as_ref().unwrap().state, PyState::Untrusted);
        let k = run.missing[0].clone();
        assert_eq!(run.keys["PY"], k);
        let (id, key) = (k.id.clone(), k.key.clone());
        let mut tmp = Engine::new();
        tmp.run(&[calc("T", "a", "y = 4")]);
        let plain = serde_json::to_value(tmp.vars.get("y").unwrap().to_plain()).unwrap();
        let record = RunRecord {
            cell_id: id,
            input_hash: key,
            base_hash: Some(k.base.clone()),
            status: RunStatus::Done,
            values: serde_json::json!([{ "name": "y", "summary": "4", "plain": plain }]),
            outputs: serde_json::json!([{ "kind": "text", "text": "hi" }]),
            stdout: String::new(),
            stderr: String::new(),
            error: None,
            duration_ms: 3,
            env: String::new(),
            computed_at_us: 1,
            computed_by: "user:them@there".into(),
            doc_version: 1,
        };
        assert_eq!(e.use_stored(&cells, vec![record]), 1);
        let run = e.evaluate(&cells);
        let py = run.results["PY"].python.as_ref().unwrap();
        assert_eq!(py.state, PyState::Done);
        assert!(py.stored.as_ref().unwrap().elsewhere);
        assert_eq!(py.outputs, vec![OutputView::Text { text: "hi".into() }]);
        assert!(run.missing.is_empty() && run.jobs.is_empty());
        assert!(run.results["B"].text.ends_with('5'), "{:?}", run.results["B"]);
    }
}
