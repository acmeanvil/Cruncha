// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Evaluation of Hurmet's RPN (`evaluate.js`: `evalRpn`, `evalCustomFunction`) and the
//! conversion of a variable's stored value into an operand (`operand.js`: `fromAssignment`).

use crate::Vars;
use crate::dataframe;
use crate::error::{err, err_msg, err0, unit_err};
use crate::format::{Mode, fmt};
use crate::functions::{self, Arity, multivar_function};
use crate::matrix;
use crate::operations::{self as ops, Shape, binary_shapes, shape_of};
use crate::value::{
    FnStatement, OUnit, Operand, Parameter, Udf, UnitObj, V, Value, cpx, dt, is_matrix, is_vector,
};
use cruncha_units::{ALL_ZEROS, Currencies, Dims, Rational, Unit, units_are_compatible};
use num_bigint::BigInt;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;

const SEP: char = '\u{a0}';
/// Nested user-defined function calls allowed before evaluation gives up.
const MAX_CALL_DEPTH: u32 = 32;
/// Statements a user-defined function may execute (guards against endless loops).
const MAX_STEPS: u64 = 1_000_000;

/// What evaluation needs besides the variables.
pub(crate) struct Ctx<'a> {
    pub decimal_format: &'a str,
    pub date_format: Option<&'a str>,
    pub currencies: &'a Currencies,
    pub depth: Cell<u32>,
    pub format_spec: String,
}

impl Ctx<'_> {
    pub fn unit(&self, name: &str) -> Result<Unit, Operand> {
        cruncha_units::unit_from_name(name, self.currencies).map_err(|e| unit_err(&e))
    }
}

/// Variables visible to an evaluation: locals (function parameters, `∑` indices) over the
/// document's variables.
pub(crate) struct Scope<'a> {
    pub locals: HashMap<String, Value>,
    pub globals: Option<&'a Vars>,
}

impl<'a> Scope<'a> {
    pub fn global(vars: &'a Vars) -> Self {
        Scope {
            locals: HashMap::new(),
            globals: Some(vars),
        }
    }
    pub fn empty() -> Scope<'static> {
        Scope {
            locals: HashMap::new(),
            globals: None,
        }
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.locals
            .get(name)
            .or_else(|| self.globals.and_then(|g| g.get(name)))
    }
}

/// Hurmet's `fromAssignment(cellAttrs, unitAware)`.
pub(crate) fn from_assignment(cell: &Value, unit_aware: bool) -> Operand {
    if matches!(cell.value, V::Null) {
        let insert = cell
            .name
            .clone()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "?".into());
        return err("NULL", &insert);
    }
    let mut o = Operand {
        value: V::Undef,
        unit: OUnit::Null,
        dtype: cell.dtype,
        name: cell.name.clone(),
    };
    let d = cell.dtype;
    o.unit = if d == dt::STRING
        || d == dt::BOOLEAN
        || d == dt::DRAWING
        || d == dt::MODULE
        || d == dt::NULL
    {
        OUnit::Null
    } else {
        cell.unit.clone()
    };
    if d & dt::QUANTITY != 0 {
        if d & dt::MAP != 0 {
            o.value = match &cell.value {
                V::Df(f) => {
                    let mut fr = (**f).clone();
                    if unit_aware {
                        if let Some(base) = fr.data_base.take() {
                            fr.data = base;
                        }
                    } else {
                        fr.data_base = None;
                    }
                    V::Df(Arc::new(fr))
                }
                other => other.clone(),
            };
        } else {
            o.value = match &cell.value {
                V::Q(q) => {
                    if unit_aware {
                        q.1.clone()
                    } else {
                        q.0.clone()
                    }
                }
                other => other.clone(),
            };
        }
        o.dtype = d - dt::QUANTITY;
        if unit_aware {
            o.unit = OUnit::Obj(Box::new(UnitObj {
                expos: o.unit.get_expos().copied(),
                is_converted: true,
                ..Default::default()
            }));
        }
    } else if d == dt::STRING {
        let s = cell.value.str();
        o.value = V::S(if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
            s[1..s.len() - 1].trim().to_string()
        } else {
            s.trim().to_string()
        });
    } else {
        o.value = cell.value.clone();
    }
    o
}

fn is_not_a_number(o: &Operand) -> bool {
    o.dtype == dt::ERROR || o.dtype & dt::RATIONAL == 0
}

fn string_from_operand(o: &Operand, ctx: &Ctx) -> String {
    if o.dtype == dt::STRING {
        o.value.str().to_string()
    } else if o.dtype == dt::RATIONAL {
        fmt(&o.value.r(), "h15", ctx.decimal_format, Mode::Plain)
    } else if is_matrix(o.dtype) {
        matrix::display_alt(
            &o.value,
            o.dtype,
            "h15",
            ctx.decimal_format,
            Mode::Plain,
            "",
        )
    } else if o.dtype & dt::MAP != 0 {
        match &o.value {
            V::Df(f) => dataframe::display_alt(f, "h15", ctx.decimal_format, false),
            _ => String::new(),
        }
    } else {
        matrix::js_string(&o.value)
    }
}

fn add_expos(a: Option<&Dims>, b: Option<&Dims>, sign: f64) -> Option<Dims> {
    match (a, b) {
        (Some(a), Some(b)) => {
            let mut c = *a;
            for (x, y) in c.iter_mut().zip(b) {
                *x += sign * y;
            }
            Some(c)
        }
        _ => None,
    }
}

fn scale_expos(a: Option<&Dims>, f: f64) -> Option<Dims> {
    a.map(|a| a.map(|x| x * f))
}

fn expos_unit(e: Option<Dims>) -> OUnit {
    OUnit::Obj(Box::new(UnitObj {
        expos: e,
        ..Default::default()
    }))
}

fn date_operand(seconds: BigInt) -> Operand {
    let mut d = ALL_ZEROS;
    d[2] = 1.0;
    Operand::new(
        V::R(Rational::new(seconds, BigInt::from(1))),
        OUnit::expos(d),
        dt::DATE,
    )
}

fn parse_rational_token(t: &str) -> Option<Rational> {
    let (n, d) = t.split_once('/')?;
    Some(Rational::new(n.parse().ok()?, d.parse().ok()?))
}

fn unary_fn_names(t: &str) -> bool {
    matches!(
        t,
        "abs"
            | "cos"
            | "sin"
            | "tan"
            | "acos"
            | "asin"
            | "atan"
            | "sec"
            | "csc"
            | "cot"
            | "asec"
            | "acsc"
            | "acot"
            | "exp"
            | "log"
            | "ln"
            | "log10"
            | "log2"
            | "cosh"
            | "sinh"
            | "tanh"
            | "sech"
            | "csch"
            | "coth"
            | "acosh"
            | "asinh"
            | "atanh"
            | "asech"
            | "acsch"
            | "acoth"
            | "gamma"
            | "Γ"
            | "lgamma"
            | "lfact"
            | "factorial"
            | "cosd"
            | "sind"
            | "tand"
            | "acosd"
            | "asind"
            | "atand"
            | "secd"
            | "cscd"
            | "cotd"
            | "asecd"
            | "acscd"
            | "acotd"
            | "real"
            | "imag"
            | "angle"
            | "conj"
            | "ceil"
            | "floor"
            | "Char"
            | "round"
            | "sqrt"
            | "sign"
    )
}

fn today_seconds() -> i64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    secs.div_euclid(86_400) * 86_400
}

fn random_f64() -> f64 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64),
    );
    (h.finish() >> 11) as f64 / (1u64 << 53) as f64
}

