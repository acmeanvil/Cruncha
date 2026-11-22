// SPDX-License-Identifier: GPL-3.0-or-later
//! What a Rust cell gets (ADR-0006): `Quantity` (a value with dimensions, convertible to any
//! unit Cruncha knows), and the plumbing between the cell and the document (`Plain` JSON in
//! and out). A cell's source is placed inside a function; its reads arrive as typed `let`s,
//! and its top-level `let`s go back as values.

pub mod bindings {
    wit_bindgen::generate!({
        world: "cell",
        path: "../../wit",
        pub_export_macro: true,
        export_macro_name: "export_cell",
    });
}

pub use bindings::Guest;
pub use serde_json::{self, Value, json};

use std::ops::{Add, Div, Mul, Neg, Sub};

/// Dimension exponents: length, mass, time, current, temperature, amount, luminous intensity,
/// money (as in `cruncha_units`).
pub type Dims = [f64; 8];

/// A unit to show a quantity in: `value_in_unit = si / factor - gauge`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    pub name: &'static str,
    pub factor: f64,
    pub gauge: f64,
}

/// Unit names are interned, so `Quantity` is `Copy` (formulas like `w * L * L` just work).
fn intern(name: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut set = NAMES.get_or_init(Mutex::default).lock().unwrap();
    if let Some(n) = set.get(name) {
        return n;
    }
    let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
    set.insert(leaked);
    leaked
}

/// A number with dimensions, held in SI base units. Arithmetic checks dimensions: adding a
/// length to a force panics with a message, as a calc would refuse it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    pub si: f64,
    pub dims: Dims,
    /// The unit it is shown in (set by `to`, or kept from the calc it came from).
    pub unit: Option<Unit>,
}

fn same(a: &Dims, b: &Dims) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
}

fn dims_text(d: &Dims) -> String {
    const NAMES: [&str; 8] = ["length", "mass", "time", "current", "temperature", "amount", "luminosity", "money"];
    let parts: Vec<String> = d
        .iter()
        .zip(NAMES)
        .filter(|(e, _)| **e != 0.0)
        .map(|(e, n)| if *e == 1.0 { n.to_string() } else { format!("{n}^{e}") })
        .collect();
    if parts.is_empty() { "dimensionless".into() } else { parts.join("·") }
}

/// A unit by name (`kip*ft`, `in²`, `MPa`), from Cruncha's unit table.
pub fn unit(name: &str) -> Result<(Unit, Dims), String> {
    let u = cruncha_units::unit_from_name(name, &Default::default()).map_err(|e| format!("unit {name}: {e:?}"))?;
    Ok((Unit { name: intern(&u.name), factor: u.factor.to_f64(), gauge: u.gauge.to_f64() }, u.expos))
}

impl Quantity {
    /// `value` in `unit_name`: `Quantity::new(12.0, "kip")`.
    #[track_caller]
    pub fn new(value: f64, unit_name: &str) -> Quantity {
        let (u, dims) = unit(unit_name).unwrap_or_else(|e| panic!("{e}"));
        Quantity { si: (value + u.gauge) * u.factor, dims, unit: Some(u) }
    }

    /// A dimensionless quantity.
    pub fn number(x: f64) -> Quantity {
        Quantity { si: x, dims: [0.0; 8], unit: None }
    }

    /// The value in SI base units.
    pub fn si(&self) -> f64 {
        self.si
    }

    /// The same quantity shown in `unit_name`; panics if the dimensions differ.
    #[track_caller]
    pub fn to(&self, unit_name: &str) -> Quantity {
        let (u, dims) = unit(unit_name).unwrap_or_else(|e| panic!("{e}"));
        if !same(&dims, &self.dims) {
            panic!("can't show {} in {unit_name} ({})", dims_text(&self.dims), dims_text(&dims));
        }
        Quantity { si: self.si, dims: self.dims, unit: Some(u) }
    }

    /// The number in its display unit (SI when it has none).
    pub fn value(&self) -> f64 {
        match &self.unit {
            Some(u) => self.si / u.factor - u.gauge,
            None => self.si,
        }
    }

    /// The number in `unit_name`.
    #[track_caller]
    pub fn value_in(&self, unit_name: &str) -> f64 {
        self.to(unit_name).value()
    }

    pub fn is_dimensionless(&self) -> bool {
        self.dims.iter().all(|d| *d == 0.0)
    }

    pub fn powi(&self, n: i32) -> Quantity {
        Quantity { si: self.si.powi(n), dims: self.dims.map(|d| d * n as f64), unit: None }
    }

