// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Data frames and maps (Hurmet's `dataframe.js` and `map.js`).
//!
//! A data frame is read from TSV between double backticks. Its cells stay strings until they
//! are accessed. If the TSV has no unit row and all its data columns have one type, it is a
//! *map* instead, whose cells are values and which arithmetic can operate on.

use crate::complex;
use crate::error::{err, err0};
use crate::eval::Ctx;
use crate::format::{Mode, fmt, formatted_decimal};
use crate::matrix;
use crate::value::{Frame, OUnit, Operand, UnitObj, V, dt, is_matrix};
use cruncha_lang::hurmet::{add_text_escapes, tabless_trim, unit_tex_from_string};
use cruncha_units::{ALL_ZEROS, NUMBER_PATTERN, Rational};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

/// Hurmet's data frame `numberRegEx`: `^(?:=|<number>$)`.
pub(crate) fn is_number_cell(s: &str) -> bool {
    if s.starts_with('=') {
        return true;
    }
    match cruncha_units::rational::number_regex_captures(s) {
        Some(c) => c[0].as_deref().map_or(0, str::len) == s.len(),
        None => false,
    }
}

static MIXED_FRACTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-?(?:[0-9]+(?: [0-9]+/[0-9]+))$").unwrap());
static INTERPOLATE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\$\{[^}\s]+\}").unwrap());
static IDENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^{}{}′*$",
        cruncha_lang::hurmet::IDENTIFIER_START,
        cruncha_lang::hurmet::IDENTIFIER_BODY
    ))
    .unwrap()
});
#[allow(dead_code)]
static NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(NUMBER_PATTERN).unwrap());

pub(crate) fn is_valid_identifier(s: &str) -> bool {
    IDENT_RE.is_match(s)
}

fn value_from_datum(datum: &V) -> V {
    match datum {
        V::S(s) if s == "true" => V::Bool(true),
        V::S(s) if s == "false" => V::Bool(false),
        V::S(s) if is_number_cell(s) => V::R(Rational::from_str_or_zero(s)),
        V::S(s) if s.is_empty() => V::Undef,
        other => other.clone(),
    }
}

fn datum_from_value(value: &V, dtype: i64, format_spec: &str) -> V {
    match value {
        V::Bool(true) => V::S("true".into()),
        V::Bool(false) => V::S("false".into()),
        V::Undef => V::S(String::new()),
        V::R(r) if dtype == dt::RATIONAL => V::S(fmt(r, format_spec, "1000000.", Mode::Tex)),
        other => other.clone(),
    }
}

fn frame_of(o: &Operand) -> Option<&Arc<Frame>> {
    match &o.value {
        V::Df(f) => Some(f),
        _ => None,
    }
}

fn to_int(v: &V) -> i64 {
    let x = v.r().to_f64();
    if x.is_finite() { x as i64 } else { 0 }
}

type RangeSpec = (Vec<String>, Vec<Option<i64>>, i64, i64);

