// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Exact rational numbers (Hurmet's `rational.js`, `Rnl`).
//!
//! A rational is a numerator and a denominator, both big integers. As in Hurmet, values are
//! **not** reduced automatically: `add` with equal denominators and `multiply`/`divide` keep
//! whatever factors they produce, and only some operations call [`Rational::normalize`]. The
//! unreduced form is observable (a zero denominator signals division by zero, and Hurmet's
//! rounding works on the stored form), so the port keeps it.
//!
//! Hurmet throws a JavaScript `TypeError` when a floating-point step yields NaN or ±∞. The port
//! instead stores those as a zero denominator (`0/0` for NaN, `±1/0` for ±∞); no operation
//! here panics.

use crate::{jsmath, jsnum};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::sync::LazyLock;

/// An exact rational, `num / den`. Negative values have a negative numerator.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rational {
    pub num: BigInt,
    pub den: BigInt,
}

/// Hurmet's `numberPattern`, the syntax of a number literal.
pub const NUMBER_PATTERN: &str = r"^(-?)(?:(0x[0-9A-Fa-f]+)|([0-9]+)(?: ([0-9]+)/([0-9]+)|(?:\.([0-9]+))?(?:e([+-]?[0-9]+)|(%))?))";

/// The largest result, in bits, that `power` computes exactly. JavaScript's BigInt allows about
/// 2^30 bits before it throws (and Hurmet then falls back to floating point); the port falls
/// back much earlier, to keep evaluation fast.
const MAX_EXACT_POWER_BITS: u64 = 1 << 20;

fn big(n: i64) -> BigInt {
    BigInt::from(n)
}

/// JavaScript BigInt `/` (truncates toward zero). Division by zero, which throws in JavaScript,
/// gives zero here.
fn idiv(a: &BigInt, b: &BigInt) -> BigInt {
    if b.is_zero() { BigInt::zero() } else { a / b }
}

/// JavaScript BigInt `%` (sign of the dividend). `x % 0` gives zero here.
fn irem(a: &BigInt, b: &BigInt) -> BigInt {
    if b.is_zero() { BigInt::zero() } else { a % b }
}

fn gcdi(a: &BigInt, b: &BigInt) -> BigInt {
    a.abs().gcd(&b.abs())
}

pub static PI: LazyLock<Rational> = LazyLock::new(|| {
    Rational::new(
        "31415926535897932384626433832795028841971693993751"
            .parse()
            .unwrap(),
        "10000000000000000000000000000000000000000000000000"
            .parse()
            .unwrap(),
    )
});
pub static E: LazyLock<Rational> = LazyLock::new(|| {
    Rational::new(
        "2718281828459045235360287471352662497757247093699959574966"
            .parse()
            .unwrap(),
        "1000000000000000000000000000000000000000000000000000000000"
            .parse()
            .unwrap(),
    )
});
/// The reduced Planck constant.
pub static HBAR: LazyLock<Rational> = LazyLock::new(|| {
    Rational::new(
        big(1054571817),
        "10000000000000000000000000000000000000000000"
            .parse()
            .unwrap(),
    )
});

impl Rational {
    pub fn new(num: BigInt, den: BigInt) -> Self {
        Rational { num, den }
    }
    pub fn int(n: i64) -> Self {
        Rational::new(big(n), BigInt::one())
    }
    pub fn zero() -> Self {
        Rational::int(0)
    }
    pub fn one() -> Self {
        Rational::int(1)
    }
    pub fn two() -> Self {
        Rational::int(2)
    }
    pub fn pi() -> Self {
        PI.clone()
    }
    pub fn e() -> Self {
        E.clone()
    }
    pub fn hbar() -> Self {
        HBAR.clone()
    }

    /// True for the NaN/∞ stand-ins (a zero denominator).
    pub fn is_finite(&self) -> bool {
        !self.den.is_zero()
    }

