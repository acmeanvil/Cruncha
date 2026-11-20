// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! The Cruncha calc evaluator: a port of Hurmet's unit-aware calculation engine (spike S2b,
//! `docs/spikes/S2b-calc-evaluator.md`).
//!
//! A calc statement such as `M = w L^2 / 8 = ?? kip·ft` is first **prepared** ([`prepare`],
//! Hurmet's `compile`): it is split into the name it defines, the expression (compiled to
//! Hurmet's RPN by [`cruncha_lang::parse_calc`]), the display selector and the display unit.
//! A prepared [`Statement`] depends only on its source and the [`Options`], so it can be cached.
//! It is then **evaluated** ([`evaluate`]) against the values of the variables it reads, held in
//! [`Vars`]. The [`Outcome`] carries the values to store for later statements, the TeX of the
//! whole display line (with the blue echo) and Hurmet's plain-text result.
//!
//! ```
//! use cruncha_eval::{calculate, Options, Vars};
//! let opts = Options::default();
//! let mut vars = Vars::new();
//! calculate("w = 1.5 kip/ft", &mut vars, &opts);
//! calculate("L = 24 ft", &mut vars, &opts);
//! assert_eq!(calculate("M = w L^2 / 8 = ?? kip·ft", &mut vars, &opts), "M = w L^2 / 8 = 108 kip·ft");
//! assert_eq!(calculate("M = @@ kip·ft", &mut vars, &opts), "108 kip·ft");
//! ```
//!
//! ## Selectors and unit awareness
//!
//! Hurmet's selectors are `?` (show the expression, its echo with values, and the result), `@`
//! (result only), `%` (no echo) and `!` (compute but do not show). Doubled (`??`, `@@`, `%%`,
//! `!!`), they make the calculation unit-aware. [`UnitAware::Hurmet`] keeps that rule, and the
//! golden tests run in that mode. [`UnitAware::Always`] (ADR-0007) evaluates every statement
//! unit-aware; selectors then only control display, and a result with dimensions but no
//! display unit is shown in SI base units.
//!
//! ## The API
//!
//! ```text
//! pub fn prepare(source: &str, opts: &Options) -> Result<Statement, Error>;
//! pub fn evaluate(stmt: &Statement, vars: &Vars, opts: &Options) -> Outcome;   // never panics
//! pub fn calculate(input: &str, vars: &mut Vars, opts: &Options) -> String;    // Hurmet's calculate()
//! ```
//!
//! - [`Statement`]: `defines` (names assigned; several for `a, b = …`), `reads` (the
//!   expression's variables, from the parse), `display` ([`Display`], the selector), `unit` (the
//!   display unit), and the prepared templates and RPN (private).
//! - [`Outcome`]: `values` (what to insert into [`Vars`]), `tex` (the display line for Temml),
//!   `result` (plain text, exactly what Hurmet's `calculate` returns) and `error`.
//! - [`Value`]: what a variable holds. [`Value::kind`], [`Value::as_f64`] (as written),
//!   [`Value::as_base_f64`] (in SI base units), [`Value::unit_name`], [`Value::dims`] and
//!   [`Value::tex`] (the TeX that echoes of later statements show).
//!
//! A literal assignment (`L = 3 ft`) and a function definition are complete after `prepare`;
//! `evaluate` returns their value without computing anything.
//!
//! ## Exchanging values (the Python worker)
//!
//! [`Value::to_plain`] gives a serde-serializable [`Plain`] (number with an exact rational and
//! a [`UnitView`], array, data frame, text, …), and [`Value::from_plain`] builds a value from
//! one that later statements read like an assigned value, echo included. A unit name that
//! does not parse (or has other dimensions) falls back to SI. See the `exchange` module source
//! for the full mapping and what is lossy.
//!
//! ## Errors
//!
//! `prepare` fails only when the expression is malformed ([`Error::Parse`]). Everything else is
//! reported by `evaluate`: `Outcome::error` holds Hurmet's message (for example
//! `Error. Divide by zero.` or `Error. Variable $q$ not found.`, with `$…$` around names as in
//! Hurmet), `Outcome::result` has the message in place of the selector, and `Outcome::tex` has
//! `\textcolor{firebrick}{\text{…}}` there. A statement that defines a name still returns it in
//! `values`, with a missing value, so later statements report `Error. Missing value for $x$.`
//! rather than "not found" (Hurmet's behavior).
//!
//! ## Thread safety
//!
//! [`Statement`], [`Value`], [`Vars`], [`Outcome`] and [`Options`] are `Send + Sync`.
//! Evaluation does not mutate shared state, so statements that do not depend on each other can
//! be evaluated in parallel against the same `Vars`.

