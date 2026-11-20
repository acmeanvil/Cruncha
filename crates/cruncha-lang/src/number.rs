// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Number literals: Hurmet's `Rnl.numberPattern` and `Rnl.fromString` (rational.js) and the
//! TeX number formatting from `format.js` (`formattedDecimal`, `texFromMixedFraction`).

use regex::{Captures, Regex};
use std::sync::LazyLock;

/// Capturing groups:
/// 1 sign, 2 hexadecimal integer, 3 integer part, 4 numerator of a mixed fraction,
/// 5 denominator of a mixed fraction, 6 decimal fraction, 7 exponent, 8 percent sign.
pub(crate) const NUMBER_PATTERN: &str = r"^(-?)(?:(0x[0-9A-Fa-f]+)|([0-9]+)(?: ([0-9]+)/([0-9]+)|(?:\.([0-9]+))?(?:e([+-]?[0-9]+)|(%))?))";

pub(crate) static NUMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(NUMBER_PATTERN).unwrap());

fn group<'a>(c: &'a Captures, i: usize) -> &'a str {
    c.get(i).map_or("", |m| m.as_str())
}

/// TeX for a number literal (lexer.js `texFromNumStr`).
pub(crate) fn tex_from_num_parts(parts: &Captures, decimal_format: &str) -> String {
    let mut num;
    if !group(parts, 2).is_empty() {
        num = format!("\\mathrm{{{}}}", group(parts, 2));
    } else if !group(parts, 5).is_empty() {
        // texFromMixedFraction
        return format!(
            "{}{}\\,\\sfrac{{{}}}{{{}}}",
            if group(parts, 1).is_empty() { "" } else { "-" },
            group(parts, 3),
            group(parts, 4),
            group(parts, 5)
        );
    } else {
        num = group(parts, 3).to_string();
        if !group(parts, 6).is_empty() {
            num.push('.');
            num.push_str(group(parts, 6));
        }
        num = formatted_decimal(&num, decimal_format);
        if !group(parts, 8).is_empty() {
            num.push_str("\\%");
        } else if let Some(e) = parts.get(7).map(|m| m.as_str()).filter(|e| !e.is_empty()) {
            if let Some(rest) = e.strip_prefix('-') {
                num.push_str("\\text{e-}");
                num.push_str(rest);
            } else {
                num.push_str("\\text{e}");
                num.push_str(e);
            }
        }
    }
    if !group(parts, 1).is_empty() {
        num = format!("\\text{{-}}{num}");
    }
    num
}

pub fn formatted_integer(int_str: &str, decimal_format: &str) -> String {
    let sep = decimal_format.chars().nth(1).unwrap_or(',');
    if sep == '0' {
        return int_str.to_string();
    }
    let n = int_str.chars().count();
    let digits: Vec<char> = int_str.chars().collect();
    let mut out = String::new();
    if decimal_format == "1,00,000." {
        // /(\d)(?=(\d\d)+\d$)/g → "$1,"
        for (i, d) in digits.iter().enumerate() {
            out.push(*d);
            let remaining = n - i - 1;
            if remaining >= 3 && remaining % 2 == 1 {
                out.push(',');
            }
        }
        return out;
    }
    let (group, sep_str) = if decimal_format == "1,0000,0000." {
        // Hurmet replaces /\B(?=(\d{4})+$)/g with "$1,", but that regex has no group 1,
        // so JavaScript inserts a literal "$1,".
        (4, "$1,".to_string())
    } else {
        let s = match sep {
            ',' => ",",
            ' ' => "\u{2008}",
            '’' => "’",
            _ => ".",
        };
        (3, s.to_string())
    };
    for (i, d) in digits.iter().enumerate() {
        // `\B`: only between two digits (a leading minus sign gets no separator).
        if i > 0 && digits[i - 1].is_ascii_digit() && (n - i).is_multiple_of(group) {
            out.push_str(&sep_str);
        }
        out.push(*d);
    }
    out
}

/// format.js `formattedDecimal` without trailing-zero truncation.
pub fn formatted_decimal(num_str: &str, decimal_format: &str) -> String {
    match num_str.find('.') {
        None => formatted_integer(num_str, decimal_format),
        Some(pos) => {
            let frac_sep = if decimal_format.ends_with('.') {
                "."
            } else {
                ","
            };
            format!(
                "{}{}{}",
                formatted_integer(&num_str[..pos], decimal_format),
                frac_sep,
                &num_str[pos + 1..]
            )
        }
    }
}

fn strip_leading_zeros(s: &str) -> &str {
    let t = s.trim_start_matches('0');
    if t.is_empty() { "0" } else { t }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// The RPN form of a number literal, `®numerator/denominator` (parser.js `rationalRPN`).
/// Decimals are not reduced, matching `Rnl.fromString`.
pub(crate) fn rational_rpn(num_str: &str) -> String {
    let Some(parts) = NUMBER_RE.captures(num_str) else {
        return "®NaN".to_string();
    };
    let negative = !group(&parts, 1).is_empty();
    let (mut numer, denom): (String, String) = if !group(&parts, 5).is_empty() {
        let d: u128 = group(&parts, 5).parse().unwrap_or(1);
        let i: u128 = group(&parts, 3).parse().unwrap_or(0);
        let n: u128 = group(&parts, 4).parse().unwrap_or(0);
        let numerator = i * d + n;
        if d == 1 {
            (numerator.to_string(), "1".to_string())
        } else {
            let g = gcd(numerator, d);
            ((numerator / g).to_string(), (d / g).to_string())
        }
    } else if !group(&parts, 2).is_empty() {
        let v = u128::from_str_radix(&group(&parts, 2)[2..], 16).unwrap_or(0);
        (v.to_string(), "1".to_string())
    } else {
        let frac = group(&parts, 6);
        let digits = format!("{}{}", group(&parts, 3), frac);
        let numerator = strip_leading_zeros(&digits).to_string();
        let frac_len = frac.len() as i64;
        let exp = if let Some(e) = parts.get(7) {
            e.as_str().parse::<i64>().unwrap_or(0) - frac_len
        } else if !group(&parts, 8).is_empty() {
            -2 - frac_len
        } else {
            -frac_len
        };
        if exp < 0 {
            (numerator, format!("1{}", "0".repeat((-exp) as usize)))
        } else if numerator == "0" {
            (numerator, "1".to_string())
        } else {
            (
                format!("{numerator}{}", "0".repeat(exp as usize)),
                "1".to_string(),
            )
        }
    };
    if negative && numer != "0" {
        numer.insert(0, '-');
    }
    format!("®{numer}/{denom}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpn_numbers() {
        assert_eq!(rational_rpn("1.2"), "®12/10");
        assert_eq!(rational_rpn("-4.5"), "®-45/10");
        assert_eq!(rational_rpn("0.9999375"), "®9999375/10000000");
        assert_eq!(rational_rpn("5 2/3"), "®17/3");
        assert_eq!(rational_rpn("1e3"), "®1000/1");
    }

    #[test]
    fn grouping() {
        assert_eq!(formatted_decimal("4500", "1,000,000."), "4,500");
        assert_eq!(formatted_decimal("1234567.5", "1.000.000,"), "1.234.567,5");
        assert_eq!(formatted_decimal("1234567", "1,00,000."), "12,34,567");
    }
}
