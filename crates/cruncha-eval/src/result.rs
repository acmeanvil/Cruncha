// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Results: unit checks and conversion (`evaluate.js`: `conditionResult`, `errorResult`),
//! formatting for display (`result.js`: `formatResult`, `writeSavedResult`), tuples
//! (`tuple.js`) and dates (`date.js`).

use crate::attrs::{Attrs, Names};
use crate::complex;
use crate::dataframe;
use crate::error::err0;
use crate::eval::Ctx;
use crate::eval::normalize_value;
use crate::format::{Mode, fmt, format};
use crate::literal::tex;
use crate::matrix;
use crate::value::{OUnit, Operand, UnitObj, V, dt, is_matrix, is_vector};
use cruncha_lang::hurmet::{add_text_escapes, unit_tex_from_string};
use cruncha_units::{ALL_ZEROS, Rational, units_are_compatible};
use std::sync::Arc;

/// JavaScript `String.prototype.replace(pattern, replacement)` for one match at
/// `start..end`, including the `$$`, `$&`, `` $` `` and `$'` patterns.
pub(crate) fn js_replace_at(s: &str, start: usize, end: usize, replacement: &str) -> String {
    let mut rep = String::new();
    let mut chars = replacement.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('$') => {
                    rep.push('$');
                    chars.next();
                }
                Some('&') => {
                    rep.push_str(&s[start..end]);
                    chars.next();
                }
                Some('`') => {
                    rep.push_str(&s[..start]);
                    chars.next();
                }
                Some('\'') => {
                    rep.push_str(&s[end..]);
                    chars.next();
                }
                _ => rep.push('$'),
            }
        } else {
            rep.push(c);
        }
    }
    format!("{}{rep}{}", &s[..start], &s[end..])
}

/// `s.replace(/@@?/, rep)`.
fn replace_at_selector(s: &str, rep: &str) -> String {
    match s.find('@') {
        Some(p) => {
            let end = if s[p + 1..].starts_with('@') {
                p + 2
            } else {
                p + 1
            };
            js_replace_at(s, p, end, rep)
        }
        None => s.to_string(),
    }
}

/// `s.replace(/[?%] *[?%]|[?%]/, rep)`.
fn replace_q_selector(s: &str, rep: &str) -> String {
    let Some(p) = s.find(['?', '%']) else {
        return s.to_string();
    };
    let rest = &s[p + 1..];
    let spaces = rest.len() - rest.trim_start_matches(' ').len();
    let end = if rest[spaces..].starts_with(['?', '%']) {
        p + 1 + spaces + 1
    } else {
        p + 1
    };
    js_replace_at(s, p, end, rep)
}

/// `s.replace(/ *\\⦂/, "")`.
fn strip_percent_placeholder(s: &str) -> String {
    match s.find("\\⦂") {
        Some(p) => {
            let start = s[..p].trim_end_matches(' ').len();
            format!("{}{}", &s[..start], &s[p + "\\⦂".len()..])
        }
        None => s.to_string(),
    }
}

// ---------------------------------------------------------------------------------------
// Dates
// ---------------------------------------------------------------------------------------

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn ymd(value: &Rational) -> Option<(i64, i64, i64)> {
    let secs = value.to_f64();
    let ms = (secs * 1000.0).trunc();
    if !ms.is_finite() || ms.abs() > 8.64e15 {
        return None;
    }
    Some(civil_from_days((ms / 86_400_000.0).floor() as i64))
}

