// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Vectors and matrices (Hurmet's `matrix.js`). A vector is an array of elements, a matrix an
//! array of rows; the `dtype` flags say which (row vector, column vector, matrix).

use crate::complex;
use crate::error::{err, err0};
use crate::format::{Mode, fmt};
use crate::value::{OUnit, Operand, V, dt, is_vector};
use cruncha_units::Rational;

/// The most elements a range or a generated matrix may have (JavaScript has no limit).
pub(crate) const MAX_ELEMENTS: usize = 1_000_000;

pub(crate) fn transpose(o: &Operand) -> Operand {
    if is_vector(o.dtype) {
        let delta =
            if o.dtype & dt::ROWVECTOR != 0 { 1 } else { -1 } * (dt::COLUMNVECTOR - dt::ROWVECTOR);
        Operand {
            value: o.value.clone(),
            unit: o.unit.clone(),
            dtype: o.dtype + delta,
            name: None,
        }
    } else if o.dtype & dt::MATRIX != 0 {
        let rows = o.value.arr();
        let ncols = rows.first().map_or(0, V::len);
        let t: Vec<V> = (0..ncols)
            .map(|i| V::A(rows.iter().map(|row| row.get(i)).collect()))
            .collect();
        Operand {
            value: V::A(t),
            unit: o.unit.clone(),
            dtype: o.dtype,
            name: None,
        }
    } else {
        err0("BAD_TRANS")
    }
}

fn map_elems(value: &V, vector: bool, f: &dyn Fn(&V) -> V) -> V {
    if vector {
        V::A(value.arr().iter().map(f).collect())
    } else {
        V::A(
            value
                .arr()
                .iter()
                .map(|row| V::A(row.arr().iter().map(f).collect()))
                .collect(),
        )
    }
}

pub(crate) fn convert_from_base_units(
    value: &V,
    dtype: i64,
    gauge: &Rational,
    factor: &Rational,
) -> V {
    let vector = is_vector(dtype);
    let conversion = map_elems(value, vector, &|e| V::R(e.r().divide(factor)));
    if !gauge.is_zero() {
        // Hurmet bug, reproduced: with a gauge, it subtracts the gauge from the base-unit
        // value and discards the division by the factor.
        return map_elems(value, vector, &|e| V::R(e.r().subtract(gauge)));
    }
    conversion
}

pub(crate) fn convert_to_base_units(
    value: &V,
    dtype: i64,
    gauge: &Rational,
    factor: &Rational,
) -> V {
    let vector = is_vector(dtype);
    let mut conversion = value.clone();
    if !gauge.is_zero() {
        conversion = map_elems(value, vector, &|e| V::R(e.r().add(gauge)));
    }
    map_elems(&conversion, vector, &|e| V::R(e.r().multiply(factor)))
}

pub(crate) fn element_display(value: &V, dtype: i64, spec: &str, df: &str, mode: Mode) -> String {
    if matches!(value, V::Undef) {
        return String::new();
    }
    if dtype & dt::RATIONAL != 0 {
        match value {
            V::R(r) => fmt(r, spec, df, mode),
            _ => js_string(value),
        }
    } else if dtype & dt::COMPLEX != 0 {
        complex::display(&value.cpx(), spec, df).0
    } else if dtype & dt::STRING != 0 {
        let s = js_string(value);
        match mode {
            Mode::Literal => format!("\"{}\"", s.replace('"', "\"\"")),
            Mode::Plain => s,
            Mode::Tex => format!("\\text{{{s}}}"),
        }
    } else if dtype & dt::BOOLEAN != 0 {
        if mode == Mode::Tex {
            format!("\\text{{{}}}", js_string(value))
        } else {
            js_string(value)
        }
    } else {
        js_string(value)
    }
}

/// JavaScript `String(value)` for the values that can appear in a matrix.
pub(crate) fn js_string(v: &V) -> String {
    match v {
        V::Undef => "undefined".into(),
        V::Null => "null".into(),
        V::Bool(b) => b.to_string(),
        V::Num(n) => cruncha_units::jsnum::to_string(*n),
        V::S(s) => s.clone(),
        V::R(r) => format!("{},{}", r.num, r.den),
        V::C(z) => format!("{},{},{},{}", z[0].num, z[0].den, z[1].num, z[1].den),
        V::A(a) => a.iter().map(js_string).collect::<Vec<_>>().join(","),
        _ => "[object Object]".into(),
    }
}

