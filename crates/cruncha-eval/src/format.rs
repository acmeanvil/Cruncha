// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Number formatting (Hurmet's `format.js`): rounding to a format spec such as `h15`, `f2`,
//! `r3`, `e3`, `n3`, `k3`, `m` or `%1`, then adding the document's thousands and decimal
//! separators.

use crate::error::{err, err0};
use crate::value::{Operand, dt};
use cruncha_lang::hurmet::formatted_integer;
use cruncha_units::{Rational, jsnum};
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// TeX for the rendered result.
    Tex,
    /// Plain text, with the document's decimal format, for screen display.
    Plain,
    /// A valid Hurmet number literal, for results saved in a Markdown file.
    Literal,
}

const SI_PREFIXES: [&str; 17] = [
    "y", "z", "a", "f", "p", "n", "µ", "m", "", "k", "M", "G", "T", "P", "E", "Z", "Y",
];

fn superscript(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            _ => char::from_u32(c as u32 + 0x2040).unwrap_or(c),
        })
        .collect()
}

fn subscript(s: &str) -> String {
    s.chars()
        .map(|c| char::from_u32(c as u32 + 0x2050).unwrap_or(c))
        .collect()
}

/// Hurmet's `texFromMixedFraction`.
#[allow(dead_code)]
pub(crate) fn tex_from_mixed_fraction(neg: bool, whole: &str, num: &str, den: &str) -> String {
    format!(
        "{}{whole}\\,\\sfrac{{{num}}}{{{den}}}",
        if neg { "-" } else { "" }
    )
}

fn shift_decimal_point(s: &str, mut places: i64) -> String {
    if places == 0 {
        return s.to_string();
    }
    let (sign, s) = match s.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", s),
    };
    let (int_part, frac_part) = s.split_once('.').unwrap_or((s, ""));
    let mut int_part = int_part.to_string();
    let mut frac_part = frac_part.to_string();
    while places > 0 {
        if frac_part.is_empty() {
            frac_part = "0".into();
        }
        int_part.push_str(&frac_part[..1]);
        frac_part = frac_part[1..].to_string();
        places -= 1;
    }
    if frac_part.is_empty() {
        format!("{sign}{int_part}")
    } else {
        format!("{sign}{int_part}.{frac_part}")
    }
}

struct Spec {
    ftype: char,
    /// `None` when the spec has no digits (JavaScript's `undefined`).
    n: Option<f64>,
}

fn int_abs(i: &BigInt) -> BigInt {
    i.abs()
}