/// `evalRpn(rpn, vars, formats, unitAware, lib)`.
pub(crate) fn eval_rpn(
    rpn: &str,
    scope: &mut Scope,
    ctx: &Ctx,
    unit_aware: bool,
    lib: Option<&HashMap<String, Value>>,
) -> Operand {
    let tokens: Vec<&str> = rpn.split(SEP).collect();
    let mut stack: Vec<Operand> = Vec::new();
    let mut o_prev: Option<Operand> = None;
    macro_rules! pop {
        () => {
            match stack.pop() {
                Some(o) => o,
                None => return err0("ERROR"),
            }
        };
    }
    macro_rules! tryv {
        ($e:expr) => {
            match $e {
                Ok(v) => v,
                Err(e) => return e,
            }
        };
    }
    let next_num = |i: usize| -> usize {
        tokens
            .get(i + 1)
            .and_then(|t| t.parse::<usize>().ok())
            .unwrap_or(0)
    };
    let mut i = 0usize;
    while i < tokens.len() {
        let tkn = tokens[i];
        let ch = tkn.chars().next().unwrap_or(' ');

        if tkn == "⁅isnan" {
            let mut depth = 1;
            let mut j = i + 1;
            while j < tokens.len() {
                if tokens[j] == "⁅isnan" {
                    depth += 1;
                } else if tokens[j] == "isnan" {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            let sub = tokens[i + 1..j.min(tokens.len())].join("\u{a0}");
            let arg = eval_rpn(&sub, scope, ctx, unit_aware, lib);
            stack.push(Operand::boolean(is_not_a_number(&arg)));
            i = j + 1;
            continue;
        }

        if ch == '®' {
            let Some(r) = parse_rational_token(&tkn['®'.len_utf8()..]) else {
                return err0("ERROR");
            };
            stack.push(Operand::rational(r));
        } else if ch == '⌾' {
            let Ok(s) = tkn['⌾'.len_utf8()..].parse::<BigInt>() else {
                return err0("ERROR");
            };
            stack.push(date_operand(s));
        } else if ch == '©' {
            let ints: Vec<Option<BigInt>> = tkn['©'.len_utf8()..]
                .split(',')
                .map(|s| s.parse().ok())
                .collect();
            if ints.len() != 4 || ints.iter().any(Option::is_none) {
                return err0("ERROR");
            }
            let ints: Vec<BigInt> = ints.into_iter().flatten().collect();
            let z = cpx(
                Rational::new(ints[0].clone(), ints[1].clone()),
                Rational::new(ints[2].clone(), ints[3].clone()),
            );
            stack.push(Operand::new(z, OUnit::zeros(), dt::COMPLEX));
        } else if ch == '¿' {
            let var_name = &tkn['¿'.len_utf8()..];
            if var_name == "undefined" {
                stack.push(Operand::new(V::Undef, OUnit::Null, 0));
            } else if var_name == "T"
                && tokens.get(i + 1) == Some(&"^")
                && stack.last().is_some_and(|o| is_matrix(o.dtype))
            {
                i += 1;
                let m = pop!();
                stack.push(matrix::transpose(&m));
            } else if var_name == "j" && scope.get("j").is_none() {
                stack.push(Operand::new(
                    cpx(Rational::zero(), Rational::one()),
                    OUnit::zeros(),
                    dt::COMPLEX,
                ));
            } else {
                let Some(cell) = scope.get(var_name) else {
                    return err("V_NAME", var_name);
                };
                let o = from_assignment(cell, unit_aware);
                if o.is_error() {
                    return o;
                }
                stack.push(o);
            }
        } else if ch == '"' {
            let s = if tkn.len() >= 2 && tkn.ends_with('"') {
                &tkn[1..tkn.len() - 1]
            } else if tkn == "\"" {
                ""
            } else {
                tkn
            };
            stack.push(Operand::string(s));
        } else if tkn.starts_with("``") {
            let inner = if tkn.len() >= 4 {
                &tkn[2..tkn.len() - 2]
            } else {
                ""
            };
            let interp = |name: &str| -> Result<String, Operand> {
                if name == "undefined" {
                    return Ok(String::new());
                }
                if name == "j" && scope.get("j").is_none() {
                    return Ok("j".into());
                }
                let cell = scope.get(name).ok_or_else(|| err("V_NAME", name))?;
                let o = from_assignment(cell, false);
                if o.is_error() {
                    return Err(o);
                }
                Ok(match &o.value {
                    V::R(r) => cruncha_units::jsnum::to_string(r.to_f64()),
                    other => matrix::js_string(other),
                })
            };
            let df = dataframe::from_tsv(dataframe::tsv_trim(inner), Some(&interp), ctx);
            stack.push(df);
        } else if ch == '`' {
            let s = if tkn.len() >= 2 && tkn.ends_with('`') {
                tkn[1..tkn.len() - 1].trim()
            } else {
                tkn.trim()
            };
            stack.push(Operand::new(V::S(s.to_string()), OUnit::Null, dt::RICHTEXT));
        } else {
            match tkn {
                "true" | "false" => stack.push(Operand::boolean(tkn == "true")),
                "pi" | "π" => stack.push(Operand::rational(Rational::pi())),
                "e" => {
                    // Hurmet pushes the *string* "e" as a rational here (a bug); "e" in an
                    // expression normally compiles to a rational literal instead.
                    stack.push(Operand::rational(Rational::e()));
                }
                "ℏ" => {
                    let e = if unit_aware {
                        [2.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0]
                    } else {
                        ALL_ZEROS
                    };
                    stack.push(Operand::new(
                        V::R(Rational::hbar()),
                        OUnit::expos(e),
                        dt::RATIONAL,
                    ));
                }
                "∠" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    if o1.dtype != dt::RATIONAL || o2.dtype != dt::RATIONAL {
                        return err0("NAN_OP");
                    }
                    let theta = o2.value.r().to_f64();
                    let r = o1.value.r();
                    let z = cpx(
                        r.multiply(&Rational::from_number(cruncha_units::jsmath::cos(theta))),
                        r.multiply(&Rational::from_number(cruncha_units::jsmath::sin(theta))),
                    );
                    stack.push(Operand::new(z, OUnit::zeros(), dt::COMPLEX));
                }
                "+" | ".+" | "-" | ".-" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    let op = if tkn == "+" || tkn == ".+" {
                        "add"
                    } else {
                        "subtract"
                    };
                    let numeric = |d: i64| d & (dt::RATIONAL | dt::DATE | dt::COMPLEX) != 0;
                    if !(numeric(o1.dtype) && numeric(o2.dtype)) {
                        return err0("NAN_OP");
                    }
                    if unit_aware && !units_are_compatible(o1.unit.get_expos(), o2.unit.get_expos())
                    {
                        return err0("UNIT_ADD");
                    }
                    let (s1, s2, _) = binary_shapes(&o1, &o2);
                    let value = tryv!(ops::binary(s1, s2, op, &o1.value, &o2.value));
                    let Some(dtype) = ops::op_dtype(s1, s2, o1.dtype, o2.dtype, tkn) else {
                        return err0("ERROR");
                    };
                    stack.push(Operand::new(value, o1.unit, dtype));
                }
                "~" => {
                    let o1 = pop!();
                    if !(o1.dtype & dt::RATIONAL != 0 || o1.dtype == dt::COMPLEX) {
                        return err0("NAN_OP");
                    }
                    let value = tryv!(ops::unary(shape_of(&o1), "negate", &o1.value));
                    stack.push(Operand::new(value, o1.unit, o1.dtype));
                }
                "×" | "·" | "*" | "∗" | "∘" | "⌧" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    if (tkn == "*" || tkn == "∗") && o1.dtype == dt::STRING {
                        // Julia's string concatenation operator. Hurmet tests o1 twice and
                        // returns the result at once, ignoring the rest of the expression.
                        let s = string_from_operand(&o1, ctx) + &string_from_operand(&o2, ctx);
                        return Operand::string(s);
                    }
                    let num1 = o1.dtype & (dt::RATIONAL | dt::COMPLEX) != 0;
                    let num2 =
                        o2.dtype & (dt::RATIONAL | dt::COMPLEX) != 0 || o2.dtype == dt::DATAFRAME;
                    if !(num1 && num2) {
                        return err0("NAN_OP");
                    }
                    let unit = if unit_aware {
                        if (o1.dtype == dt::DATAFRAME && o2.dtype == dt::RATIONAL)
                            || (o1.dtype == dt::RATIONAL && o2.dtype == dt::DATAFRAME)
                        {
                            if o1.dtype == dt::DATAFRAME {
                                o1.unit.clone()
                            } else {
                                o2.unit.clone()
                            }
                        } else {
                            expos_unit(add_expos(o1.unit.get_expos(), o2.unit.get_expos(), 1.0))
                        }
                    } else {
                        OUnit::zeros()
                    };
                    let unit = if o2.dtype == dt::DATAFRAME {
                        o2.unit.clone()
                    } else {
                        unit
                    };
                    let (s1, s2, breakdown) = binary_shapes(&o1, &o2);
                    let op = if breakdown {
                        match tkn {
                            "×" => "cross",
                            "·" => "dot",
                            "∘" => "circ",
                            _ => "multiply",
                        }
                    } else {
                        "multiply"
                    };
                    use Shape::*;
                    let dtype = if tkn == "∘"
                        || matches!(s1, Scalar | Map | Complex)
                        || matches!(s2, Scalar | Map | Complex)
                    {
                        match ops::op_dtype(s1, s2, o1.dtype, o2.dtype, op) {
                            Some(d) => d,
                            None => return err0("ERROR"),
                        }
                    } else if tkn == "·" {
                        dt::RATIONAL
                    } else if tkn == "×" {
                        dt::COLUMNVECTOR
                    } else {
                        matrix::mult_result_type(o1.dtype, o2.dtype)
                    };
                    let value = tryv!(ops::binary(s1, s2, op, &o1.value, &o2.value));
                    stack.push(Operand::new(value, unit, dtype));
                }
                "/" | "./" | "//" | "///" | "\u{2215}" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    let num = |d: i64| d & dt::RATIONAL != 0 || d == dt::COMPLEX;
                    if !(num(o1.dtype) && num(o2.dtype)) {
                        return err0("NAN_OP");
                    }
                    let unit = if unit_aware {
                        expos_unit(add_expos(o1.unit.get_expos(), o2.unit.get_expos(), -1.0))
                    } else {
                        OUnit::zeros()
                    };
                    let (s1, s2, _) = binary_shapes(&o1, &o2);
                    let value = tryv!(ops::binary(s1, s2, "divide", &o1.value, &o2.value));
                    let Some(dtype) = ops::op_dtype(s1, s2, o1.dtype, o2.dtype, "divide") else {
                        return err0("ERROR");
                    };
                    let q = Operand::new(value, unit, dtype);
                    if ops::is_div_by_zero(&q.value, shape_of(&q)) {
                        return err0("DIV");
                    }
                    stack.push(q);
                }
                "^" | ".^" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    let num = |d: i64| d & dt::RATIONAL != 0 || d == dt::COMPLEX;
                    let transpose = is_matrix(o1.dtype) && matches!(&o2.value, V::S(s) if s == "T");
                    if !((num(o1.dtype) && num(o2.dtype)) || transpose) {
                        return err0("NAN_OP");
                    }
                    let unit = if unit_aware {
                        expos_unit(scale_expos(o1.unit.get_expos(), o2.value.r().to_f64()))
                    } else {
                        OUnit::zeros()
                    };
                    let (s1, s2, _) = binary_shapes(&o1, &o2);
                    let mut value = tryv!(ops::binary(s1, s2, "power", &o1.value, &o2.value));
                    let dtype = if value.is_complex_pair() {
                        // Hurmet's Cpx.isComplex also accepts any two-element vector of
                        // rationals, so `[2, 3]^2` becomes the complex number 4 + j 9 (a bug).
                        let z = value.cpx();
                        value = cpx(z[0].clone(), z[1].clone());
                        dt::COMPLEX
                    } else {
                        match ops::op_dtype(s1, s2, o1.dtype, o2.dtype, tkn) {
                            Some(d) => d,
                            None => return err0("ERROR"),
                        }
                    };
                    stack.push(Operand::new(value, unit, dtype));
                }
                "modulo" => {
                    if unit_aware {
                        return err("UNIT_UN", "modulo");
                    }
                    let o2 = pop!();
                    let o1 = pop!();
                    if o1.dtype & dt::RATIONAL == 0 || o2.dtype & dt::RATIONAL == 0 {
                        return err0("NAN_OP");
                    }
                    let (s1, s2, _) = binary_shapes(&o1, &o2);
                    let Some(dtype) = ops::op_dtype(s1, s2, o1.dtype, o2.dtype, "modulo") else {
                        return err0("ERROR");
                    };
                    let value = tryv!(ops::binary(s1, s2, "modulo", &o1.value, &o2.value));
                    stack.push(Operand::new(value, OUnit::Bare, dtype));
                }
                "&" | "hcat" | "vcat" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    let op_name = if tkn == "vcat" { "unshift" } else { "concat" };
                    let (s1, s2, _) = binary_shapes(&o1, &o2);
                    let o3 = if o1.dtype == dt::STRING {
                        // Hurmet tests `o1.dtype === STRING` twice (it meant o2 for one).
                        Operand::string(
                            string_from_operand(&o1, ctx) + &string_from_operand(&o2, ctx),
                        )
                    } else if (o1.dtype & dt::DATAFRAME != 0
                        && is_vector(o2.dtype)
                        && tkn != "vcat")
                        || (is_vector(o1.dtype) && o2.dtype & dt::DATAFRAME != 0 && tkn != "vcat")
                        || (o1.dtype & dt::DATAFRAME != 0 && s2 == Shape::Scalar)
                        || (s1 == Shape::Scalar && o2.dtype & dt::DATAFRAME != 0)
                    {
                        let r = dataframe::append(&o1, &o2, &ctx.format_spec, unit_aware, ctx);
                        if r.is_error() {
                            return r;
                        }
                        r
                    } else if o1.dtype & dt::MAP != 0 || o2.dtype & dt::MAP != 0 {
                        // Hurmet's map.append calls `map.value.set`, which maps do not have.
                        return err("NOT_SUPPORTED", "&");
                    } else {
                        if unit_aware
                            && !units_are_compatible(o1.unit.get_expos(), o2.unit.get_expos())
                        {
                            return err0("UNIT_ADD");
                        }
                        let value = tryv!(ops::binary(s1, s2, op_name, &o1.value, &o2.value));
                        let Some(mut dtype) = ops::op_dtype(s1, s2, o1.dtype, o2.dtype, tkn) else {
                            return err0("ERROR");
                        };
                        if o1.dtype == dt::COLUMNVECTOR && s2 == Shape::Scalar {
                            dtype = o1.dtype + o2.dtype;
                        }
                        Operand::new(value, o1.unit.clone(), dtype)
                    };
                    stack.push(o3);
                }
                "√" | "∛" | "∜" => {
                    let index: i64 = match tkn {
                        "√" => 2,
                        "∛" => 3,
                        _ => 4,
                    };
                    let pow = V::R(Rational::new(1.into(), index.into()));
                    let o1 = pop!();
                    if o1.dtype & (dt::RATIONAL | dt::COMPLEX) == 0 {
                        return err0("NAN_OP");
                    }
                    let unit = if unit_aware {
                        expos_unit(scale_expos(o1.unit.get_expos(), 1.0 / index as f64))
                    } else {
                        OUnit::zeros()
                    };
                    let s1 = shape_of(&o1);
                    let value = tryv!(ops::binary(s1, Shape::Scalar, "power", &o1.value, &pow));
                    let dtype = if value.is_complex_pair() {
                        dt::COMPLEX
                    } else {
                        match ops::op_dtype(s1, Shape::Scalar, o1.dtype, dt::RATIONAL, tkn) {
                            Some(d) => d,
                            None => return err0("ERROR"),
                        }
                    };
                    let value = if dtype == dt::COMPLEX {
                        let z = value.cpx();
                        cpx(z[0].clone(), z[1].clone())
                    } else {
                        value
                    };
                    stack.push(Operand::new(value, unit, dtype));
                }
                "root" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    if o1.dtype & dt::RATIONAL == 0 || o2.dtype & dt::RATIONAL == 0 {
                        return err0("NAN_OP");
                    }
                    let unit = if unit_aware {
                        let n = cruncha_units::jsnum::bigint_to_f64(&o1.value.r().num);
                        expos_unit(scale_expos(o2.unit.get_expos(), 1.0 / n))
                    } else {
                        OUnit::zeros()
                    };
                    let pow = V::R(o1.value.r().reciprocal());
                    // Hurmet takes the shape of the index, not of the radicand.
                    let s1 = shape_of(&o1);
                    let value = tryv!(ops::binary(s1, Shape::Scalar, "power", &o2.value, &pow));
                    let Some(dtype) = ops::op_dtype(s1, Shape::Scalar, o1.dtype, dt::RATIONAL, tkn)
                    else {
                        return err0("ERROR");
                    };
                    stack.push(Operand::new(value, unit, dtype));
                }
                "." => {
                    let o2 = pop!();
                    let o1 = pop!();
                    let is_df = o1.dtype == dt::DATAFRAME;
                    let in_cols = |o: &Operand, key: &str| match &o.value {
                        V::Df(f) => f.column_map.contains_key(key),
                        _ => false,
                    };
                    let property = if is_df && tokens.len() - i > 2 && tokens[i + 2] == "." {
                        let o3 = Operand::string(tokens[i + 1].replace('"', ""));
                        let args = if in_cols(&o1, o2.value.str()) {
                            vec![o3, o2]
                        } else {
                            vec![o2, o3]
                        };
                        i += 2;
                        dataframe::range(&o1, &args, unit_aware, ctx)
                    } else if is_df
                        && tokens.len() - i > 3
                        && tokens[i + 2] == "[]"
                        && tokens[i + 3] == "1"
                        && tokens[i + 1].starts_with('"')
                    {
                        let o3 = Operand::string(tokens[i + 1].replace('"', ""));
                        let args = if in_cols(&o1, o2.value.str()) {
                            vec![o3, o2]
                        } else {
                            vec![o2, o3]
                        };
                        i += 3;
                        dataframe::range(&o1, &args, unit_aware, ctx)
                    } else {
                        property_from_dot_accessor(&o1, &o2, unit_aware, ctx)
                    };
                    if property.is_error() {
                        return property;
                    }
                    stack.push(property);
                }
                "[]" => {
                    let num_args = next_num(i);
                    i += 1;
                    if stack.len() < num_args + 1 {
                        return err0("ERROR");
                    }
                    let args: Vec<Operand> = stack.split_off(stack.len() - num_args);
                    let o1 = pop!();
                    let in_cols = |key: &str| match &o1.value {
                        V::Df(f) => f.column_map.contains_key(key),
                        _ => false,
                    };
                    let property = if o1.dtype & dt::DATAFRAME != 0 {
                        if args.len() == 1
                            && args[0].dtype == dt::STRING
                            && tokens.len() - i > 2
                            && tokens[i + 2] == "."
                        {
                            let o3 = Operand::string(tokens[i + 1].replace('"', ""));
                            let o2 = args[0].clone();
                            let new_args = if in_cols(o2.value.str()) {
                                vec![o3, o2]
                            } else {
                                vec![o2, o3]
                            };
                            i += 2;
                            dataframe::range(&o1, &new_args, unit_aware, ctx)
                        } else if args.len() == 1
                            && args[0].dtype == dt::STRING
                            && tokens.len() - i > 3
                            && tokens[i + 2] == "[]"
                            && tokens[i + 3] == "1"
                            && tokens[i + 1].starts_with('"')
                        {
                            let o3 = Operand::string(tokens[i + 1].replace('"', ""));
                            let o2 = args[0].clone();
                            let new_args = if in_cols(o2.value.str()) {
                                vec![o3, o2]
                            } else {
                                vec![o2, o3]
                            };
                            i += 3;
                            dataframe::range(&o1, &new_args, unit_aware, ctx)
                        } else {
                            dataframe::range(&o1, &args, unit_aware, ctx)
                        }
                    } else if o1.dtype & dt::MAP != 0 {
                        dataframe::map_range(&o1, &args)
                    } else if o1.dtype == dt::STRING {
                        match args.first() {
                            Some(a) => text_range(o1.value.str(), a),
                            None => err0("ERROR"),
                        }
                    } else if o1.dtype == dt::MODULE {
                        match &o1.value {
                            V::Module(m) => {
                                if num_args == 1 {
                                    match m.get(args[0].value.str()) {
                                        Some(v) => from_assignment(v, unit_aware),
                                        None => err0("ERROR"),
                                    }
                                } else {
                                    let mut t = Vec::new();
                                    for a in &args {
                                        let name = a.value.str().to_string();
                                        let v = match m.get(&name) {
                                            Some(v) => from_assignment(v, unit_aware),
                                            None => return err0("ERROR"),
                                        };
                                        t.push((name, v));
                                    }
                                    Operand::new(V::Tuple(t), OUnit::Null, dt::TUPLE)
                                }
                            }
                            _ => err0("ERROR"),
                        }
                    } else {
                        let Some(row_index) = args.first() else {
                            return err0("ERROR");
                        };
                        let zero = Operand::new(V::R(Rational::zero()), OUnit::Bare, dt::RATIONAL);
                        let col_index = if num_args == 2 {
                            Some(&args[1])
                        } else if is_vector(o1.dtype) {
                            None
                        } else {
                            Some(&zero)
                        };
                        matrix::submatrix(&o1, row_index, col_index)
                    };
                    if property.is_error() {
                        return property;
                    }
                    stack.push(property);
                }
                ":" => {
                    let end = pop!();
                    let o1 = pop!();
                    if !(o1.dtype == dt::RATIONAL || o1.dtype == dt::RANGE) {
                        return err0("NAN_OP");
                    }
                    let is_inf = matches!(&end.value, V::S(s) if s == "∞");
                    let value = if o1.dtype == dt::RATIONAL {
                        let start = o1.value.r();
                        let step = if is_inf || start.less_than_or_equal(&end.value.r()) {
                            Rational::one()
                        } else {
                            Rational::int(-1)
                        };
                        vec![V::R(start), V::R(step), end.value.clone()]
                    } else {
                        vec![o1.value.get(0), o1.value.get(2), end.value.clone()]
                    };
                    stack.push(Operand::new(V::A(value), OUnit::Null, dt::RANGE));
                }
                "normal" | "uniform" | "lognormal" => {
                    // Hurmet pops the two ends and pushes nothing (a TODO).
                    return err("NOT_SUPPORTED", tkn);
                }
                "!" | "‼" | "!!" => {
                    let o1 = pop!();
                    if o1.dtype & dt::RATIONAL == 0 {
                        return err0("NAN_OP");
                    }
                    if unit_aware && !units_are_compatible(o1.unit.get_expos(), Some(&ALL_ZEROS)) {
                        return err0("FACT");
                    }
                    let x = o1.value.r();
                    if !x.is_integer() || x.is_negative() {
                        return err0("FACT");
                    }
                    // Not in Hurmet: keep exact double factorials (a product of n/2 big integers)
                    // and Lanczos' floating point within reach.
                    let limit = if tkn == "!" { 100_000 } else { 10_000 };
                    if x.greater_than(&Rational::int(limit)) {
                        return err0("NOT_FINITE");
                    }
                    let op = if tkn == "!" {
                        "factorial"
                    } else {
                        "doubleFactorial"
                    };
                    let value = tryv!(ops::unary(shape_of(&o1), op, &o1.value));
                    stack.push(Operand::new(value, OUnit::Bare, dt::RATIONAL));
                }
                "%" => {
                    let o1 = pop!();
                    if o1.dtype & dt::RATIONAL == 0 {
                        return err0("NAN_OP");
                    }
                    // Hurmet returns the bare value here (a bug: it tests the value as if it
                    // were an error). The port pushes the percentage.
                    let value = tryv!(ops::unary(shape_of(&o1), "percent", &o1.value));
                    stack.push(Operand::new(value, o1.unit, o1.dtype));
                }
                "|" | "‖" => {
                    let o1 = pop!();
                    if !(o1.dtype & dt::RATIONAL != 0 || o1.dtype == dt::COMPLEX) {
                        return err0("NAN_OP");
                    }
                    let op = if tkn == "|" { "abs" } else { "norm" };
                    let value = tryv!(ops::unary(shape_of(&o1), op, &o1.value));
                    stack.push(Operand::new(value, o1.unit, dt::RATIONAL));
                }
                "matrix" => {
                    let num_rows = next_num(i);
                    let num_cols = tokens
                        .get(i + 2)
                        .and_then(|t| t.parse::<usize>().ok())
                        .unwrap_or(0);
                    i += 2;
                    let result = if stack.last().is_some_and(|o| o.dtype == dt::RANGE) {
                        if unit_aware {
                            return err0("UNIT_IN_EL");
                        }
                        let r = pop!();
                        matrix::operand_from_range(&r.value)
                    } else {
                        matrix::operand_from_token_stack(&mut stack, num_rows, num_cols)
                    };
                    if result.is_error() {
                        return result;
                    }
                    stack.push(result);
                }
                "tuple" => {
                    let n = next_num(i);
                    i += 1;
                    if stack.len() < n {
                        return err0("ERROR");
                    }
                    let items = stack.split_off(stack.len() - n);
                    let t = items
                        .into_iter()
                        .enumerate()
                        .map(|(k, o)| (k.to_string(), o))
                        .collect();
                    stack.push(Operand::new(V::Tuple(t), OUnit::Null, dt::TUPLE));
                }
                "startSvg" | "beamDiagram" => return err("NOT_SUPPORTED", tkn),
                t if unary_fn_names(t) => {
                    let arg = pop!();
                    if arg.dtype & (dt::RATIONAL | dt::COMPLEX) == 0 {
                        return err("UNREAL", tkn);
                    }
                    let expos = if unit_aware {
                        tryv!(functions::function_expos(tkn, &[&arg]))
                    } else {
                        Some(ALL_ZEROS)
                    };
                    let unit = if tkn == "Char" {
                        OUnit::Null
                    } else {
                        expos_unit(expos)
                    };
                    let complex_arg = arg.dtype & dt::RATIONAL == 0;
                    let f = |e: &V| functions::unary_elem(tkn, complex_arg, e);
                    let value = if arg.dtype & dt::MAP != 0 {
                        match &arg.value {
                            V::Df(fr) => {
                                let mut fr2 = (**fr).clone();
                                for col in fr2.data.iter_mut() {
                                    if matches!(col.first(), Some(V::R(_))) {
                                        for e in col.iter_mut() {
                                            *e = tryv!(f(e));
                                        }
                                    }
                                }
                                V::Df(Arc::new(fr2))
                            }
                            other => other.clone(),
                        }
                    } else if is_vector(arg.dtype) {
                        V::A(tryv!(
                            arg.value.arr().iter().map(f).collect::<Result<Vec<_>, _>>()
                        ))
                    } else if is_matrix(arg.dtype) {
                        let mut rows = Vec::new();
                        for row in arg.value.arr() {
                            rows.push(V::A(tryv!(
                                row.arr().iter().map(f).collect::<Result<Vec<_>, _>>()
                            )));
                        }
                        V::A(rows)
                    } else {
                        tryv!(f(&arg.value))
                    };
                    let arfn = ["abs", "angle", "imag", "real", "Γ", "gamma"];
                    let mut dtype = if tkn == "Char" {
                        arg.dtype - dt::RATIONAL + dt::STRING
                    } else if arg.dtype & dt::COMPLEX != 0 && arfn.contains(&tkn) {
                        arg.dtype - dt::COMPLEX + dt::RATIONAL
                    } else {
                        arg.dtype
                    };
                    // Not in Hurmet (which then fails): a real function with a complex result
                    // (√ of a negative number) or a complex function with a real result.
                    if dtype == dt::RATIONAL && matches!(value, V::C(_)) {
                        dtype = dt::COMPLEX;
                    } else if dtype == dt::COMPLEX && matches!(value, V::R(_)) {
                        dtype = dt::RATIONAL;
                    }
                    stack.push(Operand::new(value, unit, dtype));
                }
                "logn" | "atan2" | "gcd" | "rms" | "binomial" | "ones" | "zeros" | "mod"
                | "rem" => {
                    let a2 = pop!();
                    let a1 = pop!();
                    let args = vec![a1, a2];
                    if args[0].dtype & dt::RATIONAL == 0 {
                        return err0("");
                    }
                    let expos = if unit_aware {
                        functions::function_expos(tkn, &[&args[0], &args[1]]).unwrap_or(None)
                    } else {
                        Some(ALL_ZEROS)
                    };
                    let (value, dtype) = tryv!(multivar_function(Arity::Binary, tkn, &args));
                    stack.push(Operand::new(value, expos_unit(expos), dtype));
                }
                "hypot" => {
                    let num_args = next_num(i);
                    i += 1;
                    let mut acc = pop!();
                    if acc.dtype & dt::RATIONAL == 0 {
                        return err0("");
                    }
                    for _ in 0..num_args.saturating_sub(1) {
                        let next = pop!();
                        if next.dtype & dt::RATIONAL == 0 {
                            return err0("");
                        }
                        let expos = if unit_aware {
                            functions::function_expos("hypot", &[&acc, &next]).unwrap_or(None)
                        } else {
                            Some(ALL_ZEROS)
                        };
                        let pair = [acc, next];
                        let (value, dtype) =
                            tryv!(multivar_function(Arity::Binary, "hypot", &pair));
                        acc = Operand::new(value, expos_unit(expos), dtype);
                    }
                    stack.push(acc);
                }
                "today" | "savedate" => {
                    let secs = if tkn == "today" {
                        today_seconds()
                    } else {
                        let Some(saved) = scope.get("@savedate") else {
                            return err0("UNSAVED");
                        };
                        let s = format!("'{}'", saved.value.str());
                        match cruncha_lang::hurmet::seconds_from_iso_string(&s) {
                            Some(x) => x,
                            None => return err0("UNSAVED"),
                        }
                    };
                    stack.push(date_operand(BigInt::from(secs)));
                }
                "Int" => {
                    let arg = pop!();
                    if arg.dtype & dt::BOOLEAN == 0 {
                        return err("LOGIC", "Int");
                    }
                    let b = |e: &V| V::R(Rational::int(i64::from(e.truthy())));
                    let value = if is_vector(arg.dtype) {
                        V::A(arg.value.arr().iter().map(b).collect())
                    } else if is_matrix(arg.dtype) {
                        V::A(
                            arg.value
                                .arr()
                                .iter()
                                .map(|r| V::A(r.arr().iter().map(b).collect()))
                                .collect(),
                        )
                    } else {
                        b(&arg.value)
                    };
                    stack.push(Operand::new(
                        value,
                        OUnit::zeros(),
                        arg.dtype - dt::BOOLEAN + dt::RATIONAL,
                    ));
                }
                "number" => {
                    let arg = pop!();
                    if arg.dtype & dt::STRING == 0 {
                        return err0("STRING");
                    }
                    let n = |e: &V| V::R(Rational::from_str_or_zero(e.str()));
                    let value = if is_vector(arg.dtype) {
                        V::A(arg.value.arr().iter().map(n).collect())
                    } else if is_matrix(arg.dtype) {
                        V::A(
                            arg.value
                                .arr()
                                .iter()
                                .map(|r| V::A(r.arr().iter().map(n).collect()))
                                .collect(),
                        )
                    } else {
                        n(&arg.value)
                    };
                    stack.push(Operand::new(
                        value,
                        OUnit::zeros(),
                        arg.dtype - dt::STRING + dt::RATIONAL,
                    ));
                }
                "findmax" => {
                    let arg = pop!();
                    if !(is_vector(arg.dtype) && arg.dtype & dt::RATIONAL != 0) {
                        return err("NOT_VECTOR", "findmax");
                    }
                    let a = arg.value.arr();
                    let Some(first) = a.first() else {
                        return err0("ERROR");
                    };
                    let mut max = first.r();
                    let mut index = Rational::one();
                    for (k, e) in a.iter().enumerate().skip(1) {
                        if e.r().greater_than(&max) {
                            max = e.r();
                            index = Rational::int(k as i64 + 1);
                        }
                    }
                    let t = vec![
                        (
                            "max".to_string(),
                            Operand::new(V::R(max), OUnit::Bare, dt::RATIONAL),
                        ),
                        (
                            "index".to_string(),
                            Operand::new(V::R(index), OUnit::Bare, dt::RATIONAL),
                        ),
                    ];
                    stack.push(Operand::new(V::Tuple(t), OUnit::Null, dt::TUPLE));
                }
                "findfirst" => {
                    let num_args = next_num(i);
                    i += 1;
                    let mut args = vec![pop!()];
                    if num_args == 2 {
                        args.insert(0, pop!());
                    }
                    let is_string = num_args == 2 && args[1].dtype & dt::STRING != 0;
                    let ff = |needle: &str, hay: &str| -> V {
                        // String index + 1, counting a surrogate pair as one character.
                        match hay.find(needle) {
                            Some(p) => V::R(Rational::int(hay[..p].chars().count() as i64 + 1)),
                            None => V::R(Rational::int(0)),
                        }
                    };
                    let value = if is_string && is_vector(args[1].dtype) {
                        V::A(
                            args[1]
                                .value
                                .arr()
                                .iter()
                                .map(|e| ff(args[0].value.str(), e.str()))
                                .collect(),
                        )
                    } else if is_string && is_matrix(args[1].dtype) {
                        V::A(
                            args[1]
                                .value
                                .arr()
                                .iter()
                                .map(|r| {
                                    V::A(
                                        r.arr()
                                            .iter()
                                            .map(|e| ff(args[0].value.str(), e.str()))
                                            .collect(),
                                    )
                                })
                                .collect(),
                        )
                    } else if is_string {
                        ff(args[0].value.str(), args[1].value.str())
                    } else if num_args == 1 {
                        V::R(tryv!(matrix::findfirst(&V::Bool(true), &args[0])))
                    } else if is_vector(args[1].dtype) {
                        V::R(tryv!(matrix::findfirst(&args[0].value, &args[1])))
                    } else {
                        return err("ERR_FUNC", "Error. Did not understand arguments");
                    };
                    let dtype = if is_string {
                        args[1].dtype - dt::STRING + dt::RATIONAL
                    } else {
                        dt::RATIONAL
                    };
                    stack.push(Operand::new(value, OUnit::zeros(), dtype));
                }
                "roundn" | "string" => {
                    let spec = pop!();
                    let num = pop!();
                    if num.dtype & dt::RATIONAL == 0 || spec.dtype & dt::STRING == 0 {
                        return err0("");
                    }
                    let s = spec.value.str();
                    let valid = s.char_indices().any(|(k, c)| {
                        (c == 'f' || c == 'r')
                            && s[k + 1..].starts_with(|d: char| d.is_ascii_digit())
                    });
                    if !valid {
                        return err0("");
                    }
                    let fixed = s.starts_with('f');
                    let n: i64 = s.get(1..).and_then(|x| x.parse().ok()).unwrap_or(0);
                    let f = |e: &V| -> V {
                        let r = e.r();
                        let st = if fixed {
                            r.to_fixed(n)
                        } else {
                            r.to_string_significant(n)
                        };
                        if tkn == "string" {
                            V::S(st)
                        } else {
                            V::R(Rational::from_str_or_zero(&st))
                        }
                    };
                    if num.dtype & dt::MAP != 0 {
                        // Hurmet looks up `Functions.binary[funcName][tkn]` here (a TypeError).
                        return err("NOT_SUPPORTED", tkn);
                    }
                    let value = if is_vector(num.dtype) {
                        V::A(num.value.arr().iter().map(f).collect())
                    } else if is_matrix(num.dtype) {
                        V::A(
                            num.value
                                .arr()
                                .iter()
                                .map(|r| V::A(r.arr().iter().map(f).collect()))
                                .collect(),
                        )
                    } else {
                        f(&num.value)
                    };
                    let (unit, dtype) = if tkn == "string" {
                        (OUnit::Null, num.dtype - dt::RATIONAL + dt::STRING)
                    } else {
                        (num.unit.clone(), num.dtype)
                    };
                    let mut o = Operand::new(value, unit, dtype);
                    o.name = num.name.clone();
                    stack.push(o);
                }
                "dataframe" | "max" | "min" | "sum" | "product" | "range" | "mean" | "median"
                | "variance" | "stddev" | "accumulate" => {
                    let num_args = next_num(i);
                    i += 1;
                    if stack.len() < num_args {
                        return err0("ERROR");
                    }
                    let args = stack.split_off(stack.len() - num_args);
                    if tkn != "dataframe" {
                        if args.iter().any(|a| a.dtype & dt::RATIONAL == 0) {
                            return err("NANARG", tkn);
                        }
                    } else {
                        let df = dataframe::from_vectors(&args, &ctx.format_spec, ctx);
                        if df.is_error() {
                            return df;
                        }
                        stack.push(df);
                        i += 1;
                        continue;
                    }
                    if args.is_empty() {
                        return err0("ERROR");
                    }
                    let expos = if unit_aware {
                        let refs: Vec<&Operand> = args.iter().collect();
                        functions::function_expos(tkn, &refs).unwrap_or(None)
                    } else {
                        Some(ALL_ZEROS)
                    };
                    let (value, dtype) = tryv!(multivar_function(Arity::Reduce, tkn, &args));
                    stack.push(Operand::new(value, expos_unit(expos), dtype));
                }
                "spreadsheetSum" => return err("NOT_SUPPORTED", "spreadsheet"),
                "rand" => {
                    let num_args = next_num(i);
                    i += 1;
                    match num_args {
                        0 => stack.push(Operand::new(
                            V::R(Rational::from_number(random_f64())),
                            OUnit::Bare,
                            dt::RATIONAL,
                        )),
                        1 => {
                            let n = pop!().value.r().to_f64();
                            if n.fract() != 0.0 || !(0.0..=1e6).contains(&n) {
                                return err("INT_ARG", "rand");
                            }
                            let v = (0..n as usize)
                                .map(|_| V::R(Rational::from_number(random_f64())))
                                .collect();
                            stack.push(Operand::new(
                                V::A(v),
                                OUnit::Bare,
                                dt::RATIONAL + dt::COLUMNVECTOR,
                            ));
                        }
                        2 => {
                            let n = pop!().value.r().to_f64();
                            let m = pop!().value.r().to_f64();
                            if n.fract() != 0.0
                                || m.fract() != 0.0
                                || !(0.0..=1e3).contains(&n)
                                || !(0.0..=1e3).contains(&m)
                            {
                                return err("INT_ARG", "rand");
                            }
                            let v = (0..m as usize)
                                .map(|_| {
                                    V::A(
                                        (0..n as usize)
                                            .map(|_| V::R(Rational::from_number(random_f64())))
                                            .collect(),
                                    )
                                })
                                .collect();
                            stack.push(Operand::new(
                                V::A(v),
                                OUnit::Bare,
                                dt::RATIONAL + dt::MATRIX,
                            ));
                        }
                        _ => return err("BAD_ARGS", "rand"),
                    }
                }
                "isnan" => {
                    let o = pop!();
                    stack.push(Operand::boolean(is_not_a_number(&o)));
                }
                "length" => {
                    let arg = pop!();
                    let len = if is_vector(arg.dtype) {
                        arg.value.len()
                    } else if arg.dtype & dt::MATRIX != 0 {
                        arg.value.len() * arg.value.get(0).len()
                    } else if arg.dtype == dt::STRING {
                        arg.value.str().chars().count()
                    } else if arg.dtype & dt::MAP != 0 {
                        // Hurmet calls `arg.keys()`, which operands do not have.
                        return err("NOT_SUPPORTED", "length");
                    } else {
                        0
                    };
                    stack.push(Operand::rational(Rational::int(len as i64)));
                }
                "count" => {
                    let pattern = pop!();
                    let s = pop!();
                    if pattern.dtype != dt::STRING || s.dtype != dt::STRING {
                        return err0("COUNT");
                    }
                    let (p, h) = (pattern.value.str(), s.value.str());
                    let n = if p.is_empty() {
                        cruncha_lang::hurmet::utf16_len(h) as i64 - 1
                    } else {
                        h.matches(p).count() as i64
                    };
                    stack.push(Operand::rational(Rational::int(n)));
                }
                "lerp" => {
                    let a3 = pop!();
                    let a2 = pop!();
                    let a1 = pop!();
                    let r = functions::lerp(&[a1, a2, a3], unit_aware);
                    if r.is_error() {
                        return r;
                    }
                    stack.push(r);
                }
                "matrix2table" => {
                    let num_args = next_num(i);
                    i += 1;
                    let row_names = if num_args == 3 {
                        pop!().value.arr().to_vec()
                    } else {
                        Vec::new()
                    };
                    let col_names = pop!().value.arr().to_vec();
                    let m = pop!();
                    let r = dataframe::matrix2table(&m, &col_names, &row_names);
                    if r.is_error() {
                        return r;
                    }
                    stack.push(r);
                }
                "transpose" => {
                    let o = pop!();
                    stack.push(matrix::transpose(&o));
                }
                // Hurmet calls the non-existent Matrix.trace.
                "trace" => return err("NOT_SUPPORTED", "trace"),
                "fetch" => return err0("FETCH"),
                "function" => {
                    let name = tokens.get(i + 1).copied().unwrap_or("");
                    let num_args = tokens
                        .get(i + 2)
                        .and_then(|t| t.parse::<usize>().ok())
                        .unwrap_or(0);
                    i += 2;
                    if stack.len() < num_args {
                        return err0("ERROR");
                    }
                    let args = stack.split_off(stack.len() - num_args);
                    let o = if tokens.get(i + 1) == Some(&".") {
                        let module = pop!();
                        i += 1;
                        match &module.value {
                            V::Module(m) => match m.get(name) {
                                Some(v) => match &v.value {
                                    V::Udf(u) => {
                                        eval_custom_function(u, args, ctx, unit_aware, Some(m))
                                    }
                                    _ if v.dtype == dt::ERROR => return from_assignment(v, false),
                                    _ => return err("F_NAME", name),
                                },
                                None => return err("F_NAME", name),
                            },
                            _ => return err("F_NAME", name),
                        }
                    } else if let Some(udf) =
                        lib.and_then(|l| l.get(name)).and_then(|v| match &v.value {
                            V::Udf(u) => Some(u.clone()),
                            _ => None,
                        })
                    {
                        eval_custom_function(&udf, args, ctx, unit_aware, lib)
                    } else if let Some(udf) =
                        scope.get(name).and_then(|v| match (&v.value, v.dtype) {
                            (V::Udf(u), d) if d == dt::MODULE => Some(u.clone()),
                            _ => None,
                        })
                    {
                        eval_custom_function(&udf, args, ctx, unit_aware, None)
                    } else {
                        return err("BAD_FUN_NM", name);
                    };
                    if o.is_error() {
                        return o;
                    }
                    stack.push(o);
                }
                "=" | "==" | "⩵" | "<" | ">" | "<=" | "≤" | ">=" | "≥" | "≠" | "!=" | "∈"
                | "in" | "∉" | "!in" | "∋" | "∌" | "⊂" | "⊄" | "⊃" | "⊅" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    if unit_aware
                        && !(o1.dtype & dt::STRING != 0
                            || o2.dtype & dt::STRING != 0
                            || o1.dtype == dt::NULL
                            || o2.dtype == dt::NULL)
                        && !units_are_compatible(o1.unit.get_expos(), o2.unit.get_expos())
                    {
                        return err0("UNIT_COMP");
                    }
                    let prev_value = if o1.dtype == dt::BOOLEANFROMCOMPARISON {
                        o_prev.as_ref().map(|p| p.value.clone())
                    } else {
                        None
                    };
                    let set_cmp = ["in", "!in", "∈", "∉", "∋", "∌", "⊂", "⊄", "⊃", "⊅"];
                    let (value, mut dtype) = if set_cmp.contains(&tkn) {
                        (
                            tryv!(ops::compare(tkn, &o1.value, &o2.value, prev_value.as_ref())),
                            o1.dtype + dt::BOOLEANFROMCOMPARISON,
                        )
                    } else {
                        let (s1, s2, _) = binary_shapes(&o1, &o2);
                        let v = tryv!(ops::relate(
                            s1,
                            s2,
                            tkn,
                            &o1.value,
                            &o2.value,
                            prev_value.as_ref()
                        ));
                        let Some(d) = ops::op_dtype(s1, s2, o1.dtype, o2.dtype, tkn) else {
                            return err0("ERROR");
                        };
                        (v, d + dt::BOOLEANFROMCOMPARISON)
                    };
                    for flag in [dt::RATIONAL, dt::COMPLEX, dt::STRING] {
                        if dtype & flag != 0 {
                            dtype -= flag;
                        }
                    }
                    o_prev = Some(o2);
                    stack.push(Operand::new(value, OUnit::Null, dtype));
                }
                "and" | "&&" | "or" | "||" | "∧" | "∨" | "⊻" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    if o1.dtype & dt::BOOLEAN == 0 || o2.dtype & dt::BOOLEAN == 0 {
                        return err("LOGIC", tkn);
                    }
                    let op = match tkn {
                        "and" | "&&" | "∧" => "and",
                        "or" | "||" | "∨" => "or",
                        _ => "xor",
                    };
                    let (s1, s2, _) = binary_shapes(&o1, &o2);
                    let value = tryv!(ops::binary(s1, s2, op, &o1.value, &o2.value));
                    let Some(dtype) = ops::op_dtype(s1, s2, o1.dtype, o2.dtype, tkn) else {
                        return err0("ERROR");
                    };
                    stack.push(Operand::new(value, OUnit::Null, dtype));
                }
                "not" | "¬" => {
                    let o1 = pop!();
                    if o1.dtype & dt::BOOLEAN == 0 {
                        return err("LOGIC", tkn);
                    }
                    let value = tryv!(ops::unary(shape_of(&o1), "not", &o1.value));
                    // Hurmet sets BOOLEAN here even for a vector.
                    stack.push(Operand::new(value, OUnit::Null, dt::BOOLEAN));
                }
                "cases" => {
                    let num_args = next_num(i);
                    i += 1;
                    if stack.len() < num_args {
                        return err0("ERROR");
                    }
                    let conditions = stack.split_off(stack.len() - num_args);
                    for (j, c) in conditions.iter().enumerate() {
                        if c.dtype & dt::BOOLEAN == 0 {
                            return err("LOGIC", "if");
                        }
                        if ops::condition(shape_of(c), &c.value) {
                            let local = tokens
                                .get(i + j + 1)
                                .copied()
                                .unwrap_or("")
                                .replace('§', "\u{a0}");
                            let o = eval_rpn(&local, scope, ctx, unit_aware, lib);
                            if o.is_error() {
                                return o;
                            }
                            stack.push(o);
                            break;
                        }
                    }
                    i += num_args;
                }
                "applyUnit" => {
                    let unit_name = tokens.get(i + 1).copied().unwrap_or("");
                    if !unit_aware {
                        return err("UNIT_AWARE", unit_name);
                    }
                    let o1 = pop!();
                    if o1.dtype & dt::RATIONAL == 0 {
                        return err0("QUANT_NUM");
                    }
                    i += 1;
                    let u = tryv!(ctx.unit(unit_name));
                    let o = if is_matrix(o1.dtype) {
                        Operand::new(
                            matrix::convert_to_base_units(&o1.value, o1.dtype, &u.gauge, &u.factor),
                            expos_unit(add_expos(o1.unit.get_expos(), Some(&u.expos), 1.0)),
                            o1.dtype,
                        )
                    } else if o1.dtype & dt::MAP != 0 {
                        let value = match &o1.value {
                            V::Df(f) => {
                                let mut fr = (**f).clone();
                                fr.data =
                                    dataframe::map_convert(&fr.data, &u.gauge, &u.factor, true);
                                V::Df(Arc::new(fr))
                            }
                            other => other.clone(),
                        };
                        Operand::new(value, OUnit::from_unit(&u), o1.dtype)
                    } else {
                        Operand::new(
                            V::R(o1.value.r().add(&u.gauge).multiply(&u.factor)),
                            expos_unit(add_expos(o1.unit.get_expos(), Some(&u.expos), 1.0)),
                            o1.dtype,
                        )
                    };
                    stack.push(o);
                }
                "rem%" => {
                    let o2 = pop!();
                    let o1 = pop!();
                    if o1.dtype & dt::RATIONAL == 0 || o2.dtype & dt::RATIONAL == 0 {
                        return err0("NAN_OP");
                    }
                    let (s1, s2, _) = binary_shapes(&o1, &o2);
                    let value = tryv!(ops::binary(s1, s2, "rem", &o1.value, &o2.value));
                    let Some(dtype) = ops::op_dtype(s1, s2, o1.dtype, o2.dtype, tkn) else {
                        return err0("ERROR");
                    };
                    stack.push(Operand::new(value, OUnit::zeros(), dtype));
                }
                "⎾⏋" | "⎿⏌" => {
                    let o1 = pop!();
                    if o1.dtype & dt::RATIONAL == 0 {
                        return err0("NAN_OP");
                    }
                    if unit_aware && !units_are_compatible(o1.unit.get_expos(), Some(&ALL_ZEROS)) {
                        return err0("");
                    }
                    let op = if tkn == "⎾⏋" { "ceil" } else { "floor" };
                    let value = tryv!(ops::unary(shape_of(&o1), op, &o1.value));
                    stack.push(Operand::new(value, o1.unit, o1.dtype));
                }
                "()" => {
                    let a2 = pop!();
                    let a1 = pop!();
                    if unit_aware
                        && (!units_are_compatible(a1.unit.get_expos(), Some(&ALL_ZEROS))
                            || !units_are_compatible(a2.unit.get_expos(), Some(&ALL_ZEROS)))
                    {
                        return err0("BINOM");
                    }
                    let (value, dtype) =
                        tryv!(multivar_function(Arity::Binary, "binomial", &[a1, a2]));
                    stack.push(Operand::new(value, OUnit::zeros(), dtype));
                }
                "→" => {
                    let body = pop!().value.str().replace('§', "\u{a0}");
                    let parameter = pop!().value.str().to_string();
                    let udf = Udf {
                        name: String::new(),
                        parameters: vec![Parameter {
                            name: parameter,
                            default: None,
                        }],
                        statements: vec![FnStatement {
                            rpn: body,
                            stype: "return".into(),
                            ..Default::default()
                        }],
                    };
                    stack.push(Operand::new(V::Udf(Arc::new(udf)), OUnit::Null, dt::MODULE));
                }
                "∑" => {
                    let body = pop!().value.str().replace('§', "\u{a0}");
                    let end = pop!().value.r();
                    let mut index = pop!().value.r();
                    let parameter = pop!().value.str().to_string();
                    let mut sum = Rational::zero();
                    let saved = scope.locals.remove(&parameter);
                    let mut steps = 0u64;
                    while index.less_than_or_equal(&end) {
                        steps += 1;
                        if steps > MAX_STEPS {
                            return err_msg("Error. Too many terms in the sum.");
                        }
                        scope.locals.insert(
                            parameter.clone(),
                            Value {
                                value: V::R(index.clone()),
                                unit: OUnit::Bare,
                                dtype: dt::RATIONAL,
                                ..Default::default()
                            },
                        );
                        let local = eval_rpn(&body, scope, ctx, false, None);
                        if local.is_error() {
                            scope.locals.remove(&parameter);
                            return local;
                        }
                        sum = sum.add(&local.value.r());
                        index = index.add(&Rational::one());
                    }
                    // Hurmet deletes the variable, even a document variable of the same name;
                    // the port only removes its own binding.
                    scope.locals.remove(&parameter);
                    if let Some(s) = saved {
                        scope.locals.insert(parameter, s);
                    }
                    stack.push(Operand::new(V::R(sum), OUnit::Bare, dt::RATIONAL));
                }
                "throw" => {
                    let o = pop!();
                    return Operand::new(o.value, OUnit::Null, dt::ERROR);
                }
                "\\blue" | "\\gray" | "\\green" | "\\orange" | "\\pink" | "\\purple" | "\\red" => {
                    let mut c = pop!();
                    if c.dtype == dt::STRING {
                        c.unit = OUnit::Color(tkn[1..].to_string());
                    }
                    stack.push(c);
                }
                _ => {
                    // Hurmet ignores a token it does not know.
                }
            }
        }
        i += 1;
    }
    let o = pop!();
    if !stack.is_empty() {
        return err0("ERROR");
    }
    o
}

