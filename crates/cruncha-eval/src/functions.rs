// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Built-in functions (Hurmet's `functions.js`).

use crate::complex::{self, Exponent, Z};
use crate::error::{err, err0};
use crate::matrix;
use crate::value::{Operand, V, cpx, dt, is_matrix, is_vector};
use cruncha_units::rational::lanczos;
use cruncha_units::{ALL_ZEROS, Dims, Rational, jsmath, units_are_compatible};

/// `Functions.functionExpos`: the unit exponents of a function's result.
pub(crate) fn function_expos(name: &str, args: &[&Operand]) -> Result<Option<Dims>, Operand> {
    let expos = if args.len() == 1 {
        args[0].unit.get_expos().copied()
    } else {
        None
    };
    // Every argument must be dimensionless (ADR-0022 item 13: Hurmet checked only a single
    // argument, so `gcd(12 'm', 18 's')` passed).
    let dimensionless = || {
        for a in args {
            if !units_are_compatible(a.unit.get_expos(), Some(&ALL_ZEROS)) {
                return Err(err("UNIT_IN", name));
            }
        }
        Ok(Some(ALL_ZEROS))
    };
    // `sum(A, 1)` and `sum(A, 2)`: the second argument is an axis, not a term.
    if name == "sum" && args.len() == 2 && crate::value::is_matrix(args[0].dtype) {
        return Ok(args[0].unit.get_expos().copied());
    }
    match name {
        // |A| of an n × n matrix is its determinant, in the unit to the nth power.
        "abs" if args.len() == 1 && args[0].dtype & dt::MATRIX != 0 => {
            let n = args[0].value.arr().len() as f64;
            Ok(expos.map(|e| e.map(|x| x * n)))
        }
        "abs" | "round" | "roundn" | "sign" | "trace" | "fetch" => Ok(expos),
        "cos" | "sin" | "tan" | "sec" | "csc" | "cot" | "acos" | "arccos" | "asin" | "arcsin"
        | "atan" | "arctan" | "asec" | "arcsec" | "acsc" | "arccsc" | "acot" | "arccot"
        | "cosd" | "sind" | "tand" | "secd" | "cscd" | "cotd" | "acosd" | "asind" | "atand"
        | "asecd" | "acscd" | "acotd" | "gud" => dimensionless(),
        "exp" | "log" | "ln" | "log10" | "log2" | "logn" | "cosh" | "sinh" | "tanh" | "sech"
        | "csch" | "coth" | "acosh" | "asinh" | "atanh" | "asech" | "acsch" | "acoth"
        | "binomial" | "gamma" | "Γ" | "lgamma" | "lfact" | "factorial" => dimensionless(),
        "sqrt" => Ok(expos.map(|e| e.map(|x| x / 2.0))),
        "gcd" | "mht" | "ones" | "zeros" => dimensionless(),
        // ADR-0022: Char wasn't listed, so unit-aware it was "not found".
        "Char" => dimensionless(),
        "atan2" | "hypot" | "rms" | "mod" | "rem" | "ceil" | "floor" | "sum" | "mean" | "median" | "min"
        | "max" | "range" | "stddev" | "variance" | "accumulate" => {
            let x = args.first().and_then(|a| a.unit.get_expos().copied());
            for a in &args[1..] {
                if a.unit.get_expos().copied() != x {
                    return Err(err("UNIT_ARG", name));
                }
            }
            Ok(match name {
                "atan2" => Some(ALL_ZEROS),
                // The variance is in the square of the arguments' unit (ADR-0022).
                "variance" => x.map(|e| e.map(|v| v * 2.0)),
                _ => x,
            })
        }
        "real" | "imag" | "angle" | "conj" => Ok(Some(ALL_ZEROS)),
        // The product's unit is the product of the units: the exponents add up, n times
        // for the n elements of one vector or matrix (ADR-0022 item 12; Hurmet summed the
        // exponents and threw the sum away).
        "product" => {
            fn elements(v: &crate::value::V) -> usize {
                match v {
                    crate::value::V::A(a) => a.iter().map(elements).sum(),
                    _ => 1,
                }
            }
            let mut total = ALL_ZEROS;
            for a in args {
                let Some(e) = a.unit.get_expos().copied() else { return Ok(None) };
                let n = if args.len() == 1 { elements(&a.value) } else { 1 } as f64;
                for (t, x) in total.iter_mut().zip(e) {
                    *t += x * n;
                }
            }
            Ok(Some(total))
        }
        _ => Err(err("F_NAME", name)),
    }
}

