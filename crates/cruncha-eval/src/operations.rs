// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Operators overloaded on operand shapes (Hurmet's `operations.js`) and comparisons
//! (`compare.js`).
//!
//! Hurmet dispatches `Operators.binary[shape1][shape2][op](x, y)` through nested object
//! literals. Combinations that are missing there make JavaScript throw a `TypeError`; the port
//! returns Hurmet's generic "does not understand" error instead.

use crate::complex::{self, Exponent};
use crate::error::{err, err0};
use crate::matrix;
use crate::value::{Frame, Operand, V, cpx, dt, is_matrix, is_vector};
use cruncha_units::Rational;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    Scalar,
    Complex,
    Vector,
    RowVector,
    ColumnVector,
    Matrix,
    DataFrame,
    Map,
    Other,
}

pub(crate) fn shape_of(o: &Operand) -> Shape {
    if o.dtype == dt::COMPLEX {
        Shape::Complex
    } else if o.dtype < 128 {
        Shape::Scalar
    } else if is_vector(o.dtype) {
        Shape::Vector
    } else if o.dtype & dt::MATRIX != 0 {
        Shape::Matrix
    } else if o.dtype == dt::DATAFRAME {
        Shape::DataFrame
    } else if o.dtype & dt::MAP != 0 {
        Shape::Map
    } else {
        Shape::Other
    }
}

/// Hurmet's `binaryShapesOf`. When both operands are matrices, vectors are refined into row
/// and column vectors. (Hurmet tests `o1.dtyp & MAP`, a typo, so a map as the first operand
/// never counts.)
pub(crate) fn binary_shapes(o1: &Operand, o2: &Operand) -> (Shape, Shape, bool) {
    let mut s1 = shape_of(o1);
    let mut s2 = shape_of(o2);
    let mut breakdown = false;
    if is_matrix(o1.dtype) && (is_matrix(o2.dtype) || o2.dtype & dt::MAP != 0) {
        breakdown = true;
        if s1 == Shape::Vector {
            s1 = if o1.dtype & dt::ROWVECTOR != 0 {
                Shape::RowVector
            } else {
                Shape::ColumnVector
            };
        }
        if s2 == Shape::Vector {
            s2 = if o2.dtype & dt::ROWVECTOR != 0 {
                Shape::RowVector
            } else {
                Shape::ColumnVector
            };
        }
    }
    (s1, s2, breakdown)
}

fn unknown() -> Operand {
    err0("ERROR")
}

/// `Operators.dtype[shape1][shape2](t0, t1, tkn)`.
pub(crate) fn op_dtype(s1: Shape, s2: Shape, t0: i64, t1: i64, tkn: &str) -> Option<i64> {
    use Shape::*;
    Some(match (s1, s2) {
        (Scalar, Scalar) => {
            if t0 == dt::DATE || t1 == dt::DATE {
                if t0 == t1 { dt::RATIONAL } else { dt::DATE }
            } else if tkn == "&" || tkn == "hcat" {
                t0 + dt::ROWVECTOR
            } else if tkn == "vcat" {
                t0 + dt::COLUMNVECTOR
            } else {
                t0
            }
        }
        (Scalar, Complex | Vector | Matrix | DataFrame | Map) => t1,
        (Complex, Scalar | Complex) => t0,
        (Vector, Scalar) => t0,
        (Vector, Map) => t1 + (t0 & dt::ROWVECTOR) + (t0 & dt::COLUMNVECTOR),
        (RowVector, RowVector) => {
            if tkn == "vcat" {
                t0 - dt::ROWVECTOR + dt::MATRIX
            } else {
                t0
            }
        }
        (RowVector, ColumnVector) => t0,
        (RowVector, Matrix) => {
            if tkn == "multiply" {
                t0
            } else {
                t1
            }
        }
        (ColumnVector, RowVector) => match tkn {
            "dot" => dt::RATIONAL,
            "cross" => t0,
            _ => t0 - dt::COLUMNVECTOR + dt::MATRIX,
        },
        (ColumnVector, ColumnVector) => {
            if tkn == "&" || tkn == "hcat" {
                t0 - dt::COLUMNVECTOR + dt::MATRIX
            } else {
                t0
            }
        }
        (ColumnVector, Matrix) => t1,
        (Matrix, Scalar | RowVector | Matrix) => t0,
        (Matrix, ColumnVector) => {
            if tkn == "*" || tkn == "⌧" {
                t1
            } else {
                t0
            }
        }
        (Matrix, Map) => 0,
        (DataFrame, Scalar) => t0,
        (Map, Scalar | Vector | Map) => t0,
        (Map, Matrix) => 0,
        _ => return None,
    })
}

/// JavaScript `x && y`, `x || y`, `x !== y` on Hurmet values.
fn logic(op: &str, x: &V, y: &V) -> V {
    match op {
        "and" => {
            if x.truthy() {
                y.clone()
            } else {
                x.clone()
            }
        }
        "or" => {
            if x.truthy() {
                x.clone()
            } else {
                y.clone()
            }
        }
        _ => V::Bool(!strict_equals(x, y)),
    }
}