/// `isoDateString`.
pub(crate) fn iso_date_string(value: &Rational) -> String {
    match ymd(value) {
        Some((y, m, d)) if (0..=9999).contains(&y) => format!("{y:04}-{m:02}-{d:02}"),
        Some((y, m, d)) => format!(
            "{}{:06}-{m:02}-{d:02}",
            if y < 0 { "-" } else { "+" },
            y.abs()
        ),
        None => "Invalid Date".into(),
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// `plainDate`: a date in the document's date format. Only English month names are ported
/// (Hurmet uses `Intl.DateTimeFormat` for the language in the format).
pub(crate) fn plain_date(value: &Rational, spec: Option<&str>) -> String {
    let Some(spec) = spec.filter(|s| *s != "yyyy-mm-dd") else {
        return iso_date_string(value);
    };
    static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"^(?:(w{3,4}), )?(y{1,4}|m{3,4}|d{1,2})([- .]| de )(m{2,4}|d{1,2})([- .]|, | de )(y{4}|d{2})(?: ([a-z]{2,3}))?$").unwrap()
    });
    let (Some(c), Some((y, m, d))) = (RE.captures(spec), ymd(value)) else {
        return iso_date_string(value);
    };
    let part = |p: &str| -> String {
        match p {
            "d" => d.to_string(),
            "dd" => format!("{d:02}"),
            "mm" => format!("{m:02}"),
            "mmm" => MONTHS[(m - 1) as usize][..3].to_string(),
            "mmmm" => MONTHS[(m - 1) as usize].to_string(),
            _ => y.to_string(),
        }
    };
    let mut s = String::new();
    if let Some(w) = c.get(1) {
        // Hurmet compares `match[1].length === "3"` (always false), so the weekday is long.
        let _ = w;
        let days = [
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
        ];
        let dn = (value.to_f64() / 86_400.0).floor() as i64;
        s.push_str(days[dn.rem_euclid(7) as usize]);
        s.push_str(", ");
    }
    s.push_str(&part(&c[2]));
    s.push_str(&c[3]);
    s.push_str(&part(&c[4]));
    s.push_str(&c[5]);
    s.push_str(&part(&c[6]));
    s
}

/// `formatDate`: TeX for a date result.
pub(crate) fn format_date(value: &Rational, spec: Option<&str>) -> String {
    format!(
        "\\class{{date-result}}{{\\text{{{}}}}}",
        plain_date(value, spec)
    )
}

// ---------------------------------------------------------------------------------------
// Tuples
// ---------------------------------------------------------------------------------------

fn tuple_display(t: &[(String, Operand)], spec: &str, df: &str) -> String {
    if t.is_empty() {
        return String::new();
    }
    let mut s = String::from("\\begin{array}{c}");
    if t.iter().any(|(_, o)| o.unit.name().is_some()) {
        let mut row = String::new();
        for (_, o) in t {
            if let Some(n) = o.unit.name() {
                row.push_str(&unit_tex_from_string(n).replacen("\\;\\, ", "", 1));
            }
            row.push('&');
        }
        row.pop();
        s.push_str(&row);
        s.push_str(" \\\\ \\hline ");
    }
    let mut bot = String::new();
    for (_, o) in t {
        bot.push_str(&fmt(&o.value.r(), spec, df, Mode::Tex));
        bot.push_str(" & ");
    }
    bot.pop();
    s.push_str(&bot);
    s.push_str("\\end{array}");
    s
}

fn tuple_display_alt(t: &[(String, Operand)], spec: &str) -> String {
    if t.is_empty() {
        return String::new();
    }
    let mut s = String::from("``");
    if t.iter().any(|(_, o)| o.unit.name().is_some()) {
        let mut row = String::new();
        for (_, o) in t {
            if let Some(n) = o.unit.name() {
                row.push_str(n);
            }
            row.push('\t');
        }
        row.pop();
        s.push_str(&row);
        s.push('\n');
    }
    let mut bot = String::new();
    for (_, o) in t {
        bot.push_str(&fmt(&o.value.r(), spec, "100000.", Mode::Tex));
        bot.push('\t');
    }
    bot.pop();
    s.push_str(&bot);
    s + "``"
}

// ---------------------------------------------------------------------------------------
// conditionResult / errorResult
// ---------------------------------------------------------------------------------------