pub(crate) fn gamma(x: &Rational) -> Result<Rational, Operand> {
    if x.is_zero() {
        Err(err0("Γ0"))
    } else if x.is_positive() && x.is_integer() && x.less_than(&Rational::int(101)) {
        Ok(x.subtract(&Rational::one()).factorial())
    } else if x.is_negative() && x.is_integer() {
        Err(err0("ΓPOLE"))
    } else if x.less_than(&Rational::new(1.into(), 2.into())) {
        let pi = std::f64::consts::PI;
        let g = gamma(&Rational::one().subtract(x))?.to_f64();
        Ok(Rational::from_number(pi / jsmath::sin(pi * x.to_f64()) * g))
    } else {
        Ok(lanczos(x))
    }
}

fn lgamma(r: &Rational) -> Result<Rational, Operand> {
    if r.is_zero() {
        return Err(err0("Γ0"));
    }
    if r.is_negative() {
        return Err(err0("LOGΓ"));
    }
    if r.are_equal(&Rational::one()) || r.are_equal(&Rational::two()) {
        return Ok(Rational::zero());
    }
    if r.less_than_or_equal(&Rational::int(14)) {
        return Ok(Rational::from_number(jsmath::log(gamma(r)?.to_f64())));
    }
    let x = r.to_f64();
    let pi = std::f64::consts::PI;
    let y = (x - 0.5) * jsmath::log(x) - x + 0.5 * jsmath::log(2.0 * pi) + 1.0 / (12.0 * x)
        - 1.0 / (360.0 * jsmath::pow(x, 3.0))
        + 1.0 / (1260.0 * jsmath::pow(x, 5.0))
        - 1.0 / (1680.0 * jsmath::pow(x, 7.0))
        + 5.0 / (540.0 * jsmath::pow(x, 9.0));
    Ok(Rational::from_number(y))
}

fn binomial(n: &Rational, k: &Rational) -> Result<Rational, Operand> {
    if n.are_equal(k) {
        return Ok(Rational::one());
    }
    if n.is_zero() || k.is_negative() || n.less_than(k) {
        return Ok(Rational::zero());
    }
    if n.is_integer() && k.is_integer() && n.is_positive() && k.is_positive() {
        Ok(n.factorial()
            .divide(&k.factorial().multiply(&n.subtract(k).factorial())))
    } else if n.is_integer() && k.is_integer() && k.is_positive() {
        let sign = Rational::int(-1).power(k).unwrap_or_else(Rational::one);
        let multiset = binomial(&n.negate().add(&k.decrement()), k)?;
        Ok(sign.multiply(&multiset))
    } else {
        Ok(gamma(&n.increment())?
            .divide(&gamma(&k.increment())?.multiply(&gamma(&n.subtract(k).increment())?)))
    }
}

fn pi_over_180() -> Rational {
    Rational::pi().divide(&Rational::int(180))
}

