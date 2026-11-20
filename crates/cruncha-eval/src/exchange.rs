// SPDX-License-Identifier: GPL-3.0-or-later

//! The value exchange bridge: a calc [`Value`] to and from [`Plain`], a serde-serializable
//! form that carries values to the Python worker (ADR-0005, design 02 §3.3) and brings its
//! results back, so later calc statements can read them like any assigned value.
//!
//! ## Kinds
//!
//! | Hurmet dtype                                   | `Plain`                                  |
//! |------------------------------------------------|------------------------------------------|
//! | missing value (`null`)                         | `Missing`                                |
//! | `BOOLEAN`                                      | `Bool`                                   |
//! | `STRING`, `RICHTEXT`                           | `Text`                                   |
//! | `RATIONAL` (± `QUANTITY`), scalar              | `Number` (always `Num::Exact`, see below) |
//! | `COMPLEX`                                      | `Complex` (f64 parts)                    |
//! | `RATIONAL` + `ROWVECTOR`/`COLUMNVECTOR`/`MATRIX` (± `QUANTITY`) | `Array` (f64, row-major) |
//! | `DATAFRAME`                                    | `Frame`                                  |
//! | `ERROR`, `DATE`, `MAP`, `TUPLE`, `MODULE` (functions), `RANGE`, `DRAWING`, `MACRO`, arrays of non-reals | `Unsupported` |
//!
//! ## Numbers
//!
//! Every Hurmet number is a rational, so `to_plain` writes a finite scalar as
//! `Num::Exact { num, den }` (decimal big-integer strings, in Hurmet's stored, possibly
//! unreduced form). `Num::Float` appears only for the NaN/∞ stand-ins (zero denominator).
//! `from_plain` keeps `Num::Exact` exactly (a negative denominator moves its sign to the
//! numerator; a zero denominator is an error), and converts `Num::Float` with Hurmet's
//! `Rnl.fromNumber` ([`Rational::from_number`]: integers exactly, otherwise the shortest
//! round-trip decimal). Array, complex and frame entries are f64.
//!
//! ## Units
//!
//! [`UnitView`] follows Hurmet's convention (`cruncha_units::Unit`): a value `v` written in the
//! unit is `(v + gauge) × factor` in SI base units. `°C` has gauge 273.15 and factor 1; `°F`
//! has gauge 459.67 and factor 5/9. `dims` are the exponents of length (m), mass (kg), time
//! (s), current (A), temperature (K), amount (`unit`), luminous intensity (cd) and money (EUR),
//! in that order ([`cruncha_units::DIM_NAMES`]). `factor` and `gauge` are f64 views of Hurmet's
//! exact rationals.
//!
//! `to_plain` gives a quantity's display unit (name, factor, gauge) with the value's own
//! dimensions; a quantity without a display unit (unit-aware result with no unit requested)
//! has `name: None`, factor 1 and gauge 0, and its value is in SI base units. A number that
//! is not a quantity, or a dimensionless one with no name, factor 1 and gauge 0, has
//! `unit: None`.
//!
//! `from_plain`: if `name` parses with [`cruncha_units::unit_from_name`] and its dimensions
//! equal `dims`, Hurmet's unit is used: the value is as written in it, and the base value comes
//! from Hurmet's exact factor and gauge (the view's `factor`/`gauge` are then ignored).
//! Otherwise the value is converted to SI with the view's `factor` and `gauge` and shown in the
//! coherent SI unit built from `dims` (`kg·m·s⁻²`, as ADR-0007 shows a result without a display
//! unit). A dimensionless value becomes a plain number; dimensions with no SI name (fractional
//! exponents, or money without exchange rates) give a quantity with no unit name, which still
//! takes part in unit-aware arithmetic.
//!
//! ## Frames
//!
//! `Frame` has `row_names` (Hurmet's `#` on the first heading: the first column names the
//! rows) in addition to `columns`. A number column's values are as written in its unit; with
//! an SI fallback unit they are converted to SI. Text columns hold Hurmet's cell strings
//! (booleans as `"true"`/`"false"`).
//!
//! ## Lossy points
//!
//! - Array, complex and frame values are f64 (an exact rational in a vector becomes a float).
//! - `UnitView::factor`/`gauge` are f64; with a parsable name Hurmet's exact factor is used.
//! - A quantity with no unit name comes back named in SI units (`m`, `kg·m·s⁻²`).
//! - Hurmet's evaluator trims text and strips surrounding quotes when it reads a string;
//!   a frame text cell that looks like a number or a boolean is read as one.

