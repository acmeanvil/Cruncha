// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Hurmet's operand model (`constants.js`, `operand.js`).
//!
//! Hurmet is dynamically typed: an operand is `{ value, unit, dtype }`, where `dtype` is a sum
//! of bit flags (`RATIONAL + COLUMNVECTOR + QUANTITY`, …) and `value` is whatever JavaScript
//! value fits (a rational is a two-element array of BigInts, a vector is an array of those, a
//! matrix an array of arrays). The port keeps that shape on purpose, so that the many places
//! where Hurmet does arithmetic on `dtype` or inspects a value's shape behave the same.

use cruncha_units::{Dims, Rational};
use std::collections::HashMap;
use std::sync::Arc;

/// Data-type flags (Hurmet's `dt`). An operand can be two types at once, e.g.
/// `RATIONAL + MATRIX`.
pub(crate) mod dt {
    pub const NULL: i64 = 0;
    pub const RATIONAL: i64 = 1;
    pub const COMPLEX: i64 = 2;
    pub const BOOLEAN: i64 = 4;
    pub const BOOLEANFROMCOMPARISON: i64 = 12;
    pub const STRING: i64 = 16;
    pub const QUANTITY: i64 = 32;
    pub const DATE: i64 = 64;
    pub const RANGE: i64 = 128;
    pub const TUPLE: i64 = 256;
    pub const MAP: i64 = 512;
    pub const ROWVECTOR: i64 = 1024;
    pub const COLUMNVECTOR: i64 = 2048;
    pub const MATRIX: i64 = 4096;
    pub const DATAFRAME: i64 = 8192;
    pub const MODULE: i64 = 16384;
    pub const ERROR: i64 = 32768;
    pub const DRAWING: i64 = 131072;
    pub const RICHTEXT: i64 = 262144;
    pub const MACRO: i64 = 1048576;
}

/// A JavaScript value, as far as Hurmet operands use them.
#[derive(Clone, Debug, Default)]
pub(crate) enum V {
    /// `undefined`
    #[default]
    Undef,
    /// `null`: a variable whose value is missing.
    Null,
    Bool(bool),
    /// A plain JavaScript number (rare: Hurmet mostly uses rationals).
    Num(f64),
    R(Rational),
    /// A complex number, `[re, im]`.
    C(Box<[Rational; 2]>),
    S(String),
    /// An array: a vector, a matrix (array of rows), or a range `[start, step, end]`.
    A(Vec<V>),
    /// `{ plain, inBaseUnits }`: a quantity's value as written and in SI base units.
    Q(Box<(V, V)>),
    /// A data frame or a map (shared, copied on write).
    Df(Arc<Frame>),
    /// A tuple (multiple return values), in insertion order.
    Tuple(Vec<(String, Operand)>),
    /// A user-defined function.
    Udf(Arc<Udf>),
    /// A module: named functions and values.
    Module(Arc<HashMap<String, Value>>),
}

impl V {
    pub fn r(&self) -> Rational {
        match self {
            V::R(r) => r.clone(),
            V::Q(q) => q.0.r(),
            _ => Rational::zero(),
        }
    }
    pub fn as_r(&self) -> Option<&Rational> {
        match self {
            V::R(r) => Some(r),
            _ => None,
        }
    }
    pub fn is_rational(&self) -> bool {
        matches!(self, V::R(_))
    }
    pub fn arr(&self) -> &[V] {
        match self {
            V::A(a) => a,
            _ => &[],
        }
    }
    pub fn len(&self) -> usize {
        match self {
            V::A(a) => a.len(),
            V::S(s) => cruncha_lang::hurmet::utf16_len(s),
            _ => 0,
        }
    }
    pub fn get(&self, i: usize) -> V {
        match self {
            V::A(a) => a.get(i).cloned().unwrap_or(V::Undef),
            _ => V::Undef,
        }
    }
    pub fn str(&self) -> &str {
        match self {
            V::S(s) => s,
            _ => "",
        }
    }
    /// JavaScript truthiness.
    pub fn truthy(&self) -> bool {
        match self {
            V::Undef | V::Null => false,
            V::Bool(b) => *b,
            V::Num(n) => *n != 0.0 && !n.is_nan(),
            V::S(s) => !s.is_empty(),
            _ => true,
        }
    }
    pub fn is_complex_pair(&self) -> bool {
        // Hurmet's Cpx.isComplex: an array of exactly two rationals.
        match self {
            V::C(_) => true,
            V::A(a) => a.len() == 2 && a[0].is_rational() && a[1].is_rational(),
            _ => false,
        }
    }
    pub fn cpx(&self) -> [Rational; 2] {
        match self {
            V::C(z) => (**z).clone(),
            V::A(a) if a.len() == 2 => [a[0].r(), a[1].r()],
            V::R(r) => [r.clone(), Rational::zero()],
            _ => [Rational::zero(), Rational::zero()],
        }
    }
}