pub(crate) fn display(value: &V, dtype: i64, spec: &str, df: &str) -> String {
    let mut s = String::new();
    if dtype & dt::MATRIX != 0 {
        s.push_str("\\begin{pmatrix}");
        let rows = value.arr();
        let ncols = rows.first().map_or(0, V::len);
        for row in rows {
            for j in 0..ncols {
                s.push_str(&element_display(&row.get(j), dtype, spec, df, Mode::Tex));
                s.push_str(" &");
            }
            s.pop();
            s.push_str(" \\\\ ");
        }
        for _ in 0..3 {
            s.pop();
        }
        let t = s.trim().to_string();
        s = t;
        s.push_str("\\end{pmatrix}");
    } else {
        let elems = match value {
            V::Q(q) => q.0.arr().to_vec(),
            _ => value.arr().to_vec(),
        };
        if elems.is_empty() {
            s.push_str("[\\,]");
        } else {
            s.push_str("\\begin{bmatrix}");
            let sep = if dtype & dt::ROWVECTOR != 0 {
                " & "
            } else {
                " \\\\ "
            };
            let n = elems.len();
            for (i, e) in elems.iter().enumerate() {
                s.push_str(&element_display(e, dtype, spec, df, Mode::Tex));
                if i < n - 1 {
                    s.push_str(sep);
                }
            }
            s.push_str("\\end{bmatrix}");
        }
    }
    s
}

pub(crate) fn display_alt(
    value: &V,
    dtype: i64,
    spec: &str,
    df: &str,
    mode: Mode,
    suffix: &str,
) -> String {
    let mut s = String::new();
    if dtype & dt::MATRIX != 0 {
        s.push('(');
        let rows = value.arr();
        let ncols = rows.first().map_or(0, V::len);
        for row in rows {
            for j in 0..ncols {
                s.push_str(&element_display(&row.get(j), dtype, spec, df, mode));
                s.push_str(suffix);
                s.push_str(", ");
            }
            s.pop();
            s.pop();
            s.push_str("; ");
        }
        s.pop();
        s.pop();
        let t = s.trim().to_string();
        s = t;
        s.push(')');
    } else {
        s.push('[');
        let sep = if dtype & dt::ROWVECTOR != 0 {
            ", "
        } else {
            "; "
        };
        let elems = match value {
            V::Q(q) => q.0.arr().to_vec(),
            _ => value.arr().to_vec(),
        };
        let n = elems.len();
        for (i, e) in elems.iter().enumerate() {
            s.push_str(&element_display(e, dtype, spec, df, mode));
            s.push_str(suffix);
            if i + 1 < n {
                s.push_str(sep);
            }
        }
        s.push(']');
    }
    s
}

/// `Matrix.findfirst`: 1-based index of the first element equal to `el`, or 0.
pub(crate) fn findfirst(el: &V, array: &Operand) -> Result<Rational, Operand> {
    if !is_vector(array.dtype) {
        return Err(err("NOT_VECTOR", "findfirst"));
    }
    let numeric = el.is_rational();
    for (i, val) in array.value.arr().iter().enumerate() {
        let eq_num = numeric && val.is_rational() && el.r().are_equal(&val.r());
        let eq = match (val, el) {
            (V::Bool(a), V::Bool(b)) => a == b,
            (V::S(a), V::S(b)) => a == b,
            _ => false,
        };
        if eq_num || eq {
            return Ok(Rational::int(i as i64 + 1));
        }
    }
    Ok(Rational::zero())
}

pub(crate) fn identity(n: usize) -> Vec<Vec<Rational>> {
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i == j {
                        Rational::one()
                    } else {
                        Rational::zero()
                    }
                })
                .collect()
        })
        .collect()
}