/// `identifyRange`: which rows and columns `df[args…]` selects.
fn identify_range(df: &Operand, args: &[Operand]) -> Result<RangeSpec, Operand> {
    let Some(f) = frame_of(df) else {
        return Err(err0("ERROR"));
    };
    let col_idx = |name: &str| f.column_map.get(name).map(|&i| i as i64);
    let num_rows = f.data.first().map_or(0, Vec::len) as i64;
    let num_cols = f.data.len() as i64;
    let in_rows = |name: &str| {
        f.row_map
            .as_ref()
            .and_then(|m| m.get(name))
            .map(|&i| i as i64)
    };
    let mut row_list: Vec<String> = Vec::new();
    let mut column_list: Vec<Option<i64>> = Vec::new();
    let (mut i_start, mut i_end) = (0, 0);
    if num_rows == 1 {
        // A single-row data frame: each argument calls a column.
        for a in args {
            if a.dtype == dt::STRING {
                column_list.push(col_idx(a.value.str()));
            } else if a.dtype == dt::RATIONAL {
                // Hurmet does not subtract 1 here (unlike every other numeric index).
                column_list.push(Some(to_int(&a.value)));
            } else if a.dtype == dt::RANGE {
                let r = a.value.arr();
                let js = to_int(&r.first().cloned().unwrap_or_default()) - 1;
                let je = to_int(&r.get(2).cloned().unwrap_or_default()) - 1;
                for j in js..=je.min(js + 100_000) {
                    column_list.push(Some(j));
                }
            }
        }
    } else if args.len() == 1 && args[0].dtype == dt::RATIONAL {
        i_end = num_rows - 1;
        column_list.push(Some(to_int(&args[0].value) - 1));
    } else if args.len() == 1 && args[0].dtype == dt::RANGE {
        let r = args[0].value.arr();
        let js = to_int(&r.first().cloned().unwrap_or_default()) - 1;
        let je = to_int(&r.get(2).cloned().unwrap_or_default()) - 1;
        for j in js..=je.min(js + 100_000) {
            column_list.push(Some(j));
        }
        i_end = num_rows - 1;
    } else if args.len() == 1 && args[0].dtype == dt::STRING {
        let key = args[0].value.str();
        if let Some(row) = in_rows(key) {
            i_start = row;
            i_end = row;
            column_list = (0..num_cols).map(Some).collect();
        } else if let Some(c) = col_idx(key) {
            i_end = num_rows - 1;
            column_list.push(Some(c));
        } else if f.row_map.is_none() {
            // Without row names, the key can only have meant a column.
            return Err(err("BAD_COLUMN_NAME", key));
        } else {
            return Err(err("BAD_ROW_NAME", key));
        }
    } else if args.len() == 1 && args[0].dtype == dt::STRING + dt::COLUMNVECTOR {
        for name in args[0].value.arr() {
            row_list.push(name.str().to_string());
        }
        column_list = (0..num_cols).map(Some).collect();
    } else if args.len() == 1 && args[0].dtype == dt::STRING + dt::ROWVECTOR {
        // Columns by name (ADR-0022 item 14; Hurmet read `df.columnIndicator`, which doesn't
        // exist, so every index was undefined).
        i_end = num_rows;
        for name in args[0].value.arr() {
            match col_idx(name.str()) {
                Some(c) => column_list.push(Some(c)),
                None => return Err(err("BAD_COLUMN_NAME", name.str())),
            }
        }
    } else if args.len() == 2 {
        let a0 = &args[0];
        if a0.dtype == dt::STRING {
            if let Some(row) = in_rows(a0.value.str()) {
                i_start = row;
                i_end = row;
            } else if a0.value.str() == "end" {
                i_start = num_rows - 1;
                i_end = i_start;
            } else {
                // A column name first: every row (ADR-0022 item 14; Hurmet used the number
                // of columns here).
                i_start = 0;
                i_end = num_rows - 1;
            }
        } else if a0.dtype == dt::RATIONAL {
            i_start = to_int(&a0.value) - 1;
            i_end = i_start;
        } else if a0.dtype == dt::RANGE {
            let r = a0.value.arr();
            i_start = to_int(&r.first().cloned().unwrap_or_default()) - 1;
            i_end = to_int(&r.get(2).cloned().unwrap_or_default()) - 1;
        }
        if a0.dtype == dt::STRING
            && let Some(c) = col_idx(a0.value.str())
        {
            column_list.push(Some(c));
        }
        let a1 = &args[1];
        if a1.dtype == dt::STRING && col_idx(a1.value.str()).is_some() {
            column_list.push(col_idx(a1.value.str()));
        } else if a1.dtype == dt::STRING && a1.value.str() == "end" {
            column_list.push(Some(f.headings.len() as i64 - 1));
        } else if a1.dtype == dt::RATIONAL {
            column_list.push(Some(to_int(&a1.value) - 1));
        } else if a1.dtype == dt::RANGE {
            // A range of columns (ADR-0022 item 14; Hurmet looked up `columnMap[NaN]`).
            let r = a1.value.arr();
            let js = to_int(&r.first().cloned().unwrap_or_default()) - 1;
            let je = to_int(&r.get(2).cloned().unwrap_or_default()) - 1;
            for j in js..=je.min(js + 100_000) {
                column_list.push(Some(j));
            }
        }
    } else if !args.is_empty() {
        // A list of row names or column names.
        i_end = args.len() as i64;
        let last = args[args.len() - 1].value.str();
        if in_rows(last).is_some_and(|r| r != 0) {
            row_list = args.iter().map(|a| a.value.str().to_string()).collect();
            column_list = (0..num_cols).map(Some).collect();
        } else {
            column_list = args.iter().map(|a| col_idx(a.value.str())).collect();
        }
    }
    Ok((row_list, column_list, i_start, i_end))
}

fn col(data: &[Vec<V>], j: Option<i64>) -> &[V] {
    match j {
        Some(j) if j >= 0 => data.get(j as usize).map_or(&[], Vec::as_slice),
        _ => &[],
    }
}

fn cell(data: &[Vec<V>], j: Option<i64>, i: i64) -> V {
    if i < 0 {
        return V::Undef;
    }
    col(data, j).get(i as usize).cloned().unwrap_or(V::Undef)
}

fn slice(c: &[V], start: i64, end_inclusive: i64) -> Vec<V> {
    let s = start.max(0) as usize;
    let e = ((end_inclusive + 1).max(0) as usize).min(c.len());
    if s >= e { Vec::new() } else { c[s..e].to_vec() }
}

fn heading(f: &Frame, j: Option<i64>) -> String {
    match j {
        Some(j) if j >= 0 => f
            .headings
            .get(j as usize)
            .cloned()
            .unwrap_or_else(|| "undefined".into()),
        _ => "undefined".into(),
    }
}