pub(crate) fn cpx(re: Rational, im: Rational) -> V {
    V::C(Box::new([re, im]))
}

/// A unit attached to an operand. Hurmet stores several shapes here.
#[derive(Clone, Debug, Default)]
pub(crate) enum OUnit {
    #[default]
    Null,
    /// `{ name?, factor?, gauge?, expos?, isConverted? }`
    Obj(Box<UnitObj>),
    /// Hurmet sometimes stores the exponent array itself as the unit; `unit.expos` is then
    /// `undefined`.
    Bare,
    /// A colour name, for coloured text (Hurmet's "hack to return a color").
    Color(String),
    /// A data frame's map from unit name to unit.
    Frame(HashMap<String, UnitObj>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UnitObj {
    pub name: Option<String>,
    pub factor: Option<Rational>,
    pub gauge: Option<Rational>,
    pub expos: Option<Dims>,
    pub is_converted: bool,
}

impl OUnit {
    pub fn expos(d: Dims) -> OUnit {
        OUnit::Obj(Box::new(UnitObj {
            expos: Some(d),
            ..Default::default()
        }))
    }
    pub fn zeros() -> OUnit {
        OUnit::expos(cruncha_units::ALL_ZEROS)
    }
    pub fn get_expos(&self) -> Option<&Dims> {
        match self {
            OUnit::Obj(u) => u.expos.as_ref(),
            _ => None,
        }
    }
    pub fn obj(&self) -> Option<&UnitObj> {
        match self {
            OUnit::Obj(u) => Some(u),
            _ => None,
        }
    }
    pub fn name(&self) -> Option<&str> {
        self.obj().and_then(|u| u.name.as_deref())
    }
    pub fn truthy(&self) -> bool {
        !matches!(self, OUnit::Null)
    }
    pub fn from_unit(u: &cruncha_units::Unit) -> OUnit {
        OUnit::Obj(Box::new(UnitObj::from_unit(u)))
    }
}

impl UnitObj {
    pub fn from_unit(u: &cruncha_units::Unit) -> UnitObj {
        UnitObj {
            name: Some(u.name.clone()),
            factor: Some(u.factor.clone()),
            gauge: Some(u.gauge.clone()),
            expos: Some(u.expos),
            is_converted: false,
        }
    }
    pub fn factor(&self) -> Rational {
        self.factor.clone().unwrap_or_else(Rational::one)
    }
    pub fn gauge(&self) -> Rational {
        self.gauge.clone().unwrap_or_else(Rational::zero)
    }
}

/// An operand on the evaluation stack.
#[derive(Clone, Debug, Default)]
pub(crate) struct Operand {
    pub value: V,
    pub unit: OUnit,
    pub dtype: i64,
    pub name: Option<String>,
}

impl Operand {
    pub fn new(value: V, unit: OUnit, dtype: i64) -> Operand {
        Operand {
            value,
            unit,
            dtype,
            name: None,
        }
    }
    pub fn is_error(&self) -> bool {
        self.dtype == dt::ERROR
    }
    pub fn rational(r: Rational) -> Operand {
        Operand::new(V::R(r), OUnit::zeros(), dt::RATIONAL)
    }
    pub fn boolean(b: bool) -> Operand {
        Operand::new(V::Bool(b), OUnit::Null, dt::BOOLEAN)
    }
    pub fn string(s: impl Into<String>) -> Operand {
        Operand::new(V::S(s.into()), OUnit::Null, dt::STRING)
    }
    pub fn error_message(&self) -> &str {
        self.value.str()
    }
}

/// Hurmet's `isMatrix`: a row vector, a column vector or a 2-D matrix.
pub(crate) fn is_matrix(dtype: i64) -> bool {
    dtype & (dt::ROWVECTOR | dt::COLUMNVECTOR | dt::MATRIX) != 0
}

pub(crate) fn is_vector(dtype: i64) -> bool {
    dtype & (dt::ROWVECTOR | dt::COLUMNVECTOR) != 0
}

/// A data frame (`dtype` present) or a map (`dtype` absent): data stored column by column.
#[derive(Clone, Debug, Default)]
pub(crate) struct Frame {
    /// Columns. In a data frame the cells are the strings from the TSV; in a map, values.
    pub data: Vec<Vec<V>>,
    /// For a map with a unit: the data in base units (`data` is then `data.plain`).
    pub data_base: Option<Vec<Vec<V>>>,
    pub headings: Vec<String>,
    pub column_map: HashMap<String, usize>,
    pub row_map: Option<HashMap<String, usize>>,
    /// Unit names per column (data frames only): strings, or null/undefined as in Hurmet.
    pub units: Option<Vec<V>>,
    /// Data type per column (data frames only).
    pub dtype: Option<Vec<i64>>,
}

/// One statement of a user-defined function.
#[derive(Clone, Debug, Default)]
pub(crate) struct FnStatement {
    pub name: Vec<String>,
    pub rpn: String,
    pub stype: String,
    pub end_of_block: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Parameter {
    pub name: String,
    pub default: Option<Value>,
}

/// A user-defined function (Hurmet's `funcObj`), or an anonymous `x → expr` function.
#[derive(Clone, Debug, Default)]
pub(crate) struct Udf {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub statements: Vec<FnStatement>,
}

/// A value that a statement assigns to a variable (Hurmet's cell attributes).
///
/// It holds the value as written and, for quantities, in SI base units, together with the
/// unit, the data type, and the TeX that the blue echo of later statements shows for it.
#[derive(Clone, Debug, Default)]
pub struct Value {
    pub(crate) name: Option<String>,
    pub(crate) value: V,
    pub(crate) unit: OUnit,
    pub(crate) dtype: i64,
    pub(crate) resultdisplay: String,
    pub(crate) expos: Option<Dims>,
    pub(crate) error: bool,
}

/// The kind of a [`Value`], for callers that do not need Hurmet's flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Number,
    Complex,
    Boolean,
    Text,
    Date,
    Vector,
    Matrix,
    DataFrame,
    Map,
    Tuple,
    Function,
    Error,
    Missing,
    Other,
}

impl Value {
    pub(crate) fn from_operand(o: &Operand) -> Value {
        Value {
            name: o.name.clone(),
            value: o.value.clone(),
            unit: o.unit.clone(),
            dtype: o.dtype,
            ..Default::default()
        }
    }

    /// What sort of value this is.
    pub fn kind(&self) -> Kind {
        let d = self.dtype;
        if matches!(self.value, V::Null) {
            Kind::Missing
        } else if d == dt::ERROR || self.error {
            Kind::Error
        } else if d & dt::DATAFRAME != 0 {
            Kind::DataFrame
        } else if d & dt::MAP != 0 {
            Kind::Map
        } else if d & dt::MATRIX != 0 {
            Kind::Matrix
        } else if is_vector(d) {
            Kind::Vector
        } else if d & dt::MODULE != 0 {
            Kind::Function
        } else if d & dt::TUPLE != 0 {
            Kind::Tuple
        } else if d & dt::COMPLEX != 0 {
            Kind::Complex
        } else if d & dt::DATE != 0 {
            Kind::Date
        } else if d & dt::RATIONAL != 0 {
            Kind::Number
        } else if d & dt::BOOLEAN != 0 {
            Kind::Boolean
        } else if d & (dt::STRING | dt::RICHTEXT) != 0 {
            Kind::Text
        } else {
            Kind::Other
        }
    }

    /// The number as written (before unit conversion), for a scalar number.
    pub fn as_f64(&self) -> Option<f64> {
        if self.dtype & (dt::RATIONAL | dt::DATE) == 0 || is_matrix(self.dtype) {
            return None;
        }
        match &self.value {
            V::R(r) => Some(r.to_f64()),
            V::Q(q) => q.0.as_r().map(|r| r.to_f64()),
            _ => None,
        }
    }

    /// The number in SI base units, for a scalar quantity (or the plain number).
    pub fn as_base_f64(&self) -> Option<f64> {
        match &self.value {
            V::Q(q) => q.1.as_r().map(|r| r.to_f64()),
            _ => self.as_f64(),
        }
    }

    /// The exact rational, for a scalar number.
    pub fn as_rational(&self) -> Option<cruncha_units::Rational> {
        match &self.value {
            V::R(r) if self.dtype & dt::RATIONAL != 0 => Some(r.clone()),
            V::Q(q) => q.0.as_r().cloned(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.value {
            V::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            V::S(s) if self.dtype & (dt::STRING | dt::RICHTEXT) != 0 => Some(s),
            _ => None,
        }
    }

    /// The display unit's name, if the value is a quantity with a named unit.
    pub fn unit_name(&self) -> Option<&str> {
        self.unit.name()
    }

    /// Dimension exponents, for a quantity.
    pub fn dims(&self) -> Option<Dims> {
        self.expos.or_else(|| self.unit.get_expos().copied())
    }

    /// The TeX that a later statement's echo shows for this value.
    pub fn tex(&self) -> &str {
        &self.resultdisplay
    }

    /// Hurmet's data type flags (see Hurmet's `constants.js`).
    pub fn hurmet_dtype(&self) -> i64 {
        self.dtype
    }
}