/// `writeSavedResult`: the Markdown for a saved document, and the display selector.
pub(crate) fn write_saved_result(stmt: &mut Attrs, saved: &str) {
    let template = stmt.resulttemplate.clone().unwrap_or_default();
    if template.contains('@') {
        stmt.display_selector = if stmt.altresulttemplate.contains("@@") {
            "@@"
        } else {
            "@"
        }
        .into();
        if stmt.entry.starts_with("@test ") || stmt.entry.starts_with("@@test ") {
            stmt.md = Some(format!("{} 〔{saved}〕", stmt.entry));
        } else if let Some(pos) = stmt.entry.rfind(stmt.display_selector.as_str()) {
            let after = &stmt.entry[pos + stmt.display_selector.len()..];
            stmt.md = Some(format!("{}〔{saved}〕{after}", &stmt.entry[..pos]));
        }
    } else if template.contains('?') || template.contains('%') {
        let ch = if template.contains('?') { "?" } else { "%" };
        let double = format!("{ch}{ch}");
        stmt.display_selector = if stmt.altresulttemplate.contains(&double) {
            double
        } else {
            ch.to_string()
        };
        if let Some(pos) = stmt.alt.rfind(stmt.display_selector.as_str()) {
            let after = &stmt.alt[pos + stmt.display_selector.len()..];
            stmt.md = Some(format!("{}〔{saved}〕{after}", &stmt.alt[..pos]));
        }
    }
    if let Some(md) = &stmt.md {
        stmt.md = Some(strip_percent_placeholder(md));
    }
}

/// `errorResult(stmt, result)`.
pub(crate) fn error_result(stmt: &mut Attrs, result: &Operand) {
    let msg = result.error_message().to_string();
    stmt.value = V::Null;
    stmt.result_display_err = format!(
        "\\textcolor{{firebrick}}{{\\text{{{}}}}}",
        msg.replace('%', "\\%")
    );
    stmt.alt_result_display = msg.clone();
    stmt.error = true;
    stmt.dtype = dt::ERROR;
    let template = stmt.resulttemplate.clone().unwrap_or_default();
    if !template.contains('!') {
        write_saved_result(stmt, &msg);
    }
    if template.contains('!') {
        stmt.tex.push_str("= ");
        stmt.tex.push_str(&stmt.result_display_err.clone());
        stmt.alt.push_str(&msg);
    } else if template.contains('@') {
        stmt.tex = replace_at_selector(&template, &stmt.result_display_err);
        stmt.alt = replace_at_selector(&stmt.altresulttemplate, &msg);
    } else {
        stmt.tex = replace_q_selector(&stmt.tex, &stmt.result_display_err);
        stmt.alt = replace_q_selector(&stmt.alt, &msg);
    }
}

fn unit_obj(u: &OUnit) -> Option<&UnitObj> {
    u.obj()
}