/// `DataFrame.range(df, args, unitAware)`.
pub(crate) fn range(df: &Operand, args: &[Operand], unit_aware: bool, ctx: &Ctx) -> Operand {
    let (row_list, column_list, i_start, i_end) = match identify_range(df, args) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let Some(f) = frame_of(df) else {
        return err0("ERROR");
    };
    let dtypes = f.dtype.clone().unwrap_or_default();
    let units = f.units.clone().unwrap_or_default();
    let dtype_of =
        |j: Option<i64>| -> i64 { j.and_then(|j| dtypes.get(j as usize).copied()).unwrap_or(0) };
    let unit_name = |j: Option<i64>| -> Option<String> {
        match j.and_then(|j| units.get(j as usize)) {
            Some(V::S(s)) if !s.is_empty() => Some(s.clone()),
            _ => None,
        }
    };
    let frame_units = match &df.unit {
        OUnit::Frame(m) => Some(m),
        _ => None,
    };
    if row_list.is_empty() && i_start == i_end && column_list.len() == 1 {
        let j = column_list[0];
        let mut dtype = dtype_of(j);
        if dtype & dt::QUANTITY != 0 {
            dtype -= dt::QUANTITY;
        }
        let mut value = value_from_datum(&cell(&f.data, j, i_start));
        let mut unit = if dtype & dt::RATIONAL != 0 {
            OUnit::zeros()
        } else {
            OUnit::expos_none()
        };
        if unit_aware && let Some(name) = unit_name(j) {
            match ctx.unit(&name) {
                Ok(u) => {
                    value = V::R(value.r().add(&u.gauge).multiply(&u.factor));
                    unit = OUnit::expos(u.expos);
                }
                Err(e) => return e,
            }
        }
        return Operand::new(value, unit, dtype);
    }
    if column_list.len() == 1 {
        let j = column_list[0];
        let unit = unit_name(j)
            .and_then(|n| frame_units.and_then(|m| m.get(&n)).cloned())
            .map(|u| OUnit::Obj(Box::new(u)))
            .unwrap_or_else(OUnit::expos_none);
        let mut value: Vec<V> = slice(col(&f.data, j), i_start, i_end)
            .iter()
            .map(value_from_datum)
            .collect();
        let mut dtype = dtype_of(j) + dt::COLUMNVECTOR;
        if dtype & dt::QUANTITY != 0 {
            dtype -= dt::QUANTITY;
        }
        if dtype & dt::RATIONAL != 0
            && unit_aware
            && let OUnit::Obj(u) = &unit
            && let (Some(g), Some(fa)) = (&u.gauge, &u.factor)
        {
            value = value
                .iter()
                .map(|e| V::R(e.r().add(g).multiply(fa)))
                .collect();
        }
        let mut o = Operand::new(V::A(value), unit, dtype);
        o.name = Some(heading(f, j));
        return o;
    }
    let mut out = Frame {
        row_map: if row_list.is_empty() {
            None
        } else {
            Some(HashMap::new())
        },
        units: Some(Vec::new()),
        dtype: Some(Vec::new()),
        ..Default::default()
    };
    let mut unit_map: HashMap<String, UnitObj> = HashMap::new();
    for (k, &j) in column_list.iter().enumerate() {
        out.headings.push(heading(f, j));
        let un = j
            .and_then(|j| units.get(j as usize).cloned())
            .unwrap_or(V::Undef);
        if let V::S(name) = &un
            && !name.is_empty()
            && !unit_map.contains_key(name)
            && let Some(u) = frame_units.and_then(|m| m.get(name))
        {
            unit_map.insert(name.clone(), u.clone());
        }
        if let Some(u) = out.units.as_mut() {
            u.push(un);
        }
        if let Some(d) = out.dtype.as_mut() {
            d.push(dtype_of(j));
        }
        // Keyed by the selected column's heading (ADR-0022 item 14; Hurmet used the k-th
        // heading of the source).
        out.column_map.insert(heading(f, j), k);
        if !row_list.is_empty() {
            let mut elements = Vec::new();
            for (i, row_name) in row_list.iter().enumerate() {
                let r = f
                    .row_map
                    .as_ref()
                    .and_then(|m| m.get(row_name))
                    .map_or(-1, |&r| r as i64);
                elements.push(cell(&f.data, j, r));
                if let Some(m) = out.row_map.as_mut() {
                    m.insert(row_name.clone(), i);
                }
            }
            out.data.push(elements);
        } else {
            out.data.push(slice(col(&f.data, j), i_start, i_end));
        }
    }
    Operand::new(V::Df(Arc::new(out)), OUnit::Frame(unit_map), dt::DATAFRAME)
}

/// `map.range(map, keys)`.
pub(crate) fn map_range(map: &Operand, keys: &[Operand]) -> Operand {
    let (row_list, column_list, i_start, i_end) = match identify_range(map, keys) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let Some(f) = frame_of(map) else {
        return err0("ERROR");
    };
    let mut unit = map.unit.clone();
    if row_list.is_empty() && i_start == i_end && column_list.len() == 1 {
        let value = cell(&f.data, column_list[0], i_start);
        return Operand::new(value, unit, map.dtype - dt::MAP);
    }
    if column_list.len() == 1 {
        let j = column_list[0];
        let value = slice(col(&f.data, j), i_start, i_end);
        let dtype = if j == Some(0) {
            dt::COLUMNVECTOR
                + if matches!(value.first(), Some(V::S(_))) {
                    dt::STRING
                } else {
                    map.dtype - dt::MAP
                }
        } else {
            map.dtype - dt::MAP + dt::COLUMNVECTOR
        };
        if j == Some(-1) {
            unit = OUnit::Null;
        }
        return Operand::new(V::A(value), unit, dtype);
    }
    let mut out = Frame {
        row_map: if row_list.is_empty() {
            None
        } else {
            Some(HashMap::new())
        },
        ..Default::default()
    };
    for (k, &j) in column_list.iter().enumerate() {
        out.headings.push(heading(f, j));
        // Keyed by the selected column's heading, as in `range` (ADR-0022 item 14).
        out.column_map.insert(heading(f, j), k);
        if !row_list.is_empty() {
            let mut elements = Vec::new();
            for (i, row_name) in row_list.iter().enumerate() {
                let r = f
                    .row_map
                    .as_ref()
                    .and_then(|m| m.get(row_name))
                    .map_or(-1, |&r| r as i64);
                elements.push(cell(&f.data, j, r));
                if let Some(m) = out.row_map.as_mut() {
                    m.insert(row_name.clone(), i);
                }
            }
            out.data.push(elements);
        } else {
            out.data.push(slice(col(&f.data, j), i_start, i_end));
        }
    }
    Operand::new(V::Df(Arc::new(out)), unit, map.dtype)
}

