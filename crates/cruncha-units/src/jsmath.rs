// SPDX-License-Identifier: GPL-3.0-or-later
//
// `sin`, `cos`, `log` and `log10` below are ported from fdlibm 5.3, as adapted in V8's
// `src/base/ieee754.cc`:
//
//   Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//   Developed at SunSoft, a Sun Microsystems, Inc. business.
//   Permission to use, copy, modify, and distribute this software is freely granted,
//   provided that this notice is preserved.

//! JavaScript's `Math` functions, bit for bit.
//!
//! V8 implements `Math.sin`, `Math.log` and friends with fdlibm, which is not always correctly
//! rounded. The platform's libm (or the `libm` crate, a port of musl, which took newer FreeBSD
//! and ARM code) can differ in the last bit, and that shows in Hurmet's 15-digit results
//! (`sin(2.34)` is …261 in V8 and …263 in macOS libm). So `sin`, `cos`, `log` and `log10` are
//! ported from fdlibm here, and the rest use the `libm` crate, whose results match V8's on the
//! test vectors in `tests/fixtures/v8_math.json`.
//!
//! Everything here is deterministic across platforms (pure Rust, no platform libm), except
//! that V8 itself is not: its `Math.pow` calls the C library's `pow`. This module uses the
//! `libm` crate's `pow` (fdlibm's algorithm) instead, so results do not depend on the OS; it
//! differs from Node.js on macOS in the last bit for about 1 argument in 10.

fn hi(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}
fn lo(x: f64) -> u32 {
    x.to_bits() as u32
}
fn with_hi(x: f64, h: i32) -> f64 {
    f64::from_bits(((h as u32 as u64) << 32) | (lo(x) as u64))
}
fn from_words(h: u32, l: u32) -> f64 {
    f64::from_bits(((h as u64) << 32) | l as u64)
}

fn kernel_sin(x: f64, y: f64, iy: i32) -> f64 {
    const HALF: f64 = 0.5;
    let s1 = from_words(0xBFC55555, 0x55555549);
    let s2 = from_words(0x3F811111, 0x1110F8A6);
    let s3 = from_words(0xBF2A01A0, 0x19C161D5);
    let s4 = from_words(0x3EC71DE3, 0x57B1FE7D);
    let s5 = from_words(0xBE5AE5E6, 0x8A2B9CEB);
    let s6 = from_words(0x3DE5D93A, 0x5ACFD57C);
    let ix = hi(x) & 0x7fffffff;
    if ix < 0x3e400000 && (x as i32) == 0 {
        return x;
    }
    let z = x * x;
    let v = z * x;
    let r = s2 + z * (s3 + z * (s4 + z * (s5 + z * s6)));
    if iy == 0 {
        x + v * (s1 + z * r)
    } else {
        x - ((z * (HALF * y - v * r) - y) - v * s1)
    }
}

fn kernel_cos(x: f64, y: f64) -> f64 {
    const ONE: f64 = 1.0;
    let c1 = from_words(0x3FA55555, 0x5555554C);
    let c2 = from_words(0xBF56C16C, 0x16C15177);
    let c3 = from_words(0x3EFA01A0, 0x19CB1590);
    let c4 = from_words(0xBE927E4F, 0x809C52AD);
    let c5 = from_words(0x3E21EE9E, 0xBDB4B1C4);
    let c6 = from_words(0xBDA8FAE9, 0xBE8838D4);
    let ix = hi(x) & 0x7fffffff;
    if ix < 0x3e400000 && (x as i32) == 0 {
        return ONE;
    }
    let z = x * x;
    let r = z * (c1 + z * (c2 + z * (c3 + z * (c4 + z * (c5 + z * c6)))));
    if ix < 0x3FD33333 {
        ONE - (0.5 * z - (z * r - x * y))
    } else {
        let qx = if ix > 0x3fe90000 {
            0.28125
        } else {
            from_words((ix - 0x00200000) as u32, 0)
        };
        let hz = 0.5 * z - qx;
        let a = ONE - qx;
        a - (hz - (z * r - x * y))
    }
}

