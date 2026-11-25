// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Literal values on the right of an assignment (Hurmet's `literal.js`, `valueFromLiteral`):
//! numbers with units, strings, booleans, matrices, data frames, complex numbers and dates.

use crate::dataframe;
use crate::eval::{Ctx, Scope, eval_rpn};
use crate::format::validate_format_spec;
use crate::matrix;
use crate::value::{OUnit, Operand, V, cpx, dt};
use cruncha_lang::hurmet::unit_tex_from_string;
use cruncha_units::Rational;
use regex::Regex;
use std::sync::LazyLock;

static MATRIX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^[(\[] *(?:(?:-?[0-9.]+|"(?:[^"]|"")*"|true|false) *[,;\t]? *)+[)\]]"#).unwrap()
});
static UNIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zäöōA-Z0-9 μµ#$£¥'′″°Δ√()Å₨₪€℃℉ΩΩKÅ^+\-/*.•×\-−·₂⁰¹²³\x{2074}-\x{2079}]+$")
        .unwrap()
});
const NUM: &str =
    r"(-?(?:0x[0-9A-Fa-f]+|[0-9]+(?: [0-9]+/[0-9]+|(?:\.[0-9]+)?(?:e[+-]?[0-9]+|%)?)))";
const NON_NEG: &str = r"(0x[0-9A-Fa-f]+|[0-9]+(?: [0-9]+/[0-9]+|(?:\.[0-9]+)?(?:e[+-]?[0-9]+|%)?))";
pub(crate) static COMPLEX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^{NUM}(?: *([+-]) *(?: j *{NON_NEG}|{NON_NEG} *∠{NUM}(°)?))"
    ))
    .unwrap()
});
static DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^'(\d{4})-(\d{1,2})-(\d{1,2})'$").unwrap());
static DISTRIBUTION_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[({\[].* to ").unwrap());

/// (value, unit, dtype, TeX display). An error has dtype `ERROR` and the message (or "") as
/// display.
pub(crate) type Literal = (V, OUnit, i64, String);

fn error(display: impl Into<String>) -> Literal {
    (V::Num(0.0), OUnit::Null, dt::ERROR, display.into())
}

pub(crate) fn tex(s: &str, ctx: &Ctx) -> String {
    let opts = lang_opts(ctx);
    cruncha_lang::parse_tex(s, &opts)
        .unwrap_or_else(|_| format!("\\text{{{}}}", cruncha_lang::hurmet::add_text_escapes(s)))
}

pub(crate) fn lang_opts(ctx: &Ctx) -> cruncha_lang::Options {
    cruncha_lang::Options {
        formats: cruncha_lang::Formats {
            decimal_format: ctx.decimal_format.to_string(),
            date_format: ctx.date_format.map(str::to_string),
        },
        ..Default::default()
    }
}

fn eval_literal(s: &str, ctx: &Ctx) -> Option<(String, Operand)> {
    let calc = cruncha_lang::parse_calc(s, &lang_opts(ctx)).ok()?;
    let mut scope = Scope::empty();
    let o = eval_rpn(&calc.rpn, &mut scope, ctx, false, None);
    Some((calc.tex, o))
}

fn unit_from_string(s: &str, ctx: &Ctx) -> Result<(OUnit, String), Operand> {
    if s.is_empty() {
        return Ok((OUnit::Null, String::new()));
    }
    let name = s.replace('\'', "");
    let name = name.trim();
    let u = ctx.unit(name)?;
    Ok((OUnit::from_unit(&u), unit_tex_from_string(name)))
}

fn literal_with_unit(o: Operand, tex: String, unit_str: &str, ctx: &Ctx) -> Literal {
    let mut unit = if o.dtype & dt::RATIONAL != 0 {
        OUnit::zeros()
    } else {
        OUnit::Null
    };
    let mut display = String::new();
    let mut value = o.value.clone();
    if !unit_str.is_empty() {
        match unit_from_string(unit_str, ctx) {
            Ok((u, d)) => {
                unit = u;
                display = d;
            }
            Err(e) => return error(e.error_message()),
        }
        let (gauge, factor) = match unit.obj() {
            Some(u) => (u.gauge(), u.factor()),
            None => (Rational::zero(), Rational::one()),
        };
        let base = if o.dtype == dt::RATIONAL {
            V::R(o.value.r().add(&gauge).multiply(&factor))
        } else {
            matrix::convert_to_base_units(&o.value, o.dtype, &gauge, &factor)
        };
        value = V::Q(Box::new((o.value, base)));
    }
    if !display.is_empty() {
        (
            value,
            unit,
            o.dtype + dt::QUANTITY,
            format!("{tex}\\,{display}"),
        )
    } else {
        (value, unit, o.dtype, tex)
    }
}

/// Hurmet's `valueFromLiteral(str, name, formats)`.
pub(crate) fn value_from_literal(s: &str, name: &str, ctx: &Ctx) -> Literal {
    if DISTRIBUTION_RE.is_match(s) {
        return error("");
    }
    if s == "true" || s == "false" {
        return (
            V::Bool(s == "true"),
            OUnit::Null,
            dt::BOOLEAN,
            format!("\\mathord{{\\text{{{s}}}}}"),
        );
    }
    if s.len() >= 6 && s.starts_with("'''") && s.ends_with("'''") {
        let body = &s[3..s.len() - 3];
        let body = body
            .strip_prefix("\r\n")
            .or_else(|| body.strip_prefix('\n'))
            .unwrap_or(body);
        let body = body
            .strip_suffix("\r\n")
            .or_else(|| body.strip_suffix('\n'))
            .unwrap_or(body);
        return (
            V::S(body.to_string()),
            OUnit::Null,
            dt::MACRO,
            String::new(),
        );
    }
    // /^\x22.+\x22/
    if s.starts_with('"') && s.len() >= 3 && s[1..].char_indices().any(|(k, c)| c == '"' && k >= 1)
    {
        let inner = &s[1..s.len() - 1];
        if name == "format" {
            let spec = inner.replace("\"\"", "\"");
            let spec = spec.trim();
            let (dtype, display) = validate_format_spec(spec);
            return (V::S(spec.to_string()), OUnit::Null, dtype, display);
        }
        return (
            V::S(inner.replace("\"\"", "\"")),
            OUnit::Null,
            dt::STRING,
            tex(s, ctx),
        );
    }
    if let Some(m) = MATRIX_RE.find(s) {
        let matrix_str = m.as_str();
        let Some((tex, o)) = eval_literal(matrix_str, ctx) else {
            return error("");
        };
        let unit_str = s[matrix_str.len()..].trim();
        if !unit_str.is_empty() && !UNIT_RE.is_match(unit_str) {
            return error("");
        }
        if o.is_error() {
            return error(o.error_message());
        }
        return literal_with_unit(o, tex, unit_str, ctx);
    }
    if let Some(rest) = s.strip_prefix("``") {
        let pos = rest.find("``").map_or(s.len(), |p| p + 2);
        let tsv = dataframe::tsv_trim(&s[2..pos.min(s.len())]);
        let o = dataframe::from_tsv(tsv, None, ctx);
        if o.is_error() {
            return error(o.error_message());
        }
        let V::Df(f) = &o.value else { return error("") };
        let display = dataframe::display(f, "h3", ctx.decimal_format, false);
        if o.dtype == dt::DATAFRAME {
            return (o.value.clone(), o.unit, dt::DATAFRAME, display);
        }
        // A map.
        let unit_str = s.get(pos + 2..).unwrap_or("").trim();
        let mut dtype = o.dtype;
        let mut unit = OUnit::Null;
        let mut unit_display = String::new();
        let mut value = o.value.clone();
        if !unit_str.is_empty() {
            match unit_from_string(unit_str, ctx) {
                Ok((u, d)) => {
                    let (g, fa) = u.obj().map_or((Rational::zero(), Rational::one()), |x| {
                        (x.gauge(), x.factor())
                    });
                    let mut fr = (**f).clone();
                    fr.data_base = Some(dataframe::map_convert(&fr.data, &g, &fa, true));
                    value = V::Df(std::sync::Arc::new(fr));
                    unit = u;
                    unit_display = d;
                    dtype = dt::MAP + dt::RATIONAL + dt::QUANTITY;
                }
                Err(_) => return error(""),
            }
        }
        return (value, unit, dtype, format!("{display}\\;{unit_display}"));
    }
    if let Some(c) = COMPLEX_RE.captures(s) {
        let display = tex(s, ctx);
        let g = |k: usize| c.get(k).map(|m| m.as_str());
        let (re, im) = if let Some(b) = g(3) {
            let re = Rational::from_str_or_zero(g(1).unwrap_or("0"));
            let mut im = Rational::from_str_or_zero(b);
            if g(2) == Some("-") {
                im = im.negate();
            }
            (re, im)
        } else {
            let r = Rational::from_str_or_zero(g(4).unwrap_or("0"));
            let mut theta = Rational::from_str_or_zero(g(5).unwrap_or("0"));
            if g(6).is_some() {
                theta = theta.multiply(&Rational::pi()).divide(&Rational::int(180));
            }
            let t = theta.to_f64();
            (
                r.multiply(&Rational::from_number(cruncha_units::jsmath::cos(t))),
                r.multiply(&Rational::from_number(cruncha_units::jsmath::sin(t))),
            )
        };
        // Hurmet's unit here is the bare exponent array; compile() wraps it as { expos }.
        return (cpx(re, im), OUnit::zeros(), dt::COMPLEX, display);
    }
    if DATE_RE.is_match(s) {
        let Some(secs) = cruncha_lang::hurmet::seconds_from_iso_string(s) else {
            return error("");
        };
        let value = V::R(Rational::int(secs));
        let mut e = cruncha_units::ALL_ZEROS;
        e[2] = 1.0;
        let display = crate::result::format_date(&value.r(), ctx.date_format);
        return (value, OUnit::expos(e), dt::DATE, display);
    }
    if let Some(caps) = cruncha_units::rational::number_regex_captures(s) {
        let num_str = caps[0].clone().unwrap_or_default();
        let unit_str = s[num_str.len()..].trim();
        if !unit_str.is_empty() && !UNIT_RE.is_match(unit_str) {
            return error("");
        }
        let Some((tex, o)) = eval_literal(&num_str, ctx) else {
            return error("");
        };
        if o.is_error() {
            return error(o.error_message());
        }
        return literal_with_unit(o, tex, unit_str, ctx);
    }
    error("")
}