fn has_unit_row(lines: &[&str]) -> bool {
    if lines.len() < 3 {
        return false;
    }
    if lines[1].split('\t').map(str::trim).any(is_number_cell) {
        return false;
    }
    lines[2].split('\t').map(str::trim).any(is_number_cell)
}

/// Resolves `${name}` in a TSV to the variable's value.
pub(crate) type Interpolate<'a> = &'a dyn Fn(&str) -> Result<String, Operand>;

/// `DataFrame.dataFrameFromTSV(str, vars)`. `interpolate` resolves `${name}`.
pub(crate) fn from_tsv(input: &str, interpolate: Option<Interpolate>, ctx: &Ctx) -> Operand {
    let mut s = input.strip_prefix('`').unwrap_or(input).to_string();
    if let Some(interp) = interpolate {
        let matches: Vec<(usize, usize)> = INTERPOLATE_RE
            .find_iter(&s)
            .map(|m| (m.start(), m.end()))
            .collect();
        for &(a, b) in matches.iter().rev() {
            let name = cruncha_lang::hurmet::check_for_numeric_subscript(&s[a + 2..b - 1]);
            let value = match interp(&name) {
                Ok(v) => v,
                Err(e) => return e,
            };
            s.replace_range(a..b, &value);
        }
    }
    let lines: Vec<&str> = s
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let got_units = has_unit_row(&lines);
    let mut frame = Frame {
        units: Some(Vec::new()),
        dtype: Some(Vec::new()),
        ..Default::default()
    };
    let mut cols: Vec<String> = lines[0].split('\t').map(str::to_string).collect();
    if cols[0].starts_with('#') {
        frame.row_map = Some(HashMap::new());
        cols[0] = cols[0][1..].to_string();
    } else if cols[0].starts_with("\\#") {
        cols[0] = cols[0][1..].to_string();
    }
    for (c, datum) in cols.iter().enumerate() {
        let d = datum.trim().to_string();
        frame.column_map.insert(d.clone(), c);
        frame.headings.push(d);
        frame.data.push(Vec::new());
    }
    let mut unit_map: HashMap<String, UnitObj> = HashMap::new();
    let mut units: Vec<V> = Vec::new();
    if got_units {
        for name in lines[1].split('\t') {
            let name = name.trim().to_string();
            if !name.is_empty() && !unit_map.contains_key(&name) {
                // Hurmet stores whatever unitFromUnitName returns, an error object included.
                let u = ctx
                    .unit(&name)
                    .map(|u| UnitObj::from_unit(&u))
                    .unwrap_or_default();
                unit_map.insert(name.clone(), u);
            }
            units.push(V::S(name));
        }
    }
    let mut row: usize = 0;
    let start = if got_units { 2 } else { 1 };
    for (k, line) in lines.iter().enumerate().skip(start) {
        let r = k - start;
        row = r;
        for (c, datum) in line.split('\t').enumerate() {
            let mut datum = datum.trim().to_string();
            if datum == "=sum(up)" {
                let mut sum = Rational::zero();
                if let Some(column) = frame.data.get(c) {
                    for num in column {
                        if !cruncha_units::jsnum::parse_number(num.str()).is_nan() {
                            sum = sum.add(&Rational::from_str_or_zero(num.str()));
                        }
                    }
                }
                datum = cruncha_units::jsnum::to_string(sum.to_f64());
            }
            if c >= frame.data.len() {
                // JavaScript would throw a TypeError on the missing column.
                return err0("ERROR");
            }
            if c == 0
                && let Some(m) = frame.row_map.as_mut()
            {
                m.insert(datum.clone(), r);
            }
            frame.data[c].push(V::S(datum));
        }
    }
    let _ = row;
    // Each column's type is that of its first non-empty datum.
    let mut dtypes: Vec<i64> = Vec::new();
    let nrows = frame.data.first().map_or(0, Vec::len);
    for (j, column) in frame.data.iter().enumerate() {
        for i in 0..nrows {
            let datum = column.get(i).map_or("", V::str);
            if datum.is_empty() {
                continue;
            }
            let has_unit = units.get(j).is_some_and(|u| !u.str().is_empty());
            dtypes.push(if is_number_cell(datum) {
                dt::RATIONAL
                    + if !units.is_empty() && has_unit {
                        dt::QUANTITY
                    } else {
                        0
                    }
            } else if datum == "true" || datum == "false" {
                dt::BOOLEAN
            } else {
                dt::STRING
            });
            break;
        }
    }
    let mut is_map = false;
    let mut i_start = 0;
    if unit_map.is_empty() {
        is_map = true;
        i_start = if frame.row_map.is_some() { 1 } else { 0 };
        for i in i_start + 1..dtypes.len() {
            if dtypes[i] != dtypes[i_start] {
                is_map = false;
                break;
            }
        }
    }
    if is_map {
        let base = dtypes.get(i_start).copied().unwrap_or(0);
        if base == dt::RATIONAL {
            for (i, column) in frame.data.iter_mut().enumerate() {
                if dtypes.get(i) == Some(&dt::RATIONAL) {
                    for e in column.iter_mut() {
                        *e = V::R(Rational::from_str_or_zero(e.str()));
                    }
                }
            }
        }
        frame.units = None;
        frame.dtype = None;
        let unit = if dtypes.first() == Some(&dt::RATIONAL) {
            OUnit::zeros()
        } else {
            OUnit::Null
        };
        Operand::new(V::Df(Arc::new(frame)), unit, dt::MAP + base)
    } else {
        frame.units = Some(units);
        frame.dtype = Some(dtypes);
        Operand::new(
            V::Df(Arc::new(frame)),
            OUnit::Frame(unit_map),
            dt::DATAFRAME,
        )
    }
}