/// `textRange(str, index)`: characters of a string by 1-based index or range.
fn text_range(s: &str, index: &Operand) -> Operand {
    if index.dtype != dt::RATIONAL && index.dtype != dt::RANGE {
        return err0("STR_INDEX");
    }
    let chars: Vec<char> = s.chars().collect();
    let at = |p: i64| -> String {
        let n = chars.len() as i64;
        let p = if p < 0 { n + p } else { p };
        if (0..n).contains(&p) {
            chars[p as usize].to_string()
        } else {
            "undefined".into()
        }
    };
    let value = if index.dtype == dt::RATIONAL {
        let pos = index.value.r().to_f64() as i64 - 1;
        at(pos)
    } else {
        let r = index.value.arr();
        let start = r.first().map_or(0.0, |v| v.r().to_f64()) as i64;
        let step = r.get(1).map_or(1.0, |v| v.r().to_f64()) as i64;
        let end = match r.get(2) {
            Some(V::S(x)) if x == "∞" => cruncha_lang::hurmet::utf16_len(s) as i64,
            Some(v) => v.r().to_f64() as i64,
            None => 0,
        };
        if step == 1 {
            let a = (start - 1).clamp(0, chars.len() as i64) as usize;
            let b = end.clamp(0, chars.len() as i64) as usize;
            if a < b {
                chars[a..b].iter().collect()
            } else {
                String::new()
            }
        } else {
            let mut out = String::new();
            let step = if step <= 0 { 1 } else { step };
            let mut k = start - 1;
            while k < end && k - start < 10_000 {
                out.push_str(&at(k));
                k += step;
            }
            out
        }
    };
    Operand::string(value)
}

