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
use cruncha_store::{Cell, CellKind};
use serde::Serialize;

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
    /// A past version: Python output isn't kept with the history yet.
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
        }
    }
}

/// Rich output, ready for the web view (images as data URLs).
#[derive(Debug, Clone, PartialEq, Serialize)]
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
                src: format!(
                    "data:{mime};base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(data)
                ),
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
        }
    }
}

/// A Python cell that is due to run.
#[derive(Debug, Clone)]
pub struct PyJob {
    pub id: String,
    pub key: u64,
    pub request: RunRequest,
}

/// The outcome of a Python cell's last run.
struct PyOutcome {
    key: u64,
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
    running: Option<(String, u64)>,
    /// Is there a Python environment? Is the document trusted to run code?
    pub python_available: bool,
    pub trusted: bool,
    /// Evaluating a past version: Python cells don't run.
    pub past: bool,
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
            past: false,
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
            if c.kind == CellKind::Python {
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
        self.running = job.map(|j| (j.id.clone(), j.key));
    }

    /// Store the outcome of a job.
    pub fn record(&mut self, job: &PyJob, result: Result<RunResult, String>) {
        let mut values = Vec::new();
        let mut defined = Vec::new();
        let view = match result {
            Ok(r) => {
                for d in &r.defines {
                    defined.push((d.name.clone(), d.summary.clone()));
                    let plain: Option<Plain> = serde_json::from_value(d.plain.clone()).ok();
                    if let Some(v) = plain
                        .as_ref()
                        .and_then(|p| Value::from_plain_with(p, &self.opts).ok())
                    {
                        values.push((d.name.clone(), v));
                    }
                }
                PyView {
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
                }
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
                key: job.key,
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
    pub fn evaluate(&mut self, cells: &[Cell]) -> Run {
        self.evaluated = 0;
        // Graph nodes: calcs from their prepared statements, Python cells from the worker's
        // scan (when there is one), other cells from their stored names.
        let mut nodes = Vec::with_capacity(cells.len());
        let mut kind_of: HashMap<&str, CellKind> = HashMap::new();
        for c in cells {
            kind_of.insert(c.id.as_str(), c.kind);
            let (defines, reads) = match c.kind {
                CellKind::Calc => self.names(&c.source),
                CellKind::Python => match self.py_names.get(&c.source) {
                    Some(n) => (n.defines.clone(), n.reads.clone()),
                    None => (c.defines.clone(), c.reads.clone()),
                },
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
        let graph = Graph::new(nodes);
        let cell_of: HashMap<&str, &Cell> = cells.iter().map(|c| (c.id.as_str(), c)).collect();

        let mut vars = Vars::new();
        let mut out = HashMap::new();
        let mut jobs = Vec::new();
        let mut live: HashSet<String> = HashSet::new();
        // Python cells that will produce values later (queued or running) / their keys.
        let mut py_pending: HashSet<String> = HashSet::new();
        let mut py_keys: HashMap<String, u64> = HashMap::new();
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
                CellKind::Python => {
                    live.insert(id.clone());
                    if let Some(n) = self.py_names.get(&cell.source) {
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
                    // Rust (and other) cells take part in the graph but don't run yet.
                    if !node.defines.is_empty() {
                        let label = if cell.kind == CellKind::Rust {
                            "Rust cells"
                        } else {
                            "these cells"
                        };
                        result.error = Some(format!("{label} can't run yet"));
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
            result.error = outcome.error.map(|e| clean_message(&e));
            out.insert(id.clone(), result);
        }
        self.memo.retain(|id, _| live.contains(id));
        self.py_memo.retain(|id, _| live.contains(id));
        self.vars = vars;
        Run { results: out, jobs }
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
                Some(CellKind::Python) => {
                    if vars.contains(name) {
                        continue;
                    }
                    if py_pending.contains(definer) {
                        return Some(Blocked::Pending);
                    }
                    if let Some(what) = opaque.get(name) {
                        return Some(Blocked::Because(format!(
                            "{name} is a Python {what}; calcs can use numbers, quantities, vectors, matrices and frames"
                        )));
                    }
                    let why = if self.past {
                        "Python results aren't kept for past versions yet"
                    } else if !self.python_available {
                        "Python isn't set up yet"
                    } else if !self.trusted {
                        "this document isn't trusted to run code"
                    } else {
                        "it hasn't run successfully"
                    };
                    return Some(Blocked::Because(format!(
                        "{name} comes from a Python cell; {why}"
                    )));
                }
                Some(CellKind::Calc) | None => {}
                Some(CellKind::Rust) => {
                    return Some(Blocked::Because(format!(
                        "{name} comes from a Rust cell, which can't run yet"
                    )));
                }
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
        py_keys: &mut HashMap<String, u64>,
        opaque: &mut HashMap<String, String>,
    ) {
        let id = cell.id.as_str();
        let set = |r: &mut CellResult, state, note: Option<String>| {
            r.python = Some(PyView::state(state, note))
        };
        if cell.source.trim().is_empty() {
            return;
        }
        if let Some(e) = self
            .py_names
            .get(&cell.source)
            .and_then(|n| n.syntax_error.clone())
        {
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
        let mut h = DefaultHasher::new();
        cell.source.hash(&mut h);
        let mut request = RunRequest {
            cell: cell.id.clone(),
            source: cell.source.clone(),
            defines: result.defines.clone(),
            ..Default::default()
        };
        let mut waiting = false;
        let mut blocked: Option<String> = None;
        for name in &result.reads {
            name.hash(&mut h);
            let definer = graph.definer(name);
            let definer_kind = definer.and_then(|d| cell_of.get(d)).map(|c| c.kind);
            match definer_kind {
                Some(CellKind::Python) => {
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
                    py_keys.get(d).hash(&mut h);
                    request.python_inputs.push(name.clone());
                }
                Some(CellKind::Calc) => match vars.get(name) {
                    Some(v) => {
                        format!("{v:?}").hash(&mut h);
                        match v.to_plain() {
                            Plain::Unsupported { kind, .. } => {
                                blocked.get_or_insert(format!(
                                    "{name} ({kind}) can't be passed to Python yet"
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
                        blocked
                            .get_or_insert(format!("{name} has no value (its calc has an error)"));
                    }
                },
                Some(_) => {
                    blocked.get_or_insert(format!("{name} comes from a cell that can't run yet"));
                }
                // Not defined anywhere: Python raises NameError, which says it best.
                None => {}
            }
        }
        let key = h.finish();
        py_keys.insert(id.to_owned(), key);

        let memo_hit = self.py_memo.get(id).filter(|o| o.key == key);
        if let Some(o) = memo_hit {
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
            return;
        }
        if self.running.as_ref() == Some(&(id.to_owned(), key)) {
            py_pending.insert(id.to_owned());
            return set(result, PyState::Running, None);
        }
        if self.past {
            return set(result, PyState::Past, None);
        }
        if !self.python_available {
            return set(result, PyState::NoPython, None);
        }
        if !self.trusted {
            return set(result, PyState::Untrusted, None);
        }
        if let Some(why) = blocked {
            return set(result, PyState::Blocked, Some(why));
        }
        if !self.may_run(cell) {
            return set(result, PyState::Edited, None);
        }
        py_pending.insert(id.to_owned());
        if !waiting {
            jobs.push(PyJob {
                id: id.to_owned(),
                key,
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
            values: value_views(&o.values),
            defines: stmt.defines,
            reads: stmt.reads,
            ..CellResult::default()
        }
    }
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
}
