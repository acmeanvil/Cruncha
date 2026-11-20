// SPDX-License-Identifier: GPL-3.0-or-later

//! JavaScript number semantics that Hurmet's output depends on.
//!
//! Hurmet converts between its exact rationals and IEEE doubles through JavaScript's
//! `Number(bigint)`, `Number.prototype.toExponential`, `toPrecision` and `String(number)`.
//! These helpers reproduce those algorithms exactly (ECMA-262 §21.1.3), so that rounded results
//! match Hurmet digit for digit. Rust's own float formatting rounds ties to even, where
//! JavaScript rounds them up, and it never switches to exponent notation; both differences
//! are handled here.

use num_bigint::{BigInt, Sign};
use num_traits::{ToPrimitive, Zero};

/// `Number(bigint)`: the nearest double, ties to even; ±Infinity when too large.
pub fn bigint_to_f64(n: &BigInt) -> f64 {
    let (sign, mag) = n.clone().into_parts();
    let bits = mag.bits();
    let value = if bits <= 64 {
        mag.to_u64().map_or(f64::INFINITY, |u| u as f64)
    } else {
        // Keep the top 64 bits, with a sticky bit for anything below, then scale.
        let shift = bits - 64;
        let top = (&mag >> shift).to_u64().unwrap_or(u64::MAX);
        let sticky = !(&mag & ((num_bigint::BigUint::from(1u8) << shift) - 1u8)).is_zero();
        let top = if sticky { top | 1 } else { top };
        if shift > 2000 {
            f64::INFINITY
        } else {
            (top as f64) * 2f64.powi(shift as i32)
        }
    };
    if sign == Sign::Minus { -value } else { value }
}

/// Decimal digits and exponent of `x` (finite, non-zero, positive), rounded to `p` significant
/// digits with JavaScript's tie rule (round half up). Returns (digits, e) with x ≈ d.ddd × 10^e.
fn rounded_digits(x: f64, p: usize) -> (String, i32) {
    let parse = |s: &str| -> (String, i32) {
        let (mant, exp) = s.split_once('e').unwrap_or((s, "0"));
        let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
        (digits, exp.parse().unwrap_or(0))
    };
    let (mut digits, mut e) = parse(&format!("{:.*e}", p + 24, x));
    let tail = &digits[p..];
    if tail.starts_with("4999999999") || tail.starts_with("5000000000") {
        // Possibly an exact tie: use the exact expansion (a double has at most 767 digits).
        let exact = parse(&format!("{:.1100e}", x));
        digits = exact.0;
        e = exact.1;
    }
    let round_up = digits.as_bytes()[p] >= b'5';
    let mut head: Vec<u8> = digits.as_bytes()[..p].to_vec();
    if round_up {
        let mut i = p;
        loop {
            if i == 0 {
                head.insert(0, b'1');
                head.pop();
                e += 1;
                break;
            }
            i -= 1;
            if head[i] == b'9' {
                head[i] = b'0';
            } else {
                head[i] += 1;
                break;
            }
        }
    }
    (String::from_utf8(head).unwrap_or_default(), e)
}

/// Shortest round-trip digits and exponent: x ≈ d.ddd × 10^e.
fn shortest_digits(x: f64) -> (String, i32) {
    let s = format!("{:e}", x.abs());
    let (mant, exp) = s.split_once('e').unwrap_or((&s, "0"));
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    (digits, exp.parse().unwrap_or(0))
}

fn non_finite(x: f64) -> Option<String> {
    if x.is_nan() {
        Some("NaN".into())
    } else if x.is_infinite() {
        Some(if x > 0.0 { "Infinity" } else { "-Infinity" }.into())
    } else {
        None
    }
}

fn exp_suffix(e: i32) -> String {
    if e >= 0 {
        format!("e+{e}")
    } else {
        format!("e-{}", -e)
    }
}