/// `propertyFromDotAccessor(parent, index, unitAware)`.
pub(crate) fn property_from_dot_accessor(
    parent: &Operand,
    index: &Operand,
    unit_aware: bool,
    ctx: &Ctx,
) -> Operand {
    if parent.dtype & dt::MAP != 0 {
        dataframe::map_range(parent, std::slice::from_ref(index))
    } else if parent.dtype & dt::DATAFRAME != 0 {
        dataframe::range(parent, std::slice::from_ref(index), unit_aware, ctx)
    } else if parent.dtype == dt::MODULE {
        match &parent.value {
            V::Module(m) => match m.get(index.value.str()) {
                Some(v) => from_assignment(v, unit_aware),
                None => err("BAD_KEY", index.value.str()),
            },
            _ => err("NO_PROP", parent.name.as_deref().unwrap_or("undefined")),
        }
    } else {
        // Hurmet also has a branch for strings and `dt.ARRAY`, which does not exist (so the
        // test is `parent.dtype === STRING && index is a number`); indexing a string uses [].
        if parent.dtype == dt::STRING && index.dtype == dt::RATIONAL {
            let k = index.value.r().to_f64() as i64;
            let units: Vec<u16> = parent.value.str().encode_utf16().collect();
            let a = (k - 1).clamp(0, units.len() as i64) as usize;
            let b = k.clamp(0, units.len() as i64) as usize;
            let s = if a < b {
                String::from_utf16_lossy(&units[a..b])
            } else {
                String::new()
            };
            return Operand::new(V::S(s), parent.unit.clone(), parent.dtype);
        }
        err("NO_PROP", parent.name.as_deref().unwrap_or("undefined"))
    }
}