use crate::eval::Ctx;
use crate::value::{Frame, OUnit, Operand, UnitObj, V, Value, cpx, dt, is_matrix};
use crate::{Options, ctx_for, dataframe, result, si_unit_name};
use cruncha_lang::hurmet::unit_tex_from_string;
use cruncha_units::{ALL_ZEROS, Dims, Rational, jsnum};
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// A calc value in a plain, serializable form.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Plain {
    Missing,
    Bool {
        value: bool,
    },
    Text {
        value: String,
    },
    /// A real number as written in `unit` (exact when the calc value was an exact rational).
    Number {
        value: Num,
        unit: Option<UnitView>,
    },
    Complex {
        re: f64,
        im: f64,
    },
    /// A vector or matrix of reals, row-major, as written in `unit`. `shape` is `[n]` for a
    /// vector or `[rows, cols]` for a matrix; `orientation` is `"row"` or `"column"` for a
    /// vector and `"matrix"` for a matrix.
    Array {
        shape: Vec<usize>,
        orientation: String,
        values: Vec<f64>,
        unit: Option<UnitView>,
    },
    /// A data frame: columns with a name, an optional unit, and number or text data (`None`
    /// is an empty cell). `row_names`: the first column names the rows (Hurmet's `#name`).
    Frame {
        columns: Vec<PlainColumn>,
        #[serde(default)]
        row_names: bool,
    },
    /// Anything else (functions, modules, maps, tuples, dates, errors, …): its kind and a
    /// short summary.
    Unsupported {
        kind: String,
        summary: String,
    },
}

/// A real number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Num {
    /// `num / den`, decimal big integers.
    Exact {
        num: String,
        den: String,
    },
    Float {
        value: f64,
    },
}

/// A unit of measure. A value `v` in this unit is `(v + gauge) * factor` in SI base units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitView {
    /// Unit name in Hurmet syntax (`kip·ft`, `°C`, `m/s²`), or `None` for a quantity without
    /// a display unit.
    pub name: Option<String>,
    pub factor: f64,
    pub gauge: f64,
    /// Dimension exponents: length, mass, time, current, temperature, amount, luminous
    /// intensity, money ([`cruncha_units::DIM_NAMES`]).
    pub dims: [f64; 8],
}

/// A data frame column.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlainColumn {
    pub name: String,
    pub unit: Option<UnitView>,
    pub data: ColumnData,
}

/// A column's cells.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ColumnData {
    Number { values: Vec<Option<f64>> },
    Text { values: Vec<Option<String>> },
}

// ---------------------------------------------------------------------------------------
// Value → Plain
// ---------------------------------------------------------------------------------------

fn unsupported(kind: &str, summary: impl Into<String>) -> Plain {
    let mut summary: String = summary.into();
    if summary.chars().count() > 200 {
        summary = summary.chars().take(199).collect::<String>() + "…";
    }
    Plain::Unsupported {
        kind: kind.into(),
        summary,
    }
}

fn num_of(r: &Rational) -> Num {
    if r.den.is_zero() {
        let value = if r.num.is_zero() {
            f64::NAN
        } else if r.num.is_negative() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        Num::Float { value }
    } else {
        Num::Exact {
            num: r.num.to_string(),
            den: r.den.to_string(),
        }
    }
}

fn view_of_obj(u: &UnitObj, dims: Dims) -> UnitView {
    UnitView {
        name: u.name.clone().filter(|n| !n.is_empty()),
        factor: u.factor().to_f64(),
        gauge: u.gauge().to_f64(),
        dims,
    }
}

/// The unit of a number or array value, or `None` for a plain number.
fn unit_view(v: &Value) -> Option<UnitView> {
    if v.dtype & dt::QUANTITY == 0 {
        return None;
    }
    let dims = v.dims().unwrap_or(ALL_ZEROS);
    let view = match v.unit.obj() {
        Some(u) => view_of_obj(u, dims),
        None => UnitView {
            name: None,
            factor: 1.0,
            gauge: 0.0,
            dims,
        },
    };
    let trivial = view.name.is_none()
        && view.factor == 1.0
        && view.gauge == 0.0
        && view.dims.iter().all(|d| *d == 0.0);
    (!trivial).then_some(view)
}

fn reals(elems: &[V]) -> Option<Vec<f64>> {
    elems
        .iter()
        .map(|e| e.as_r().map(Rational::to_f64))
        .collect()
}