    /// `Rnl.fromNumber`: an integer converts exactly; otherwise the shortest round-trip decimal
    /// (`toExponential()`) is read back as a decimal fraction.
    pub fn from_number(x: f64) -> Self {
        if x.is_nan() {
            return Rational::new(BigInt::zero(), BigInt::zero());
        }
        if x.is_infinite() {
            return Rational::new(big(if x > 0.0 { 1 } else { -1 }), BigInt::zero());
        }
        if x.fract() == 0.0 {
            return Rational::new(f64_int_to_bigint(x), BigInt::one());
        }
        let s = jsnum::to_exponential(x, None);
        let (mant, exp) = s.split_once('e').unwrap_or((&s, "0"));
        let neg = mant.starts_with('-');
        let mant = mant.trim_start_matches('-');
        let (int_part, frac) = mant.split_once('.').unwrap_or((mant, ""));
        let exp: i64 = exp.trim_start_matches('+').parse().unwrap_or(0);
        let exp = exp - frac.len() as i64;
        let digits = format!("{}{int_part}{frac}", if neg { "-" } else { "" });
        let n: BigInt = digits.parse().unwrap_or_default();
        if exp < 0 {
            Rational::new(n, pow10(-exp))
        } else {
            Rational::new(n * pow10(exp), BigInt::one()).normalize()
        }
    }

    /// `Rnl.fromString`: read a number literal (decimal, `0x` hex, mixed fraction `5 2/3`,
    /// scientific notation, or a percentage). `None` if the string does not start with one.
    pub fn from_str_hurmet(s: &str) -> Option<Self> {
        let caps = number_regex_captures(s)?;
        let g = |i: usize| caps[i].as_deref();
        let mut r = if let Some(den) = g(5) {
            let den: BigInt = den.parse().ok()?;
            let whole: BigInt = g(3)?.parse().ok()?;
            let numer: BigInt = g(4)?.parse().ok()?;
            Rational::new(whole * &den + numer, den).normalize()
        } else if let Some(hex) = g(2) {
            Rational::new(
                BigInt::parse_bytes(&hex.as_bytes()[2..], 16)?,
                BigInt::one(),
            )
        } else {
            let frac = g(6).unwrap_or("");
            let numerator: BigInt = format!("{}{frac}", g(3)?).parse().ok()?;
            let exp: i64 = if let Some(e) = g(7) {
                e.trim_start_matches('+').parse::<i64>().ok()? - frac.len() as i64
            } else if g(8).is_some() {
                -2 - frac.len() as i64
            } else {
                -(frac.len() as i64)
            };
            if exp < 0 {
                Rational::new(numerator, pow10(-exp))
            } else {
                Rational::new(numerator * pow10(exp), BigInt::one()).normalize()
            }
        };
        if g(1) == Some("-") {
            r = r.negate();
        }
        Some(r)
    }

    /// `Rnl.fromString`, with zero for input that is not a number (Hurmet would throw).
    pub fn from_str_or_zero(s: &str) -> Self {
        Rational::from_str_hurmet(s).unwrap_or_else(Rational::zero)
    }

    pub fn normalize(&self) -> Self {
        if self.den.is_one() {
            return self.clone();
        }
        let g = gcdi(&self.num, &self.den);
        if g.is_one() || g.is_zero() {
            self.clone()
        } else {
            Rational::new(&self.num / &g, &self.den / &g)
        }
    }