fn element_from_iterable(
    iterable: &Operand,
    index: &Rational,
    step: &Rational,
) -> (Operand, Rational) {
    let mut next = index.increment();
    let i = index.to_f64() as i64;
    let (value, dtype) = if iterable.dtype == dt::RANGE {
        next = index.add(step);
        (V::R(index.clone()), dt::RATIONAL)
    } else if iterable.dtype == dt::STRING {
        // Hurmet indexes UTF-16 code units, with special handling for one surrogate prefix.
        let units: Vec<u16> = iterable.value.str().encode_utf16().collect();
        let get = |a: i64, b: i64| -> String {
            let n = units.len() as i64;
            let (a, b) = (a.clamp(0, n) as usize, b.clamp(0, n) as usize);
            if a < b {
                String::from_utf16_lossy(&units[a..b])
            } else {
                String::new()
            }
        };
        if i >= 1 && units.get((i - 1) as usize) == Some(&0xD835) {
            next = index.add(&Rational::one());
            (V::S(format!("\u{FFFD}{}", get(i + 1, i + 2))), dt::STRING)
        } else {
            (V::S(get(i, i + 1)), dt::STRING)
        }
    } else {
        let v = if i >= 0 {
            iterable.value.get(i as usize)
        } else {
            V::Undef
        };
        let d = if iterable.dtype & dt::STRING != 0 {
            dt::STRING
        } else if iterable.dtype & dt::ROWVECTOR != 0 {
            iterable.dtype - dt::ROWVECTOR
        } else if iterable.dtype & dt::COLUMNVECTOR != 0 {
            iterable.dtype - dt::COLUMNVECTOR
        } else {
            iterable.dtype - dt::MATRIX
        };
        (v, d)
    };
    (Operand::new(value, iterable.unit.clone(), dtype), next)
}