fn array_to_plain(v: &Value) -> Plain {
    let plain = match &v.value {
        V::Q(q) => &q.0,
        other => other,
    };
    if v.dtype & dt::RATIONAL == 0 {
        return unsupported("array", v.tex());
    }
    let elems = plain.arr();
    let (shape, orientation, values) = if v.dtype & dt::MATRIX != 0 {
        let cols = elems.first().map_or(0, V::len);
        let mut values = Vec::with_capacity(elems.len() * cols);
        for row in elems {
            if row.len() != cols {
                return unsupported("array", "ragged matrix");
            }
            match reals(row.arr()) {
                Some(r) => values.extend(r),
                None => return unsupported("array", v.tex()),
            }
        }
        (vec![elems.len(), cols], "matrix", values)
    } else {
        let Some(values) = reals(elems) else {
            return unsupported("array", v.tex());
        };
        let o = if v.dtype & dt::ROWVECTOR != 0 {
            "row"
        } else {
            "column"
        };
        (vec![elems.len()], o, values)
    };
    Plain::Array {
        shape,
        orientation: orientation.into(),
        values,
        unit: unit_view(v),
    }
}

fn cell_str(c: &V) -> Option<String> {
    match c {
        V::Undef | V::Null => None,
        V::S(s) if s.is_empty() => None,
        V::S(s) => Some(s.clone()),
        other => Some(crate::matrix::js_string(other)),
    }
}

fn cell_num(c: &V) -> Option<f64> {
    match c {
        V::R(r) => Some(r.to_f64()),
        V::Num(x) => Some(*x),
        V::S(s) if !s.is_empty() => Some(Rational::from_str_or_zero(s).to_f64()),
        _ => None,
    }
}

fn frame_to_plain(v: &Value) -> Plain {
    let V::Df(f) = &v.value else {
        return unsupported("dataframe", "");
    };
    let unit_map = match &v.unit {
        OUnit::Frame(m) => Some(m),
        _ => None,
    };
    let mut columns = Vec::with_capacity(f.data.len());
    for (j, col) in f.data.iter().enumerate() {
        // Hurmet's per-column dtype list skips empty columns, so infer from the cells.
        let is_number = col.iter().find_map(|c| match c {
            V::R(_) | V::Num(_) => Some(true),
            V::S(s) if s.is_empty() => None,
            V::S(s) => Some(dataframe::is_number_cell(s)),
            V::Undef | V::Null => None,
            _ => Some(false),
        });
        let unit_name = f
            .units
            .as_ref()
            .and_then(|u| u.get(j))
            .map(|u| u.str().to_string())
            .filter(|n| !n.is_empty());
        let unit = unit_name.as_ref().and_then(|n| {
            let obj = unit_map.and_then(|m| m.get(n))?;
            let mut view = view_of_obj(obj, obj.expos.unwrap_or(ALL_ZEROS));
            view.name = Some(n.clone());
            Some(view)
        });
        let data = if is_number == Some(true) {
            ColumnData::Number {
                values: col.iter().map(cell_num).collect(),
            }
        } else {
            ColumnData::Text {
                values: col.iter().map(cell_str).collect(),
            }
        };
        columns.push(PlainColumn {
            name: f.headings.get(j).cloned().unwrap_or_default(),
            unit: if is_number == Some(true) { unit } else { None },
            data,
        });
    }
    Plain::Frame {
        columns,
        row_names: f.row_map.is_some(),
    }
}