fn rounded_string(r: &Rational, spec: &Spec) -> Result<String, Operand> {
    let bad_prec = |n: Option<f64>| n.is_some_and(|n| !(1.0..=15.0).contains(&n) || n.is_nan());
    match spec.ftype {
        'h' => {
            let integer_str = r.to_fixed(0);
            if let Some(n) = spec.n
                && integer_str.replace('-', "").len() as f64 >= n
            {
                return Ok(integer_str);
            }
            if bad_prec(spec.n) {
                return Err(err0("BAD_PREC"));
            }
            Ok(match spec.n {
                Some(n) => jsnum::to_precision(r.to_f64(), n as usize),
                None => jsnum::to_string(r.to_f64()),
            })
        }
        'f' | '%' => Ok(r.to_fixed(spec.n.unwrap_or(0.0) as i64)),
        'r' | 'p' => {
            if bad_prec(spec.n) {
                return Err(err0("BAD_PREC"));
            }
            let num_str = match spec.n {
                Some(n) => jsnum::to_precision(r.to_f64(), n as usize),
                None => jsnum::to_string(r.to_f64()),
            };
            Ok(if num_str.contains('e') {
                jsnum::to_string(jsnum::parse_number(&num_str))
            } else {
                num_str
            })
        }
        's' | 'S' | 'e' | 'E' | 'n' | 'N' | 'k' => {
            if bad_prec(spec.n) {
                return Err(err0("BAD_PREC"));
            }
            let digits = spec.n.map(|n| (n as usize).saturating_sub(1));
            Ok(jsnum::to_exponential(r.to_f64(), digits).replacen('+', "", 1))
        }
        'm' | 't' | 'b' | 'x' | 'X' => {
            let r = r.normalize();
            let sign = if r.is_negative() { "-" } else { "" };
            let numerator = int_abs(&r.num);
            let denominator = r.den.clone();
            if denominator.is_zero() {
                return Err(err0("NOT_FINITE"));
            }
            match spec.ftype {
                'm' => {
                    let quotient = (&numerator / &denominator).to_string();
                    let remainder = &numerator % &denominator;
                    if remainder.is_zero() {
                        return Ok(format!("{sign}{quotient}"));
                    }
                    Ok(format!(
                        "{sign}{quotient}\u{a0}{}⁄{}",
                        superscript(&remainder.to_string()),
                        subscript(&denominator.to_string())
                    ))
                }
                't' => Ok(format!("{sign}{}", &numerator / &denominator)),
                _ => {
                    if denominator != BigInt::from(1) {
                        return Err(err("INT_NUM", &spec.ftype.to_string()));
                    }
                    match numerator.to_u64() {
                        Some(n) if n <= 9007199254740991 => Ok(match spec.ftype {
                            'b' => format!("{sign}0b{n:b}"),
                            'x' => format!("{sign}0x{n:x}"),
                            _ => format!("{sign}0x{n:X}"),
                        }),
                        _ => Ok(String::new()),
                    }
                }
            }
        }
        _ => Err(err("BAD_FORMAT", &spec.ftype.to_string())),
    }
}

/// Hurmet's `formattedDecimal`.
pub(crate) fn formatted_decimal(num_str: &str, decimal_format: &str, truncate: bool) -> String {
    match num_str.find('.') {
        None => formatted_integer(num_str, decimal_format),
        Some(pos) => {
            let sep = if decimal_format.ends_with('.') {
                '.'
            } else {
                ','
            };
            let mut frac = format!("{sep}{}", &num_str[pos + 1..]);
            if truncate {
                // frac.replace(/[.,]?0+$/, "")
                let trimmed = frac.trim_end_matches('0');
                if trimmed.len() < frac.len() {
                    let t = trimmed.strip_suffix(['.', ',']).unwrap_or(trimmed);
                    frac = t.to_string();
                }
            }
            format!(
                "{}{frac}",
                formatted_integer(&num_str[..pos], decimal_format)
            )
        }
    }
}

fn superscript_exponent(s: &str) -> String {
    let neg = s.starts_with('-');
    let digits = s.trim_start_matches(['-', '+']);
    format!("{}{}", if neg { "⁻" } else { "" }, superscript(digits))
}

fn mixed_fraction_literal(r: &Rational) -> String {
    let r = r.normalize();
    let sign = if r.is_negative() { "-" } else { "" };
    let numerator = int_abs(&r.num);
    if r.den.is_zero() {
        return "NaN".into();
    }
    let quotient = &numerator / &r.den;
    let remainder = &numerator % &r.den;
    if remainder.is_zero() {
        format!("{sign}{quotient}")
    } else {
        format!("{sign}{quotient} {remainder}/{}", r.den)
    }
}

fn literal_from_rounded_string(num_str: &str, spec: &Spec, r: &Rational) -> String {
    match spec.ftype {
        'h' => {
            let s = num_str.replacen("e+", "e", 1);
            // /^[^e]*\.[^e]*$/ → strip /\.?0+$/
            if !s.contains('e') && s.contains('.') {
                let t = s.trim_end_matches('0');
                if t.len() < s.len() {
                    return t.strip_suffix('.').unwrap_or(t).to_string();
                }
            }
            s
        }
        '%' | 'p' => format!("{}%", num_str.replacen("e+", "e", 1)),
        'm' => mixed_fraction_literal(r),
        'b' => r.normalize().num.to_string(),
        _ => num_str.replacen("e+", "e", 1),
    }
}