fn real_fn(name: &str, x: &Rational) -> Result<Rational, Operand> {
    let n = || x.to_f64();
    let f = Rational::from_number;
    Ok(match name {
        "abs" => x.abs(),
        "angle" | "real" | "imag" | "conj" => return Err(err("NA_REAL", name)),
        "cos" => x.cos(),
        "sin" => x.sin(),
        "tan" => x.tan().ok_or_else(|| err("TAN90", "π/2"))?,
        "cosh" => x.cosh(),
        "sinh" => x.sinh(),
        "tanh" => x.tanh(),
        "acos" => {
            if x.abs().greater_than(&Rational::one()) {
                return Err(err("ATRIG", "acos"));
            }
            f(jsmath::acos(n()))
        }
        "asin" => {
            if x.abs().greater_than(&Rational::one()) {
                return Err(err("ATRIG", "asin"));
            }
            f(jsmath::asin(n()))
        }
        "atan" => f(jsmath::atan(n())),
        "sec" => f(1.0 / jsmath::cos(n())),
        "csc" => f(1.0 / jsmath::sin(n())),
        "cot" => {
            if x.is_zero() {
                return Err(err("COT", "cotangent"));
            }
            f(1.0 / jsmath::tan(n()))
        }
        // ADR-0022: Hurmet inverted this range check and then called a missing `Math.atn`.
        "asec" | "arcsec" | "acsc" | "arccsc" => {
            if x.abs().less_than(&Rational::one()) {
                let label = if name.ends_with("sec") { "asec" } else { "acsc" };
                return Err(err("ASEC", label));
            }
            let inv = 1.0 / n();
            f(if name.ends_with("sec") { jsmath::acos(inv) } else { jsmath::asin(inv) })
        }
        "acot" | "arccot" => {
            if x.is_zero() {
                Rational::pi().divide(&Rational::two())
            } else {
                f(jsmath::atan(1.0 / n()))
            }
        }
        "exp" => x.exp(),
        "log" | "ln" => {
            if x.is_zero() {
                return Err(err0("LOG_ZERO"));
            }
            f(jsmath::log(n()))
        }
        "log10" => {
            if x.is_zero() {
                return Err(err0("LOG_ZERO"));
            }
            f(jsmath::log10(n()))
        }
        "log2" => {
            if x.is_zero() {
                return Err(err0("LOG_ZERO"));
            }
            f(jsmath::log2(n()))
        }
        "sech" => {
            let v = n();
            f(2.0 / (jsmath::exp(v) + jsmath::exp(-v)))
        }
        "csch" => {
            let v = n();
            f(2.0 / (jsmath::exp(v) - jsmath::exp(-v)))
        }
        "coth" => {
            let v = n();
            f((jsmath::exp(v) + jsmath::exp(-v)) / (jsmath::exp(v) - jsmath::exp(-v)))
        }
        "acosh" => {
            let v = n();
            f(jsmath::log(v + (v - 1.0).sqrt() * (v + 1.0).sqrt()))
        }
        "asinh" => {
            let v = n();
            f(jsmath::log(v + (jsmath::pow(v, 2.0) + 1.0).sqrt()))
        }
        "atanh" => {
            let v = n();
            f((jsmath::log(1.0 + v) - jsmath::log(1.0 - v)) / 2.0)
        }
        "asech" => {
            if x.is_zero() {
                return Err(err0("DIV"));
            }
            let v = n();
            f(jsmath::log(((-v * v + 1.0).sqrt() + 1.0) / v))
        }
        // Hurmet's table spells this "ascsh" (a TypeError there); asinh(1/x) (ADR-0022).
        "acsch" => {
            if x.is_zero() {
                return Err(err0("DIV"));
            }
            real_fn("asinh", &x.reciprocal())?
        }
        "acoth" => {
            if x.is_zero() {
                return Err(err0("DIV"));
            }
            let v = n();
            f((jsmath::log(1.0 + 1.0 / v) - jsmath::log(1.0 - 1.0 / v)) / 2.0)
        }
        "ceil" => x.ceil(),
        "floor" => x.floor(),
        "gamma" | "Γ" => gamma(x)?,
        "lgamma" => {
            if x.is_negative() || x.is_zero() {
                return Err(err0("LOGΓ"));
            }
            lgamma(x)?
        }
        "lfact" => {
            if x.is_negative() || !x.is_integer() {
                return Err(err0("FACT"));
            }
            lgamma(&x.add(&Rational::one()))?
        }
        "factorial" => x.factorial(),
        "sign" => x.sign(),
        "cosd" => {
            if x.are_equal(&Rational::int(90)) {
                Rational::zero()
            } else if x.are_equal(&Rational::int(60)) {
                Rational::new(1.into(), 2.into())
            } else {
                real_fn("cos", &x.multiply(&pi_over_180()))?
            }
        }
        "sind" => {
            if x.are_equal(&Rational::int(30)) {
                Rational::new(1.into(), 2.into())
            } else {
                real_fn("sin", &x.multiply(&pi_over_180()))?
            }
        }
        "tand" => {
            if x.are_equal(&Rational::int(45)) {
                Rational::one()
            } else if x.are_equal(&Rational::int(90)) {
                return Err(err("TAN90", "90°"));
            } else {
                real_fn("tan", &x.multiply(&pi_over_180()))?
            }
        }
        "cotd" => real_fn("cot", &x.multiply(&pi_over_180()))?,
        "cscd" => real_fn("csc", &x.multiply(&pi_over_180()))?,
        "secd" => real_fn("sec", &x.multiply(&pi_over_180()))?,
        "acosd" | "asind" | "atand" | "acotd" | "acscd" | "asecd" => {
            real_fn(&name[..name.len() - 1], x)?.divide(&pi_over_180())
        }
        "round" => Rational::from_str_or_zero(&x.to_fixed(0)),
        _ => return Err(err0("ERROR")),
    })
}