impl Value {
    /// This value in plain, serializable form (see the `exchange` module docs for the mapping).
    pub fn to_plain(&self) -> Plain {
        let d = self.dtype;
        if d == dt::ERROR || self.error {
            let msg = match &self.value {
                V::S(s) => s.clone(),
                _ => String::new(),
            };
            return unsupported("error", msg);
        }
        if matches!(self.value, V::Null | V::Undef) {
            return Plain::Missing;
        }
        if d & dt::DATAFRAME != 0 {
            return frame_to_plain(self);
        }
        if d & dt::MAP != 0 {
            return unsupported("map", self.tex());
        }
        if is_matrix(d) {
            return array_to_plain(self);
        }
        if d & dt::MODULE != 0 {
            let kind = if matches!(self.value, V::Module(_)) {
                "module"
            } else {
                "function"
            };
            return unsupported(kind, self.name.clone().unwrap_or_default());
        }
        if d & dt::TUPLE != 0 {
            return unsupported("tuple", self.tex());
        }
        if d & dt::DATE != 0 {
            let summary = match &self.value {
                V::R(r) => result::iso_date_string(r),
                _ => self.tex().to_string(),
            };
            return unsupported("date", summary);
        }
        if d & dt::RANGE != 0 {
            return unsupported("range", self.tex());
        }
        if d & (dt::DRAWING | dt::MACRO) != 0 {
            return unsupported("other", self.tex());
        }
        if d & dt::COMPLEX != 0 {
            let z = match &self.value {
                V::Q(q) => q.0.cpx(),
                other => other.cpx(),
            };
            return Plain::Complex {
                re: z[0].to_f64(),
                im: z[1].to_f64(),
            };
        }
        if d & dt::RATIONAL != 0 {
            let r = match &self.value {
                V::R(r) => r,
                V::Q(q) => match q.0.as_r() {
                    Some(r) => r,
                    None => return unsupported("number", self.tex()),
                },
                _ => return unsupported("number", self.tex()),
            };
            return Plain::Number {
                value: num_of(r),
                unit: unit_view(self),
            };
        }
        if d & dt::BOOLEAN != 0
            && let V::Bool(b) = self.value
        {
            return Plain::Bool { value: b };
        }
        if d & (dt::STRING | dt::RICHTEXT) != 0
            && let V::S(s) = &self.value
        {
            return Plain::Text { value: s.clone() };
        }
        unsupported("other", self.tex())
    }

    /// Build a value from its plain form, with the default [`Options`] (no currency rates).
    /// It can go into [`Vars`](crate::Vars) and is read by later statements exactly like an
    /// assigned value: unit-aware arithmetic, and `tex()` for their echoes.
    pub fn from_plain(p: &Plain) -> Result<Value, String> {
        Value::from_plain_with(p, &Options::default())
    }

    /// [`Value::from_plain`] with explicit options: the decimal format of the echo TeX, and
    /// the currency rates that monetary unit names need.
    pub fn from_plain_with(p: &Plain, opts: &Options) -> Result<Value, String> {
        let ctx = ctx_for(opts, "h15");
        from_plain_ctx(p, &ctx)
    }
}

// ---------------------------------------------------------------------------------------
// Plain → Value
// ---------------------------------------------------------------------------------------

/// How a given unit view resolves.
struct Resolved {
    /// The unit to attach; `None` for a plain number.
    obj: Option<UnitObj>,
    /// `Some((gauge, factor))`: the given value is in the view's unit, not in `obj`; convert
    /// to SI with these. `None`: the value is already as written in `obj`.
    to_si: Option<(Rational, Rational)>,
}

fn finite(x: f64, what: &str) -> Result<f64, String> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(format!("{what} is not finite: {x}"))
    }
}

fn resolve(view: Option<&UnitView>, ctx: &Ctx) -> Result<Resolved, String> {
    let Some(u) = view else {
        return Ok(Resolved {
            obj: None,
            to_si: None,
        });
    };
    if let Some(name) = u.name.as_deref().filter(|n| !n.trim().is_empty())
        && let Ok(unit) = ctx.unit(name.trim())
        && unit.expos == u.dims
    {
        return Ok(Resolved {
            obj: Some(UnitObj::from_unit(&unit)),
            to_si: None,
        });
    }
    let factor = finite(u.factor, "unit factor")?;
    let gauge = finite(u.gauge, "unit gauge")?;
    if factor == 0.0 {
        return Err("unit factor is zero".into());
    }
    let conv = if factor == 1.0 && gauge == 0.0 {
        None
    } else {
        Some((Rational::from_number(gauge), Rational::from_number(factor)))
    };
    if u.dims.iter().any(|d| !d.is_finite()) {
        return Err("unit dimensions are not finite".into());
    }
    if u.dims.iter().all(|d| *d == 0.0) {
        return Ok(Resolved {
            obj: None,
            to_si: conv,
        });
    }
    let obj = match si_unit_name(&u.dims).and_then(|n| ctx.unit(&n).ok()) {
        Some(unit) if unit.expos == u.dims => UnitObj::from_unit(&unit),
        _ => UnitObj {
            factor: Some(Rational::one()),
            gauge: Some(Rational::zero()),
            expos: Some(u.dims),
            ..Default::default()
        },
    };
    Ok(Resolved {
        obj: Some(obj),
        to_si: Some(conv.unwrap_or((Rational::zero(), Rational::one()))),
    })
}