/// `conditionResult(stmt, oprnd, unitAware)`: check the result's unit against the requested
/// display unit and convert. Returns `None` after reporting an error into `stmt`.
pub(crate) fn condition_result(
    stmt: &mut Attrs,
    oprnd: Operand,
    unit_aware: bool,
) -> Option<Operand> {
    let mut result = oprnd;
    if result.dtype == dt::COMPLEX {
        let z = result.value.cpx();
        if z[1].is_zero() {
            result.value = V::R(z[0].clone());
            result.dtype = 1;
        }
    }
    let rd_has_bang = stmt.resultdisplay.contains('!');
    let stmt_expos = stmt.unit.get_expos().copied();
    if result.dtype != dt::ERROR
        && unit_aware
        && !rd_has_bang
        && (stmt_expos.is_some() || result.unit.get_expos().is_some())
    {
        let mut time = ALL_ZEROS;
        time[2] = 1.0;
        let expos = if result.dtype == dt::DATE && !stmt.unit.truthy() {
            time
        } else {
            stmt_expos.unwrap_or(ALL_ZEROS)
        };
        if !units_are_compatible(result.unit.get_expos(), Some(&expos)) {
            let code = if stmt_expos.is_some() {
                "UNIT_RES"
            } else {
                "UNIT_MISS"
            };
            result = err0(code);
        }
    }
    if result.dtype == dt::ERROR {
        error_result(stmt, &result);
        return None;
    }
    let rt_has_bang = stmt
        .resulttemplate
        .as_deref()
        .is_some_and(|t| t.contains('!'));
    if rt_has_bang
        && !(result.dtype == dt::DATAFRAME
            || result.dtype & dt::MAP != 0
            || is_matrix(result.dtype)
            || result.dtype & dt::TUPLE != 0)
    {
        error_result(stmt, &err0("BAD_DISPLAY"));
        return None;
    }
    if result.dtype & dt::RATIONAL != 0 {
        result.value = normalize_value(&result.value, result.dtype);
    } else if result.dtype == dt::COMPLEX {
        let z = result.value.cpx();
        result.value = crate::value::cpx(z[0].normalize(), z[1].normalize());
    }
    stmt.dtype = result.dtype;

    let su = unit_obj(&stmt.unit).cloned();
    // `stmt.unit.gauge` is an array (truthy even when zero), so any parsed unit counts.
    let unit_in_spec = su.as_ref().is_some_and(|u| {
        u.factor
            .as_ref()
            .is_some_and(|f| !f.are_equal(&Rational::one()) || u.gauge.is_some())
    });
    let is_percent = stmt.unit.name() == Some("\\⦂");
    if result.dtype & dt::DATAFRAME != 0 || stmt.resultdisplay.contains('!') {
        stmt.unit = result.unit.clone();
    } else if unit_aware && result.dtype & dt::RATIONAL != 0 && !is_percent {
        if !unit_in_spec && units_are_compatible(result.unit.get_expos(), Some(&ALL_ZEROS)) {
            stmt.unit = OUnit::Obj(Box::new(UnitObj {
                factor: Some(Rational::one()),
                gauge: Some(Rational::zero()),
                expos: Some(ALL_ZEROS),
                ..Default::default()
            }));
        }
        let (gauge, factor) = match unit_obj(&stmt.unit) {
            Some(u) => (u.gauge(), u.factor()),
            None => (Rational::zero(), Rational::one()),
        };
        if result.dtype & dt::MAP != 0 {
            if let V::Df(f) = &result.value {
                let mut fr = (**f).clone();
                let base = fr.data.clone();
                fr.data = dataframe::map_convert(&base, &gauge, &factor, false);
                fr.data_base = Some(base);
                result.value = V::Df(Arc::new(fr));
            }
        } else {
            let plain = if is_matrix(result.dtype) {
                matrix::convert_from_base_units(&result.value, result.dtype, &gauge, &factor)
            } else {
                V::R(result.value.r().divide(&factor).subtract(&gauge))
            };
            result.value = V::Q(Box::new((plain, result.value.clone())));
        }
        stmt.dtype += dt::QUANTITY;
        stmt.expos = result.unit.get_expos().copied();
    } else if unit_in_spec && !is_percent {
        let (gauge, factor) = su
            .as_ref()
            .map_or((Rational::zero(), Rational::one()), |u| {
                (u.gauge(), u.factor())
            });
        if result.dtype & dt::MAP != 0 {
            if let V::Df(f) = &result.value {
                let mut fr = (**f).clone();
                fr.data_base = Some(dataframe::map_convert(&fr.data, &gauge, &factor, true));
                result.value = V::Df(Arc::new(fr));
            }
        } else {
            let base = if is_matrix(result.dtype) {
                matrix::convert_to_base_units(&result.value, result.dtype, &gauge, &factor)
            } else {
                V::R(result.value.r().add(&gauge).multiply(&factor))
            };
            result.value = V::Q(Box::new((result.value.clone(), base)));
        }
        stmt.dtype += dt::QUANTITY;
    } else if result.dtype & (dt::RATIONAL | dt::COMPLEX) != 0 && !is_percent {
        stmt.unit = OUnit::zeros();
    }
    stmt.value = result.value.clone();
    Some(result)
}

// ---------------------------------------------------------------------------------------
// formatResult
// ---------------------------------------------------------------------------------------