/// `DataFrame.dataFrameFromVectors`, for `dataframe(v1, v2, …)`.
pub(crate) fn from_vectors(vectors: &[Operand], format_spec: &str, ctx: &Ctx) -> Operand {
    let mut frame = Frame {
        units: Some(Vec::new()),
        dtype: Some(Vec::new()),
        ..Default::default()
    };
    let first_is_name = vectors.first().and_then(|v| v.name.as_deref()) == Some("name");
    if first_is_name {
        frame.row_map = Some(HashMap::new());
    }
    let mut unit_map: HashMap<String, UnitObj> = HashMap::new();
    for (j, v) in vectors.iter().enumerate() {
        let vector_type = if v.dtype & dt::ROWVECTOR != 0 {
            dt::ROWVECTOR
        } else if v.dtype & dt::COLUMNVECTOR != 0 {
            dt::COLUMNVECTOR
        } else {
            return err("NOT_VECTOR", "dataframe");
        };
        let name = v.name.clone().unwrap_or_else(|| "undefined".into());
        frame.headings.push(name.clone());
        frame.column_map.insert(name, j);
        let col_dtype = v.dtype - vector_type;
        frame.data.push(
            v.value
                .arr()
                .iter()
                .map(|e| datum_from_value(e, col_dtype, format_spec))
                .collect(),
        );
        if let Some(d) = frame.dtype.as_mut() {
            d.push(col_dtype);
        }
        match v.unit.name() {
            Some(n) if !n.is_empty() => {
                if let Some(u) = frame.units.as_mut() {
                    u.push(V::S(n.to_string()));
                }
                if !unit_map.contains_key(n) {
                    let u = ctx
                        .unit(n)
                        .map(|u| UnitObj::from_unit(&u))
                        .unwrap_or_default();
                    unit_map.insert(n.to_string(), u);
                }
            }
            _ => {
                if let Some(u) = frame.units.as_mut() {
                    u.push(V::Null);
                }
            }
        }
        if first_is_name {
            for (i, n) in vectors[0].value.arr().iter().enumerate() {
                if let Some(m) = frame.row_map.as_mut() {
                    m.insert(crate::matrix::js_string(n), i);
                }
            }
        }
    }
    Operand::new(
        V::Df(Arc::new(frame)),
        OUnit::Frame(unit_map),
        dt::DATAFRAME,
    )
}

/// `DataFrame.matrix2table`.
pub(crate) fn matrix2table(m: &Operand, headings: &[V], row_headings: &[V]) -> Operand {
    let mut headings: Vec<String> = headings.iter().map(crate::matrix::js_string).collect();
    if !row_headings.is_empty() {
        headings.insert(0, String::new());
    }
    let mut frame = Frame::default();
    for (i, h) in headings.iter().enumerate() {
        frame.column_map.insert(h.clone(), i);
    }
    if !row_headings.is_empty() {
        let mut rm = HashMap::new();
        for (i, h) in row_headings.iter().enumerate() {
            rm.insert(crate::matrix::js_string(h), i);
        }
        frame.row_map = Some(rm);
    }
    let rows = m.value.arr();
    let ncols = rows.first().map_or(0, V::len);
    let delta = if row_headings.is_empty() { 0 } else { 1 };
    let mut data: Vec<Vec<V>> = vec![Vec::new(); headings.len().max(ncols + delta)];
    if delta == 1 {
        data[0] = row_headings.to_vec();
    }
    for j in 0..ncols {
        data[j + delta] = rows.iter().map(|r| r.get(j)).collect();
    }
    frame.data = data;
    frame.headings = headings;
    Operand::new(
        V::Df(Arc::new(frame)),
        m.unit.clone(),
        m.dtype - dt::MATRIX + dt::MAP,
    )
}