/// Hurmet's `format(num, specStr, decimalFormat, mode)`.
pub(crate) fn format(
    num: &Rational,
    spec_str: &str,
    decimal_format: &str,
    mode: Mode,
) -> Result<String, Operand> {
    if !num.is_finite() {
        // Not in Hurmet, which throws on NaN and ±∞ (see Rational's docs).
        return Ok(if num.is_zero() {
            "NaN"
        } else if num.is_negative() {
            "-Infinity"
        } else {
            "Infinity"
        }
        .into());
    }
    if num.is_zero() {
        return Ok("0".into());
    }
    let plain = mode == Mode::Plain;
    let ftype = spec_str.chars().next().unwrap_or('h');
    let spec_body = spec_str.trim_end_matches(['∠', '°']);
    let n = if spec_body.chars().count() > 1 {
        let rest: String = spec_body.chars().skip(1).collect();
        Some(jsnum::parse_number(&rest))
    } else {
        None
    };
    let spec = Spec { ftype, n };
    let num = if ftype == '%' || ftype == 'p' {
        Rational::new(&num.num * BigInt::from(100), num.den.clone())
    } else {
        num.clone()
    };
    if (ftype == 'b' || ftype == 'x') && !num.is_integer() {
        return Err(err0("FORM_FRAC"));
    }
    let num_str = rounded_string(&num, &spec)?;
    if mode == Mode::Literal {
        return Ok(literal_from_rounded_string(&num_str, &spec, &num));
    }
    Ok(match ftype {
        'f' | 'r' | 'h' => formatted_decimal(&num_str, decimal_format, ftype == 'h'),
        't' => formatted_integer(&num_str, decimal_format),
        '%' | 'p' => format!(
            "{}{}",
            formatted_decimal(&num_str, decimal_format, false),
            if plain { "%" } else { "\\%" }
        ),
        'm' | 'b' | 'x' | 'X' => num_str,
        _ => {
            let pos = num_str.find('e').unwrap_or(num_str.len());
            let mut significand = num_str[..pos].to_string();
            let exp_str = num_str.get(pos + 1..).unwrap_or("");
            let comma = decimal_format.ends_with(',');
            if comma {
                significand = significand.replacen('.', ",", 1);
            }
            match ftype {
                'e' | 'E' => {
                    if plain {
                        format!("{significand}{ftype}{exp_str}")
                    } else if let Some(rest) = exp_str.strip_prefix('-') {
                        format!("{significand}\\text{{{ftype}-}}{rest}")
                    } else {
                        format!("{significand}\\text{{{ftype}}}{exp_str}")
                    }
                }
                's' | 'S' => {
                    if plain {
                        format!(
                            "{significand}{}10{}",
                            if ftype == 'S' { "×" } else { "·" },
                            superscript_exponent(exp_str)
                        )
                    } else {
                        let op = if ftype == 'S' {
                            "×"
                        } else {
                            "\\mkern2mu{\\cdot}\\mkern1mu"
                        };
                        format!("{significand}{op}10^{{{exp_str}}}")
                    }
                }
                'n' | 'N' => {
                    let exponent: i64 = exp_str.parse().unwrap_or(0);
                    let m = exponent.rem_euclid(3);
                    let new_exponent = exponent - m;
                    let mut sig = num_str[..pos].to_string();
                    if m != 0 {
                        sig = shift_decimal_point(&sig, m);
                    }
                    if comma {
                        sig = sig.replacen('.', ",", 1);
                    }
                    if new_exponent == 0 {
                        sig
                    } else if plain {
                        format!(
                            "{sig}{}10{}",
                            if ftype == 'N' { "×" } else { "·" },
                            superscript_exponent(&new_exponent.to_string())
                        )
                    } else {
                        let op = if ftype == 'N' {
                            "×"
                        } else {
                            "\\mkern2mu{\\cdot}\\mkern1mu"
                        };
                        format!("{sig}{op}10^{{{new_exponent}}}")
                    }
                }
                'k' => {
                    let exponent: i64 = exp_str.parse().unwrap_or(0);
                    let q = exponent / 3; // truncates toward zero, like Math.floor/ceil there
                    let modulo = exponent % 3;
                    if modulo != 0 {
                        significand = jsnum::to_string(
                            jsnum::parse_number(&significand)
                                * cruncha_units::jsmath::pow(10.0, modulo as f64),
                        );
                    }
                    let idx = 8 + q;
                    let prefix = if (0..17).contains(&idx) {
                        SI_PREFIXES[idx as usize]
                    } else {
                        ""
                    };
                    format!("{significand}{prefix}")
                }
                _ => num_str,
            }
        }
    })
}