// Hurmet reports an error as an operand (`dtype: ERROR`), and the port keeps that: `Err` is an
// `Operand`. Errors are not on the hot path, so they are not boxed.
#![allow(clippy::result_large_err)]

mod attrs;
mod compile;
mod complex;
mod dataframe;
mod echo;
mod error;
mod eval;
mod exchange;
mod format;
mod functions;
mod literal;
mod matrix;
mod module;
mod operations;
mod result;
mod value;

use attrs::{Attrs, Names};
use cruncha_units::Dims;
use eval::{Ctx, Scope, eval_rpn};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;
use value::{OUnit, UnitObj, V, dt, is_matrix};

pub use cruncha_lang::Formats;
pub use cruncha_units::{Currencies, Rational};
pub use exchange::{ColumnData, Num, Plain, PlainColumn, UnitView};
pub use value::{Kind, Value};

/// Whether calculations convert to SI base units and check dimensions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnitAware {
    /// Hurmet's rule: `?`, `@`, `%`, `!` compute with plain numbers as written; `??`, `@@`,
    /// `%%`, `!!` are unit-aware. The golden tests use this mode.
    #[default]
    Hurmet,
    /// ADR-0007: every calculation is unit-aware; the selector only controls the display.
    Always,
}

/// Evaluation options.
#[derive(Clone, Debug)]
pub struct Options {
    /// Decimal and date formats for results and TeX. The default is Hurmet's
    /// `calculate()` default: `1,000,000.` and `yyyy-mm-dd`.
    pub formats: Formats,
    pub unit_aware: UnitAware,
    /// Exchange rates for monetary units (code → units per euro). Empty by default, which makes
    /// any monetary unit an error, as in Hurmet without rates.
    pub currencies: Arc<Currencies>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            formats: Formats {
                decimal_format: "1,000,000.".into(),
                date_format: Some("yyyy-mm-dd".into()),
            },
            unit_aware: UnitAware::Hurmet,
            currencies: Arc::new(Currencies::new()),
        }
    }
}

/// How a statement shows its result (Hurmet's display selector).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    /// No selector: an assignment of a literal value, a function definition, or plain math.
    None,
    /// `?`: expression, echo with values, and result.
    Echo,
    /// `??`
    EchoUnitAware,
    /// `@`: the result only.
    ResultOnly,
    /// `@@`
    ResultOnlyUnitAware,
    /// `%`: expression and result, no echo.
    NoEcho,
    /// `%%`
    NoEchoUnitAware,
    /// `!`: compute and store, but show only the expression.
    Suppress,
    /// `!!`
    SuppressUnitAware,
    /// `@test expr`: an assertion.
    Test,
    /// `@@test expr`
    TestUnitAware,
}

impl Display {
    /// Is this a unit-aware (doubled) selector?
    pub fn is_unit_aware(self) -> bool {
        matches!(
            self,
            Display::EchoUnitAware
                | Display::ResultOnlyUnitAware
                | Display::NoEchoUnitAware
                | Display::SuppressUnitAware
                | Display::TestUnitAware
        )
    }
    fn from_selector(sel: &str, is_test: bool) -> Display {
        let double =
            sel.contains("??") || sel.contains("@@") || sel.contains("%%") || sel.contains("!!");
        if is_test {
            return if double {
                Display::TestUnitAware
            } else {
                Display::Test
            };
        }
        if sel.contains('!') {
            if double {
                Display::SuppressUnitAware
            } else {
                Display::Suppress
            }
        } else if sel.contains('@') {
            if double {
                Display::ResultOnlyUnitAware
            } else {
                Display::ResultOnly
            }
        } else if sel.contains('%') {
            if double {
                Display::NoEchoUnitAware
            } else {
                Display::NoEcho
            }
        } else if sel.contains('?') {
            if double {
                Display::EchoUnitAware
            } else {
                Display::Echo
            }
        } else {
            Display::None
        }
    }
}