    pub fn is_integer(&self) -> bool {
        self.den.is_one() || irem(&self.num, &self.den).is_zero()
    }
    pub fn is_zero(&self) -> bool {
        self.num.is_zero()
    }
    pub fn is_negative(&self) -> bool {
        self.num.is_negative()
    }
    pub fn is_positive(&self) -> bool {
        self.num.is_positive()
    }
    pub fn sign(&self) -> Self {
        if self.is_positive() {
            Rational::one()
        } else if self.is_zero() {
            Rational::zero()
        } else {
            Rational::int(-1)
        }
    }
    pub fn negate(&self) -> Self {
        Rational::new(-&self.num, self.den.clone())
    }
    pub fn abs(&self) -> Self {
        Rational::new(self.num.abs(), self.den.clone())
    }
    pub fn increment(&self) -> Self {
        Rational::new(&self.num + &self.den, self.den.clone())
    }
    pub fn decrement(&self) -> Self {
        Rational::new(&self.num - &self.den, self.den.clone())
    }
    pub fn floor(&self) -> Self {
        let q = idiv(&self.num, &self.den);
        if irem(&self.num, &self.den).is_zero() || !self.num.is_negative() {
            Rational::new(q, BigInt::one())
        } else {
            Rational::new(q - 1, BigInt::one())
        }
    }
    pub fn ceil(&self) -> Self {
        let q = idiv(&self.num, &self.den);
        if irem(&self.num, &self.den).is_zero() || self.num.is_negative() {
            Rational::new(q, BigInt::one())
        } else {
            Rational::new(q + 1, BigInt::one())
        }
    }
    pub fn add(&self, b: &Self) -> Self {
        if self.den == b.den {
            Rational::new(&self.num + &b.num, self.den.clone())
        } else {
            Rational::new(&self.num * &b.den + &b.num * &self.den, &self.den * &b.den).normalize()
        }
    }
    pub fn subtract(&self, b: &Self) -> Self {
        if self.den == b.den {
            Rational::new(&self.num - &b.num, self.den.clone())
        } else {
            Rational::new(&self.num * &b.den - &b.num * &self.den, &self.den * &b.den).normalize()
        }
    }
    pub fn multiply(&self, b: &Self) -> Self {
        Rational::new(&self.num * &b.num, &self.den * &b.den)
    }
    pub fn divide(&self, b: &Self) -> Self {
        let mut n = &self.num * &b.den;
        let mut d = &self.den * &b.num;
        if d.is_negative() {
            n = -n;
            d = -d;
        }
        Rational::new(n, d)
    }
    pub fn reciprocal(&self) -> Self {
        let (mut n, mut d) = (self.den.clone(), self.num.clone());
        if d.is_negative() {
            n = -n;
            d = -d;
        }
        Rational::new(n, d)
    }

    /// `Rnl.power`. An integer exponent is exact; a fractional one goes through floating point.
    /// Returns `None` for an even root of a negative number (Hurmet's `BAD_ROOT`).
    pub fn power(&self, b: &Self) -> Option<Self> {
        if b.num.is_zero() {
            return Some(Rational::one());
        }
        let b = b.normalize();
        if b.is_integer() {
            let e = idiv(&b.num, &b.den);
            if let Some(exact) = self.exact_power(&e) {
                return Some(exact);
            }
            // JavaScript throws a RangeError here; Hurmet's catch uses floating point.
            return Some(Rational::from_number(js_pow(self.to_f64(), b.to_f64())));
        }
        if self.is_positive() || b.greater_than(&Rational::one()) || b.less_than(&Rational::int(-1))
        {
            Some(Rational::from_number(js_pow(self.to_f64(), b.to_f64())))
        } else if b.modulo(&Rational::two()) == Rational::one() {
            Some(Rational::from_number(-js_pow(-self.to_f64(), b.to_f64())))
        } else {
            None
        }
    }

    fn exact_power(&self, e: &BigInt) -> Option<Self> {
        let neg = e.is_negative();
        let ee = e.abs().to_u32()?;
        let bits = self.num.bits().max(self.den.bits()).max(1);
        if bits.saturating_mul(ee as u64) > MAX_EXACT_POWER_BITS {
            return None;
        }
        if neg {
            // [a[1] ** -b, a[0] ** -b]: the sign is not moved to the numerator (as in Hurmet).
            Some(Rational::new(self.den.pow(ee), self.num.pow(ee)))
        } else {
            Some(Rational::new(self.num.pow(ee), self.den.pow(ee)))
        }
    }

    pub fn sqrt(&self) -> Self {
        Rational::from_number(self.to_f64().sqrt())
    }
    pub fn exp(&self) -> Self {
        Rational::from_number(jsmath::exp(self.to_f64()))
    }

    /// `Rnl.hypot`.
    pub fn hypot(&self, b: &Self) -> Self {
        let a = self.abs();
        let b = b.abs();
        let maximum = a.max(&b);
        let minimum = a.min(&b);
        let r = minimum.divide(&maximum);
        maximum.multiply(&r.multiply(&r).increment().sqrt())
    }