    pub fn powf(&self, p: f64) -> Quantity {
        Quantity { si: self.si.powf(p), dims: self.dims.map(|d| d * p), unit: None }
    }

    pub fn sqrt(&self) -> Quantity {
        self.powf(0.5)
    }

    pub fn abs(&self) -> Quantity {
        Quantity { si: self.si.abs(), ..*self }
    }

    #[track_caller]
    pub fn max(&self, other: &Quantity) -> Quantity {
        self.check(other, "max");
        if other.si > self.si { *other } else { *self }
    }

    #[track_caller]
    pub fn min(&self, other: &Quantity) -> Quantity {
        self.check(other, "min");
        if other.si < self.si { *other } else { *self }
    }

    #[track_caller]
    fn check(&self, other: &Quantity, op: &str) {
        if !same(&self.dims, &other.dims) {
            panic!("{op}: {} and {} have different dimensions", dims_text(&self.dims), dims_text(&other.dims));
        }
    }
}

impl std::fmt::Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.unit {
            Some(u) => write!(f, "{} {}", fmt_num(self.value()), u.name),
            None if self.is_dimensionless() => write!(f, "{}", fmt_num(self.si)),
            None => write!(f, "{} (SI: {})", fmt_num(self.si), dims_text(&self.dims)),
        }
    }
}

fn fmt_num(x: f64) -> String {
    if x != 0.0 && (x.abs() >= 1e9 || x.abs() < 1e-4) { format!("{x:e}") } else { format!("{}", (x * 1e9).round() / 1e9) }
}

impl PartialOrd for Quantity {
    #[track_caller]
    fn partial_cmp(&self, other: &Quantity) -> Option<std::cmp::Ordering> {
        self.check(other, "compare");
        self.si.partial_cmp(&other.si)
    }
}

impl Add for Quantity {
    type Output = Quantity;
    #[track_caller]
    fn add(self, o: Quantity) -> Quantity {
        self.check(&o, "+");
        Quantity { si: self.si + o.si, ..self }
    }
}

impl Sub for Quantity {
    type Output = Quantity;
    #[track_caller]
    fn sub(self, o: Quantity) -> Quantity {
        self.check(&o, "-");
        Quantity { si: self.si - o.si, ..self }
    }
}

impl Neg for Quantity {
    type Output = Quantity;
    fn neg(self) -> Quantity {
        Quantity { si: -self.si, ..self }
    }
}

impl Mul for Quantity {
    type Output = Quantity;
    fn mul(self, o: Quantity) -> Quantity {
        let mut dims = self.dims;
        dims.iter_mut().zip(o.dims).for_each(|(a, b)| *a += b);
        Quantity { si: self.si * o.si, dims, unit: None }
    }
}

impl Div for Quantity {
    type Output = Quantity;
    fn div(self, o: Quantity) -> Quantity {
        let mut dims = self.dims;
        dims.iter_mut().zip(o.dims).for_each(|(a, b)| *a -= b);
        Quantity { si: self.si / o.si, dims, unit: None }
    }
}

impl Mul<f64> for Quantity {
    type Output = Quantity;
    fn mul(self, k: f64) -> Quantity {
        Quantity { si: self.si * k, ..self }
    }
}

impl Div<f64> for Quantity {
    type Output = Quantity;
    fn div(self, k: f64) -> Quantity {
        Quantity { si: self.si / k, ..self }
    }
}

impl Mul<Quantity> for f64 {
    type Output = Quantity;
    fn mul(self, q: Quantity) -> Quantity {
        q * self
    }
}

impl Div<Quantity> for f64 {
    type Output = Quantity;
    fn div(self, q: Quantity) -> Quantity {
        Quantity { si: self / q.si, dims: q.dims.map(|d| -d), unit: None }
    }
}

macro_rules! by_ref {
    ($($tr:ident $m:ident),*) => {$(
        impl $tr<&Quantity> for &Quantity {
            type Output = Quantity;
            #[track_caller]
            fn $m(self, o: &Quantity) -> Quantity { (*self).$m(*o) }
        }
        impl $tr<f64> for &Quantity {
            type Output = Quantity;
            #[track_caller]
            fn $m(self, k: f64) -> Quantity { (*self).$m(Quantity::number(k)) }
        }
    )*};
}
by_ref!(Add add, Sub sub, Mul mul, Div div);

// ---- values in and out (the Plain exchange format) -----------------------------------------

/// A value from the document, converted to the Rust type its `let` declares.
pub trait FromPlain: Sized {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String>;
}