/// Format, turning an error into its message (as Hurmet's callers do when they concatenate).
pub(crate) fn fmt(num: &Rational, spec: &str, decimal_format: &str, mode: Mode) -> String {
    match format(num, spec, decimal_format, mode) {
        Ok(s) => s,
        Err(e) => e.error_message().to_string(),
    }
}

/// Hurmet's `validateFormatSpec`: returns (dtype, TeX display).
pub(crate) fn validate_format_spec(s: &str) -> (i64, String) {
    // /^([beENfhkmnprsStx%])?(-?[\d]+)?([∠°]{0,2})?$/
    let mut chars = s.chars().peekable();
    let mut ftype = None;
    if let Some(&c) = chars.peek()
        && "beENfhkmnprsStx%".contains(c)
    {
        ftype = Some(c);
        chars.next();
    }
    let rest: String = chars.collect();
    let digits_end = {
        let b = rest.as_bytes();
        let mut i = 0;
        if b.first() == Some(&b'-') && b.get(1).is_some_and(u8::is_ascii_digit) {
            i = 1;
        }
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == 1 && b.first() == Some(&b'-') {
            0
        } else {
            i
        }
    };
    let (digits, ctype) = rest.split_at(digits_end);
    let valid = ctype.chars().count() <= 2 && ctype.chars().all(|c| c == '∠' || c == '°');
    if !valid {
        let message = err("BAD_FORMAT", s).value.str().to_string();
        return (dt::ERROR, format!("\\text{{{message}}}"));
    }
    let ftype = ftype.unwrap_or('h');
    let mut n: i64 = if digits.is_empty() {
        15
    } else {
        digits.parse().unwrap_or(15)
    };
    match ftype {
        'b' | 'x' | 'X' => return (dt::STRING, format!("\\text{{{ftype}{ctype}}}")),
        't' => n = 0,
        'f' | '%' => {}
        _ => {
            if !(1..=15).contains(&n) {
                let message = err0("BAD_PREC").value.str().to_string();
                return (dt::ERROR, format!("\\text{{{message}}}"));
            }
        }
    }
    let ftype = if ftype == '%' {
        "\\%".to_string()
    } else {
        ftype.to_string()
    };
    (dt::STRING, format!("\\text{{{ftype}{n}{ctype}}}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(s: &str, spec: &str) -> String {
        fmt(
            &Rational::from_str_or_zero(s),
            spec,
            "1,000,000.",
            Mode::Tex,
        )
    }

    #[test]
    fn formats() {
        assert_eq!(f("33000", "h15"), "33,000");
        assert_eq!(f("0.03555", "h15"), "0.03555");
        assert_eq!(f("-88.1", "h15"), "-88.1");
        assert_eq!(f("2.345", "f2"), "2.35");
        assert_eq!(f("123456", "r3"), "123,000");
        assert_eq!(f("123456", "e3"), "1.23\\text{e}5");
        assert_eq!(f("0.4", "%1"), "40.0\\%");
        assert_eq!(f("12345", "k3"), "12.3k");
        assert_eq!(f("-123", "h15"), "-123");
    }
}