fn test_value(o: &Operand) -> bool {
    if is_vector(o.dtype) {
        o.value.arr().iter().all(V::truthy)
    } else if is_matrix(o.dtype) {
        o.value.arr().iter().all(|r| r.arr().iter().all(V::truthy))
    } else if o.dtype & dt::MAP != 0 {
        match &o.value {
            V::Df(f) => f.data.iter().all(|c| c.iter().all(V::truthy)),
            _ => true,
        }
    } else {
        o.value.truthy()
    }
}

fn comparison_at_end(rpn: &str) -> Option<&str> {
    // / ([⩵≠><>≤≥∋∈∉∌⊂⊃⊄⊅]|==|in|!in|!=|=>|<=)$/
    let pos = rpn.rfind('\u{a0}')?;
    let op = &rpn[pos + '\u{a0}'.len_utf8()..];
    let single = op.chars().count() == 1 && "⩵≠><>≤≥∋∈∉∌⊂⊃⊄⊅".contains(op);
    if single || matches!(op, "==" | "in" | "!in" | "!=" | "=>" | "<=") {
        Some(op)
    } else {
        None
    }
}

fn negated(op: &str) -> (&'static str, &'static str) {
    match op {
        "⩵" | "==" => ("≠", "≠"),
        "≠" | "!=" => ("==", "=="),
        ">" => ("\\ngtr ", "!>"),
        "<" => ("\\nless ", "!<"),
        "≤" => ("\\nleq ", "!≤"),
        "≥" | "=>" | "<=" => ("\\ngeq ", "!≥"),
        "∋" => ("∌", "∌"),
        "∈" | "in" => ("∉", "∉"),
        "⊂" => ("⊄", "⊄"),
        "⊃" => ("⊅", "⊅"),
        "∉" => ("∈", "∈"),
        "∌" => ("∋", "∋"),
        "⊄" => ("⊂", "⊂"),
        "⊅" => ("⊃", "⊃"),
        "!in" => ("in", "in"),
        _ => ("", ""),
    }
}

fn strip_test_prefix(s: &str) -> &str {
    s.strip_prefix("@@test ")
        .or_else(|| s.strip_prefix("@test "))
        .unwrap_or(s)
}