fn zv(z: Z) -> V {
    let [a, b] = z;
    cpx(a, b)
}

fn complex_fn(name: &str, z: &Z) -> Result<V, Operand> {
    let half_pi = Rational::pi().divide(&Rational::two());
    let two = [Rational::two(), Rational::zero()];
    Ok(match name {
        "abs" => V::R(complex::abs(z)),
        "angle" => V::R(complex::angle(z)?),
        "real" => V::R(z[0].clone()),
        "imag" => V::R(z[1].clone()),
        "conj" => zv(complex::conjugate(z)),
        "cos" => zv(complex::cos(z)),
        "sin" => zv(complex::sin(z)),
        "asin" => zv(complex::asin(z)?),
        "atan" => zv(complex::atan(z)?),
        "acos" => zv(complex::subtract(
            &[half_pi, Rational::zero()],
            &complex::asin(z)?,
        )),
        "tan" => zv(complex::divide(&complex::sin(z), &complex::cos(z))?),
        "cot" => zv(complex::divide(&complex::cos(z), &complex::sin(z))?),
        "sec" => zv(complex::inverse(&complex::cos(z))?),
        "csc" => zv(complex::inverse(&complex::sin(z))?),
        "asec" => {
            let inv = complex::inverse(z)?;
            zv(complex::subtract(
                &[half_pi, Rational::zero()],
                &complex::asin(&inv)?,
            ))
        }
        "acot" => zv(complex::atan(&complex::inverse(z)?)?),
        "acsc" => zv(complex::asin(&complex::inverse(z)?)?),
        "exp" => zv(complex::exp(z)),
        "log" | "ln" => zv(complex::log(z)?),
        // Hurmet passes the complex number to Rnl.toNumber here, which reads its real part's
        // numerator and denominator as numbers (a bug); the port uses the real part.
        "log10" => V::R(Rational::from_number(jsmath::log10(z[0].to_f64()))),
        "log2" => V::R(Rational::from_number(jsmath::log2(z[0].to_f64()))),
        "cosh" => zv(complex::divide(
            &complex::add(&complex::exp(z), &complex::exp(&complex::negate(z))),
            &two,
        )?),
        "sinh" => zv(complex::divide(
            &complex::subtract(&complex::exp(z), &complex::exp(&complex::negate(z))),
            &two,
        )?),
        "tanh" => {
            let ez = complex::exp(z);
            let emz = complex::exp(&complex::negate(z));
            zv(complex::divide(
                &complex::subtract(&ez, &emz),
                &complex::add(&ez, &emz),
            )?)
        }
        "sech" => zv(complex::divide(
            &two,
            &complex::add(&complex::exp(z), &complex::exp(&complex::negate(z))),
        )?),
        "csch" => zv(complex::divide(
            &two,
            &complex::subtract(&complex::exp(z), &complex::exp(&complex::negate(z))),
        )?),
        "coth" => {
            let ez = complex::exp(z);
            let emz = complex::exp(&complex::negate(z));
            zv(complex::divide(
                &complex::add(&ez, &emz),
                &complex::subtract(&ez, &emz),
            )?)
        }
        "acosh" => zv(complex::acosh(z)?),
        "asinh" => zv(complex::asinh(z)?),
        "atanh" => zv(complex::atanh(z)?),
        "asech" => zv(complex::acosh(&complex::inverse(z)?)?),
        "acsch" => zv(complex::asinh(&complex::inverse(z)?)?),
        "acoth" => zv(complex::atanh(&complex::inverse(z)?)?),
        "ceil" | "floor" => return Err(err("NA_COMPL_OP", "ceil")),
        // Hurmet calls the non-existent Cpx.gamma here.
        "gamma" | "Γ" => return Err(err("NOT_SUPPORTED", "Γ")),
        "lgamma" | "factorial" | "round" => return Err(err("NA_COMPL_OP", name)),
        "sign" => {
            if z[1].is_zero() && z[0].is_positive() {
                V::R(Rational::one())
            } else if z[1].is_zero() && z[0].is_negative() {
                V::R(Rational::int(-1))
            } else {
                zv(complex::divide(z, &[complex::abs(z), Rational::zero()])?)
            }
        }
        "sqrt" | "Char" | "cosd" | "sind" | "tand" | "acosd" | "asind" | "atand" | "secd"
        | "cscd" | "cotd" | "asecd" | "acscd" | "acotd" | "lfact" => {
            return Err(err0("ERROR"));
        }
        _ => return Err(err0("ERROR")),
    })
}

