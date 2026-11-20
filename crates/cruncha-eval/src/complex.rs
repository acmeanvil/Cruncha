// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Complex numbers with rational parts (Hurmet's `complex.js`).

use crate::error::{err, err0};
use crate::format::{Mode, fmt};
use crate::value::Operand;
use cruncha_units::{Rational, jsmath, jsnum};
use num_bigint::BigInt;

pub(crate) type Z = [Rational; 2];
type R = Result<Z, Operand>;

fn r0() -> Rational {
    Rational::zero()
}

pub(crate) fn j() -> Z {
    [r0(), Rational::one()]
}

pub(crate) fn abs(z: &Z) -> Rational {
    z[0].hypot(&z[1])
}
pub(crate) fn negate(z: &Z) -> Z {
    [z[0].negate(), z[1].negate()]
}
pub(crate) fn conjugate(z: &Z) -> Z {
    [z[0].clone(), z[1].negate()]
}

pub(crate) fn angle(z: &Z) -> Result<Rational, Operand> {
    if z[0].is_zero() && z[1].is_zero() {
        Err(err("ORIGIN", "angle"))
    } else if z[1].is_zero() {
        Ok(if z[0].is_positive() {
            r0()
        } else {
            Rational::pi()
        })
    } else if z[0].is_zero() {
        let half = Rational::pi().divide(&Rational::two());
        Ok(if z[1].is_positive() {
            half
        } else {
            half.negate()
        })
    } else {
        Ok(Rational::from_number(jsmath::atan2(
            z[1].to_f64(),
            z[0].to_f64(),
        )))
    }
}

pub(crate) fn add(x: &Z, y: &Z) -> Z {
    [x[0].add(&y[0]), x[1].add(&y[1])]
}
pub(crate) fn subtract(x: &Z, y: &Z) -> Z {
    [x[0].subtract(&y[0]), x[1].subtract(&y[1])]
}
pub(crate) fn multiply(x: &Z, y: &Z) -> Z {
    [
        x[0].multiply(&y[0]).subtract(&x[1].multiply(&y[1])),
        x[0].multiply(&y[1]).add(&x[1].multiply(&y[0])),
    ]
}

pub(crate) fn divide(x: &Z, y: &Z) -> R {
    if !x[1].is_zero() && !y[1].is_zero() {
        if y[1].abs().less_than(&y[0].abs()) {
            let ratio = y[1].divide(&y[0]);
            let denom = y[0].add(&y[1].multiply(&ratio));
            Ok([
                x[0].add(&x[1].multiply(&ratio)).divide(&denom),
                x[1].subtract(&x[0].multiply(&ratio)).divide(&denom),
            ])
        } else {
            let ratio = y[0].divide(&y[1]);
            let denom = y[1].add(&y[0].multiply(&ratio));
            Ok([
                x[1].add(&x[0].multiply(&ratio)).divide(&denom),
                x[0].negate().add(&x[1].multiply(&ratio)).divide(&denom),
            ])
        }
    } else if x[1].is_zero() {
        if y[1].abs().less_than(&y[0].abs()) {
            let ratio = y[1].divide(&y[0]);
            let denom = y[0].add(&y[1].multiply(&ratio));
            Ok([
                x[0].divide(&denom),
                x[0].multiply(&ratio.divide(&denom)).negate(),
            ])
        } else {
            let ratio = y[0].divide(&y[1]);
            let denom = y[1].add(&y[0].multiply(&ratio));
            Ok([
                x[0].multiply(&ratio).divide(&denom),
                x[0].divide(&denom).negate(),
            ])
        }
    } else if y[1].is_zero() {
        if y[0].is_zero() {
            // Hurmet returns undefined here (a TODO); the port reports division by zero.
            Err(err0("DIV"))
        } else {
            Ok([x[0].divide(&y[0]), x[1].divide(&y[0])])
        }
    } else if y[0].is_zero() {
        Err(err0("DIV"))
    } else {
        Ok([x[0].divide(&y[0]), r0()])
    }
}

pub(crate) fn increment(z: &Z) -> Z {
    [z[0].increment(), z[1].clone()]
}
pub(crate) fn decrement(z: &Z) -> Z {
    [z[0].decrement(), z[1].clone()]
}

pub(crate) fn inverse(z: &Z) -> R {
    if z[1].is_zero() {
        if z[0].is_zero() {
            return Err(err0("DIV"));
        }
        // Hurmet calls the non-existent Rnl.inverse here (a TypeError); this is its intent.
        Ok([z[0].reciprocal(), r0()])
    } else {
        divide(&[Rational::one(), r0()], z)
    }
}

pub(crate) fn cos(z: &Z) -> Z {
    [
        z[0].cos().multiply(&z[1].cosh()),
        z[0].sin().negate().multiply(&z[1].sinh()),
    ]
}
pub(crate) fn sin(z: &Z) -> Z {
    [
        z[0].sin().multiply(&z[1].cosh()),
        z[0].cos().multiply(&z[1].sinh()),
    ]
}