#[derive(Clone)]
struct Control {
    kind: &'static str,
    condition: bool,
    end_of_block: usize,
    start: usize,
    rpn: String,
    dummy: String,
    index: Rational,
    step: Rational,
    end_index: Rational,
    next_index: Rational,
    iterable: Option<Operand>,
}

impl Control {
    fn new(kind: &'static str, condition: bool, end_of_block: usize) -> Self {
        Control {
            kind,
            condition,
            end_of_block,
            start: 0,
            rpn: String::new(),
            dummy: String::new(),
            index: Rational::zero(),
            step: Rational::one(),
            end_index: Rational::zero(),
            next_index: Rational::zero(),
            iterable: None,
        }
    }
}

/// `evalCustomFunction(udf, args, formats, isUnitAware, lib)`.
pub(crate) fn eval_custom_function(
    udf: &Udf,
    args: Vec<Operand>,
    ctx: &Ctx,
    unit_aware: bool,
    lib: Option<&HashMap<String, Value>>,
) -> Operand {
    if args.len() > udf.parameters.len() {
        return err("NUMARGS", &udf.name);
    }
    let depth = ctx.depth.get();
    if depth >= MAX_CALL_DEPTH {
        return err_msg("Error. Too many nested function calls.");
    }
    ctx.depth.set(depth + 1);
    let r = run_function(udf, args, ctx, unit_aware, lib);
    ctx.depth.set(depth);
    r
}