    /// `Rnl.mod`: |remainder| of the normalized quotient, as an integer.
    pub fn modulo(&self, b: &Self) -> Self {
        let q = self.normalize().divide(&b.normalize());
        Rational::new(irem(&q.num, &q.den).abs(), BigInt::one())
    }
    /// `Rnl.rem`.
    pub fn rem(&self, b: &Self) -> Self {
        let q = self.normalize().divide(&b.normalize());
        Rational::new(irem(&q.num, &q.den), BigInt::one())
    }
    /// `Rnl.gcd`; `None` unless both are integers.
    pub fn gcd(&self, b: &Self) -> Option<Self> {
        if !self.is_integer() || !b.is_integer() {
            return None;
        }
        Some(Rational::new(
            gcdi(&idiv(&self.num, &self.den), &idiv(&b.num, &b.den)),
            BigInt::one(),
        ))
    }

    pub fn are_equal(&self, b: &Self) -> bool {
        if self.den == b.den {
            self.num == b.num
        } else {
            &self.num * &b.den == &self.den * &b.num
        }
    }
    pub fn less_than(&self, b: &Self) -> bool {
        if self.is_negative() != b.is_negative() {
            self.is_negative()
        } else {
            self.subtract(b).is_negative()
        }
    }
    pub fn greater_than(&self, b: &Self) -> bool {
        if self.is_positive() != b.is_positive() {
            self.is_positive()
        } else {
            self.subtract(b).is_positive()
        }
    }
    pub fn less_than_or_equal(&self, b: &Self) -> bool {
        self.less_than(b) || self.are_equal(b)
    }
    pub fn greater_than_or_equal(&self, b: &Self) -> bool {
        self.greater_than(b) || self.are_equal(b)
    }
    pub fn max(&self, b: &Self) -> Self {
        if self.greater_than(b) {
            self.clone()
        } else {
            b.clone()
        }
    }
    pub fn min(&self, b: &Self) -> Self {
        if self.less_than(b) {
            self.clone()
        } else {
            b.clone()
        }
    }

    pub fn cos(&self) -> Self {
        if self.are_equal(&PI.divide(&Rational::two())) {
            Rational::zero()
        } else {
            Rational::from_number(jsmath::cos(self.to_f64()))
        }
    }
    pub fn sin(&self) -> Self {
        Rational::from_number(jsmath::sin(self.to_f64()))
    }
    /// `None` at exactly π/2 (Hurmet's `TAN90`).
    pub fn tan(&self) -> Option<Self> {
        if self.are_equal(&PI.divide(&Rational::two())) {
            None
        } else {
            Some(Rational::from_number(jsmath::tan(self.to_f64())))
        }
    }
    pub fn cosh(&self) -> Self {
        let x = self.to_f64();
        Rational::from_number((jsmath::exp(x) + jsmath::exp(-x)) / 2.0)
    }
    pub fn sinh(&self) -> Self {
        let x = self.to_f64();
        Rational::from_number((jsmath::exp(x) - jsmath::exp(-x)) / 2.0)
    }
    pub fn tanh(&self) -> Self {
        let x = self.to_f64();
        Rational::from_number(
            (jsmath::exp(x) - jsmath::exp(-x)) / (jsmath::exp(x) + jsmath::exp(-x)),
        )
    }

    /// `Rnl.toNumber`: `Number(num) / Number(den)`, or, when that overflows, the value
    /// rounded to 20 significant digits and read back.
    pub fn to_f64(&self) -> f64 {
        let x = jsnum::bigint_to_f64(&self.num) / jsnum::bigint_to_f64(&self.den);
        if !x.is_nan() && x != f64::INFINITY {
            return x;
        }
        if self.den.is_zero() {
            // JavaScript reaches `toStringSignificant` here: 0/0 reads back as 0, and n/0 throws.
            return if self.num.is_zero() { 0.0 } else { x };
        }
        jsnum::parse_number(&self.to_string_significant(20))
    }