pub(crate) fn log(x: &Z) -> R {
    if x[0].is_zero() && x[1].is_zero() {
        return Err(err("ORIGIN", "log"));
    }
    Ok([
        Rational::from_number(jsmath::log(abs(x).to_f64())),
        angle(x)?,
    ])
}

fn is_small(x: &Rational) -> bool {
    x.abs().less_than(&Rational::new(
        BigInt::from(1),
        BigInt::from(100_000_000_000_000i64),
    ))
}

pub(crate) fn exp(x: &Z) -> Z {
    if is_small(&x[1]) {
        [x[0].exp(), r0()]
    } else if x[0].is_zero() {
        let mut re = x[1].cos();
        if is_small(&re) {
            re = r0();
        }
        let mut im = x[1].sin();
        if is_small(&im) {
            im = r0();
        }
        [re, im]
    } else {
        let real_exp = x[0].exp();
        [
            real_exp.multiply(&x[1].cos()),
            real_exp.multiply(&x[1].sin()),
        ]
    }
}

/// `Cpx.power(x, y)`; `y` is either a real (`Err` side of the enum) or a complex number.
pub(crate) fn power(x: &Z, y: &Exponent) -> R {
    let mut z = match y {
        Exponent::Real(y) => {
            let l = log(x)?;
            [l[0].multiply(y), l[1].multiply(y)]
        }
        Exponent::Complex(y) if y[1].is_zero() => {
            let l = log(x)?;
            [l[0].multiply(&y[0]), l[1].multiply(&y[0])]
        }
        Exponent::Complex(y) if x[1].is_zero() && !x[0].is_negative() => {
            let lx = Rational::from_number(jsmath::log(x[0].to_f64()));
            [lx.multiply(&y[0]), lx.multiply(&y[1])]
        }
        Exponent::Complex(y) => {
            let l = log(x)?;
            [
                l[0].multiply(&y[0]).subtract(&l[1].multiply(&y[1])),
                l[1].multiply(&y[0]).add(&l[0].multiply(&y[1])),
            ]
        }
    };
    z = exp(&z);
    if is_small(&z[1]) {
        z[1] = r0();
    }
    if is_small(&z[0]) {
        z[0] = r0();
    }
    Ok(z)
}

pub(crate) enum Exponent {
    Real(Rational),
    Complex(Z),
}

pub(crate) fn sqrt(x: &Z) -> R {
    let mut z = log(x)?;
    z[0] = z[0].divide(&Rational::two());
    z[1] = z[1].divide(&Rational::two());
    Ok(exp(&z))
}

pub(crate) fn acosh(z: &Z) -> R {
    log(&add(
        z,
        &multiply(&sqrt(&decrement(z))?, &sqrt(&increment(z))?),
    ))
}

pub(crate) fn asinh(z: &Z) -> R {
    let s = sqrt(&add(&multiply(z, z), &[Rational::one(), r0()]))?;
    log(&add(z, &s))
}

pub(crate) fn atanh(z: &Z) -> R {
    divide(
        &subtract(
            &log(&increment(z))?,
            &log(&subtract(&[Rational::one(), r0()], z))?,
        ),
        &[Rational::two(), r0()],
    )
}

pub(crate) fn asin(z: &Z) -> R {
    divide(&asinh(&multiply(&j(), z))?, &j())
}

pub(crate) fn atan(z: &Z) -> R {
    let jz = multiply(&j(), z);
    let term1 = log(&increment(&jz))?;
    let term2 = log(&subtract(&[Rational::one(), r0()], &jz))?;
    divide(&subtract(&term1, &term2), &[r0(), Rational::two()])
}

/// `Cpx.display`: (TeX, plain text).
pub(crate) fn display(z: &Z, format_spec: &str, decimal_format: &str) -> (String, String) {
    let complex_spec = if format_spec.contains(['∠', '°']) {
        format_spec.chars().last().unwrap_or('j')
    } else {
        'j'
    };
    if complex_spec == 'j' {
        let real = fmt(&z[0], format_spec, decimal_format, Mode::Tex);
        let im = fmt(&z[1], format_spec, decimal_format, Mode::Tex);
        if let Some(stripped) = im.strip_prefix('-') {
            let _ = stripped;
            // JavaScript `-imPart` coerces the string to a number and negates it.
            let neg = jsnum::to_string(-jsnum::parse_number(&im));
            (format!("{real} - j\\,{neg}"), format!("{real} - j {neg}"))
        } else {
            (format!("{real} + j\\, {im}"), format!("{real} + j {im}"))
        }
    } else {
        let mag = z[0].hypot(&z[1]);
        let mut theta = angle(z).unwrap_or_else(|_| r0());
        let in_degrees = complex_spec == '°';
        if in_degrees {
            theta = theta.multiply(&Rational::int(180)).divide(&Rational::pi());
        }
        let s = format!(
            "{}∠{}{}",
            fmt(&mag, format_spec, decimal_format, Mode::Tex),
            fmt(&theta, format_spec, decimal_format, Mode::Tex),
            if in_degrees { "°" } else { "" }
        );
        (s.clone(), s)
    }
}