/// A prepared statement (Hurmet's `compile` result). Cheap to clone and `Send + Sync`.
#[derive(Clone, Debug)]
pub struct Statement {
    /// Names this statement assigns: empty for a bare expression; several for a multiple
    /// assignment such as `a, b = …`.
    pub defines: Vec<String>,
    /// Variables the expression reads (from the parse).
    pub reads: Vec<String>,
    pub display: Display,
    /// The display unit after the selector (`kip·ft` in `= ?? kip·ft`), if any.
    pub unit: Option<String>,
    attrs: Arc<Attrs>,
}

/// Why a statement could not be prepared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The expression is malformed (Hurmet's parser would throw).
    Parse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for Error {}

/// Variable values, by name.
#[derive(Clone, Debug, Default)]
pub struct Vars {
    map: HashMap<String, Value>,
}

impl Vars {
    pub fn new() -> Self {
        Self::default()
    }
    /// Insert a value. A value without a name (for example one from [`Value::from_plain`])
    /// takes `name`, which Hurmet's error messages and data frame headings use.
    pub fn insert(&mut self, name: impl Into<String>, mut value: Value) -> Option<Value> {
        let name = name.into();
        if value.name.is_none() {
            value.name = Some(name.clone());
        }
        self.map.insert(name, value)
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.map.get(name)
    }
    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.map.remove(name)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.map.iter()
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    /// Set the number format for results (Hurmet's `format = "h15"` statement), e.g. `"f2"`.
    pub fn set_format(&mut self, spec: &str) {
        let v = Value {
            name: Some("format".into()),
            value: V::S(spec.into()),
            dtype: dt::STRING,
            ..Default::default()
        };
        self.map.insert("format".into(), v);
    }
}

/// The result of evaluating a statement.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// What the statement defines, to insert into [`Vars`] (empty for a bare expression).
    pub values: Vec<(String, Value)>,
    /// The whole display line as TeX for Temml, with the blue echo where the selector asks
    /// for it. For an error, the result position holds `\textcolor{firebrick}{\text{…}}`.
    pub tex: String,
    /// The plain-text line (or result), exactly as Hurmet's `calculate()` returns it in draft
    /// mode. For an error, the message replaces the selector.
    pub result: String,
    /// The error message, if evaluation failed.
    pub error: Option<String>,
}

pub(crate) fn ctx_for<'a>(opts: &'a Options, format_spec: &str) -> Ctx<'a> {
    Ctx {
        decimal_format: &opts.formats.decimal_format,
        date_format: opts.formats.date_format.as_deref(),
        currencies: &opts.currencies,
        depth: Cell::new(0),
        format_spec: format_spec.to_string(),
    }
}

fn format_spec_of(vars: &Vars) -> String {
    vars.get("format")
        .map_or_else(|| "h15".to_string(), |v| v.value.str().to_string())
}

/// Prepare a statement (Hurmet's `compile`).
pub fn prepare(source: &str, opts: &Options) -> Result<Statement, Error> {
    let ctx = ctx_for(opts, "h15");
    let c = compile::compile(source, &ctx).map_err(Error::Parse)?;
    let display = if c.is_calc {
        Display::from_selector(&c.selector, c.is_test)
    } else {
        Display::None
    };
    Ok(Statement {
        defines: c.attrs.name.list(),
        reads: c.attrs.dependencies.clone(),
        display,
        unit: c.unit_str,
        attrs: Arc::new(c.attrs),
    })
}

const BASE_UNITS: [&str; 8] = ["m", "kg", "s", "A", "K", "unit", "cd", "EUR"];

/// The SI base unit for a dimension vector, e.g. `kg·m²·s⁻²` (ADR-0007's display for a
/// result without a display unit). `None` for fractional exponents.
pub(crate) fn si_unit_name(d: &Dims) -> Option<String> {
    let order = [1usize, 0, 2, 3, 4, 5, 6, 7];
    let mut parts = Vec::new();
    for &k in &order {
        let e = d[k];
        if e == 0.0 {
            continue;
        }
        if e.fract() != 0.0 || e.abs() > 99.0 {
            return None;
        }
        let e = e as i64;
        let sup: String = if e == 1 {
            String::new()
        } else {
            e.to_string()
                .chars()
                .map(|c| match c {
                    '-' => '⁻',
                    '1' => '¹',
                    '2' => '²',
                    '3' => '³',
                    c => char::from_u32(c as u32 + 0x2040).unwrap_or(c),
                })
                .collect()
        };
        parts.push(format!("{}{sup}", BASE_UNITS[k]));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("·"))
    }
}