/// `DataFrame.append(o1, o2, formatSpec, unitAware)`: add a column to a data frame.
pub(crate) fn append(
    o1: &Operand,
    o2: &Operand,
    format_spec: &str,
    unit_aware: bool,
    ctx: &Ctx,
) -> Operand {
    let o1_is_df = o1.dtype == dt::DATAFRAME;
    let (df, addend) = if o1_is_df { (o1, o2) } else { (o2, o1) };
    let Some(f) = frame_of(df) else {
        return err0("ERROR");
    };
    let mut fr = (**f).clone();
    let mut unit_map = match &df.unit {
        OUnit::Frame(m) => m.clone(),
        _ => HashMap::new(),
    };
    let name = addend.name.clone().unwrap_or_else(|| "undefined".into());
    if o1_is_df {
        fr.column_map.insert(name.clone(), fr.headings.len());
        fr.headings.push(name);
    } else {
        for v in fr.column_map.values_mut() {
            *v += 1;
        }
        fr.column_map.insert(name.clone(), 0);
        fr.headings.insert(0, name);
    }
    let mut unit = None;
    if let Some(n) = addend.unit.name().filter(|n| !n.is_empty()) {
        let units = fr.units.get_or_insert_with(Vec::new);
        if o1_is_df {
            units.push(V::S(n.to_string()));
        } else {
            units.insert(0, V::S(n.to_string()));
        }
        let u = ctx.unit(n).ok();
        if !unit_map.contains_key(n) {
            unit_map.insert(
                n.to_string(),
                u.as_ref().map(UnitObj::from_unit).unwrap_or_default(),
            );
        }
        unit = u;
    }
    // Hurmet tests `addend.dype` (a typo), so the first branch never applies.
    let dtype = if !is_matrix(addend.dtype) {
        addend.dtype
    } else if addend.dtype & dt::COLUMNVECTOR != 0 {
        addend.dtype - dt::COLUMNVECTOR
    } else {
        addend.dtype - dt::ROWVECTOR
    };
    let num_rows = fr.data.first().map_or(0, Vec::len);
    let new_col: Vec<V> = if num_rows == 1 && !is_matrix(addend.dtype) {
        let v = match &unit {
            Some(u) if unit_aware && dtype == dt::RATIONAL => {
                V::R(addend.value.r().divide(&u.factor).subtract(&u.gauge))
            }
            _ => addend.value.clone(),
        };
        vec![datum_from_value(&v, dtype, format_spec)]
    } else if let (Some(u), true) = (&unit, unit_aware && dtype == dt::RATIONAL) {
        let v = matrix::convert_from_base_units(&addend.value, addend.dtype, &u.gauge, &u.factor);
        v.arr()
            .iter()
            .map(|e| datum_from_value(e, dtype, format_spec))
            .collect()
    } else {
        addend
            .value
            .arr()
            .iter()
            .map(|e| datum_from_value(e, dtype, format_spec))
            .collect()
    };
    if o1_is_df {
        fr.data.push(new_col);
        fr.dtype.get_or_insert_with(Vec::new).push(dtype);
    } else {
        fr.data.insert(0, new_col);
        fr.dtype.get_or_insert_with(Vec::new).insert(0, dtype);
    }
    Operand::new(V::Df(Arc::new(fr)), OUnit::Frame(unit_map), dt::DATAFRAME)
}

// ---------------------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------------------

static ACCENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([^\x{0300}-\x{0308}\x{030A}\x{030C}\x{0332}\x{20d0}\x{20d1}\x{20d6}\x{20d7}\x{20e1}]+)([\x{0300}-\x{0308}\x{030A}\x{030C}\x{0332}\x{20d0}\x{20d1}\x{20d6}\x{20d7}\x{20e1}])(.+)?").unwrap()
});
static SUBSCRIPT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([^_]+)(_[^']+)?(.*)?").unwrap());

fn accent_from_char(c: &str) -> &'static str {
    match c {
        "\u{0300}" => "\\grave",
        "\u{0301}" => "\\acute",
        "\u{0302}" => "\\hat",
        "\u{0303}" => "\\tilde",
        "\u{0304}" | "\u{0305}" => "\\bar",
        "\u{0307}" => "\\dot",
        "\u{0308}" => "\\ddot",
        "\u{030A}" => "\\mathring",
        "\u{030C}" => "\\check",
        "\u{0332}" => "\\underline",
        "\u{20d0}" => "\\overleftharpoon",
        "\u{20d1}" => "\\overrightharpoon",
        "\u{20d6}" => "\\overleftarrow",
        "\u{20d7}" => "\\vec",
        "\u{20e1}" => "\\overleftrightarrow",
        _ => "undefined",
    }
}

/// `formatColumnName`.
pub(crate) fn format_column_name(s: &str) -> String {
    if !is_valid_identifier(s) {
        return format!("\\text{{{}}}", add_text_escapes(s));
    }
    let s = s.replace('′', "'");
    if let Some(c) = ACCENT_RE.captures(&s) {
        return format!(
            "{}{{{}}}{}",
            accent_from_char(&c[2]),
            &c[1],
            c.get(3).map_or("", |m| m.as_str())
        );
    }
    match SUBSCRIPT_RE.captures(&s) {
        Some(c) => {
            let p1 = &c[1];
            let mut result = if p1.chars().count() > 1 {
                format!("\\text{{{p1}}}")
            } else {
                p1.to_string()
            };
            if let Some(p2) = c.get(2) {
                result.push_str(&format!("_\\text{{{}}}", &p2.as_str()[1..]));
            }
            result + c.get(3).map_or("", |m| m.as_str())
        }
        None => s,
    }
}

fn is_not_empty(units: Option<&Vec<V>>) -> bool {
    match units {
        None => false,
        Some(u) => u
            .iter()
            // An undefined unit (a selection from a frame without a units row) is empty too;
            // Hurmet printed a row of "undefined" (ADR-0022 item 14).
            .any(|x| !matches!(x, V::Null | V::Undef) && !(matches!(x, V::S(s) if s.is_empty()))),
    }
}

fn js_str(v: &V) -> String {
    crate::matrix::js_string(v)
}

struct ColInfo {
    has_align: bool,
    max_before: usize,
    max_after: usize,
}

/// `DataFrame.display`: TeX.
#[allow(clippy::needless_range_loop)] // Mirrors Hurmet's column-major loops.
pub(crate) fn display(
    f: &Frame,
    format_spec: &str,
    decimal_format: &str,
    omit_heading: bool,
) -> String {
    let data = &f.data;
    if data.is_empty() {
        return String::new();
    }
    let num_rows = data[0].len();
    let num_cols = data.len();
    let write_row_nums = num_rows > 5 && f.row_map.is_none();
    let is_map = f.dtype.is_none();
    let mut s =
        String::from("\\renewcommand{\\arraycolsep}{8pt}\\renewcommand{\\arraystretch}{1.15}");
    s.push_str("\\begin{array}{");
    s.push_str(if f.row_map.is_some() {
        "l|"
    } else if write_row_nums {
        "r|"
    } else {
        ""
    });
    for _ in 0..num_cols {
        s.push('c');
    }
    s.push('}');
    if !omit_heading {
        if write_row_nums {
            s.push('&');
        }
        for j in 0..num_cols {
            let h = f
                .headings
                .get(j)
                .cloned()
                .unwrap_or_else(|| "undefined".into());
            s.push_str(&format!("{{{}}}&", format_column_name(&h)));
        }
        s.pop();
        s.push_str(" \\\\ ");
    }
    if is_not_empty(f.units.as_ref()) {
        if write_row_nums {
            s.push('&');
        }
        let units = f.units.as_ref().map_or(&[][..], Vec::as_slice);
        for j in 0..num_cols {
            match units.get(j) {
                Some(V::S(u)) if !u.is_empty() => {
                    s.push_str(&unit_tex_from_string(u).replacen("\\;\\, ", "", 1));
                }
                _ => {}
            }
            s.push('&');
        }
        s.pop();
        s.push_str(" \\\\ ");
    }
    s.push_str("\\hline ");

    // Decimal alignment info (data frames only; map cells are not strings).
    let dtypes = f.dtype.clone().unwrap_or_default();
    let mut col_info: Vec<Option<ColInfo>> = Vec::new();
    let mut cell_info: Vec<Vec<(bool, usize, usize)>> = Vec::new();
    for j in 0..num_cols {
        if !is_map && dtypes.get(j).is_some_and(|d| d & dt::RATIONAL != 0) {
            let mut ci = ColInfo {
                has_align: false,
                max_before: 0,
                max_after: 0,
            };
            let mut cells = Vec::new();
            for i in 0..num_rows {
                let datum = js_str(&data[j].get(i).cloned().unwrap_or_default());
                let len = datum.chars().count();
                let pos = datum.find('.').map(|p| datum[..p].chars().count());
                let (has, before, after) = match pos {
                    Some(p) => (true, p, len - p - 1),
                    None => (false, len, 0),
                };
                cells.push((has, before, after));
                if has {
                    ci.has_align = true;
                    ci.max_after = ci.max_after.max(after);
                }
                ci.max_before = ci.max_before.max(before);
            }
            col_info.push(Some(ci));
            cell_info.push(cells);
        } else {
            col_info.push(None);
            cell_info.push(Vec::new());
        }
    }
    let total_re = |v: &V| {
        let t = js_str(v).to_lowercase();
        t.starts_with("total") || t.starts_with("sum")
    };
    for i in 0..num_rows {
        if i == num_rows - 1 && total_re(&data[0][i]) {
            s.push_str("\\hline ");
        }
        if write_row_nums {
            s.push_str(&format!("{} & ", i + 1));
        }
        for j in 0..num_cols {
            let datum = data[j].get(i).cloned().unwrap_or_default();
            if is_map {
                match &datum {
                    V::Undef => s.push_str(" & "),
                    V::R(r) => s.push_str(&format!(
                        "{}&",
                        fmt(r, format_spec, decimal_format, Mode::Tex)
                    )),
                    V::C(z) => s.push_str(&format!(
                        "{}&",
                        complex::display(z, format_spec, decimal_format).0
                    )),
                    other => {
                        s.push_str(&format!("\\text{{{}}} &", add_text_escapes(&js_str(other))))
                    }
                }
            } else {
                let d = js_str(&datum);
                if MIXED_FRACTION_RE.is_match(&d) {
                    s.push_str(&format!(
                        "{}&",
                        fmt(
                            &Rational::from_str_or_zero(&d),
                            format_spec,
                            decimal_format,
                            Mode::Tex
                        )
                    ));
                } else if is_number_cell(&d) {
                    let (ci, cell) = match (&col_info[j], cell_info[j].get(i)) {
                        (Some(ci), Some(c)) => (ci, *c),
                        _ => {
                            s.push_str(&format!(
                                "{}&",
                                formatted_decimal(&d, decimal_format, false)
                            ));
                            continue;
                        }
                    };
                    let mut t = String::new();
                    let m = ci.max_before.saturating_sub(cell.1);
                    if m > 0 {
                        t.push_str(&format!("\\phantom{{{}}}", "0".repeat(m)));
                    }
                    t.push_str(&formatted_decimal(&d, decimal_format, false));
                    let n = ci.max_after.saturating_sub(cell.2);
                    if ci.has_align && (n > 0 || !cell.0) {
                        t.push_str("\\phantom{");
                        if !cell.0 {
                            t.push(if decimal_format.ends_with('.') {
                                '.'
                            } else {
                                ','
                            });
                        }
                        if n > 0 {
                            t.push_str(&"0".repeat(n));
                        }
                        t.push('}');
                    }
                    s.push_str(&t);
                    s.push('&');
                } else if d.is_empty() {
                    s.push('&');
                } else {
                    s.push_str(&format!("\\text{{{}}}&", add_text_escapes(&d)));
                }
            }
        }
        s.pop();
        s.push_str(" \\\\ ");
    }
    for _ in 0..3 {
        s.pop();
    }
    let mut s = s.trim().to_string();
    s.push_str("\\end{array}");
    s
}

/// `DataFrame.displayAlt`: plain text, as a TSV between double backticks.
#[allow(clippy::needless_range_loop)] // Mirrors Hurmet's column-major loops.
pub(crate) fn display_alt(
    f: &Frame,
    format_spec: &str,
    decimal_format: &str,
    omit_heading: bool,
) -> String {
    let data = &f.data;
    if data.is_empty() {
        return String::new();
    }
    let num_rows = data[0].len();
    let num_cols = data.len();
    let write_row_nums = num_rows > 5 && f.row_map.is_none();
    let mut s = String::from("``");
    if !omit_heading {
        if write_row_nums {
            s.push('\t');
        }
        let h0 = f
            .headings
            .first()
            .cloned()
            .unwrap_or_else(|| "undefined".into());
        s.push_str(if h0 == "name" || h0 == "item" {
            ""
        } else {
            &h0
        });
        s.push('\t');
        for j in 1..num_cols {
            s.push_str(f.headings.get(j).map_or("undefined", String::as_str));
            s.push('\t');
        }
        s.pop();
        s.push('\n');
    }
    if is_not_empty(f.units.as_ref()) {
        if write_row_nums {
            s.push('\t');
        }
        let units = f.units.as_ref().map_or(&[][..], Vec::as_slice);
        for j in 0..num_cols {
            s.push_str(&js_str(&units.get(j).cloned().unwrap_or_default()));
            s.push('\t');
        }
        s.pop();
        s.push('\n');
    }
    let is_map = f.dtype.is_none();
    for i in 0..num_rows {
        if write_row_nums {
            s.push_str(&format!("{}\t", i + 1));
        }
        for j in 0..num_cols {
            let datum = data[j].get(i).cloned().unwrap_or_default();
            if is_map {
                match &datum {
                    V::Undef => s.push('\t'),
                    V::R(r) => s.push_str(&format!(
                        "{}\t",
                        fmt(r, format_spec, decimal_format, Mode::Tex)
                    )),
                    V::C(z) => s.push_str(&format!(
                        "{}\t",
                        complex::display(z, format_spec, decimal_format).1
                    )),
                    other => s.push_str(&format!("{}\t", js_str(other))),
                }
            } else {
                let d = js_str(&datum);
                if MIXED_FRACTION_RE.is_match(&d) {
                    s.push_str(&format!(
                        "{}\t",
                        fmt(
                            &Rational::from_str_or_zero(&d),
                            format_spec,
                            "100000.",
                            Mode::Tex
                        )
                    ));
                } else {
                    s.push_str(&d);
                    s.push('\t');
                }
            }
        }
        s.pop();
        s.push('\n');
    }
    s.pop();
    let mut s = s.trim().to_string();
    s.push_str("``");
    s
}

/// Convert a map's numeric columns to or from base units.
pub(crate) fn map_convert(
    data: &[Vec<V>],
    gauge: &Rational,
    factor: &Rational,
    to_base: bool,
) -> Vec<Vec<V>> {
    data.iter()
        .map(|column| {
            if !matches!(column.first(), Some(V::R(_))) {
                return column.clone();
            }
            column
                .iter()
                .map(|e| {
                    let r = e.r();
                    V::R(if to_base {
                        r.add(gauge).multiply(factor)
                    } else {
                        let x = r.divide(factor);
                        if gauge.is_zero() {
                            x
                        } else {
                            x.subtract(gauge)
                        }
                    })
                })
                .collect()
        })
        .collect()
}

pub(crate) fn tsv_trim(s: &str) -> &str {
    tabless_trim(s)
}

impl OUnit {
    /// `{ expos: null }`
    pub(crate) fn expos_none() -> OUnit {
        OUnit::Obj(Box::default())
    }
}

#[allow(dead_code)]
pub(crate) fn zeros() -> cruncha_units::Dims {
    ALL_ZEROS
}