    /// `Rnl.toStringSignificant`.
    pub fn to_string_significant(&self, n: i64) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let quotient = idiv(&self.num, &self.den).abs();
        if quotient > BigInt::zero() {
            self.to_fixed(n - quotient.to_string().len() as i64)
        } else {
            let inv = idiv(&self.den, &self.num).abs().to_string().len() as i64;
            self.to_fixed(inv + n - 1)
        }
    }

    /// `Rnl.toString(r, numDigitsAfterDecimal)`: round half away from zero to a fixed number
    /// of decimals (a negative count rounds to tens, hundreds, …).
    pub fn to_fixed(&self, digits: i64) -> String {
        if self.is_zero() {
            return "0".into();
        }
        if digits < 0 {
            let n = -digits;
            let sig = self
                .divide(&Rational::new(pow10(n), BigInt::one()))
                .to_fixed(0);
            return format!("{sig}{}", "0".repeat(n as usize));
        }
        let r = self.normalize();
        let quotient = idiv(&r.num, &r.den);
        let remainder = irem(&r.num, &r.den);
        let mut result = quotient.to_string();
        if remainder.is_zero() && digits > 0 {
            result.push('.');
            result.push_str(&"0".repeat(digits as usize));
        } else if !remainder.is_zero() {
            let remainder = remainder.abs();
            let scale = pow10(digits);
            let new_num = &remainder * &scale;
            let mut fractus = idiv(&new_num, &r.den);
            let residue = irem(&new_num, &r.den);
            let rounds_up = (BigInt::from(2) * &residue).abs() >= r.den.abs();
            let int_sign = BigInt::from(if quotient >= BigInt::zero() { 1 } else { -1 });
            if digits == 0 {
                return if rounds_up {
                    ((quotient.abs() + BigInt::one()) * &int_sign).to_string()
                } else {
                    result
                };
            }
            if rounds_up {
                fractus += 1;
                if fractus == scale {
                    return format!(
                        "{}.{}",
                        (quotient.abs() + BigInt::one()) * &int_sign,
                        "0".repeat(digits as usize)
                    );
                }
            }
            // Hurmet bug, reproduced: for -1 < r < 0 the quotient is 0, so the sign is lost.
            result.push('.');
            let f = fractus.to_string();
            result.push_str(&"0".repeat((digits as usize).saturating_sub(f.len())));
            result.push_str(&f);
        }
        result
    }

    /// `Rnl.factorial` (exact below 101, Lanczos above).
    pub fn factorial(&self) -> Self {
        if self.less_than(&Rational::int(101)) {
            let n = self.to_f64();
            if n >= 0.0 && n.fract() == 0.0 {
                return Rational::from_str_or_zero(FACTORIALS[n as usize]);
            }
            return Rational::zero();
        }
        lanczos(&self.increment())
    }

    /// `Rnl.doubleFactorial`.
    pub fn double_factorial(&self) -> Self {
        if self.less_than(&Rational::int(29)) {
            let n = self.to_f64();
            if n >= 0.0 && n.fract() == 0.0 {
                return Rational::from_str_or_zero(DOUBLE_FACTORIALS[n as usize]);
            }
            return Rational::zero();
        }
        let mut r = self.clone();
        let mut i = self.to_f64() - 2.0;
        while i > 0.0 {
            r = r.multiply(&Rational::from_number(i));
            i -= 2.0;
        }
        r
    }
}

/// `Rnl.lanczos`: the Lanczos approximation of Γ(x), given x + 1.
pub fn lanczos(x_plus_one: &Rational) -> Rational {
    let x = x_plus_one.subtract(&Rational::one());
    let term1 = Rational::two().multiply(&E.divide(&PI).sqrt());
    let base = x.add(&Rational::from_number(11.400511)).divide(&E);
    let term2 = base
        .power(&x.add(&Rational::new(big(1), big(2))))
        .unwrap_or_else(Rational::zero);
    let d = [
        "2.48574089138753565546e-5",
        "1.05142378581721974210",
        "-3.45687097222016235469",
        "4.51227709466894823700",
        "-2.98285225323576655721",
        "1.05639711577126713077",
        "-0.195428773191645869583",
        "0.0170970543404441224307",
        "-0.000571926117404305781283",
        "0.00000463399473359905636708",
        "-0.00000000271994908488607703910",
    ];
    let mut sum = Rational::from_str_or_zero(d[0]);
    for (k, dk) in d.iter().enumerate().skip(1) {
        sum = sum
            .add(&Rational::from_str_or_zero(dk).divide(&x.add(&Rational::from_number(k as f64))));
    }
    term1.multiply(&term2).multiply(&sum)
}