/// Insert a display unit into the templates of a statement that had none (ADR-0007 mode).
fn add_display_unit(a: &mut Attrs, name: &str, ctx: &Ctx) {
    let unit_tex = literal::tex(&format!("'{name}'"), ctx);
    if let Some(rt) = a.resulttemplate.as_mut() {
        rt.push(' ');
        rt.push_str(&unit_tex);
    }
    a.altresulttemplate.push(' ');
    a.altresulttemplate.push_str(name);
    if let Some(stripped) = a.tex.strip_suffix("\\end{aligned}") {
        a.tex = format!("{stripped} {unit_tex}\\end{{aligned}}");
    } else if !a.tex.is_empty() {
        a.tex.push(' ');
        a.tex.push_str(&unit_tex);
    }
    if !a.alt.is_empty() {
        a.alt.push(' ');
        a.alt.push_str(name);
    }
}

fn run(stmt: &Statement, vars: &Vars, opts: &Options) -> Attrs {
    let mut a = (*stmt.attrs).clone();
    if a.is_static {
        return a;
    }
    let spec = format_spec_of(vars);
    let ctx = ctx_for(opts, &spec);
    a.tex = a.template.clone();
    a.alt = a.alt_template.clone();
    a.md = None;
    let rt = a.resulttemplate.clone().unwrap_or_default();
    let selector_aware = rt.contains("??")
        || rt.contains("!!")
        || rt.contains("%%")
        || rt.contains("@@")
        || rt.contains("¡¡");
    let unit_aware = selector_aware || opts.unit_aware == UnitAware::Always;
    let mut scope = Scope::global(vars);
    if a.tex.contains('〖') {
        match echo::plug_vals_into_echo(&a.tex, &scope, unit_aware, &spec, &ctx) {
            Ok(t) => a.tex = t,
            Err(e) => {
                result::error_result(&mut a, &e);
                return a;
            }
        }
    }
    let Some(rpn) = a.rpn.clone() else { return a };
    let oprnd = eval_rpn(&rpn, &mut scope, &ctx, unit_aware, None);
    if oprnd.is_error() {
        result::error_result(&mut a, &oprnd);
        return a;
    }
    if matches!(&oprnd.value, V::R(r) if !r.is_finite()) {
        // JavaScript would throw here (NaN or ±∞ cannot become a rational).
        result::error_result(&mut a, &error::err0("NOT_FINITE"));
        return a;
    }
    if opts.unit_aware == UnitAware::Always
        && a.unit.get_expos().is_none()
        && oprnd.dtype & dt::RATIONAL != 0
        && let Some(d) = oprnd
            .unit
            .get_expos()
            .filter(|d| d.iter().any(|x| *x != 0.0))
        && let Some(name) = si_unit_name(d)
        && let Ok(u) = ctx.unit(&name)
    {
        a.unit = OUnit::from_unit(&u);
        add_display_unit(&mut a, &name, &ctx);
    }
    let Some(result) = result::condition_result(&mut a, oprnd, unit_aware) else {
        return a;
    };
    if a.error {
        return a;
    }
    let assert = vars.get("assert").map(|v| v.value.str().to_string());
    result::format_result(&mut a, &result, &spec, &ctx, assert.as_deref(), unit_aware);
    a
}

fn value_of(a: &Attrs, name: &str) -> Value {
    Value {
        name: Some(name.to_string()),
        value: a.value.clone(),
        unit: a.unit.clone(),
        dtype: a.dtype,
        resultdisplay: a.resultdisplay.clone(),
        expos: a.expos,
        error: a.error,
    }
}