fn run_function(
    udf: &Udf,
    args: Vec<Operand>,
    ctx: &Ctx,
    unit_aware: bool,
    lib: Option<&HashMap<String, Value>>,
) -> Operand {
    let mut scope = Scope::empty();
    let nargs = args.len();
    for (p, a) in udf.parameters.iter().zip(args) {
        let mut v = Value::from_operand(&a);
        if v.name.is_none() {
            v.name = Some(p.name.clone());
        }
        scope.locals.insert(p.name.clone(), v);
    }
    for p in &udf.parameters[nargs..] {
        let v = p.default.clone().unwrap_or(Value {
            name: Some(p.name.clone()),
            value: V::Null,
            ..Default::default()
        });
        scope.locals.insert(p.name.clone(), v);
    }
    let n = udf.statements.len();
    let mut control: Vec<Control> = vec![Control::new("if", true, n.saturating_sub(1))];
    let mut steps: u64 = 0;
    let mut i = 0usize;
    while i < n {
        steps += 1;
        if steps > MAX_STEPS {
            return err_msg(format!(
                "Error in function {}. Too many steps (an endless loop?).",
                udf.name
            ));
        }
        let st = &udf.statements[i];
        let Some(level) = control.len().checked_sub(1) else {
            break;
        };
        let cond = control[level].condition;
        match st.stype.as_str() {
            "statement" => {
                if cond {
                    let result = eval_rpn(&st.rpn, &mut scope, ctx, unit_aware, lib);
                    if result.is_error() {
                        return result;
                    }
                    if !st.name.is_empty() {
                        assign_in_function(&mut scope, &st.name, result);
                    }
                }
            }
            "if" => {
                if cond {
                    let result = eval_rpn(&st.rpn, &mut scope, ctx, unit_aware, lib);
                    if result.is_error() {
                        return result;
                    }
                    let val = ops::condition(shape_of(&result), &result.value);
                    control.push(Control::new("if", val, st.end_of_block));
                } else {
                    i = st.end_of_block;
                }
            }
            "elseif" => {
                if control[level].kind == "if" && cond {
                    i = control[level].end_of_block;
                    control.pop();
                } else {
                    let result = eval_rpn(&st.rpn, &mut scope, ctx, unit_aware, lib);
                    if result.is_error() {
                        return result;
                    }
                    let val = ops::condition(shape_of(&result), &result.value);
                    if let Some(c) = control.last_mut() {
                        c.condition = val;
                    }
                }
            }
            "else" => {
                if control[level].kind == "if" && cond {
                    i = control[level].end_of_block;
                    control.pop();
                } else {
                    control[level].condition = true;
                }
            }
            "while" => {
                if cond {
                    let result = eval_rpn(&st.rpn, &mut scope, ctx, unit_aware, lib);
                    if result.is_error() {
                        return result;
                    }
                    if ops::condition(shape_of(&result), &result.value) {
                        let mut c = Control::new("while", true, st.end_of_block);
                        c.start = i;
                        c.rpn = st.rpn.clone();
                        control.push(c);
                    } else {
                        i = st.end_of_block;
                    }
                } else {
                    i = st.end_of_block;
                }
            }
            "for" => {
                if cond {
                    let mut toks: Vec<&str> = st.rpn.split(SEP).collect();
                    if toks.is_empty() {
                        return err0("ERROR");
                    }
                    let first = toks.remove(0);
                    let dummy = first.strip_prefix('¿').unwrap_or(first).to_string();
                    let iterable = eval_rpn(&toks.join("\u{a0}"), &mut scope, ctx, unit_aware, lib);
                    if iterable.is_error() {
                        return iterable;
                    }
                    let is_range = iterable.dtype & dt::RANGE != 0;
                    let mut c = Control::new("for", true, st.end_of_block);
                    c.start = i;
                    c.index = if is_range {
                        iterable.value.get(0).r()
                    } else {
                        Rational::zero()
                    };
                    c.step = if is_range {
                        iterable.value.get(1).r()
                    } else {
                        Rational::one()
                    };
                    c.end_index = if is_range {
                        iterable.value.get(2).r()
                    } else {
                        Rational::int(iterable.value.len() as i64 - 1)
                    };
                    let (o, next) = element_from_iterable(&iterable, &c.index, &c.step);
                    c.next_index = next;
                    c.dummy = dummy.clone();
                    c.iterable = Some(iterable);
                    control.push(c);
                    let mut v = Value::from_operand(&o);
                    v.name = Some(dummy.clone());
                    scope.locals.insert(dummy, v);
                } else {
                    i = st.end_of_block;
                }
            }
            "break" => {
                if cond {
                    let mut j = control.len() - 1;
                    while j > 0 {
                        if control[j].kind == "while" || control[j].kind == "for" {
                            i = control[j].end_of_block;
                            control.pop();
                            break;
                        } else {
                            control.pop();
                        }
                        j -= 1;
                    }
                }
            }
            "end" => {
                let top = control[level].clone();
                if top.kind == "if" && i >= top.end_of_block {
                    control.pop();
                } else if top.kind == "if" && top.condition {
                    if i < top.end_of_block {
                        i = top.end_of_block;
                    }
                    control.pop();
                } else if top.kind == "while" {
                    let result = eval_rpn(&top.rpn, &mut scope, ctx, unit_aware, lib);
                    if result.is_error() {
                        return result;
                    }
                    if result.value.truthy() {
                        i = top.start;
                    } else {
                        control.pop();
                    }
                } else if top.kind == "for" {
                    let c = &mut control[level];
                    c.index = c.next_index.clone();
                    let proceed = if c.step.is_positive() {
                        c.index.less_than_or_equal(&c.end_index)
                    } else {
                        c.index.greater_than_or_equal(&c.end_index)
                    };
                    if proceed {
                        let iterable = c.iterable.clone().unwrap_or_default();
                        let (o, next) = element_from_iterable(&iterable, &c.index, &c.step);
                        c.next_index = next;
                        let dummy = c.dummy.clone();
                        i = c.start;
                        let mut v = Value::from_operand(&o);
                        v.name = Some(dummy.clone());
                        scope.locals.insert(dummy, v);
                    } else {
                        control.pop();
                    }
                }
            }
            "return" => {
                if cond {
                    if !st.rpn.is_empty() {
                        return eval_rpn(&st.rpn, &mut scope, ctx, unit_aware, lib);
                    }
                    return Operand::rational(Rational::zero());
                }
            }
            "print" => {
                if cond && !st.rpn.is_empty() {
                    // Hurmet logs to the console; the port evaluates and discards.
                    let result = eval_rpn(&st.rpn, &mut scope, ctx, unit_aware, lib);
                    if result.is_error() {
                        return result;
                    }
                }
            }
            "throw" if cond => {
                if !st.rpn.is_empty() {
                    let result = eval_rpn(&st.rpn, &mut scope, ctx, unit_aware, lib);
                    return Operand::new(result.value, OUnit::Null, dt::ERROR);
                }
                return Operand::new(V::S(String::new()), OUnit::Null, dt::ERROR);
            }
            _ => {}
        }
        i += 1;
    }
    // Hurmet returns undefined when a function ends without `return`.
    err0("ERROR")
}