/// `__ieee754_rem_pio2` for |x| ≤ 2^19·π/2: (n, y0, y1) with x = n·π/2 + y0 + y1.
/// `None` for larger arguments.
fn rem_pio2(x: f64) -> Option<(i32, f64, f64)> {
    let invpio2 = from_words(0x3FE45F30, 0x6DC9C883);
    let pio2_1 = from_words(0x3FF921FB, 0x54400000);
    let pio2_1t = from_words(0x3DD0B461, 0x1A626331);
    let pio2_2 = from_words(0x3DD0B461, 0x1A600000);
    let pio2_2t = from_words(0x3BA3198A, 0x2E037073);
    let pio2_3 = from_words(0x3BA3198A, 0x2E000000);
    let pio2_3t = from_words(0x397B839A, 0x252049C1);
    let hx = hi(x);
    let ix = hx & 0x7fffffff;
    if ix <= 0x3fe921fb {
        return Some((0, x, 0.0));
    }
    if ix < 0x4002d97c {
        if hx > 0 {
            let mut z = x - pio2_1;
            let (y0, y1);
            if ix != 0x3ff921fb {
                y0 = z - pio2_1t;
                y1 = (z - y0) - pio2_1t;
            } else {
                z -= pio2_2;
                y0 = z - pio2_2t;
                y1 = (z - y0) - pio2_2t;
            }
            return Some((1, y0, y1));
        } else {
            let mut z = x + pio2_1;
            let (y0, y1);
            if ix != 0x3ff921fb {
                y0 = z + pio2_1t;
                y1 = (z - y0) + pio2_1t;
            } else {
                z += pio2_2;
                y0 = z + pio2_2t;
                y1 = (z - y0) + pio2_2t;
            }
            return Some((-1, y0, y1));
        }
    }
    if ix <= 0x413921fb {
        let t = x.abs();
        let n = (t * invpio2 + 0.5) as i32;
        let fnum = n as f64;
        let mut r = t - fnum * pio2_1;
        let mut w = fnum * pio2_1t;
        let npio2_hw = |k: i32| -> i32 { hi(k as f64 * std::f64::consts::FRAC_PI_2) };
        let mut y0;
        if n < 32 && ix != npio2_hw(n) {
            y0 = r - w;
        } else {
            let j = ix >> 20;
            y0 = r - w;
            let mut i = j - ((hi(y0) >> 20) & 0x7ff);
            if i > 16 {
                let t = r;
                w = fnum * pio2_2;
                r = t - w;
                w = fnum * pio2_2t - ((t - r) - w);
                y0 = r - w;
                i = j - ((hi(y0) >> 20) & 0x7ff);
                if i > 49 {
                    let t = r;
                    w = fnum * pio2_3;
                    r = t - w;
                    w = fnum * pio2_3t - ((t - r) - w);
                    y0 = r - w;
                }
            }
        }
        let y1 = (r - y0) - w;
        if hx < 0 {
            return Some((-n, -y0, -y1));
        }
        return Some((n, y0, y1));
    }
    None
}

/// `Math.sin`.
pub fn sin(x: f64) -> f64 {
    let ix = hi(x) & 0x7fffffff;
    if ix <= 0x3fe921fb {
        return kernel_sin(x, 0.0, 0);
    }
    if ix >= 0x7ff00000 {
        return f64::NAN;
    }
    match rem_pio2(x) {
        Some((n, y0, y1)) => match n & 3 {
            0 => kernel_sin(y0, y1, 1),
            1 => kernel_cos(y0, y1),
            2 => -kernel_sin(y0, y1, 1),
            _ => -kernel_cos(y0, y1),
        },
        // |x| > 2^19·π/2 needs fdlibm's __kernel_rem_pio2, which is not ported.
        None => libm::sin(x),
    }
}

/// `Math.cos`.
pub fn cos(x: f64) -> f64 {
    let ix = hi(x) & 0x7fffffff;
    if ix <= 0x3fe921fb {
        return kernel_cos(x, 0.0);
    }
    if ix >= 0x7ff00000 {
        return f64::NAN;
    }
    match rem_pio2(x) {
        Some((n, y0, y1)) => match n & 3 {
            0 => kernel_cos(y0, y1),
            1 => -kernel_sin(y0, y1, 1),
            2 => -kernel_cos(y0, y1),
            _ => kernel_sin(y0, y1, 1),
        },
        None => libm::cos(x),
    }
}

pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// `Math.log` (fdlibm `__ieee754_log`).
pub fn log(x: f64) -> f64 {
    let ln2_hi = from_words(0x3fe62e42, 0xfee00000);
    let ln2_lo = from_words(0x3dea39ef, 0x35793c76);
    let two54 = from_words(0x43500000, 0);
    let lg1 = from_words(0x3FE55555, 0x55555593);
    let lg2 = from_words(0x3FD99999, 0x9997FA04);
    let lg3 = from_words(0x3FD24924, 0x94229359);
    let lg4 = from_words(0x3FCC71C5, 0x1D8E78AF);
    let lg5 = from_words(0x3FC74664, 0x96CB03DE);
    let lg6 = from_words(0x3FC39A09, 0xD078C69F);
    let lg7 = from_words(0x3FC2F112, 0xDF3E5244);
    let mut x = x;
    let mut hx = hi(x);
    let lx = lo(x);
    let mut k: i32 = 0;
    if hx < 0x00100000 {
        if ((hx & 0x7fffffff) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= two54;
        hx = hi(x);
    }
    if hx >= 0x7ff00000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000fffff;
    let i = (hx + 0x95f64) & 0x100000;
    x = with_hi(x, hx | (i ^ 0x3ff00000));
    k += i >> 20;
    let f = x - 1.0;
    if (0x000fffff & (2 + hx)) < 3 {
        if f == 0.0 {
            if k == 0 {
                return 0.0;
            }
            let dk = k as f64;
            return dk * ln2_hi + dk * ln2_lo;
        }
        #[allow(clippy::excessive_precision)]
        let r = f * f * (0.5 - 0.33333333333333333 * f);
        if k == 0 {
            return f - r;
        }
        let dk = k as f64;
        return dk * ln2_hi - ((r - dk * ln2_lo) - f);
    }
    let s = f / (2.0 + f);
    let dk = k as f64;
    let z = s * s;
    let mut i = hx - 0x6147a;
    let w = z * z;
    let j = 0x6b851 - hx;
    let t1 = w * (lg2 + w * (lg4 + w * lg6));
    let t2 = z * (lg1 + w * (lg3 + w * (lg5 + w * lg7)));
    i |= j;
    let r = t2 + t1;
    if i > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            f - (hfsq - s * (hfsq + r))
        } else {
            dk * ln2_hi - ((hfsq - (s * (hfsq + r) + dk * ln2_lo)) - f)
        }
    } else if k == 0 {
        f - s * (f - r)
    } else {
        dk * ln2_hi - ((s * (f - r) - dk * ln2_lo) - f)
    }
}

/// `Math.log10` (fdlibm `__ieee754_log10`).
pub fn log10(x: f64) -> f64 {
    let two54 = from_words(0x43500000, 0);
    let ivln10 = from_words(0x3FDBCB7B, 0x1526E50E);
    let log10_2hi = from_words(0x3FD34413, 0x509F6000);
    let log10_2lo = from_words(0x3D59FEF3, 0x11F12B36);
    let mut x = x;
    let mut hx = hi(x);
    let lx = lo(x);
    let mut k: i32 = 0;
    if hx < 0x00100000 {
        if ((hx & 0x7fffffff) as u32 | lx) == 0 {
            return f64::NEG_INFINITY;
        }
        if hx < 0 {
            return f64::NAN;
        }
        k -= 54;
        x *= two54;
        hx = hi(x);
    }
    if hx >= 0x7ff00000 {
        return x + x;
    }
    k += (hx >> 20) - 1023;
    let i = ((k as u32 & 0x80000000) >> 31) as i32;
    hx = (hx & 0x000fffff) | ((0x3ff - i) << 20);
    let y = (k + i) as f64;
    x = with_hi(x, hx);
    let z = y * log10_2lo + ivln10 * log(x);
    z + y * log10_2hi
}

pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// `Math.pow` / `**`, with JavaScript's NaN for `±1 ** ±∞` (C's `pow` gives 1). V8 calls the
/// C library's `pow`, which differs between platforms; this uses fdlibm's algorithm (the
/// `libm` crate) everywhere. `x ** 0.5` is exactly `√x`, as in both.
pub fn pow(x: f64, y: f64) -> f64 {
    if y.is_nan() || (x.abs() == 1.0 && y.is_infinite()) {
        f64::NAN
    } else {
        libm::pow(x, y)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn matches_v8() {
        // Values printed by Node.js 24 (V8).
        assert_eq!(super::sin(2.34), 0.7184647930691261);
        assert_eq!(super::sin(-5.67), 0.5754753801952172);
        assert_eq!(super::cos(2.34), -0.695563326462902);
        assert_eq!(super::exp(5.67), 290.03453439173467);
    }
}