impl Resolved {
    /// (value as written in `obj`, value in base units) for one given number.
    fn convert(&self, x: &Rational) -> (Rational, Rational) {
        let (written, base_of) = match &self.to_si {
            Some((g, f)) => {
                let base = x.add(g).multiply(f);
                match &self.obj {
                    Some(o) => (base.divide(&o.factor()).subtract(&o.gauge()), base),
                    None => (base.clone(), base),
                }
            }
            None => {
                let base = match &self.obj {
                    Some(o) => x.add(&o.gauge()).multiply(&o.factor()),
                    None => x.clone(),
                };
                (x.clone(), base)
            }
        };
        (written, base_of)
    }

    fn unit_tex(&self) -> String {
        match self.obj.as_ref().and_then(|o| o.name.as_deref()) {
            Some(n) => format!(" \\;{}", unit_tex_from_string(n)),
            None => String::new(),
        }
    }

    fn ounit(&self) -> OUnit {
        match &self.obj {
            Some(o) => OUnit::Obj(Box::new(o.clone())),
            None => OUnit::zeros(),
        }
    }
}

fn parse_big(s: &str, what: &str) -> Result<BigInt, String> {
    s.trim()
        .parse::<BigInt>()
        .map_err(|_| format!("bad {what}: {s:?}"))
}

fn rational_of(n: &Num) -> Result<Rational, String> {
    match n {
        Num::Exact { num, den } => {
            let mut num = parse_big(num, "numerator")?;
            let mut den = parse_big(den, "denominator")?;
            if den.is_zero() {
                return Err("zero denominator".into());
            }
            if den.is_negative() {
                num = -num;
                den = -den;
            }
            Ok(Rational::new(num, den))
        }
        Num::Float { value } => Ok(Rational::from_number(finite(*value, "number")?)),
    }
}

fn simple(value: V, unit: OUnit, dtype: i64, ctx: &Ctx) -> Value {
    let o = Operand::new(value, unit, dtype);
    let resultdisplay = result::operand_tex(&o, "h15", ctx);
    Value {
        name: None,
        value: o.value,
        unit: o.unit,
        dtype,
        resultdisplay,
        expos: None,
        error: false,
    }
}

fn from_plain_ctx(p: &Plain, ctx: &Ctx) -> Result<Value, String> {
    match p {
        Plain::Missing => Ok(Value {
            value: V::Null,
            ..Default::default()
        }),
        Plain::Bool { value } => Ok(simple(V::Bool(*value), OUnit::Null, dt::BOOLEAN, ctx)),
        Plain::Text { value } => Ok(simple(V::S(value.clone()), OUnit::Null, dt::STRING, ctx)),
        Plain::Complex { re, im } => {
            let re = Rational::from_number(finite(*re, "real part")?);
            let im = Rational::from_number(finite(*im, "imaginary part")?);
            Ok(simple(cpx(re, im), OUnit::zeros(), dt::COMPLEX, ctx))
        }
        Plain::Number { value, unit } => {
            let x = rational_of(value)?;
            let res = resolve(unit.as_ref(), ctx)?;
            let (written, base) = res.convert(&x);
            let shown = Operand::rational(written.clone());
            let rd = result::operand_tex(&shown, "h15", ctx);
            if res.obj.is_none() {
                return Ok(Value {
                    value: V::R(written),
                    unit: OUnit::zeros(),
                    dtype: dt::RATIONAL,
                    resultdisplay: rd,
                    ..Default::default()
                });
            }
            Ok(Value {
                value: V::Q(Box::new((V::R(written), V::R(base)))),
                unit: res.ounit(),
                dtype: dt::RATIONAL + dt::QUANTITY,
                resultdisplay: rd + &res.unit_tex(),
                ..Default::default()
            })
        }
        Plain::Array {
            shape,
            orientation,
            values,
            unit,
        } => {
            let shape_flag = match (shape.len(), orientation.as_str()) {
                (1, "row") => dt::ROWVECTOR,
                (1, "column") => dt::COLUMNVECTOR,
                (2, "matrix") => dt::MATRIX,
                _ => {
                    return Err(format!(
                        "array shape {shape:?} does not fit orientation {orientation:?}"
                    ));
                }
            };
            let n: usize = shape.iter().product();
            if n != values.len() {
                return Err(format!(
                    "array of shape {shape:?} needs {n} values, got {}",
                    values.len()
                ));
            }
            let res = resolve(unit.as_ref(), ctx)?;
            let mut written = Vec::with_capacity(n);
            let mut base = Vec::with_capacity(n);
            for x in values {
                let (w, b) = res.convert(&Rational::from_number(finite(*x, "array value")?));
                written.push(V::R(w));
                base.push(V::R(b));
            }
            let nest = |flat: Vec<V>| -> V {
                if shape_flag == dt::MATRIX {
                    let cols = shape[1].max(1);
                    let mut rows = Vec::with_capacity(shape[0]);
                    let mut it = flat.into_iter();
                    for _ in 0..shape[0] {
                        rows.push(V::A(it.by_ref().take(cols).collect()));
                    }
                    V::A(rows)
                } else {
                    V::A(flat)
                }
            };
            let written = nest(written);
            let dtype = dt::RATIONAL + shape_flag;
            let rd = crate::matrix::display(&written, dtype, "h15", ctx.decimal_format);
            if res.obj.is_none() {
                return Ok(Value {
                    value: written,
                    unit: OUnit::zeros(),
                    dtype,
                    resultdisplay: rd,
                    ..Default::default()
                });
            }
            Ok(Value {
                value: V::Q(Box::new((written, nest(base)))),
                unit: res.ounit(),
                dtype: dtype + dt::QUANTITY,
                resultdisplay: rd + &res.unit_tex(),
                ..Default::default()
            })
        }
        Plain::Frame { columns, row_names } => frame_from_plain(columns, *row_names, ctx),
        Plain::Unsupported { kind, .. } => Err(format!("cannot bring a {kind} into a calc")),
    }
}