/// `formatResult(stmt, result, formatSpec, formats, assert, isUnitAware)`.
pub(crate) fn format_result(
    stmt: &mut Attrs,
    result: &Operand,
    spec: &str,
    ctx: &Ctx,
    assert: Option<&str>,
    unit_aware: bool,
) {
    let df = ctx.decimal_format;
    let template = stmt.resulttemplate.clone().unwrap_or_default();
    let is_percent = stmt.unit.name() == Some("\\⦂");
    let num_names = match &stmt.name {
        Names::None => 0,
        Names::One(_) => 1,
        Names::Many(v) => v.len(),
    };
    let times100 = |r: &Rational| r.multiply(&Rational::int(100));
    if template.contains(['?', '!', '@', '%']) {
        stmt.value = result.value.clone();
        if template.contains('!') {
            return;
        }
        let (mut rd, mut alt): (String, String);
        let mut md: Option<String> = None;
        let is_test = stmt.entry.starts_with("@test ") || stmt.entry.starts_with("@@test ");
        let comparison = stmt
            .rpn
            .as_deref()
            .and_then(comparison_at_end)
            .map(str::to_string);
        if result.dtype & dt::BOOLEAN != 0 && is_test && comparison.is_some() {
            let body = strip_test_prefix(&stmt.entry).to_string();
            if test_value(result) {
                rd = tex(&body, ctx) + ",\\text{ ok }✓";
                alt = format!("{body}, ok ✓");
            } else {
                let op = comparison.unwrap_or_default();
                let (neg_tex, neg_alt) = negated(&op);
                if let Some(a) = assert {
                    let a = a.strip_suffix('.').unwrap_or(a);
                    let addendum = body.replacen(&op, neg_tex, 1);
                    rd = format!("\\colorbox{{Salmon}}{{{a}, but ${}$}}", tex(&addendum, ctx));
                    alt = format!("{a}, but {}", body.replacen(&op, neg_alt, 1));
                } else {
                    rd = tex(&body.replacen(&op, neg_tex, 1), ctx) + ",\\colorbox{Salmon}{ n.g.}";
                    alt = format!("{}, n.g.", body.replacen(&op, neg_alt, 1));
                }
            }
        } else if is_matrix(result.dtype) {
            let plain_value = match &result.value {
                V::Q(q) => Some(q.0.clone()),
                _ => None,
            };
            let mvalue = if unit_aware || plain_value.is_some() {
                plain_value.unwrap_or(V::Undef)
            } else if is_percent && is_vector(result.dtype) {
                V::A(
                    result
                        .value
                        .arr()
                        .iter()
                        .map(|e| V::R(times100(&e.r())))
                        .collect(),
                )
            } else if is_percent {
                V::A(
                    result
                        .value
                        .arr()
                        .iter()
                        .map(|r| V::A(r.arr().iter().map(|e| V::R(times100(&e.r()))).collect()))
                        .collect(),
                )
            } else {
                result.value.clone()
            };
            rd = matrix::display(&mvalue, result.dtype, spec, df);
            alt = matrix::display_alt(&mvalue, result.dtype, spec, df, Mode::Plain, "");
            md = Some(matrix::display_alt(
                &mvalue,
                result.dtype,
                spec,
                df,
                Mode::Literal,
                if is_percent { "%" } else { "" },
            ));
            if is_percent {
                rd.push_str("\\%");
                alt.push('%');
            }
        } else if result.dtype == dt::DATAFRAME {
            let V::Df(f) = &result.value else { return };
            if num_names > 1 && num_names != f.data.len() {
                let m = "Error. Mismatch in number of multiple assignment.";
                rd = format!("\\textcolor{{firebrick}}{{\\text{{{m}}}}}");
                alt = m.to_string();
            } else {
                let omit = num_names > 1;
                rd = dataframe::display(f, spec, df, omit);
                // Hurmet passes omitHeading as the decimal format here (harmless).
                alt = dataframe::display_alt(f, spec, "1,000,000.", false);
            }
        } else if result.dtype & dt::MAP != 0 {
            let V::Df(f) = &result.value else { return };
            let local = if unit_aware || f.data_base.is_some() {
                let mut fr = (**f).clone();
                fr.data_base = None;
                fr
            } else {
                (**f).clone()
            };
            let omit = num_names > 1;
            rd = dataframe::display(&local, spec, df, omit);
            alt = dataframe::display_alt(&local, spec, df, omit);
        } else if result.dtype == dt::TUPLE {
            let V::Tuple(t) = &result.value else { return };
            if num_names > 1 && num_names != t.len() {
                let m = "Error. Mismatch in number of multiple assignment.";
                rd = format!("\\textcolor{{firebrick}}{{\\text{{{m}}}}}");
                alt = m.to_string();
            } else {
                rd = tuple_display(t, spec, df);
                alt = tuple_display_alt(t, spec);
            }
        } else if result.dtype & dt::STRING != 0 {
            let s = matrix::js_string(&result.value);
            rd = format!("\\text{{{}}}", add_text_escapes(&s));
            if let OUnit::Color(c) = &result.unit {
                rd = format!("\\textcolor{{{c}}}{{{rd}}}");
            }
            alt = s.clone();
            md = Some(format!("\"{}\"", s.replace('"', "\"\"")));
        } else if result.dtype & dt::RICHTEXT != 0 {
            let s = result.value.str().to_string();
            rd = tex(&s, ctx);
            alt = s;
        } else if result.dtype & dt::BOOLEAN != 0 {
            let s = matrix::js_string(&result.value);
            rd = format!("\\text{{{s}}}");
            alt = s;
        } else if result.dtype == dt::COMPLEX {
            let (a, b) = complex::display(&result.value.cpx(), spec, df);
            rd = a;
            alt = b;
        } else if result.dtype == dt::DATE {
            let r = result.value.r();
            rd = format_date(&r, ctx.date_format);
            alt = plain_date(&r, ctx.date_format);
            md = Some(format!("'{}'", iso_date_string(&r)));
        } else if let V::Q(q) = &result.value {
            let plain = q.0.r();
            match format(&plain, spec, df, Mode::Tex) {
                Ok(s) => {
                    rd = s;
                    alt = fmt(&plain, spec, df, Mode::Plain);
                    md = Some(fmt(&plain, spec, df, Mode::Literal));
                }
                Err(e) => {
                    // Hurmet reads `.value` of the TeX string it just built (undefined).
                    rd = format!(
                        "\\textcolor{{firebrick}}{{\\text{{{}}}}}",
                        e.error_message()
                    );
                    alt = "undefined".into();
                }
            }
        } else if let V::R(r) = &result.value {
            let value = if is_percent { times100(r) } else { r.clone() };
            match format(&value, spec, df, Mode::Tex) {
                Ok(s) => {
                    rd = if is_percent { s + "\\%" } else { s };
                    let pct = if is_percent { "%" } else { "" };
                    alt = fmt(&value, spec, df, Mode::Plain) + pct;
                    md = Some(fmt(&value, spec, df, Mode::Literal) + pct);
                }
                Err(e) => {
                    rd = format!(
                        "\\textcolor{{firebrick}}{{\\text{{{}}}}}",
                        e.error_message()
                    );
                    alt = "undefined".into();
                }
            }
        } else {
            rd = matrix::js_string(&result.value);
            alt = rd.clone();
        }
        let md = md.unwrap_or_else(|| alt.clone());

        // The string plugged into the echoes of dependent statements.
        if let Some((start, end)) = selector_span(&template) {
            stmt.resultdisplay = format!("{}{rd}{}", &template[..start], &template[end..]);
        }
        write_saved_result(stmt, &md);
        if template.contains('@') {
            stmt.tex = stmt.resultdisplay.clone();
            stmt.alt = replace_at_selector(&stmt.altresulttemplate, &alt);
        } else if template.contains('?') || template.contains('%') {
            let c = if template.contains('?') { '?' } else { '%' };
            if let Some(pos) = stmt.tex.rfind(c) {
                // Hurmet: tex.slice(0, pos).replace(/\? *$/, "") + resultDisplay + tex.slice(pos + 1)
                let head = &stmt.tex[..pos];
                let trimmed = head.trim_end_matches(' ');
                let head = match trimmed.strip_suffix(c) {
                    Some(h) => h,
                    None => head,
                };
                stmt.tex = format!("{head}{rd}{}", &stmt.tex[pos + 1..]);
            }
            let sel = stmt.display_selector.clone();
            if let Some(pos) = stmt.alt.rfind(sel.as_str()).filter(|_| !sel.is_empty()) {
                stmt.alt = format!("{}{alt}{}", &stmt.alt[..pos], &stmt.alt[pos + sel.len()..]);
            }
        }
    }
    stmt.tex = strip_percent_placeholder(&stmt.tex);
    stmt.alt = strip_percent_placeholder(&stmt.alt);
}

/// The span of `/(\? *\??|@ *@?|%%?)/` in a result template.
fn selector_span(t: &str) -> Option<(usize, usize)> {
    let p = t.find(['?', '@', '%'])?;
    let c = t[p..].chars().next()?;
    let rest = &t[p + 1..];
    let end = match c {
        '?' | '@' => {
            // `\? *\??`: the spaces are consumed even without a second selector character.
            let spaces = rest.len() - rest.trim_start_matches(' ').len();
            p + 1 + spaces + usize::from(rest[spaces..].starts_with(c))
        }
        _ => {
            if rest.starts_with('%') {
                p + 2
            } else {
                p + 1
            }
        }
    };
    Some((p, end))
}

/// The TeX of an operand, as `formatResult` writes it for the `@` selector.
pub(crate) fn operand_tex(o: &Operand, spec: &str, ctx: &Ctx) -> String {
    let mut a = Attrs {
        resulttemplate: Some("@".into()),
        altresulttemplate: "@".into(),
        ..Default::default()
    };
    format_result(&mut a, o, spec, ctx, None, false);
    a.resultdisplay
}
