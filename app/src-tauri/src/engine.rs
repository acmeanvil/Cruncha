// SPDX-License-Identifier: GPL-3.0-or-later
//! Evaluating a document: calc cells run in dependency order (`cruncha-graph`), each with
//! `cruncha-eval`, and results are memoized by their source and input values (design 02
//! §3.1, rule 6), so an edit re-runs only what it affects.
//!
//! Calculations are always unit-aware (ADR-0007). Python and Rust cells take part in the graph
//! (their names are known) but don't run yet, so a calc reading one of their names reports that.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use cruncha_eval::{Options, Outcome, Statement, UnitAware, Vars};
use cruncha_graph::{Graph, Issue, Node};
use cruncha_store::{Cell, CellKind};
use serde::Serialize;

/// What a calc cell shows.
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
}

/// One variable's value, for the variable browser.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ValueView {
    pub name: String,
    /// The value as TeX, the same form a later statement's echo shows (e.g. `24\,\text{ft}`).
    pub tex: String,
    /// `number`, `matrix`, `dataframe`, … (see [`cruncha_eval::Kind`]), lower case.
    pub kind: String,
}

fn value_views(values: &[(String, cruncha_eval::Value)]) -> Vec<ValueView> {
    values
        .iter()
        .map(|(name, v)| ValueView {
            name: name.clone(),
            tex: v.tex().to_owned(),
            kind: format!("{:?}", v.kind()).to_lowercase(),
        })
        .collect()
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
    /// Last outcome per cell, with the hash of what produced it.
    memo: HashMap<String, (u64, Outcome)>,
    /// The variables after the last run (for live previews).
    vars: Vars,
    /// Statements evaluated by the last [`Engine::run`] (the rest came from the memo).
    pub evaluated: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            opts: options(),
            prepared: HashMap::new(),
            memo: HashMap::new(),
            vars: Vars::new(),
            evaluated: 0,
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

    /// Evaluate every calc cell of `cells` (document order) and return their results by id.
    pub fn run(&mut self, cells: &[Cell]) -> HashMap<String, CellResult> {
        self.evaluated = 0;
        // Graph nodes: calcs from their prepared statements, other cells from their stored names.
        let mut nodes = Vec::with_capacity(cells.len());
        let mut kind_of: HashMap<&str, CellKind> = HashMap::new();
        for c in cells {
            kind_of.insert(c.id.as_str(), c.kind);
            let (defines, reads) = if c.kind == CellKind::Calc {
                self.names(&c.source)
            } else {
                (c.defines.clone(), c.reads.clone())
            };
            nodes.push(Node {
                id: c.id.clone(),
                defines,
                reads,
            });
        }
        if nodes.iter().any(|n| n.defines.iter().any(|d| d == FORMAT_VAR)) {
            for (n, c) in nodes.iter_mut().zip(cells) {
                if c.kind == CellKind::Calc && !n.defines.iter().any(|d| d == FORMAT_VAR) {
                    n.reads.push(FORMAT_VAR.into());
                }
            }
        }
        let graph = Graph::new(nodes);
        let source_of: HashMap<&str, &str> =
            cells.iter().map(|c| (c.id.as_str(), c.source.as_str())).collect();

        let mut vars = Vars::new();
        let mut out = HashMap::new();
        let mut live: HashSet<String> = HashSet::new();
        let order: Vec<String> = graph.order().map(str::to_owned).collect();
        for id in &order {
            let node = graph
                .nodes()
                .iter()
                .find(|n| &n.id == id)
                .expect("graph node for every cell");
            let kind = kind_of.get(id.as_str()).copied();
            if kind != Some(CellKind::Calc) {
                // Code cells take part in the graph, so the variable browser lists their names.
                if !node.defines.is_empty() {
                    let label = match kind {
                        Some(CellKind::Python) => "Python cells",
                        Some(CellKind::Rust) => "Rust cells",
                        _ => "these cells",
                    };
                    out.insert(
                        id.clone(),
                        CellResult {
                            defines: node.defines.clone(),
                            reads: node.reads.clone(),
                            error: Some(format!("{label} can't run yet")),
                            ..CellResult::default()
                        },
                    );
                }
                continue;
            }
            live.insert(id.clone());
            let issues = graph.issues(id);
            let warnings: Vec<String> = issues
                .iter()
                .filter(|i| !i.is_error())
                .map(Issue::to_string)
                .collect();
            let mut result = CellResult {
                defines: node.defines.clone(),
                reads: node.reads.iter().filter(|r| *r != FORMAT_VAR).cloned().collect(),
                warnings,
                ..CellResult::default()
            };
            let source = source_of[id.as_str()].trim();
            if source.is_empty() {
                out.insert(id.clone(), result);
                continue;
            }
            if let Some(e) = issues.iter().find(|i| i.is_error()) {
                result.error = Some(e.to_string());
                out.insert(id.clone(), result);
                continue;
            }
            // A name defined by a cell that doesn't run yet.
            if let Some(name) = node.reads.iter().find(|r| {
                graph
                    .definer(r)
                    .and_then(|d| kind_of.get(d))
                    .is_some_and(|k| *k != CellKind::Calc)
            }) {
                let kind = graph.definer(name).and_then(|d| kind_of.get(d)).copied();
                let label = match kind {
                    Some(CellKind::Python) => "a Python cell",
                    Some(CellKind::Rust) => "a Rust cell",
                    _ => "a cell",
                };
                result.error = Some(format!("{name} comes from {label}, which can't run yet"));
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
        self.vars = vars;
        out
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
            warnings: Vec::new(),
        }
    }
}

/// Hurmet marks names in messages with `$…$`; the UI shows plain text.
fn clean_message(m: &str) -> String {
    m.replace('$', "")
}

/// What determines a calc's outcome: its source and the values of what it reads.
fn input_hash(source: &str, stmt: &Statement, vars: &Vars) -> u64 {
    let mut h = DefaultHasher::new();
    source.hash(&mut h);
    for name in stmt.reads.iter().chain(std::iter::once(&FORMAT_VAR.to_string())) {
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
        assert!(r["M"].warnings.iter().any(|w| w.contains("defined further down")), "{:?}", r["M"]);
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
        assert!(r["M"].error.as_deref().is_some_and(|m| m.contains('L')), "{:?}", r["M"]);
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
        assert!(r["A"].error.as_deref().unwrap().contains("defined in 1 other cell"));
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
        assert_eq!(r["C"].error.as_deref(), Some("x comes from a Python cell, which can't run yet"));
        // The Python cell's names are reported too (for the variable browser).
        assert_eq!(r["PY"].defines, ["x"]);
        assert!(r["PY"].values.is_empty());
        // A preview uses the document's values, without storing anything.
        let p = e.preview("A = L^2 = ?? m^2");
        assert!(p.text.ends_with("9 m^2") || p.text.ends_with("9 m²"), "{}", p.text);
        // An unknown name is reported, with Hurmet's `$…$` markers removed.
        let p = e.preview("z = nope + 1 = ?");
        assert!(p.error.as_deref().is_some_and(|m| m.contains("nope") && !m.contains('$')), "{p:?}");
        assert_eq!(e.names("M = w L^2 / 8 = ?? kip·ft"), (vec!["M".to_string()], vec!["w".to_string(), "L".to_string()]));
    }

    #[test]
    fn format_statements_apply_everywhere() {
        let mut e = Engine::new();
        let r = e.run(&[calc("F", "a", "format = \"f2\""), calc("X", "b", "x = 1/3 = ?")]);
        assert!(r["X"].text.ends_with("0.33"), "{}", r["X"].text);
    }
}