/// `Functions.unary[shape][name](x)` for one real or complex element.
pub(crate) fn unary_elem(name: &str, complex_arg: bool, x: &V) -> Result<V, Operand> {
    if complex_arg {
        return complex_fn(name, &x.cpx());
    }
    match name {
        "Char" => {
            let n = x.r().to_f64();
            if n.fract() != 0.0 || !(0.0..=1_114_111.0).contains(&n) {
                // String.fromCodePoint throws a RangeError here.
                return Err(err0("ERROR"));
            }
            Ok(V::S(
                char::from_u32(n as u32)
                    .map(String::from)
                    .unwrap_or_default(),
            ))
        }
        "sqrt" => {
            let half = Rational::new(1.into(), 2.into());
            if x.is_complex_pair() || x.r().is_negative() {
                let z = complex::power(&[x.r(), Rational::zero()], &Exponent::Real(half))?;
                Ok(zv(z))
            } else {
                Ok(V::R(x.r().power(&half).ok_or_else(|| err0("BAD_ROOT"))?))
            }
        }
        _ => Ok(V::R(real_fn(name, &x.r())?)),
    }
}

/// `Functions.binary[name](list)` for two scalar arguments.
fn binary_fn(name: &str, list: &[V]) -> Result<V, Operand> {
    let a = list.first().cloned().unwrap_or_default();
    let b = list.get(1).cloned().unwrap_or_default();
    let (x, y) = (a.r(), b.r());
    Ok(match name {
        "logn" => V::R(Rational::from_number(
            jsmath::log(y.to_f64()) / jsmath::log(x.to_f64()),
        )),
        "atan2" => V::R(Rational::from_number(jsmath::atan2(y.to_f64(), x.to_f64()))),
        // ADR-0022: Rnl.hypot, with absolute values (Hurmet's copy here had none).
        "hypot" => V::R(x.hypot(&y)),
        "gcd" => V::R(x.gcd(&y).ok_or_else(|| err("INT_ARG", "gcd"))?),
        "binomial" => V::R(binomial(&x, &y)?),
        "mod" | "rem" if y.is_zero() => return Err(err0("DIV")),
        "mod" => V::R(x.modulo(&y)),
        "rem" => V::R(x.rem(&y)),
        _ => return Err(err0("ERROR")),
    })
}

/// Σ (x − about)², exactly.
fn squares(rs: &[Rational], about: &Rational) -> Rational {
    rs.iter().fold(Rational::zero(), |acc, x| {
        let d = x.subtract(about);
        acc.add(&d.multiply(&d))
    })
}