fn unit_of(v: &Value) -> Option<(Unit, Dims)> {
    let u = v.get("unit").filter(|u| !u.is_null())?;
    let dims: Dims = serde_json::from_value(u["dims"].clone()).ok()?;
    let name = u["name"].as_str().map(str::to_owned);
    let factor = u["factor"].as_f64().unwrap_or(1.0);
    let gauge = u["gauge"].as_f64().unwrap_or(0.0);
    Some((Unit { name: intern(&name.unwrap_or_default()), factor, gauge }, dims))
}

fn number_of(v: &Value) -> Result<f64, String> {
    match v["type"].as_str() {
        Some("float") => v["value"].as_f64().ok_or_else(|| "not a number".into()),
        Some("exact") => {
            let n: f64 = v["num"].as_str().and_then(|s| s.parse().ok()).ok_or("bad numerator")?;
            let d: f64 = v["den"].as_str().and_then(|s| s.parse().ok()).ok_or("bad denominator")?;
            Ok(n / d)
        }
        _ => Err("not a number".into()),
    }
}

fn quantity(v: f64, unit: Option<(Unit, Dims)>) -> Quantity {
    match unit {
        Some((u, dims)) => {
            let si = (v + u.gauge) * u.factor;
            let shown = (!u.name.is_empty()).then_some(u);
            Quantity { si, dims, unit: shown }
        }
        None => Quantity::number(v),
    }
}

impl FromPlain for Quantity {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String> {
        if v["type"] != "number" {
            return Err(format!("{name} isn't a number"));
        }
        Ok(quantity(number_of(&v["value"]).map_err(|e| format!("{name}: {e}"))?, unit_of(v)))
    }
}

impl FromPlain for f64 {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String> {
        let q = Quantity::from_plain(name, v)?;
        if !q.is_dimensionless() {
            return Err(format!("{name} has a unit; read it as a Quantity"));
        }
        Ok(q.si)
    }
}

impl FromPlain for bool {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String> {
        v["value"].as_bool().filter(|_| v["type"] == "bool").ok_or_else(|| format!("{name} isn't true or false"))
    }
}

impl FromPlain for String {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String> {
        v["value"].as_str().filter(|_| v["type"] == "text").map(str::to_owned).ok_or_else(|| format!("{name} isn't text"))
    }
}

fn array_values(name: &str, v: &Value) -> Result<(Vec<usize>, Vec<f64>), String> {
    if v["type"] != "array" {
        return Err(format!("{name} isn't a vector or matrix"));
    }
    let shape: Vec<usize> = serde_json::from_value(v["shape"].clone()).map_err(|e| e.to_string())?;
    let values: Vec<f64> = serde_json::from_value(v["values"].clone()).map_err(|e| e.to_string())?;
    Ok((shape, values))
}

impl FromPlain for Vec<f64> {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String> {
        let (_, values) = array_values(name, v)?;
        if unit_of(v).is_some_and(|(_, d)| !d.iter().all(|x| *x == 0.0)) {
            return Err(format!("{name} has a unit; read it as Vec<Quantity>"));
        }
        Ok(values)
    }
}

impl FromPlain for Vec<Quantity> {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String> {
        let (_, values) = array_values(name, v)?;
        let u = unit_of(v);
        Ok(values.into_iter().map(|x| quantity(x, u)).collect())
    }
}

impl FromPlain for Vec<Vec<f64>> {
    fn from_plain(name: &str, v: &Value) -> Result<Self, String> {
        let (shape, values) = array_values(name, v)?;
        let cols = *shape.get(1).unwrap_or(&values.len()).max(&1);
        Ok(values.chunks(cols).map(<[f64]>::to_vec).collect())
    }
}

/// A value going back to the document.
pub trait ToPlain {
    fn to_plain(&self) -> Value;
    fn summary(&self) -> String;
}

fn unit_view(q: &Quantity) -> Value {
    match &q.unit {
        Some(u) => json!({"name": u.name, "factor": u.factor, "gauge": u.gauge, "dims": q.dims}),
        None if q.is_dimensionless() => Value::Null,
        None => json!({"name": null, "factor": 1.0, "gauge": 0.0, "dims": q.dims}),
    }
}

fn float(x: f64) -> Value {
    json!({"type": "float", "value": x})
}

impl ToPlain for Quantity {
    fn to_plain(&self) -> Value {
        json!({"type": "number", "value": float(self.value()), "unit": unit_view(self)})
    }
    fn summary(&self) -> String {
        self.to_string()
    }
}