fn frame_from_plain(columns: &[PlainColumn], row_names: bool, ctx: &Ctx) -> Result<Value, String> {
    if columns.is_empty() {
        return Err("a data frame needs at least one column".into());
    }
    let len = |c: &PlainColumn| match &c.data {
        ColumnData::Number { values } => values.len(),
        ColumnData::Text { values } => values.len(),
    };
    let nrows = len(&columns[0]);
    if let Some(c) = columns.iter().find(|c| len(c) != nrows) {
        return Err(format!(
            "column {:?} has {} rows, expected {nrows}",
            c.name,
            len(c)
        ));
    }
    let mut frame = Frame::default();
    let mut unit_map: HashMap<String, UnitObj> = HashMap::new();
    let mut units: Vec<V> = Vec::new();
    let mut dtypes: Vec<i64> = Vec::new();
    for (j, c) in columns.iter().enumerate() {
        frame.headings.push(c.name.clone());
        frame.column_map.insert(c.name.clone(), j);
        match &c.data {
            ColumnData::Text { values } => {
                frame.data.push(
                    values
                        .iter()
                        .map(|v| V::S(v.clone().unwrap_or_default()))
                        .collect(),
                );
                units.push(V::S(String::new()));
                dtypes.push(dt::STRING);
            }
            ColumnData::Number { values } => {
                let res = resolve(c.unit.as_ref(), ctx)?;
                let mut cells = Vec::with_capacity(values.len());
                for v in values {
                    cells.push(V::S(match v {
                        None => String::new(),
                        Some(x) => {
                            let x = finite(*x, "data frame value")?;
                            match &res.to_si {
                                None => jsnum::to_string(x),
                                Some(_) => jsnum::to_string(
                                    res.convert(&Rational::from_number(x)).0.to_f64(),
                                ),
                            }
                        }
                    }));
                }
                frame.data.push(cells);
                match res.obj.as_ref().and_then(|o| o.name.clone()) {
                    Some(name) => {
                        unit_map.insert(name.clone(), res.obj.clone().unwrap_or_default());
                        units.push(V::S(name));
                        dtypes.push(dt::RATIONAL + dt::QUANTITY);
                    }
                    None => {
                        units.push(V::S(String::new()));
                        dtypes.push(dt::RATIONAL);
                    }
                }
            }
        }
    }
    if row_names {
        let mut m = HashMap::new();
        for (i, c) in frame.data[0].iter().enumerate() {
            m.insert(c.str().to_string(), i);
        }
        frame.row_map = Some(m);
    }
    if !unit_map.is_empty() && matches!(columns[0].data, ColumnData::Text { .. }) {
        // Hurmet's TSV convention: the units row starts with the label `unit`.
        units[0] = V::S("unit".into());
    }
    frame.units = Some(if unit_map.is_empty() {
        Vec::new()
    } else {
        units
    });
    frame.dtype = Some(dtypes);
    let resultdisplay = dataframe::display(&frame, "h3", ctx.decimal_format, false);
    Ok(Value {
        name: None,
        value: V::Df(Arc::new(frame)),
        unit: OUnit::Frame(unit_map),
        dtype: dt::DATAFRAME,
        resultdisplay,
        expos: None,
        error: false,
    })
}