/// `Functions.reduce[name](list)`.
fn reduce_fn(name: &str, list: &[V]) -> Result<V, Operand> {
    let rs: Vec<Rational> = list.iter().map(V::r).collect();
    if rs.is_empty() {
        // Array.prototype.reduce on an empty array throws a TypeError.
        return Err(err0("ERROR"));
    }
    let fold = |f: &dyn Fn(&Rational, &Rational) -> Rational| {
        let mut acc = rs[0].clone();
        for r in &rs[1..] {
            acc = f(&acc, r);
        }
        acc
    };
    let max = || fold(&|a, b| a.max(b));
    let min = || fold(&|a, b| a.min(b));
    let sum = || fold(&|a, b| a.add(b));
    Ok(V::R(match name {
        "max" => max(),
        "min" => min(),
        "sum" => sum(),
        "product" => fold(&|a, b| a.multiply(b)),
        "mean" => sum().divide(&Rational::int(rs.len() as i64)),
        // ADR-0022: Hurmet's median was the midrange, (max + min) / 2.
        "median" => {
            let mut sorted = rs.clone();
            sorted.sort_by(|a, b| {
                if a.less_than(b) {
                    std::cmp::Ordering::Less
                } else if b.less_than(a) {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            let m = sorted.len() / 2;
            if sorted.len() % 2 == 1 {
                sorted[m].clone()
            } else {
                sorted[m - 1].add(&sorted[m]).divide(&Rational::two())
            }
        }
        // ADR-0022: the root mean square (Hurmet's rms was hypot).
        "rms" => squares(&rs, &Rational::zero()).divide(&Rational::int(rs.len() as i64)).sqrt(),
        "range" => max().subtract(&min()),
        // Sample statistics, n − 1 (ADR-0022; Hurmet called a missing Rnl.pow).
        "variance" | "stddev" => {
            if rs.len() < 2 {
                return Err(err("STAT_N", name));
            }
            let mean = sum().divide(&Rational::int(rs.len() as i64));
            let var = squares(&rs, &mean).divide(&Rational::int(rs.len() as i64 - 1));
            if name == "variance" { var } else { var.sqrt() }
        }
        "accumulate" => {
            let mut v = Vec::with_capacity(rs.len());
            let mut acc = rs[0].clone();
            v.push(V::R(acc.clone()));
            for r in &rs[1..] {
                acc = acc.add(r);
                v.push(V::R(acc.clone()));
            }
            return Ok(V::A(v));
        }
        _ => return Err(err0("ERROR")),
    }))
}

/// `Functions.lerp`: linear interpolation in a table.
pub(crate) fn lerp(args: &[Operand; 3], unit_aware: bool) -> Operand {
    for a in args {
        if a.dtype & dt::RATIONAL == 0 {
            return err0("");
        }
    }
    let _ = unit_aware; // Hurmet compares `args[i].expos`, which operands do not have.
    let v0 = args[0].value.arr();
    let v1 = args[1].value.arr();
    let x = args[2].value.r();
    for i in 0..v0.len().saturating_sub(1) {
        if v0[i].r().less_than_or_equal(&x) && x.less_than_or_equal(&v0[i + 1].r()) {
            let slope = v1
                .get(i + 1)
                .map_or_else(Rational::zero, V::r)
                .subtract(&v1[i].r())
                .divide(&v0[i + 1].r().subtract(&v0[i].r()));
            let value = v1[i].r().add(&slope.multiply(&x.subtract(&v0[i].r())));
            return Operand::rational(value);
        }
    }
    // Hurmet returns undefined when x is outside the table.
    err0("ERROR")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arity {
    Binary,
    Reduce,
}

fn call(arity: Arity, name: &str, list: &[V]) -> Result<V, Operand> {
    match arity {
        Arity::Binary => match name {
            "ones" | "zeros" => {
                let m = list.first().map_or(0.0, |v| v.r().to_f64()).max(0.0) as usize;
                let n = list.get(1).map_or(0.0, |v| v.r().to_f64()).max(0.0) as usize;
                if m.saturating_mul(n) > matrix::MAX_ELEMENTS {
                    return Err(err0("ERROR"));
                }
                let x = if name == "ones" {
                    Rational::one()
                } else {
                    Rational::zero()
                };
                Ok(matrix::filled(m, n, x).value)
            }
            _ => binary_fn(name, list),
        },
        Arity::Reduce => reduce_fn(name, list),
    }
}

/// Hurmet's `multivarFunction(arity, functionName, args)`: (value, dtype).
pub(crate) fn multivar_function(
    arity: Arity,
    name: &str,
    args: &[Operand],
) -> Result<(V, i64), Operand> {
    if args.len() == 1 {
        // A reducer over a matrix reduces every element (ADR-0022 item 11). Hurmet tested
        // `args.dtype & MATRIX` on the argument list (always 0), so it folded the rows as if
        // they were numbers and returned 0.
        let flatten = arity == Arity::Reduce && name != "accumulate" && args[0].dtype & dt::MATRIX != 0;
        let list: Vec<V> = match &args[0].value {
            V::A(rows) if flatten => rows.iter().flat_map(|r| r.arr().to_vec()).collect(),
            V::A(a) => a.clone(),
            other => vec![other.clone()],
        };
        let value = call(arity, name, &list)?;
        let mut dtype = args[0].dtype;
        if arity == Arity::Reduce && name != "accumulate" {
            if dtype & dt::MATRIX != 0 {
                dtype -= dt::MATRIX;
            }
            if dtype & dt::ROWVECTOR != 0 {
                dtype -= dt::ROWVECTOR;
            }
            if dtype & dt::COLUMNVECTOR != 0 {
                dtype -= dt::COLUMNVECTOR;
            }
        }
        return Ok((value, dtype));
    }
    if name == "sum" && args.len() == 2 && is_matrix(args[0].dtype) && args[1].dtype == dt::RATIONAL
    {
        let sum_row = |row: &V| -> V {
            let mut acc: Option<Rational> = None;
            for e in row.arr() {
                acc = Some(acc.map_or(e.r(), |a| a.add(&e.r())));
            }
            acc.map_or(V::Undef, V::R)
        };
        if args[1].value.r().are_equal(&Rational::two()) {
            let v = args[0].value.arr().iter().map(sum_row).collect();
            return Ok((V::A(v), dt::COLUMNVECTOR + dt::RATIONAL));
        } else if args[1].value.r().are_equal(&Rational::one()) {
            let t = matrix::transpose(&args[0]);
            let v = t.value.arr().iter().map(sum_row).collect();
            return Ok((V::A(v), dt::ROWVECTOR + dt::RATIONAL));
        } else {
            return Err(err0("BAD_SUM"));
        }
    }
    let mut dtype = args[0].dtype;
    let mut found: Option<(usize, bool)> = None;
    for (i, a) in args.iter().enumerate() {
        if is_vector(a.dtype) {
            found = Some((i, true));
            dtype = a.dtype;
            break;
        } else if is_matrix(a.dtype) {
            found = Some((i, false));
            dtype = a.dtype;
            break;
        }
    }
    let list: Vec<V> = args.iter().map(|a| a.value.clone()).collect();
    match found {
        None => {
            if name == "zeros" || name == "ones" {
                let m = list[0].r().to_f64().max(0.0) as usize;
                let n = list.get(1).map_or(0.0, |v| v.r().to_f64()).max(0.0) as usize;
                if m.saturating_mul(n) > matrix::MAX_ELEMENTS {
                    return Err(err0("ERROR"));
                }
                let x = if name == "ones" {
                    Rational::one()
                } else {
                    Rational::zero()
                };
                let o = matrix::filled(m, n, x);
                return Ok((o.value, o.dtype));
            }
            Ok((call(arity, name, &list)?, args[0].dtype))
        }
        Some((i_arg, true)) => {
            let mut result = Vec::new();
            for e in list[i_arg].arr() {
                let mut l = list.clone();
                l[i_arg] = e.clone();
                result.push(call(arity, name, &l)?);
            }
            Ok((V::A(result), dtype))
        }
        Some((i_arg, false)) => {
            let mut result = Vec::new();
            for row in list[i_arg].arr() {
                let mut r = Vec::new();
                for e in row.arr() {
                    let mut l = list.clone();
                    l[i_arg] = e.clone();
                    r.push(call(arity, name, &l)?);
                }
                result.push(V::A(r));
            }
            Ok((V::A(result), dtype))
        }
    }
}