/// Hurmet's `insertOneHurmetVar`: the values a statement assigns.
fn assigned_values(a: &Attrs, opts: &Options) -> Vec<(String, Value)> {
    match &a.name {
        Names::None => Vec::new(),
        Names::One(n) => vec![(n.clone(), value_of(a, n))],
        Names::Many(names) => {
            if matches!(a.value, V::Null) || a.error {
                return names
                    .iter()
                    .map(|n| {
                        (
                            n.clone(),
                            Value {
                                name: Some(n.clone()),
                                value: V::Null,
                                error: a.error,
                                ..Default::default()
                            },
                        )
                    })
                    .collect();
            }
            let mut out = Vec::new();
            let is_q = a.dtype & dt::QUANTITY != 0;
            let (plain, base) = match &a.value {
                V::Q(q) => (q.0.clone(), Some(q.1.clone())),
                other => (other.clone(), None),
            };
            if is_matrix(a.dtype) {
                let flat = |v: &V| -> Vec<V> {
                    if a.dtype & dt::MATRIX != 0 {
                        v.arr().iter().flat_map(|r| r.arr().to_vec()).collect()
                    } else {
                        v.arr().to_vec()
                    }
                };
                let plains = flat(&plain);
                let bases = base.as_ref().map(flat);
                let dtype = a.dtype - (a.dtype & (dt::MATRIX | dt::ROWVECTOR | dt::COLUMNVECTOR));
                let spec = "h15";
                let df = opts.formats.decimal_format.clone();
                for (k, n) in names.iter().enumerate() {
                    let p = plains.get(k).cloned().unwrap_or_default();
                    let value = match &bases {
                        Some(b) if is_q => {
                            V::Q(Box::new((p.clone(), b.get(k).cloned().unwrap_or_default())))
                        }
                        _ => p.clone(),
                    };
                    let mut rd = matrix::element_display(&p, dtype, spec, &df, format::Mode::Tex);
                    if is_q && let Some(un) = a.unit.name() {
                        rd = format!("{rd}\\;{}", cruncha_lang::hurmet::unit_tex_from_string(un));
                    }
                    out.push((
                        n.clone(),
                        Value {
                            name: Some(n.clone()),
                            value,
                            unit: if is_q { a.unit.clone() } else { OUnit::Null },
                            dtype,
                            resultdisplay: rd,
                            expos: a.expos,
                            error: false,
                        },
                    ));
                }
            } else if let V::Tuple(t) = &a.value {
                for (k, (_, o)) in t.iter().enumerate() {
                    if let Some(n) = names.get(k) {
                        let mut v = Value::from_operand(o);
                        v.name = Some(n.clone());
                        v.resultdisplay = result::operand_tex(o, "h15", &ctx_for(opts, "h15"));
                        out.push((n.clone(), v));
                    }
                }
            } else if let V::Module(m) = &a.value {
                for n in names {
                    if let Some(v) = m.get(n) {
                        out.push((n.clone(), v.clone()));
                    }
                }
            } else {
                // Hurmet has a TODO here; each name gets the whole value.
                for n in names {
                    out.push((n.clone(), value_of(a, n)));
                }
            }
            out
        }
    }
}

/// Evaluate a prepared statement against the current variables. Never panics: errors are
/// reported in the [`Outcome`] (and, as in Hurmet, an errored assignment still defines its
/// names, with a missing value).
pub fn evaluate(stmt: &Statement, vars: &Vars, opts: &Options) -> Outcome {
    let a = run(stmt, vars, opts);
    let error = if a.error {
        Some(a.alt_result_display.clone())
            .filter(|s| !s.is_empty())
            .or_else(|| Some(a.alt.clone()))
    } else {
        None
    };
    Outcome {
        values: assigned_values(&a, opts),
        tex: a.tex.clone(),
        result: a.alt.clone(),
        error,
    }
}

/// Hurmet's `calculate(entry, vars, true)`: prepare, evaluate, store what the statement
/// defines into `vars`, and return the plain-text result. For tests and the REPL.
pub fn calculate(input: &str, vars: &mut Vars, opts: &Options) -> String {
    let stmt = match prepare(input, opts) {
        Ok(s) => s,
        Err(e) => return e.to_string(),
    };
    let out = evaluate(&stmt, vars, opts);
    for (name, value) in out.values {
        vars.insert(name, value);
    }
    out.result
}

#[allow(dead_code)]
fn unit_obj_zero() -> UnitObj {
    UnitObj::default()
}