/// Gauss–Jordan inversion; with `determinant`, returns the determinant (as Hurmet computes it:
/// the reciprocal of the product of the pivots, which is the determinant of the inverse).
pub(crate) fn invert(m: &V, determinant: bool) -> Result<V, Operand> {
    let rows = m.arr();
    let dim = rows.len();
    if dim == 0 || rows[0].len() != dim {
        return Err(err0("NONSQUARE"));
    }
    let mut c: Vec<Vec<Rational>> = rows
        .iter()
        .map(|r| r.arr().iter().map(V::r).collect())
        .collect();
    let mut id = identity(dim);
    let mut det = Rational::one();
    for i in 0..dim {
        let mut temp = c[i][i].clone();
        if temp.is_zero() {
            for ii in i + 1..dim {
                if !c[ii][i].is_zero() {
                    c.swap(i, ii);
                    id.swap(i, ii);
                    break;
                }
            }
            temp = c[i][i].clone();
            if temp.is_zero() {
                return Err(err0("SINGULAR"));
            }
        }
        if determinant {
            det = det.divide(&temp);
            if i == dim - 1 {
                return Ok(V::R(det));
            }
        }
        for j in 0..dim {
            c[i][j] = c[i][j].divide(&temp);
            id[i][j] = id[i][j].divide(&temp);
        }
        for ii in 0..dim {
            if ii == i {
                continue;
            }
            let t = c[ii][i].clone();
            for j in 0..dim {
                c[ii][j] = c[ii][j].subtract(&t.multiply(&c[i][j]));
                id[ii][j] = id[ii][j].subtract(&t.multiply(&id[i][j]));
            }
        }
    }
    Ok(V::A(
        id.into_iter()
            .map(|r| V::A(r.into_iter().map(V::R).collect()))
            .collect(),
    ))
}

fn js_slice(a: &[V], start: i64, end: i64) -> Vec<V> {
    let len = a.len() as i64;
    let norm = |x: i64| if x < 0 { (len + x).max(0) } else { x.min(len) };
    let (s, e) = (norm(start), norm(end));
    if s >= e {
        Vec::new()
    } else {
        a[s as usize..e as usize].to_vec()
    }
}

fn at(a: &[V], i: i64) -> V {
    if i < 0 {
        V::Undef
    } else {
        a.get(i as usize).cloned().unwrap_or(V::Undef)
    }
}

fn to_int(r: &V) -> i64 {
    let x = r.r().to_f64();
    if x.is_finite() { x as i64 } else { 0 }
}