/// JavaScript `===` for the primitive values Hurmet compares this way.
pub(crate) fn strict_equals(x: &V, y: &V) -> bool {
    match (x, y) {
        (V::Bool(a), V::Bool(b)) => a == b,
        (V::S(a), V::S(b)) => a == b,
        (V::Num(a), V::Num(b)) => a == b,
        (V::Undef, V::Undef) | (V::Null, V::Null) => true,
        _ => false,
    }
}

/// An element-wise operation with Hurmet's `Rnl` functions (or a logic operator).
fn rop(op: &str, a: &V, b: &V) -> Result<V, Operand> {
    let (x, y) = (a.r(), b.r());
    Ok(V::R(match op {
        "add" => x.add(&y),
        "subtract" => x.subtract(&y),
        "multiply" | "circ" => x.multiply(&y),
        "divide" => x.divide(&y),
        "power" => x.power(&y).ok_or_else(|| err0("BAD_ROOT"))?,
        "modulo" | "mod" | "rem" if y.is_zero() => return Err(err0("DIV")),
        "modulo" | "mod" => x.modulo(&y),
        "rem" => x.rem(&y),
        "hypot" => x.hypot(&y),
        "and" | "or" | "xor" => return Ok(logic(op, a, b)),
        _ => return Err(unknown()),
    }))
}

fn arr_map(v: &V, f: impl Fn(&V) -> Result<V, Operand>) -> Result<V, Operand> {
    Ok(V::A(v.arr().iter().map(f).collect::<Result<Vec<_>, _>>()?))
}

fn mat_map(m: &V, f: impl Fn(usize, usize, &V) -> Result<V, Operand>) -> Result<V, Operand> {
    let rows = m.arr();
    let mut out = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let mut r = Vec::with_capacity(row.len());
        for (j, e) in row.arr().iter().enumerate() {
            r.push(f(i, j, e)?);
        }
        out.push(V::A(r));
    }
    Ok(V::A(out))
}