/// `String(x)` / `Number.prototype.toString()`.
pub fn to_string(x: f64) -> String {
    if let Some(s) = non_finite(x) {
        return s;
    }
    if x == 0.0 {
        return "0".into();
    }
    let sign = if x < 0.0 { "-" } else { "" };
    let (digits, e) = shortest_digits(x);
    let k = digits.len() as i32;
    let n = e + 1;
    let body = if k <= n && n <= 21 {
        format!("{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let rest = if k > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        format!("{}{rest}{}", &digits[..1], exp_suffix(n - 1))
    };
    format!("{sign}{body}")
}

/// `x.toExponential()` (no argument: as many digits as needed to round-trip) or
/// `x.toExponential(f)`.
pub fn to_exponential(x: f64, fraction_digits: Option<usize>) -> String {
    if let Some(s) = non_finite(x) {
        return s;
    }
    let sign = if x < 0.0 { "-" } else { "" };
    let (digits, e) = if x == 0.0 {
        ("0".repeat(fraction_digits.unwrap_or(0) + 1), 0)
    } else {
        match fraction_digits {
            None => shortest_digits(x),
            Some(f) => rounded_digits(x.abs(), f + 1),
        }
    };
    let rest = if digits.len() > 1 {
        format!(".{}", &digits[1..])
    } else {
        String::new()
    };
    format!("{sign}{}{rest}{}", &digits[..1], exp_suffix(e))
}

/// `x.toPrecision(p)`, 1 ≤ p ≤ 100.
pub fn to_precision(x: f64, p: usize) -> String {
    if let Some(s) = non_finite(x) {
        return s;
    }
    let p = p.clamp(1, 100);
    let sign = if x < 0.0 { "-" } else { "" };
    let (digits, e) = if x == 0.0 {
        ("0".repeat(p), 0)
    } else {
        rounded_digits(x.abs(), p)
    };
    let p = p as i32;
    let body = if e < -6 || e >= p {
        let rest = if p > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        format!("{}{rest}{}", &digits[..1], exp_suffix(e))
    } else if e == p - 1 {
        digits
    } else if e >= 0 {
        format!(
            "{}.{}",
            &digits[..(e + 1) as usize],
            &digits[(e + 1) as usize..]
        )
    } else {
        format!("0.{}{digits}", "0".repeat((-(e + 1)) as usize))
    };
    format!("{sign}{body}")
}

fn is_js_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// `Number(str)` for a string: NaN when the string is not a numeric literal.
pub fn parse_number(s: &str) -> f64 {
    let t = s.trim_matches(is_js_space);
    if t.is_empty() {
        return 0.0;
    }
    match t {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    let lower = t.get(..2).map(|p| p.to_ascii_lowercase());
    if let Some(prefix) = lower.as_deref() {
        let radix = match prefix {
            "0x" => 16,
            "0o" => 8,
            "0b" => 2,
            _ => 0,
        };
        if radix > 0 {
            let body = &t[2..];
            if body.is_empty() || !body.chars().all(|c| c.is_digit(radix)) {
                return f64::NAN;
            }
            return BigInt::parse_bytes(body.as_bytes(), radix)
                .map_or(f64::NAN, |b| bigint_to_f64(&b));
        }
    }
    // StrDecimalLiteral: [+-] (digits [. [digits]] | . digits) [(e|E) [+-] digits]
    let b = t.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = i - int_start;
    let mut frac_digits = 0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        frac_digits = i - fs;
    }
    if int_digits + frac_digits == 0 {
        return f64::NAN;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let es = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return f64::NAN;
        }
    }
    if i != b.len() {
        return f64::NAN;
    }
    t.parse::<f64>().unwrap_or(f64::NAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_to_string() {
        assert_eq!(to_string(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(to_string(1e21), "1e+21");
        assert_eq!(to_string(123456789012345680000.0), "123456789012345680000");
        assert_eq!(to_string(1e-7), "1e-7");
        assert_eq!(to_string(0.000001), "0.000001");
        assert_eq!(to_string(-2.5), "-2.5");
    }

    #[test]
    fn precision_and_exponential() {
        assert_eq!(to_precision(2.5, 1), "3"); // JavaScript rounds ties up
        assert_eq!(
            to_precision(std::f64::consts::FRAC_PI_6, 15),
            "0.523598775598299"
        );
        assert_eq!(to_precision(1234.5678, 2), "1.2e+3");
        assert_eq!(to_precision(0.0000123, 2), "0.000012");
        assert_eq!(to_precision(0.0, 3), "0.00");
        assert_eq!(to_exponential(0.000001, None), "1e-6");
        assert_eq!(to_exponential(1.5, None), "1.5e+0");
        assert_eq!(to_exponential(12345.0, Some(2)), "1.23e+4");
    }

    #[test]
    fn parse() {
        assert_eq!(parse_number(" 12.5 "), 12.5);
        assert!(parse_number("1,234").is_nan());
        assert_eq!(parse_number("0x10"), 16.0);
        assert_eq!(parse_number(""), 0.0);
        assert!(parse_number("inf").is_nan());
    }

    #[test]
    fn bigint_rounding() {
        let big = BigInt::parse_bytes(b"9007199254740993", 10).unwrap(); // 2^53 + 1
        assert_eq!(bigint_to_f64(&big), 9007199254740992.0);
        let big = BigInt::parse_bytes(b"18446744073709551617", 10).unwrap(); // 2^64 + 1
        assert_eq!(bigint_to_f64(&big), 18446744073709551616.0);
        assert_eq!(bigint_to_f64(&(BigInt::from(1) << 2000u32)), f64::INFINITY);
    }
}