/// `Matrix.submatrix(oprnd, rowIndex, colIndex)`: 1-based indexing with ranges.
pub(crate) fn submatrix(o: &Operand, index: &Operand, col_index: Option<&Operand>) -> Operand {
    if index.dtype & (dt::RATIONAL | dt::RANGE) == 0 {
        return err0("BAD_INDEX");
    }
    let rows = o.value.arr();
    let mut dtype = o.dtype;
    let (start, end, step);
    if index.dtype & dt::RANGE != 0 {
        let r = index.value.arr();
        start = to_int(&at(r, 0));
        step = to_int(&at(r, 1));
        end = match at(r, 2) {
            V::S(s) if s == "∞" => rows.len() as i64,
            v => to_int(&v),
        };
    } else if index.value.r().are_equal(&Rational::zero()) {
        start = 1;
        end = rows.len() as i64;
        step = 1;
    } else {
        start = to_int(&index.value);
        end = start;
        step = 1;
    }
    let step = if step == 0 { 1 } else { step };

    if is_vector(o.dtype) {
        let value = if start == end {
            dtype = o.dtype - (o.dtype & dt::ROWVECTOR) - (o.dtype & dt::COLUMNVECTOR);
            at(rows, start - 1)
        } else if step == 1 {
            V::A(js_slice(rows, start - 1, end))
        } else {
            let mut v = Vec::new();
            let mut i = start - 1;
            while i < end && v.len() < MAX_ELEMENTS {
                v.push(at(rows, i));
                i += step;
            }
            V::A(v)
        };
        return Operand {
            value,
            unit: o.unit.clone(),
            dtype,
            name: None,
        };
    }

    let ncols = rows.first().map_or(0, V::len) as i64;
    let (mut col_start, mut col_step, mut col_end) = (0, 1, 0);
    if let Some(ci) = col_index {
        if ci.dtype & dt::RANGE != 0 {
            let r = ci.value.arr();
            col_start = to_int(&at(r, 0));
            col_step = to_int(&at(r, 1));
            col_end = match at(r, 2) {
                V::S(s) if s == "∞" => ncols,
                v => to_int(&v),
            };
        } else if ci.value.r().are_equal(&Rational::zero()) {
            col_start = 1;
            col_end = ncols;
        } else {
            col_start = to_int(&ci.value);
            col_end = col_start;
        }
    }
    let col_step = if col_step == 0 { 1 } else { col_step };
    let row = |i: i64| -> Vec<V> { at(rows, i).arr().to_vec() };
    let value;
    if start == end && col_start == col_end {
        value = at(&row(start - 1), col_start - 1);
        dtype -= dt::MATRIX;
    } else if start == end {
        let r = row(start - 1);
        value = if col_step == 1 {
            V::A(js_slice(&r, col_start - 1, col_end))
        } else {
            let mut v = Vec::new();
            let mut j = col_start - 1;
            while j < col_end && j - col_start < MAX_ELEMENTS as i64 {
                v.push(at(&r, j));
                j += col_step;
            }
            V::A(v)
        };
        dtype = dtype - dt::MATRIX + dt::ROWVECTOR;
    } else if col_start == col_end {
        let mut v = Vec::new();
        let mut i = start - 1;
        while i < end && i - start < MAX_ELEMENTS as i64 {
            v.push(at(&row(i), col_start - 1));
            i += step;
        }
        value = V::A(v);
        dtype = dtype - dt::MATRIX + dt::COLUMNVECTOR;
    } else if col_step == 1 {
        let mut v = Vec::new();
        let mut i = start - 1;
        while i < end && i - start < MAX_ELEMENTS as i64 {
            v.push(V::A(js_slice(&row(i), col_start - 1, col_end)));
            i += step;
        }
        value = V::A(v);
    } else {
        // Hurmet bug, reproduced: it reads `oprnd[i][j]` (not `oprnd.value`), which is undefined.
        let mut v = Vec::new();
        let mut i = start - 1;
        while i < end && i - start < MAX_ELEMENTS as i64 {
            let mut r = Vec::new();
            let mut j = col_start - 1;
            while j < col_end && j - col_start < MAX_ELEMENTS as i64 {
                r.push(V::Undef);
                j += col_step;
            }
            v.push(V::A(r));
            i += step;
        }
        value = V::A(v);
    }
    Operand {
        value,
        unit: o.unit.clone(),
        dtype,
        name: None,
    }
}

pub(crate) fn mult_result_type(t1: i64, t2: i64) -> i64 {
    if t1 & dt::ROWVECTOR != 0 && t2 & dt::COLUMNVECTOR != 0 {
        dt::RATIONAL
    } else if t1 & dt::MATRIX != 0 && t2 & dt::COLUMNVECTOR != 0 {
        t2
    } else if t1 & dt::ROWVECTOR != 0 && t2 & dt::MATRIX != 0 {
        t1
    } else {
        dt::MATRIX + dt::RATIONAL
    }
}

/// `[start:step:end...]`
pub(crate) fn operand_from_range(range: &V) -> Operand {
    let r = range.arr();
    let start = at(r, 0).r();
    let mut step = at(r, 1).r();
    let end = at(r, 2).r();
    if step.is_zero() {
        return err0("ZERO_STEP");
    }
    if !end.subtract(&start).sign().are_equal(&step.sign()) {
        step = step.negate();
    }
    let count = end.subtract(&start).divide(&step).abs().to_f64();
    if count.is_nan() || count >= MAX_ELEMENTS as f64 {
        return crate::error::err_msg("Error. The range has too many elements.");
    }
    let mut array = Vec::new();
    let mut j = start.clone();
    let ascending = end.greater_than(&start);
    while if ascending {
        j.less_than(&end)
    } else {
        j.greater_than_or_equal(&end)
    } {
        if array.len() >= MAX_ELEMENTS {
            return crate::error::err_msg("Error. The range has too many elements.");
        }
        array.push(V::R(j.clone()));
        j = j.add(&step);
    }
    let last_equal = array.last().is_some_and(|l| l.r().are_equal(&end));
    if !last_equal {
        array.push(V::R(end));
    }
    Operand::new(V::A(array), OUnit::zeros(), dt::RATIONAL + dt::COLUMNVECTOR)
}