fn zip_map(x: &V, y: &V, op: &str) -> Result<V, Operand> {
    let (a, b) = (x.arr(), y.arr());
    Ok(V::A(
        a.iter()
            .enumerate()
            .map(|(i, e)| rop(op, e, b.get(i).unwrap_or(&V::Undef)))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn dot_product(a: &V, b: &V) -> V {
    let (a, b) = (a.arr(), b.arr());
    let mut acc: Option<Rational> = None;
    for (i, e) in a.iter().enumerate() {
        let p = e.r().multiply(&b.get(i).map_or_else(Rational::zero, V::r));
        acc = Some(match acc {
            None => p,
            Some(s) => s.add(&p),
        });
    }
    acc.map_or(V::Undef, V::R)
}

fn cross(x: &V, y: &V) -> Result<V, Operand> {
    let (a, b) = (x.arr(), y.arr());
    if a.len() != 3 || b.len() != 3 {
        return Err(err0("CROSS"));
    }
    let (a, b): (Vec<Rational>, Vec<Rational>) =
        (a.iter().map(V::r).collect(), b.iter().map(V::r).collect());
    Ok(V::A(vec![
        V::R(a[1].multiply(&b[2]).subtract(&a[2].multiply(&b[1]))),
        V::R(a[2].multiply(&b[0]).subtract(&a[0].multiply(&b[2]))),
        V::R(a[0].multiply(&b[1]).subtract(&a[1].multiply(&b[0]))),
    ]))
}

fn mis() -> Operand {
    err0("MIS_ELNUM")
}

fn one_each(x: &V, y: &V, op: &str) -> Result<V, Operand> {
    if x.len() == 1 && y.len() == 1 {
        Ok(V::A(vec![rop(op, &x.get(0), &y.get(0))?]))
    } else {
        Err(mis())
    }
}

fn same_len(x: &V, y: &V, op: &str) -> Result<V, Operand> {
    if x.len() != y.len() {
        return Err(mis());
    }
    zip_map(x, y, op)
}

fn map_frame(
    frame: &Arc<Frame>,
    numeric: bool,
    f: impl Fn(usize, &V) -> Result<V, Operand>,
) -> Result<V, Operand> {
    let mut fr = (**frame).clone();
    for col in fr.data.iter_mut() {
        let applies = match col.first() {
            Some(V::R(_)) => numeric,
            Some(V::Bool(_)) => !numeric,
            _ => false,
        };
        if applies {
            for (i, e) in col.iter_mut().enumerate() {
                *e = f(i, e)?;
            }
        }
    }
    Ok(V::Df(Arc::new(fr)))
}

fn is_logic(op: &str) -> bool {
    matches!(op, "and" | "or" | "xor")
}

/// `Operators.binary[shape1][shape2][op](x, y)`.
pub(crate) fn binary(s1: Shape, s2: Shape, op: &str, x: &V, y: &V) -> Result<V, Operand> {
    use Shape::*;
    let cplx_err = |name: &str| Err(err("NA_COMPL_OP", name));
    match (s1, s2) {
        (Scalar, Scalar) => match op {
            "power" => {
                let (xr, yr) = (x.r(), y.r());
                if x.is_complex_pair()
                    || (xr.is_negative() && yr.is_positive() && yr.less_than(&Rational::one()))
                {
                    let z = complex::power(&[xr, Rational::zero()], &Exponent::Real(yr))?;
                    Ok(cpx(z[0].clone(), z[1].clone()))
                } else {
                    Ok(V::R(xr.power(&yr).ok_or_else(|| err0("BAD_ROOT"))?))
                }
            }
            "concat" | "unshift" => Ok(V::A(vec![x.clone(), y.clone()])),
            _ => rop(op, x, y),
        },
        (Scalar, Complex) => {
            let (xr, z) = (x.r(), y.cpx());
            let v = match op {
                "add" => [xr.add(&z[0]), z[1].clone()],
                "subtract" => [xr.subtract(&z[0]), z[1].negate()],
                "multiply" => [xr.multiply(&z[0]), xr.multiply(&z[1])],
                "divide" => complex::divide(&[xr, Rational::zero()], &z)?,
                "power" => complex::power(&[xr, Rational::zero()], &Exponent::Complex(z))?,
                "rem" | "and" | "or" | "xor" => return cplx_err(op),
                _ => return Err(unknown()),
            };
            Ok(cpx(v[0].clone(), v[1].clone()))
        }
        (Scalar, Vector) => match op {
            "concat" => {
                let mut v = vec![x.clone()];
                v.extend_from_slice(y.arr());
                Ok(V::A(v))
            }
            "add" | "subtract" | "multiply" | "divide" | "power" | "modulo" | "rem" | "and"
            | "or" | "xor" => arr_map(y, |e| rop(op, x, e)),
            _ => Err(unknown()),
        },
        (Scalar, Matrix) => match op {
            "concat" => Err(err0("BAD_CONCAT")),
            "add" | "subtract" | "multiply" | "divide" | "power" | "modulo" | "rem" | "and"
            | "or" | "xor" => mat_map(y, |_, _, e| rop(op, x, e)),
            _ => Err(unknown()),
        },
        (Scalar, DataFrame) => match (op, y) {
            ("multiply" | "divide", V::Df(frame)) => {
                let mut fr = (**frame).clone();
                for col in fr.data.iter_mut() {
                    let numeric = col
                        .first()
                        .is_some_and(|c| !cruncha_units::jsnum::parse_number(c.str()).is_nan());
                    if !numeric {
                        continue;
                    }
                    for e in col.iter_mut() {
                        let s = e.str().to_string();
                        let mut len = cruncha_lang::hurmet::utf16_len(&s) as i64;
                        // Hurmet tests `e.indexOf(".")`, which is truthy unless "." is first.
                        if s.find('.') != Some(0) {
                            len -= 1;
                        }
                        let ev = Rational::from_str_or_zero(&s);
                        let r = if op == "multiply" {
                            x.r().multiply(&ev)
                        } else {
                            x.r().divide(&ev)
                        };
                        *e = V::S(r.to_string_significant(len));
                    }
                }
                Ok(V::Df(Arc::new(fr)))
            }
            _ => Err(unknown()),
        },
        (Scalar, Map) => match (op, y) {
            ("add" | "subtract" | "multiply" | "divide" | "power" | "modulo" | "rem", V::Df(f)) => {
                map_frame(f, true, |_, e| rop(op, x, e))
            }
            ("and" | "or" | "xor", V::Df(f)) => map_frame(f, false, |_, e| rop(op, x, e)),
            _ => Err(unknown()),
        },
        (Complex, Scalar) => {
            let (z, yr) = (x.cpx(), y.r());
            let v = match op {
                "add" => [z[0].add(&yr), z[1].clone()],
                "subtract" => [z[0].subtract(&yr), z[1].clone()],
                "multiply" => [z[0].multiply(&yr), z[1].multiply(&yr)],
                "divide" => complex::divide(&z, &[yr, Rational::zero()])?,
                "power" => complex::power(&z, &Exponent::Complex([yr, Rational::zero()]))?,
                "rem" | "and" | "or" | "xor" => return cplx_err(op),
                _ => return Err(unknown()),
            };
            Ok(cpx(v[0].clone(), v[1].clone()))
        }
        (Complex, Complex) => {
            let (a, b) = (x.cpx(), y.cpx());
            let v = match op {
                "add" => complex::add(&a, &b),
                "subtract" => complex::subtract(&a, &b),
                "multiply" => complex::multiply(&a, &b),
                "divide" => complex::divide(&a, &b)?,
                "power" => complex::power(&a, &Exponent::Complex(b))?,
                "rem" | "and" | "or" | "xor" => return cplx_err(op),
                _ => return Err(unknown()),
            };
            Ok(cpx(v[0].clone(), v[1].clone()))
        }
        (Vector, Scalar) => match op {
            "concat" => {
                let mut v = x.arr().to_vec();
                v.push(y.clone());
                Ok(V::A(v))
            }
            "add" | "subtract" | "multiply" | "divide" | "power" | "modulo" | "rem" | "and"
            | "or" | "xor" => arr_map(x, |e| rop(op, e, y)),
            _ => Err(unknown()),
        },
        (RowVector, RowVector) => match op {
            "add" | "subtract" | "divide" | "circ" | "power" | "modulo" | "and" | "or" | "xor" => {
                same_len(x, y, op)
            }
            "dot" => {
                if x.len() != y.len() {
                    Err(mis())
                } else {
                    Ok(dot_product(x, y))
                }
            }
            "cross" => cross(x, y),
            "multiply" => one_each(x, y, "multiply"),
            "concat" => {
                let mut v = x.arr().to_vec();
                v.extend_from_slice(y.arr());
                Ok(V::A(v))
            }
            "unshift" => Ok(V::A(vec![x.clone(), y.clone()])),
            _ => Err(unknown()),
        },
        (RowVector, ColumnVector) => match op {
            "add" | "subtract" | "power" | "modulo" | "and" | "or" | "xor" => one_each(x, y, op),
            "dot" | "multiply" => {
                if x.len() != y.len() {
                    Err(mis())
                } else {
                    Ok(dot_product(x, y))
                }
            }
            "cross" => cross(x, y),
            "circ" => same_len(x, y, "circ"),
            "concat" | "unshift" => Err(err0("BAD_CONCAT")),
            _ => Err(unknown()),
        },
        (RowVector, Matrix) => {
            let ncols = y.get(0).len();
            match op {
                "add" | "subtract" | "circ" | "divide" | "power" | "modulo" => {
                    if x.len() != ncols {
                        return Err(mis());
                    }
                    mat_map(y, |_, j, e| rop(op, &x.get(j), e))
                }
                "multiply" => {
                    if x.len() != ncols {
                        return Err(mis());
                    }
                    let t =
                        matrix::transpose(&Operand::new(y.clone(), Default::default(), dt::MATRIX));
                    Ok(V::A(
                        t.value
                            .arr()
                            .iter()
                            .map(|row| dot_product(x, row))
                            .collect(),
                    ))
                }
                "concat" => {
                    if x.len() != ncols {
                        return Err(err0("BAD_CONCAT"));
                    }
                    Ok(V::A(
                        y.arr()
                            .iter()
                            .enumerate()
                            .map(|(i, row)| {
                                let mut r = vec![x.get(i)];
                                r.extend_from_slice(row.arr());
                                V::A(r)
                            })
                            .collect(),
                    ))
                }
                "unshift" => {
                    if x.len() != ncols {
                        return Err(err0("BAD_CONCAT"));
                    }
                    let mut v = vec![x.clone()];
                    v.extend_from_slice(y.arr());
                    Ok(V::A(v))
                }
                _ => Err(unknown()),
            }
        }
        (ColumnVector, RowVector) => match op {
            "add" | "subtract" | "power" | "modulo" | "and" | "or" | "xor" => one_each(x, y, op),
            "dot" => {
                if x.len() != y.len() {
                    Err(mis())
                } else {
                    Ok(dot_product(x, y))
                }
            }
            "cross" => cross(x, y),
            "multiply" | "divide" => {
                if op == "multiply" {
                    // Hurmet bug, reproduced: it checks `x[0].length`, the length of the first
                    // element (2 for a rational, which is a pair in Hurmet), not of x.
                    let x0_len = match x.get(0) {
                        V::R(_) | V::C(_) => 2,
                        V::A(a) => a.len(),
                        _ => 0,
                    };
                    if x0_len != y.len() {
                        return Err(mis());
                    }
                }
                let o = if op == "multiply" {
                    "multiply"
                } else {
                    "divide"
                };
                Ok(V::A(
                    x.arr()
                        .iter()
                        .map(|xe| arr_map(y, |e| rop(o, xe, e)))
                        .collect::<Result<Vec<_>, _>>()?,
                ))
            }
            "circ" => same_len(x, y, "circ"),
            "concat" | "unshift" => Err(err0("BAD_CONCAT")),
            _ => Err(unknown()),
        },
        (ColumnVector, ColumnVector) => match op {
            "add" | "subtract" | "divide" | "circ" | "power" | "modulo" | "rem" | "and" | "or"
            | "xor" => same_len(x, y, op),
            "dot" => {
                if x.len() != y.len() {
                    Err(mis())
                } else {
                    Ok(dot_product(x, y))
                }
            }
            "cross" => cross(x, y),
            "multiply" => one_each(x, y, "multiply"),
            "concat" => {
                if x.len() != y.len() {
                    return Err(mis());
                }
                Ok(V::A(
                    x.arr()
                        .iter()
                        .enumerate()
                        .map(|(i, e)| V::A(vec![e.clone(), y.get(i)]))
                        .collect(),
                ))
            }
            "unshift" => {
                let mut v = x.arr().to_vec();
                v.extend_from_slice(y.arr());
                Ok(V::A(v))
            }
            _ => Err(unknown()),
        },
        (ColumnVector, Matrix) => match op {
            "add" | "subtract" | "circ" | "divide" | "power" | "mod" => {
                if x.len() != y.len() {
                    return Err(mis());
                }
                let o = if op == "circ" { "multiply" } else { op };
                mat_map(y, |i, _, e| rop(o, &x.get(i), e))
            }
            "multiply" => {
                if y.len() != 1 {
                    return Err(mis());
                }
                mat_map(y, |i, _, e| rop("multiply", &x.get(i), e))
            }
            "concat" => {
                if x.len() != y.len() {
                    return Err(mis());
                }
                Ok(V::A(
                    y.arr()
                        .iter()
                        .enumerate()
                        .map(|(i, row)| {
                            let mut r = vec![x.get(i)];
                            r.extend_from_slice(row.arr());
                            V::A(r)
                        })
                        .collect(),
                ))
            }
            "unshift" => Err(err0("BAD_CONCAT")),
            _ => Err(unknown()),
        },
        (ColumnVector, Map) => match y {
            V::Df(f) if is_logic(op) => map_frame(f, false, |i, e| rop(op, &x.get(i), e)),
            V::Df(f)
                if matches!(
                    op,
                    "add" | "subtract" | "multiply" | "divide" | "power" | "modulo" | "rem"
                ) =>
            {
                map_frame(f, true, |i, e| rop(op, &x.get(i), e))
            }
            _ => Err(unknown()),
        },
        (Matrix, Scalar) => match op {
            "power" => {
                let m = x.arr();
                let square = m.first().is_some_and(|r| r.len() == m.len());
                if square && y.r().are_equal(&Rational::int(-1)) {
                    matrix::invert(x, false)
                } else {
                    mat_map(x, |_, _, e| rop("power", e, y))
                }
            }
            "add" | "subtract" | "multiply" | "divide" | "modulo" | "rem" => {
                mat_map(x, |_, _, e| rop(op, e, y))
            }
            _ => Err(unknown()),
        },
        (Matrix, RowVector) => match op {
            "add" | "subtract" | "multiply" | "circ" | "divide" | "power" | "modulo" | "rem" => {
                mat_map(x, |_, j, e| rop(op, e, &y.get(j)))
            }
            "unshift" => {
                if x.get(0).len() != y.len() {
                    return Err(mis());
                }
                let mut v = x.arr().to_vec();
                v.push(y.clone());
                Ok(V::A(v))
            }
            _ => Err(unknown()),
        },
        (Matrix, ColumnVector) => match op {
            "add" | "subtract" | "circ" | "divide" | "power" | "modulo" | "rem" => {
                mat_map(x, |i, _, e| rop(op, e, &y.get(i)))
            }
            "multiply" => {
                if x.get(0).len() != y.len() {
                    return Err(mis());
                }
                Ok(V::A(
                    x.arr().iter().map(|row| dot_product(row, y)).collect(),
                ))
            }
            "concat" => {
                if x.len() != y.len() {
                    return Err(mis());
                }
                Ok(V::A(
                    x.arr()
                        .iter()
                        .enumerate()
                        .map(|(i, row)| {
                            let mut r = row.arr().to_vec();
                            r.push(y.get(i));
                            V::A(r)
                        })
                        .collect(),
                ))
            }
            _ => Err(unknown()),
        },
        (Matrix, Matrix) => {
            let same_shape = x.len() == y.len() && x.get(0).len() == y.get(0).len();
            match op {
                "add" | "subtract" | "circ" | "divide" | "power" | "modulo" | "rem" | "and"
                | "or" | "xor" => {
                    if !same_shape {
                        return Err(mis());
                    }
                    let yy = y.clone();
                    mat_map(x, move |i, j, e| rop(op, e, &yy.get(i).get(j)))
                }
                "dot" => {
                    if !same_shape {
                        return Err(mis());
                    }
                    let mut acc: Option<Rational> = None;
                    for (i, row) in x.arr().iter().enumerate() {
                        let d = dot_product(row, &y.get(i)).r();
                        acc = Some(acc.map_or(d.clone(), |a| a.add(&d)));
                    }
                    Ok(acc.map_or(V::Undef, V::R))
                }
                "cross" => Err(err0("CROSS")),
                "multiply" => {
                    // Hurmet's matrix × matrix is an empty function (a TypeError follows).
                    // The port computes the matrix product.
                    if x.get(0).len() != y.len() {
                        return Err(mis());
                    }
                    let t =
                        matrix::transpose(&Operand::new(y.clone(), Default::default(), dt::MATRIX));
                    Ok(V::A(
                        x.arr()
                            .iter()
                            .map(|row| {
                                V::A(
                                    t.value
                                        .arr()
                                        .iter()
                                        .map(|col| dot_product(row, col))
                                        .collect(),
                                )
                            })
                            .collect(),
                    ))
                }
                "concat" => {
                    if x.len() != y.len() {
                        return Err(mis());
                    }
                    Ok(V::A(
                        x.arr()
                            .iter()
                            .enumerate()
                            .map(|(i, row)| {
                                let mut r = row.arr().to_vec();
                                r.extend_from_slice(y.get(i).arr());
                                V::A(r)
                            })
                            .collect(),
                    ))
                }
                "unshift" => {
                    if x.get(0).len() != y.get(0).len() {
                        return Err(mis());
                    }
                    let mut v = x.arr().to_vec();
                    v.extend_from_slice(y.arr());
                    Ok(V::A(v))
                }
                _ => Err(unknown()),
            }
        }
        (Map, Scalar) => match x {
            V::Df(f) if is_logic(op) => map_frame(f, false, |_, e| rop(op, e, y)),
            V::Df(f)
                if matches!(
                    op,
                    "add" | "subtract" | "multiply" | "divide" | "power" | "modulo" | "rem"
                ) =>
            {
                map_frame(f, true, |_, e| rop(op, e, y))
            }
            _ => Err(unknown()),
        },
        (Map, ColumnVector) => match x {
            V::Df(f) if is_logic(op) => map_frame(f, false, |i, e| rop(op, e, &y.get(i))),
            V::Df(f)
                if matches!(
                    op,
                    "add" | "subtract" | "multiply" | "divide" | "power" | "modulo" | "rem"
                ) =>
            {
                map_frame(f, true, |i, e| rop(op, e, &y.get(i)))
            }
            _ => Err(unknown()),
        },
        _ => Err(unknown()),
    }
}

/// `Operators.unary[shape][op](x)`.
pub(crate) fn unary(shape: Shape, op: &str, x: &V) -> Result<V, Operand> {
    use Shape::*;
    let one = |r: &Rational| -> Result<Rational, Operand> {
        Ok(match op {
            "abs" | "norm" => r.abs(),
            "negate" => r.negate(),
            "exp" => r.exp(),
            "floor" => r.floor(),
            "ceil" => r.ceil(),
            "percent" => Rational::new(1.into(), 100.into()).multiply(r),
            "factorial" => r.factorial(),
            "doubleFactorial" => r.double_factorial(),
            _ => return Err(unknown()),
        })
    };
    let elem = |e: &V| -> Result<V, Operand> {
        if op == "not" {
            Ok(V::Bool(!e.truthy()))
        } else {
            Ok(V::R(one(&e.r())?))
        }
    };
    match shape {
        Scalar => elem(x),
        Complex => {
            let z = x.cpx();
            Ok(match op {
                "abs" | "norm" => V::R(complex::abs(&z)),
                "conjugate" => {
                    let c = complex::conjugate(&z);
                    cpx(c[0].clone(), c[1].clone())
                }
                "negate" => {
                    let c = complex::negate(&z);
                    cpx(c[0].clone(), c[1].clone())
                }
                "exp" => {
                    let c = complex::exp(&z);
                    cpx(c[0].clone(), c[1].clone())
                }
                "floor" | "ceil" | "percent" | "factorial" | "not" => {
                    return Err(err("NA_COMPL_OP", op));
                }
                "doubleFactorial" => return Err(err("NA_COMPL_OP", "factorial")),
                _ => return Err(unknown()),
            })
        }
        Vector => match op {
            "abs" | "norm" => {
                let mut sum: Option<Rational> = None;
                for e in x.arr() {
                    let sq = e.r().multiply(&e.r());
                    sum = Some(sum.map_or(sq.clone(), |s| s.add(&sq)));
                }
                Ok(sum.map_or(V::Undef, |s| V::R(s.sqrt())))
            }
            _ => arr_map(x, elem),
        },
        Matrix => match op {
            "abs" => matrix::invert(x, true),
            "norm" => {
                let rows = x.arr();
                if rows.first().is_some_and(|r| r.len() == rows.len()) {
                    let mut sum = Rational::zero();
                    for row in rows {
                        for e in row.arr() {
                            sum = sum.add(&e.r().multiply(&e.r()));
                        }
                    }
                    Ok(V::R(sum.sqrt()))
                } else {
                    // A TODO in Hurmet: ‖matrix‖ of a non-square matrix is undefined.
                    Err(unknown())
                }
            }
            _ => mat_map(x, |_, _, e| elem(e)),
        },
        Map => match x {
            V::Df(f) => {
                if op == "not" {
                    map_frame(f, false, |_, e| elem(e))
                } else if matches!(
                    op,
                    "abs"
                        | "negate"
                        | "exp"
                        | "floor"
                        | "ceil"
                        | "percent"
                        | "factorial"
                        | "doubleFactorial"
                ) {
                    map_frame(f, true, |_, e| elem(e))
                } else {
                    Err(unknown())
                }
            }
            _ => Err(unknown()),
        },
        _ => Err(unknown()),
    }
}

/// `Operators.condition[shape](x)`: a vector or matrix condition is true if all elements are.
pub(crate) fn condition(shape: Shape, x: &V) -> bool {
    match shape {
        Shape::Vector => x.arr().iter().all(V::truthy),
        Shape::Matrix => x.arr().iter().all(|row| row.arr().iter().all(V::truthy)),
        _ => x.truthy(),
    }
}

/// `isDivByZero`.
pub(crate) fn is_div_by_zero(q: &V, shape: Shape) -> bool {
    let zero_den = |v: &V| matches!(v, V::R(r) if r.den == 0.into());
    match shape {
        Shape::Scalar => zero_den(q),
        Shape::Vector => q.arr().iter().any(zero_den),
        Shape::Matrix => q.arr().iter().any(|row| row.arr().iter().any(zero_den)),
        Shape::Map => match q {
            V::Df(f) => f.data.iter().any(|col| col.iter().any(zero_den)),
            _ => false,
        },
        _ => false,
    }
}

// ---------------------------------------------------------------------------------------
// compare.js
// ---------------------------------------------------------------------------------------

fn equals(x: &V, y: &V) -> bool {
    match (x, y) {
        (V::R(a), V::R(b)) => a.are_equal(b),
        _ => strict_equals(x, y),
    }
}

fn js_order(x: &V, y: &V) -> Option<std::cmp::Ordering> {
    match (x, y) {
        (V::S(a), V::S(b)) => Some(a.encode_utf16().cmp(b.encode_utf16())),
        (V::Bool(a), V::Bool(b)) => Some(a.cmp(b)),
        (V::Num(a), V::Num(b)) => a.partial_cmp(b),
        _ => None,
    }
}

fn frame_has(frame: &Frame, key: &str) -> bool {
    frame.headings.iter().any(|h| h == key)
        || frame.row_map.as_ref().is_some_and(|m| m.contains_key(key))
}

/// Hurmet's `compare(op, x, y, yPrev)`. `y_prev` is set inside a chained comparison
/// such as `a < b < c`.
pub(crate) fn compare(op: &str, x: &V, y: &V, y_prev: Option<&V>) -> Result<V, Operand> {
    let prev_truthy = y_prev.is_some_and(V::truthy);
    if matches!(x, V::Bool(false)) && prev_truthy {
        return Ok(V::Bool(false));
    }
    let x = if matches!(x, V::Bool(true)) && prev_truthy {
        y_prev.cloned().unwrap_or_default()
    } else {
        x.clone()
    };
    let x = &x;
    let both_r = |f: &dyn Fn(&Rational, &Rational) -> bool| match (x, y) {
        (V::R(a), V::R(b)) => Some(f(a, b)),
        _ => None,
    };
    use std::cmp::Ordering::*;
    let b = match op {
        "=" => return Err(err0("BAD_EQ")),
        "==" | "⩵" => equals(x, y),
        "≠" | "!=" | "/=" => {
            both_r(&|a, b| !a.are_equal(b)).unwrap_or_else(|| !strict_equals(x, y))
        }
        ">" => both_r(&|a, b| a.greater_than(b)).unwrap_or(js_order(x, y) == Some(Greater)),
        "<" => both_r(&|a, b| a.less_than(b)).unwrap_or(js_order(x, y) == Some(Less)),
        "≥" | ">=" => both_r(&|a, b| a.greater_than_or_equal(b))
            .unwrap_or(matches!(js_order(x, y), Some(Greater | Equal))),
        "≤" | "<=" => both_r(&|a, b| a.less_than_or_equal(b))
            .unwrap_or(matches!(js_order(x, y), Some(Less | Equal))),
        "∈" | "in" => match (x, y) {
            (V::S(a), V::S(b)) => a.chars().count() <= 1 && b.contains(a.as_str()),
            (_, V::A(arr)) if !matches!(x, V::A(_)) => arr.iter().any(|e| equals(x, e)),
            (V::S(a), V::Df(f)) => frame_has(f, a),
            _ => return Err(err0("NOT_ARRAY")),
        },
        "∋" => match (x, y) {
            (V::S(a), V::S(b)) => a.chars().count() <= 1 && b.contains(a.as_str()),
            (V::Df(f), V::S(b)) => frame_has(f, b),
            _ => return Err(err("NO_PROP", "")),
        },
        "⊃" => match (x, y) {
            (V::S(a), V::S(b)) => a.contains(b.as_str()),
            (V::A(xa), V::A(ya)) => {
                let mut result = false;
                for i in 0..xa.len() {
                    if ya.first().is_some_and(|y0| equals(y0, &xa[i])) {
                        if i + ya.len() > xa.len() {
                            break;
                        }
                        result = (1..ya.len()).all(|j| equals(&ya[j], &xa[i + j]));
                        break;
                    }
                }
                result
            }
            _ => return Err(err0("NOT_ARRAY")),
        },
        "∉" | "!in" => match (x, y) {
            (V::S(a), V::S(b)) => a.chars().count() != 1 && !b.contains(a.as_str()),
            (V::R(_), V::A(arr)) if arr.first().is_some_and(V::is_rational) => {
                !arr.iter().any(|e| equals(x, e))
            }
            (_, V::A(arr)) => !arr.iter().any(|e| strict_equals(x, e)),
            (V::S(a), V::Df(f)) => !frame_has(f, a),
            _ => return Err(err0("NOT_ARRAY")),
        },
        "∌" => match (x, y) {
            (V::S(a), V::S(b)) => b.chars().count() != 1 && !a.contains(b.as_str()),
            (V::Df(f), V::S(b)) => !frame_has(f, b),
            _ => return Err(err("NO_PROP", "")),
        },
        "⊄" | "⊅" => {
            let (small, big) = if op == "⊄" { (x, y) } else { (y, x) };
            match (x, y) {
                (V::S(a), V::S(b)) => {
                    if op == "⊄" {
                        !b.contains(a.as_str())
                    } else {
                        !a.contains(b.as_str())
                    }
                }
                (V::A(_), V::A(_)) => {
                    let (s, bg) = (small.arr(), big.arr());
                    let mut found = false;
                    for i in 0..bg.len() {
                        if s.first().is_some_and(|s0| equals(s0, &bg[i])) {
                            if i + s.len() > bg.len() {
                                continue;
                            }
                            if (1..s.len()).all(|j| equals(&s[j], &bg[i + j])) {
                                found = true;
                                break;
                            }
                        }
                    }
                    found
                }
                _ => return Err(err0("NOT_ARRAY")),
            }
        }
        // Hurmet returns undefined for an operator it does not know.
        _ => return Ok(V::Undef),
    };
    Ok(V::Bool(b))
}

fn prev_at(y_prev: Option<&V>, i: usize) -> Option<V> {
    match y_prev {
        None => None,
        Some(V::A(a)) => Some(a.get(i).cloned().unwrap_or_default()),
        Some(v) => Some(v.clone()),
    }
}

/// `Operators.relations[shape1][shape2].relate(op, x, y, yPrev)`.
pub(crate) fn relate(
    s1: Shape,
    s2: Shape,
    op: &str,
    x: &V,
    y: &V,
    y_prev: Option<&V>,
) -> Result<V, Operand> {
    use Shape::*;
    let str_ops = ["∈", "in", "∋", "⊇", "∉", "!in", "∌", "⊈", "⊉"];
    match (s1, s2) {
        (Scalar, Scalar) => compare(op, x, y, y_prev),
        (Scalar, Vector) => arr_map_i(y, |i, e| compare(op, x, e, prev_at(y_prev, i).as_ref())),
        (Scalar, Matrix) => {
            let rows = y.arr();
            let mut out = Vec::new();
            for (i, row) in rows.iter().enumerate() {
                let prev_row = prev_at(y_prev, i);
                out.push(arr_map_i(row, |j, e| {
                    let p = match &prev_row {
                        Some(V::A(a)) => Some(a.get(j).cloned().unwrap_or_default()),
                        other => other.clone(),
                    };
                    compare(op, x, e, p.as_ref())
                })?);
            }
            Ok(V::A(out))
        }
        (Scalar, Map) => match y {
            V::Df(f) if y_prev.is_none() => {
                let mut fr = (**f).clone();
                for (j, col) in fr.data.iter_mut().enumerate() {
                    let first_is_str = matches!(col.first(), Some(V::S(_)));
                    if j > 0 || !first_is_str || str_ops.contains(&op) {
                        for e in col.iter_mut() {
                            *e = compare(op, x, e, None)?;
                        }
                    }
                }
                Ok(V::Df(Arc::new(fr)))
            }
            _ => Err(unknown()),
        },
        (Vector, Scalar) => arr_map_i(x, |i, e| compare(op, e, y, prev_at(y_prev, i).as_ref())),
        (RowVector, RowVector) | (ColumnVector, ColumnVector) if y_prev.is_none() => {
            arr_map_i(x, |i, e| compare(op, e, &y.get(i), None))
        }
        (RowVector, Matrix) if y_prev.is_none() => {
            if x.len() != y.get(0).len() {
                return Err(mis());
            }
            mat_map(y, |_, j, e| compare(op, &x.get(j), e, None))
        }
        (ColumnVector, Map) => match y {
            V::Df(f) if y_prev.is_none() => {
                let mut fr = (**f).clone();
                for (j, col) in fr.data.iter_mut().enumerate() {
                    let first_is_str = matches!(col.first(), Some(V::S(_)));
                    if j > 0 || !first_is_str || str_ops.contains(&op) {
                        for (i, e) in col.iter_mut().enumerate() {
                            *e = compare(op, &x.get(i), e, None)?;
                        }
                    }
                }
                Ok(V::Df(Arc::new(fr)))
            }
            _ => Err(unknown()),
        },
        (Matrix, Scalar) => {
            let rows = x.arr();
            let mut out = Vec::new();
            for (i, row) in rows.iter().enumerate() {
                let prev_row = prev_at(y_prev, i);
                out.push(arr_map_i(row, |j, e| {
                    let p = match &prev_row {
                        Some(V::A(a)) => Some(a.get(j).cloned().unwrap_or_default()),
                        other => other.clone(),
                    };
                    compare(op, e, y, p.as_ref())
                })?);
            }
            Ok(V::A(out))
        }
        // Hurmet compares whole rows here (a bug); the port compares element by element.
        (Matrix, Matrix) if y_prev.is_none() => {
            let yy = y.clone();
            mat_map(x, move |i, j, e| compare(op, e, &yy.get(i).get(j), None))
        }
        _ => Err(unknown()),
    }
}

fn arr_map_i(v: &V, f: impl Fn(usize, &V) -> Result<V, Operand>) -> Result<V, Operand> {
    Ok(V::A(
        v.arr()
            .iter()
            .enumerate()
            .map(|(i, e)| f(i, e))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}