/// JavaScript's `**` on numbers (differs from `powf` only for `1 ** ±∞` and NaN exponents).
pub fn js_pow(a: f64, b: f64) -> f64 {
    if b.is_nan() || (a.abs() == 1.0 && b.is_infinite()) {
        f64::NAN
    } else {
        jsmath::pow(a, b)
    }
}

fn pow10(n: i64) -> BigInt {
    BigInt::from(10).pow(n.clamp(0, 100_000) as u32)
}

fn f64_int_to_bigint(x: f64) -> BigInt {
    use num_traits::FromPrimitive;
    BigInt::from_f64(x).unwrap_or_default()
}

/// The capture groups of [`NUMBER_PATTERN`] matched at the start of `s`, without the regex
/// crate. Index 0 is the whole match.
pub fn number_regex_captures(s: &str) -> Option<[Option<String>; 9]> {
    let b = s.as_bytes();
    let mut caps: [Option<String>; 9] = Default::default();
    let mut i = 0;
    if b.first() == Some(&b'-') {
        i = 1;
    }
    caps[1] = Some(s[..i].to_string());
    let digits = |from: usize| -> usize {
        let mut j = from;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        j
    };
    // (0x[0-9A-Fa-f]+)
    if b.len() > i + 2 && b[i] == b'0' && b[i + 1] == b'x' && b[i + 2].is_ascii_hexdigit() {
        let mut j = i + 2;
        while j < b.len() && b[j].is_ascii_hexdigit() {
            j += 1;
        }
        caps[2] = Some(s[i..j].to_string());
        caps[0] = Some(s[..j].to_string());
        return Some(caps);
    }
    let int_end = digits(i);
    if int_end == i {
        return None;
    }
    caps[3] = Some(s[i..int_end].to_string());
    let mut end = int_end;
    // (?: ([0-9]+)/([0-9]+) | ...)
    let mut mixed = false;
    if b.get(int_end) == Some(&b' ') {
        let n_end = digits(int_end + 1);
        if n_end > int_end + 1 && b.get(n_end) == Some(&b'/') {
            let d_end = digits(n_end + 1);
            if d_end > n_end + 1 {
                caps[4] = Some(s[int_end + 1..n_end].to_string());
                caps[5] = Some(s[n_end + 1..d_end].to_string());
                end = d_end;
                mixed = true;
            }
        }
    }
    if !mixed {
        if b.get(end) == Some(&b'.') {
            let f_end = digits(end + 1);
            if f_end > end + 1 {
                caps[6] = Some(s[end + 1..f_end].to_string());
                end = f_end;
            }
        }
        if b.get(end) == Some(&b'e') {
            let mut j = end + 1;
            if matches!(b.get(j), Some(b'+') | Some(b'-')) {
                j += 1;
            }
            let e_end = digits(j);
            if e_end > j {
                caps[7] = Some(s[end + 1..e_end].to_string());
                end = e_end;
            }
        } else if b.get(end) == Some(&b'%') {
            caps[8] = Some("%".into());
            end += 1;
        }
    }
    caps[0] = Some(s[..end].to_string());
    Some(caps)
}