/// A statement inside a function: Hurmet runs `conditionResult` with the `!` selector and
/// inserts the statement as the variable.
fn assign_in_function(scope: &mut Scope, names: &[String], result: Operand) {
    let mut value = result.value.clone();
    let mut dtype = result.dtype;
    if dtype == dt::COMPLEX {
        let z = value.cpx();
        if z[1].is_zero() {
            value = V::R(z[0].clone());
            dtype = dt::RATIONAL;
        }
    }
    if dtype & dt::RATIONAL != 0 {
        value = normalize_value(&value, dtype);
    }
    if names.len() == 1 {
        let v = Value {
            name: Some(names[0].clone()),
            value,
            unit: result.unit,
            dtype,
            ..Default::default()
        };
        scope.locals.insert(names[0].clone(), v);
    } else if let V::Tuple(t) = &value {
        for (k, (_, o)) in t.iter().enumerate() {
            if let Some(n) = names.get(k) {
                let mut v = Value::from_operand(o);
                v.name = Some(n.clone());
                scope.locals.insert(n.clone(), v);
            }
        }
    } else if is_vector(dtype) {
        for (k, e) in value.arr().iter().enumerate() {
            if let Some(n) = names.get(k) {
                let d = dtype - (dtype & (dt::ROWVECTOR | dt::COLUMNVECTOR));
                let v = Value {
                    name: Some(n.clone()),
                    value: e.clone(),
                    unit: result.unit.clone(),
                    dtype: d,
                    ..Default::default()
                };
                scope.locals.insert(n.clone(), v);
            }
        }
    }
}

/// `Rnl.normalize` on every element of a numeric value.
pub(crate) fn normalize_value(v: &V, dtype: i64) -> V {
    match v {
        V::R(r) => V::R(r.normalize()),
        V::A(a) if is_matrix(dtype) => V::A(a.iter().map(|e| normalize_value(e, dtype)).collect()),
        V::Df(f) => {
            let mut fr = (**f).clone();
            for col in fr.data.iter_mut() {
                if matches!(col.first(), Some(V::R(_))) {
                    for e in col.iter_mut() {
                        *e = V::R(e.r().normalize());
                    }
                }
            }
            V::Df(Arc::new(fr))
        }
        other => other.clone(),
    }
}