macro_rules! plain_number {
    ($($t:ty),*) => {$(
        impl ToPlain for $t {
            fn to_plain(&self) -> Value { json!({"type": "number", "value": float(*self as f64), "unit": null}) }
            fn summary(&self) -> String { self.to_string() }
        }
    )*};
}
plain_number!(f64, f32, i64, i32, u64, u32, usize, isize);

impl ToPlain for bool {
    fn to_plain(&self) -> Value {
        json!({"type": "bool", "value": self})
    }
    fn summary(&self) -> String {
        self.to_string()
    }
}

impl ToPlain for String {
    fn to_plain(&self) -> Value {
        json!({"type": "text", "value": self})
    }
    fn summary(&self) -> String {
        format!("{self:?}")
    }
}

impl ToPlain for &str {
    fn to_plain(&self) -> Value {
        json!({"type": "text", "value": self})
    }
    fn summary(&self) -> String {
        format!("{self:?}")
    }
}

impl ToPlain for Vec<f64> {
    fn to_plain(&self) -> Value {
        json!({"type": "array", "shape": [self.len()], "orientation": "column", "values": self, "unit": null})
    }
    fn summary(&self) -> String {
        format!("vector of {}", self.len())
    }
}

impl ToPlain for Vec<Quantity> {
    fn to_plain(&self) -> Value {
        let Some(first) = self.first() else {
            return json!({"type": "array", "shape": [0], "orientation": "column", "values": [], "unit": null});
        };
        for q in self {
            first.check(q, "a vector");
        }
        let shown = Quantity { si: 0.0, dims: first.dims, unit: first.unit };
        let values: Vec<f64> = self.iter().map(|q| Quantity { unit: first.unit, ..*q }.value()).collect();
        json!({"type": "array", "shape": [self.len()], "orientation": "column", "values": values, "unit": unit_view(&shown)})
    }
    fn summary(&self) -> String {
        let unit = self.first().and_then(|q| q.unit.as_ref()).map(|u| format!(" ({})", u.name)).unwrap_or_default();
        format!("vector of {}{unit}", self.len())
    }
}

impl ToPlain for Vec<Vec<f64>> {
    fn to_plain(&self) -> Value {
        let cols = self.first().map_or(0, Vec::len);
        let values: Vec<f64> = self.iter().flatten().copied().collect();
        json!({"type": "array", "shape": [self.len(), cols], "orientation": "matrix", "values": values, "unit": null})
    }
    fn summary(&self) -> String {
        format!("{} × {} matrix", self.len(), self.first().map_or(0, Vec::len))
    }
}

/// Plumbing for the generated cell code; not for cells to use.
#[doc(hidden)]
pub mod __private {
    use super::*;

    /// Autoref specialization: values with `ToPlain` go back as data; anything else as its
    /// type name (other Rust code can't use it, and calcs are told why).
    pub struct Wrap<'a, T>(pub &'a T);

    // Called as `(&&Wrap(&x)).out()`: method lookup tries `&&Wrap` first (the `ToPlain` impl,
    // when `T: ToPlain` holds), then `&Wrap` (any `T`).
    pub trait ViaPlain {
        fn out(&self) -> (Value, String);
    }
    impl<T: ToPlain> ViaPlain for &Wrap<'_, T> {
        fn out(&self) -> (Value, String) {
            (self.0.to_plain(), self.0.summary())
        }
    }

    pub trait ViaOpaque {
        fn out(&self) -> (Value, String);
    }
    impl<T> ViaOpaque for Wrap<'_, T> {
        fn out(&self) -> (Value, String) {
            let ty = std::any::type_name::<T>();
            let kind = ty.rsplit("::").next().unwrap_or(ty).to_owned();
            (json!({"type": "unsupported", "kind": kind, "summary": ty}), format!("a Rust {ty}"))
        }
    }

    pub struct Inputs(Value);

    pub fn inputs(json: &str) -> Result<Inputs, String> {
        serde_json::from_str(json).map(Inputs).map_err(|e| format!("couldn't read the inputs: {e}"))
    }

    impl Inputs {
        pub fn get<T: FromPlain>(&self, name: &str) -> Result<T, String> {
            T::from_plain(name, &self.0[name])
        }
    }

    pub struct Outputs(Vec<Value>);

    impl Outputs {
        pub fn new() -> Outputs {
            Outputs(Vec::new())
        }
        pub fn put(&mut self, name: &str, (plain, summary): (Value, String)) {
            self.0.push(json!({"name": name, "summary": summary, "plain": plain}));
        }
        pub fn finish(self) -> String {
            Value::Array(self.0).to_string()
        }
    }

    impl Default for Outputs {
        fn default() -> Self {
            Outputs::new()
        }
    }
}