const FACTORIALS: [&str; 101] = [
    "1",
    "1",
    "2",
    "6",
    "24",
    "120",
    "720",
    "5040",
    "40320",
    "362880",
    "3628800",
    "39916800",
    "479001600",
    "6227020800",
    "87178291200",
    "1307674368000",
    "20922789888000",
    "355687428096000",
    "6402373705728000",
    "121645100408832000",
    "2432902008176640000",
    "51090942171709440000",
    "1124000727777607680000",
    "25852016738884976640000",
    "620448401733239439360000",
    "15511210043330985984000000",
    "403291461126605635584000000",
    "10888869450418352160768000000",
    "304888344611713860501504000000",
    "8841761993739701954543616000000",
    "265252859812191058636308480000000",
    "8222838654177922817725562880000000",
    "263130836933693530167218012160000000",
    "8683317618811886495518194401280000000",
    "295232799039604140847618609643520000000",
    "10333147966386144929666651337523200000000",
    "371993326789901217467999448150835200000000",
    "13763753091226345046315979581580902400000000",
    "523022617466601111760007224100074291200000000",
    "20397882081197443358640281739902897356800000000",
    "815915283247897734345611269596115894272000000000",
    "33452526613163807108170062053440751665152000000000",
    "1405006117752879898543142606244511569936384000000000",
    "60415263063373835637355132068513997507264512000000000",
    "2658271574788448768043625811014615890319638528000000000",
    "119622220865480194561963161495657715064383733760000000000",
    "5502622159812088949850305428800254892961651752960000000000",
    "258623241511168180642964355153611979969197632389120000000000",
    "12413915592536072670862289047373375038521486354677760000000000",
    "608281864034267560872252163321295376887552831379210240000000000",
    "30414093201713378043612608166064768844377641568960512000000000000",
    "1551118753287382280224243016469303211063259720016986112000000000000",
    "80658175170943878571660636856403766975289505440883277824000000000000",
    "4274883284060025564298013753389399649690343788366813724672000000000000",
    "230843697339241380472092742683027581083278564571807941132288000000000000",
    "12696403353658275925965100847566516959580321051449436762275840000000000000",
    "710998587804863451854045647463724949736497978881168458687447040000000000000",
    "40526919504877216755680601905432322134980384796226602145184481280000000000000",
    "2350561331282878571829474910515074683828862318181142924420699914240000000000000",
    "138683118545689835737939019720389406345902876772687432540821294940160000000000000",
    "8320987112741390144276341183223364380754172606361245952449277696409600000000000000",
    "507580213877224798800856812176625227226004528988036003099405939480985600000000000000",
    "31469973260387937525653122354950764088012280797258232192163168247821107200000000000000",
    "1982608315404440064116146708361898137544773690227268628106279599612729753600000000000000",
    "126886932185884164103433389335161480802865516174545192198801894375214704230400000000000000",
    "8247650592082470666723170306785496252186258551345437492922123134388955774976000000000000000",
    "544344939077443064003729240247842752644293064388798874532860126869671081148416000000000000000",
    "36471110918188685288249859096605464427167635314049524593701628500267962436943872000000000000000",
    "2480035542436830599600990418569171581047399201355367672371710738018221445712183296000000000000000",
    "171122452428141311372468338881272839092270544893520369393648040923257279754140647424000000000000000",
    "11978571669969891796072783721689098736458938142546425857555362864628009582789845319680000000000000000",
    "850478588567862317521167644239926010288584608120796235886430763388588680378079017697280000000000000000",
    "61234458376886086861524070385274672740778091784697328983823014963978384987221689274204160000000000000000",
    "4470115461512684340891257138125051110076800700282905015819080092370422104067183317016903680000000000000000",
    "330788544151938641225953028221253782145683251820934971170611926835411235700971565459250872320000000000000000",
    "24809140811395398091946477116594033660926243886570122837795894512655842677572867409443815424000000000000000000",
    "1885494701666050254987932260861146558230394535379329335672487982961844043495537923117729972224000000000000000000",
    "145183092028285869634070784086308284983740379224208358846781574688061991349156420080065207861248000000000000000000",
    "11324281178206297831457521158732046228731749579488251990048962825668835325234200766245086213177344000000000000000000",
    "894618213078297528685144171539831652069808216779571907213868063227837990693501860533361810841010176000000000000000000",
    "71569457046263802294811533723186532165584657342365752577109445058227039255480148842668944867280814080000000000000000000",
    "5797126020747367985879734231578109105412357244731625958745865049716390179693892056256184534249745940480000000000000000000",
    "475364333701284174842138206989404946643813294067993328617160934076743994734899148613007131808479167119360000000000000000000",
    "39455239697206586511897471180120610571436503407643446275224357528369751562996629334879591940103770870906880000000000000000000",
    "3314240134565353266999387579130131288000666286242049487118846032383059131291716864129885722968716753156177920000000000000000000",
    "281710411438055027694947944226061159480056634330574206405101912752560026159795933451040286452340924018275123200000000000000000000",
    "24227095383672732381765523203441259715284870552429381750838764496720162249742450276789464634901319465571660595200000000000000000000",
    "2107757298379527717213600518699389595229783738061356212322972511214654115727593174080683423236414793504734471782400000000000000000000",
    "185482642257398439114796845645546284380220968949399346684421580986889562184028199319100141244804501828416633516851200000000000000000000",
    "16507955160908461081216919262453619309839666236496541854913520707833171034378509739399912570787600662729080382999756800000000000000000000",
    "1485715964481761497309522733620825737885569961284688766942216863704985393094065876545992131370884059645617234469978112000000000000000000000",
    "135200152767840296255166568759495142147586866476906677791741734597153670771559994765685283954750449427751168336768008192000000000000000000000",
    "12438414054641307255475324325873553077577991715875414356840239582938137710983519518443046123837041347353107486982656753664000000000000000000000",
    "1156772507081641574759205162306240436214753229576413535186142281213246807121467315215203289516844845303838996289387078090752000000000000000000000",
    "108736615665674308027365285256786601004186803580182872307497374434045199869417927630229109214583415458560865651202385340530688000000000000000000000",
    "10329978488239059262599702099394727095397746340117372869212250571234293987594703124871765375385424468563282236864226607350415360000000000000000000000",
    "991677934870949689209571401541893801158183648651267795444376054838492222809091499987689476037000748982075094738965754305639874560000000000000000000000",
    "96192759682482119853328425949563698712343813919172976158104477319333745612481875498805879175589072651261284189679678167647067832320000000000000000000000",
    "9426890448883247745626185743057242473809693764078951663494238777294707070023223798882976159207729119823605850588608460429412647567360000000000000000000000",
    "933262154439441526816992388562667004907159682643816214685929638952175999932299156089414639761565182862536979208272237582511852109168640000000000000000000000",
    "93326215443944152681699238856266700490715968264381621468592963895217599993229915608941463976156518286253697920827223758251185210916864000000000000000000000000",
];