/// Build a vector or matrix from the top of the stack (`matrix numRows numCols`).
pub(crate) fn operand_from_token_stack(
    stack: &mut Vec<Operand>,
    num_rows: usize,
    num_cols: usize,
) -> Operand {
    let unit_for = |dtype: i64| {
        if dtype & dt::RATIONAL != 0 {
            OUnit::zeros()
        } else {
            OUnit::Null
        }
    };
    if num_rows == 0 && num_cols == 0 {
        return Operand::new(V::A(Vec::new()), OUnit::Null, dt::COLUMNVECTOR);
    }
    if num_rows == 1 && num_cols == 1 {
        return stack.pop().unwrap_or_else(|| err0("ERROR"));
    }
    if stack.len() < num_rows * num_cols {
        return err0("ERROR");
    }
    if num_rows == 1 || num_cols == 1 {
        let num_args = num_rows.max(num_cols);
        let mut dtype = stack[stack.len() - 1].dtype;
        if num_rows == 1 && dtype & dt::COLUMNVECTOR != 0 {
            // Column vectors side by side. Hurmet bug, reproduced: it indexes the stack from
            // the bottom (`tokenStack[0]`, `tokenStack[j]`), not from the first argument.
            dtype = dtype - dt::COLUMNVECTOR + dt::MATRIX;
            let n = stack[0].value.len();
            let mut array = Vec::with_capacity(n);
            for i in 0..n {
                let mut row = Vec::with_capacity(num_args);
                for j in 0..num_args {
                    row.push(stack.get(j).map_or(V::Undef, |t| t.value.get(i)));
                }
                array.push(V::A(row));
            }
            for _ in 0..num_args {
                stack.pop();
            }
            return Operand::new(V::A(array), unit_for(dtype), dtype);
        }
        dtype += if num_rows == 1 {
            dt::ROWVECTOR
        } else {
            dt::COLUMNVECTOR
        };
        let mut array = vec![V::Undef; num_args];
        for j in (0..num_args).rev() {
            let Some(token) = stack.pop() else {
                return err0("ERROR");
            };
            if token.unit.obj().is_some_and(|u| u.is_converted) {
                return err0("UNIT_IN_MAT");
            }
            array[j] = token.value;
        }
        return Operand::new(V::A(array), unit_for(dtype), dtype);
    }
    let dtype = stack[stack.len() - 1].dtype + dt::MATRIX;
    let mut array = vec![vec![V::Undef; num_cols]; num_rows];
    for k in (0..num_rows).rev() {
        for j in (0..num_cols).rev() {
            let Some(token) = stack.pop() else {
                return err0("ERROR");
            };
            if token.unit.obj().is_some_and(|u| u.is_converted) {
                return err0("UNIT_IN_MAT");
            }
            array[k][j] = token.value;
        }
    }
    Operand::new(
        V::A(array.into_iter().map(V::A).collect()),
        unit_for(dtype),
        dtype,
    )
}

pub(crate) fn filled(m: usize, n: usize, x: Rational) -> Operand {
    if m == 1 || n == 1 {
        let dtype = dt::RATIONAL
            + if m == 1 {
                dt::ROWVECTOR
            } else {
                dt::COLUMNVECTOR
            };
        // Hurmet bug, reproduced: the length is n even for a column vector.
        Operand::new(V::A(vec![V::R(x); n]), OUnit::Bare, dtype)
    } else {
        let row = V::A(vec![V::R(x); n]);
        Operand::new(
            V::A(vec![row; m]),
            OUnit::zeros(),
            dt::RATIONAL + dt::MATRIX,
        )
    }
}