const DOUBLE_FACTORIALS: [&str; 29] = [
    "1",
    "1",
    "2",
    "3",
    "8",
    "15",
    "48",
    "105",
    "384",
    "945",
    "3840",
    "10395",
    "46080",
    "135135",
    "645120",
    "2027025",
    "10321920",
    "34459425",
    "185794560",
    "654729075",
    "3715891200",
    "13749310575",
    "81749606400",
    "316234143225",
    "1961990553600",
    "7905853580625",
    "51011754393600",
    "213458046676875",
    "1428329123020800",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> Rational {
        Rational::from_str_hurmet(s).unwrap()
    }

    #[test]
    fn literals() {
        assert_eq!(r("1.2"), Rational::new(big(12), big(10)));
        assert_eq!(r("5 2/3"), Rational::new(big(17), big(3)));
        assert_eq!(r("-3 3/8"), Rational::new(big(-27), big(8)));
        assert_eq!(r("3.3e4"), Rational::int(33000));
        assert_eq!(r("15%"), Rational::new(big(15), big(100)));
        assert_eq!(r("0x1F"), Rational::int(31));
        assert!(Rational::from_str_hurmet("abc").is_none());
    }

    #[test]
    fn from_number() {
        assert_eq!(Rational::from_number(0.1), Rational::new(big(1), big(10)));
        assert_eq!(
            Rational::from_number(1e-6),
            Rational::new(big(1), big(1_000_000))
        );
        assert_eq!(Rational::from_number(1000.0), Rational::int(1000));
    }

    #[test]
    fn fixed() {
        assert_eq!(r("2.345").to_fixed(2), "2.35");
        assert_eq!(r("-2.5").to_fixed(0), "-3");
        assert_eq!(r("1234").to_fixed(-2), "1200");
        assert_eq!(r("0.875").to_string_significant(2), "0.88");
    }

    #[test]
    fn arithmetic() {
        assert!(r("5 2/3").add(&r("1 1/3")).are_equal(&Rational::int(7)));
        assert_eq!(
            Rational::int(2).power(&Rational::int(-2)).unwrap(),
            Rational::new(big(1), big(4))
        );
        // Hurmet quirk: mod(1/2, 2) is 1, so this takes the odd-root branch.
        assert_eq!(
            Rational::int(-4)
                .power(&Rational::new(big(1), big(2)))
                .unwrap(),
            Rational::int(-2)
        );
        assert!(
            Rational::int(-4)
                .power(&Rational::new(big(2), big(3)))
                .is_none()
        );
        assert_eq!(
            Rational::int(30).double_factorial().to_fixed(0),
            "42849873690624000"
        );
    }
}
