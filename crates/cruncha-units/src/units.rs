// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Unit names → conversion factor, gauge offset and dimension exponents (Hurmet's `units.js`).
//!
//! A unit string such as `kip·ft`, `lbf/ft`, `N·m/s`, `in²`, `m^2` or `in⁻¹` is parsed with a
//! small shunting-yard pass into an RPN of simple units, numbers and the operators `·`, `/` and
//! `^`. Each simple unit is looked up in the table, after stripping a plural `s` and an SI prefix
//! (long form such as `kilo` for `siWord` units, short form such as `k` for `siSymbol` units).

use crate::rational::Rational;
use crate::table::UNIT_TABLE;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Exponents of the eight base dimensions: length (m), mass (kg), time (s), electric current
/// (A), temperature (K), amount (unit), luminous intensity (cd), money (EUR).
///
/// They are floats because Hurmet allows roots: `√(m²)` is `m`, `√m` has the exponent 0.5.
pub type Dims = [f64; 8];

/// Names of the eight base dimensions, in the order of [`Dims`].
pub const DIM_NAMES: [&str; 8] = [
    "length",
    "mass",
    "time",
    "current",
    "temperature",
    "amount",
    "luminous intensity",
    "money",
];

/// The exponents of a dimensionless number.
pub const ALL_ZEROS: Dims = [0.0; 8];

/// A unit of measure. A value `v` in this unit is `(v + gauge) × factor` in base units.
#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub name: String,
    pub factor: Rational,
    pub gauge: Rational,
    pub expos: Dims,
}

/// Currency exchange rates: currency code (e.g. `"USD"`) → units per one euro, as a decimal
/// string (e.g. `"1.1014"`). Hurmet reads them from `globalThis.currencyRates`; here the caller
/// supplies them. A monetary unit with no rate is an error.
pub type Currencies = HashMap<String, String>;

/// Why a unit name could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitError {
    /// Hurmet's `UNIT_NAME`: not in the table.
    UnknownName(String),
    /// Hurmet's `CURRENCY`: a monetary unit with no exchange rate.
    Currency,
    /// Hurmet's `DIV`: a compound unit divided by zero.
    DivideByZero,
    /// The compound unit is malformed (Hurmet would throw a JavaScript error).
    Malformed(String),
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitError::UnknownName(n) => write!(f, "Error. Unrecognized unit name: {n}"),
            UnitError::Currency => write!(
                f,
                "Error. Currency exchange rates must be defined before using a monetary unit."
            ),
            UnitError::DivideByZero => write!(f, "Error. Divide by zero."),
            UnitError::Malformed(n) => write!(f, "Error. Unrecognized unit name: {n}"),
        }
    }
}

impl std::error::Error for UnitError {}

struct Row {
    factor_num: &'static str,
    factor_den: &'static str,
    gauge: &'static str,
    misc: &'static str,
    expos: Dims,
}

static TABLE: LazyLock<HashMap<&'static str, Row>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(UNIT_TABLE.len());
    for &(name, factor_num, factor_den, gauge, misc, ex) in UNIT_TABLE {
        let mut expos = ALL_ZEROS;
        for (e, x) in expos.iter_mut().zip(ex) {
            *e = f64::from(x);
        }
        // Later rows replace earlier ones, as in JSON.parse.
        map.insert(
            name,
            Row {
                factor_num,
                factor_den,
                gauge,
                misc,
                expos,
            },
        );
    }
    map
});

/// Does the table contain this exact name?
pub fn is_known_unit(name: &str) -> bool {
    TABLE.contains_key(name)
}

fn synonym(word: &str) -> Option<&'static str> {
    Some(match word {
        "$" | "US$" | "dollar" => "USD",
        "A$" | "Australian dollar" => "AUD",
        "Brazilian real" | "real" | "R$" => "BRL",
        "British pound" | "£" => "GBP",
        "C$" | "Canadian dollar" => "CAD",
        "Chinese yuan" | "yuan" | "RMB" | "CN¥" => "CNY",
        "€" | "Euro" | "euro" => "EUR",
        "HK$" | "Hong Kong dollar" => "HKD",
        "Indian rupee" | "rupee" | "₨" | "₹" | "Rs" => "INR",
        "Israeli New Shekel" | "₪" => "ILS",
        "Mexican peso" => "MXN",
        "Swiss franc" | "franc" | "fr" | "Fr" | "₣" => "CHF",
        "Won" | "won" => "KRW",
        "yen" | "Yen" | "Japanese yen" | "¥" => "JPY",
        _ => return None,
    })
}

fn prefix_factor(prefix: &str) -> f64 {
    match prefix {
        "Y" | "yotta" => 1e24,
        "Z" | "zetta" => 1e21,
        "E" | "exa" => 1e18,
        "P" | "peta" => 1e15,
        "T" | "tera" => 1e12,
        "G" | "giga" => 1e9,
        "M" | "mega" => 1e6,
        "k" | "kilo" => 1000.0,
        "h" | "hecto" => 100.0,
        "deka" => 10.0,
        "d" | "deci" => 0.1,
        "c" | "centi" => 0.01,
        "m" | "milli" => 0.001,
        "µ" | "μ" | "micro" => 1e-6,
        "n" | "nano" => 1e-9,
        "p" | "pico" => 1e-12,
        "f" | "femto" => 1e-15,
        "a" | "atto" => 1e-18,
        "z" | "zepto" => 1e-21,
        "y" | "yocto" => 1e-24,
        "Ki" | "kibi" => 1024.0,
        "Mi" | "mebi" => 1_048_576.0,
        "Gi" | "gibi" => 1_073_741_824.0,
        "Ti" | "tebi" => 1_099_511_627_776.0,
        // Hurmet's prefixFactor lookup gives `undefined` here; fromNumber then throws.
        _ => f64::NAN,
    }
}

/// The prefixes `prefix_factor` knows, largest first: symbol (empty when only the word works),
/// word, factor. For the Units palette (`catalog::prefixes`).
pub(crate) const SI_PREFIX_LIST: &[(&str, &str, f64)] = &[
    ("Y", "yotta", 1e24),
    ("Z", "zetta", 1e21),
    ("E", "exa", 1e18),
    ("P", "peta", 1e15),
    ("T", "tera", 1e12),
    ("G", "giga", 1e9),
    ("M", "mega", 1e6),
    ("k", "kilo", 1e3),
    ("h", "hecto", 1e2),
    ("", "deka", 1e1),
    ("d", "deci", 1e-1),
    ("c", "centi", 1e-2),
    ("m", "milli", 1e-3),
    ("µ", "micro", 1e-6),
    ("n", "nano", 1e-9),
    ("p", "pico", 1e-12),
    ("f", "femto", 1e-15),
    ("a", "atto", 1e-18),
    ("z", "zepto", 1e-21),
    ("y", "yocto", 1e-24),
    ("Ki", "kibi", 1024.0),
    ("Mi", "mebi", 1_048_576.0),
    ("Gi", "gibi", 1_073_741_824.0),
    ("Ti", "tebi", 1_099_511_627_776.0),
];

const SI_PREFIXES: [&str; 20] = [
    "yotta", "zetta", "exa", "peta", "tera", "giga", "mega", "kilo", "hecto", "deka", "deci",
    "centi", "milli", "micro", "nano", "pico", "femto", "atto", "zepto", "yocto",
];

fn utf16_len(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

/// Hurmet's `unitFromWord`: one simple unit, possibly plural and possibly prefixed.
fn unit_from_word(input: &str, currencies: &Currencies) -> Result<Unit, UnitError> {
    let s = input.trim();
    let len = utf16_len(s);
    let mut u = Unit {
        name: s.to_string(),
        factor: Rational::one(),
        gauge: Rational::zero(),
        expos: ALL_ZEROS,
    };
    if input.is_empty() {
        return Ok(u);
    }
    let mut word = s.to_string();
    let mut prefix = String::new();
    let mut found: Option<&Row> = None;
    let mut got_match = false;
    let mut got_si_prefix_unit = false;

    // ADR-0022: a name in the table is that unit, before any plural or prefix is stripped.
    // Hurmet stripped first, so `Celsius`, `gauss`, `kilogram`, `kilobyte` and `centistoke`
    // were unknown and `mas` (milliarcsecond) read as milli-annum.
    let exact = TABLE.get(word.as_str());
    // Pass 1: no short SI prefix, but a long one is allowed if the word is long enough.
    if exact.is_none() && len > 2 && word.ends_with('s') {
        let second_is_p = word.chars().nth(1) == Some('p');
        if !(word == "cfs" || (len == 3 && second_is_p)) {
            word.pop();
        }
    }
    if let Some(row) = exact.or_else(|| TABLE.get(word.as_str())) {
        // The whole name, or its singular (`kilograms`).
        found = Some(row);
        got_match = true;
    } else if len > 3
        && word != "micron"
        && let Some(p) = SI_PREFIXES.iter().find(|p| word.starts_with(*p))
    {
        prefix = (*p).to_string();
        word = word[p.len()..].to_string();
    }
    if found.is_none() {
        if let Some(row) = TABLE.get(word.as_str()) {
            found = Some(row);
            got_match = true;
            if !prefix.is_empty() {
                got_match = row.misc == "siWord";
                got_si_prefix_unit = true;
            }
        } else {
            // Pass 2: try a short SI prefix.
            if let Some(first) = word.chars().next()
                && "YZEPTGMkhdcmnpfazyµμ".contains(first)
            {
                {
                    prefix = first.to_string();
                    word = word[first.len_utf8()..].to_string();
                    if let Some(row) = TABLE.get(word.as_str()) {
                        found = Some(row);
                        got_match = row.misc == "siSymbol";
                        got_si_prefix_unit = true;
                    }
                }
            }
        }
    }

    let row = match (got_match, found) {
        (true, Some(row)) => row,
        _ => return Err(UnitError::UnknownName(s.to_string())),
    };
    u.gauge = Rational::from_str_or_zero(row.gauge);
    u.expos = row.expos;
    if u.expos[7] == 1.0 {
        let code = synonym(&word).unwrap_or(&word);
        let Some(currency_row) = TABLE.get(code) else {
            return Err(UnitError::Currency);
        };
        if currency_row.factor_num == "0" {
            return Err(UnitError::Currency);
        }
        let Some(rate) = currencies.get(code) else {
            return Err(UnitError::Currency);
        };
        let rate = Rational::from_str_hurmet(rate.trim()).ok_or(UnitError::Currency)?;
        u.factor = rate.reciprocal();
    } else {
        u.factor = Rational::from_str_or_zero(row.factor_num)
            .divide(&Rational::from_str_or_zero(row.factor_den));
    }
    if got_si_prefix_unit {
        u.factor = u
            .factor
            .multiply(&Rational::from_number(prefix_factor(&prefix)));
    }
    Ok(u)
}

fn is_exponent_char(c: char) -> bool {
    matches!(c, '⁰' | '¹' | '²' | '³' | '\u{2074}'..='\u{2079}' | '⁻')
}

fn numeral_from_superscript(ch: char) -> char {
    match ch {
        '²' => '2',
        '³' => '3',
        '⁻' => '-',
        '¹' => '1',
        '⁰' => '0',
        _ => char::from_u32((ch as u32).wrapping_sub(0x2040)).unwrap_or('?'),
    }
}

fn is_op_or_num(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '·' | '-' | '/' | '^' | '(' | ')') || is_exponent_char(c)
}

/// Hurmet's `str.replace(/[*.•×\-−](?![0-9.])/g, "·")`.
fn normalize_dots(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        let next_is_num = chars
            .get(i + 1)
            .is_some_and(|n| n.is_ascii_digit() || *n == '.');
        if matches!(c, '*' | '.' | '•' | '×' | '-' | '−') && !next_is_num {
            out.push('·');
        } else {
            out.push(c);
        }
    }
    out
}

/// An entry on the exponent stack: a dimension vector, or a number from the unit string.
/// (Hurmet keeps the numbers as strings and relies on JavaScript's coercion.)
#[derive(Clone)]
enum Expo {
    Dims(Dims),
    Num(f64),
}

/// Hurmet's `unitFromUnitName`: parse a (possibly compound) unit name.
///
/// An empty name gives a dimensionless unit with factor 1.
pub fn unit_from_name(input: &str, currencies: &Currencies) -> Result<Unit, UnitError> {
    let s = normalize_dots(input.trim());
    let mut u = Unit {
        name: s.clone(),
        factor: Rational::one(),
        gauge: Rational::zero(),
        expos: ALL_ZEROS,
    };
    if s.is_empty() {
        return Ok(u);
    }
    if s == "°" || s == "°ʹ" || s == "degMinSec" {
        u.factor = Rational::from_str_or_zero("0.0174532925199433");
        return Ok(u);
    }
    if s == "feetInch" {
        // Hurmet sets `unitExpos` (a typo) here, so the exponents stay all zero.
        u.factor = Rational::from_str_or_zero("0.3048");
        return Ok(u);
    }

    const SEP: char = ';';
    let chars: Vec<char> = s.chars().collect();
    let mut in_exponent = false;
    let mut word = String::new();
    let mut rpn = String::new();
    let mut expo_rpn = String::new();
    let mut expo_queue: Vec<Dims> = Vec::new();
    let mut i_queue = 0usize;
    let mut op_stack: Vec<(char, i32)> = vec![(' ', -1)];
    let mut simple_unit: Option<Unit> = None;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_ascii_digit() || ch == '-' {
            if !in_exponent {
                rpn.push(SEP);
                expo_rpn.push(SEP);
                op_stack.push(('^', 3));
                in_exponent = true;
            }
            rpn.push(ch);
            expo_rpn.push(ch);
        } else if is_exponent_char(ch) {
            if !in_exponent {
                rpn.push(SEP);
                expo_rpn.push(SEP);
                op_stack.push(('^', 3));
                in_exponent = true;
            }
            let a = numeral_from_superscript(ch);
            rpn.push(a);
            expo_rpn.push(a);
        } else if ch == '^' {
            rpn.push(SEP);
            expo_rpn.push(SEP);
            op_stack.push(('^', 3));
            in_exponent = true;
        } else if ch == '·' || ch == '/' {
            in_exponent = false;
            rpn.push(SEP);
            expo_rpn.push(SEP);
            while op_stack.last().is_some_and(|t| t.1 >= 1) {
                let sym = op_stack.pop().map_or(' ', |t| t.0);
                rpn.push(sym);
                rpn.push(SEP);
                expo_rpn.push(sym);
                expo_rpn.push(SEP);
            }
            op_stack.push((ch, 1));
        } else if ch == '(' {
            op_stack.push(('(', 0));
        } else if ch == ')' {
            while op_stack.last().is_some_and(|t| t.1 > 0) {
                let sym = op_stack.pop().map_or(' ', |t| t.0);
                rpn.push(sym);
                rpn.push(SEP);
                expo_rpn.push(sym);
                expo_rpn.push(SEP);
            }
            if op_stack.len() > 1 {
                op_stack.pop();
            }
            in_exponent = false;
        } else {
            in_exponent = false;
            let mut j = i + 1;
            while j < chars.len() && !is_op_or_num(chars[j]) {
                j += 1;
            }
            word = chars[i..j].iter().collect();
            let su = unit_from_word(&word, currencies)?;
            rpn.push_str(&format!("{},{}", su.factor.num, su.factor.den));
            expo_rpn.push_str(&format!("¿{i_queue}"));
            expo_queue.push(su.expos);
            simple_unit = Some(su);
            i_queue += 1;
            i = j - 1;
        }
        i += 1;
    }

    if word == u.name
        && let Some(su) = simple_unit
    {
        return Ok(su);
    }

    while op_stack.len() > 1 {
        let sym = op_stack.pop().map_or(' ', |t| t.0);
        rpn.push(SEP);
        rpn.push(sym);
        expo_rpn.push(SEP);
        expo_rpn.push(sym);
    }

    let mut factors: Vec<Rational> = Vec::new();
    let mut expo_stack: Vec<Expo> = Vec::new();
    let rpn_array: Vec<&str> = rpn.split(SEP).collect();
    let expo_array: Vec<&str> = expo_rpn.split(SEP).collect();
    let malformed = || UnitError::Malformed(s.clone());
    let mut i_queue = 0usize;
    for (k, tok) in rpn_array.iter().enumerate() {
        let is_op = matches!(*tok, "·" | "/" | "-" | "^");
        let (mut val2, mut e2) = (Rational::one(), Expo::Num(1.0));
        if is_op {
            val2 = factors.pop().ok_or_else(malformed)?;
            e2 = expo_stack.pop().ok_or_else(malformed)?;
        }
        match *tok {
            "·" | "/" => {
                if *tok == "/" && val2.is_zero() {
                    return Err(UnitError::DivideByZero);
                }
                let top = factors.last_mut().ok_or_else(malformed)?;
                *top = if *tok == "·" {
                    top.multiply(&val2)
                } else {
                    top.divide(&val2)
                };
                let e1 = expo_stack.last_mut().ok_or_else(malformed)?;
                let (Expo::Dims(a), Expo::Dims(b)) = (&e1, &e2) else {
                    // JavaScript would concatenate strings here.
                    return Err(malformed());
                };
                let mut c = *a;
                for (x, y) in c.iter_mut().zip(b) {
                    *x = if *tok == "·" { *x + y } else { *x - y };
                }
                *e1 = Expo::Dims(c);
            }
            "^" => {
                let top = factors.last_mut().ok_or_else(malformed)?;
                *top = top.power(&val2).ok_or_else(malformed)?;
                let p = match e2 {
                    Expo::Num(p) => p,
                    Expo::Dims(_) => return Err(malformed()),
                };
                let e1 = expo_stack.last_mut().ok_or_else(malformed)?;
                *e1 = match e1 {
                    Expo::Dims(d) => Expo::Dims(d.map(|x| x * p)),
                    Expo::Num(n) => Expo::Num(*n * p),
                };
            }
            "-" => {
                let top = factors.last_mut().ok_or_else(malformed)?;
                *top = top.negate();
            }
            _ => {
                if let Some((n, d)) = tok.split_once(',') {
                    let n: BigInt = n.parse().map_err(|_| malformed())?;
                    let d: BigInt = d.parse().map_err(|_| malformed())?;
                    factors.push(Rational::new(n, d));
                } else {
                    // BigInt("") is 0n.
                    let n: BigInt = if tok.is_empty() {
                        BigInt::from(0)
                    } else {
                        tok.parse().map_err(|_| malformed())?
                    };
                    factors.push(Rational::new(n, BigInt::from(1)));
                }
                let et = expo_array.get(k).copied().unwrap_or("");
                if et.starts_with('¿') {
                    let d = expo_queue.get(i_queue).copied().ok_or_else(malformed)?;
                    expo_stack.push(Expo::Dims(d));
                    i_queue += 1;
                } else {
                    expo_stack.push(Expo::Num(if et.is_empty() {
                        0.0
                    } else {
                        et.parse().map_err(|_| malformed())?
                    }));
                }
            }
        }
    }
    u.factor = factors.pop().ok_or_else(malformed)?;
    u.expos = match expo_stack.pop() {
        Some(Expo::Dims(d)) => d,
        _ => return Err(malformed()),
    };
    Ok(u)
}

/// Hurmet's `unitsAreCompatible`: equal exponents (`None` is JavaScript's null/undefined).
pub fn units_are_compatible(a: Option<&Dims>, b: Option<&Dims>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_money() -> Currencies {
        Currencies::new()
    }

    fn unit(s: &str) -> Unit {
        unit_from_name(s, &no_money()).unwrap()
    }

    #[test]
    fn simple_units() {
        let ft = unit("ft");
        assert_eq!(ft.factor, Rational::from_str_or_zero("0.3048"));
        assert_eq!(ft.expos, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(unit("feet").factor.to_f64(), 0.3048);
        assert_eq!(unit("kips").expos, unit("kip").expos);
        assert_eq!(unit("km").factor.to_f64(), 1000.0);
        assert_eq!(unit("micron").factor.to_f64(), 1e-6);
        assert!(matches!(
            unit_from_name("bogus", &no_money()),
            Err(UnitError::UnknownName(_))
        ));
    }

    #[test]
    fn compound_units() {
        let u = unit("kip·ft");
        assert_eq!(u.expos, [2.0, 1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(u.name, "kip·ft");
        let u = unit("lbf/ft");
        assert_eq!(u.expos, [0.0, 1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let u = unit("N·m/s");
        assert_eq!(u.expos, [2.0, 1.0, -3.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let u = unit("in²");
        assert_eq!(u.expos[0], 2.0);
        assert_eq!(u.factor.to_f64(), 0.00064516);
        assert_eq!(unit("in^2").factor, u.factor);
        assert_eq!(unit("in⁻¹").expos[0], -1.0);
        assert_eq!(unit("N-m").expos, unit("N·m").expos);
    }

    #[test]
    fn gauge_and_angles() {
        let f = unit("°F");
        assert_eq!(f.expos[4], 1.0);
        // Hurmet has 459; 0 °F is exactly 459.67 °R, so 32 °F is exactly 273.15 K.
        assert_eq!(f.gauge.to_f64(), 459.67);
        let k = Rational::int(32).add(&f.gauge).multiply(&f.factor);
        assert_eq!(k.to_f64(), 273.15);
        assert_eq!(unit("°").factor.to_f64(), 0.0174532925199433);
        assert_eq!(unit("\\⦂").factor.to_f64(), 0.01); // the percent placeholder
    }

    #[test]
    fn currencies() {
        let mut c = Currencies::new();
        assert_eq!(unit_from_name("USD", &c), Err(UnitError::Currency));
        c.insert("USD".into(), "1.25".into());
        let usd = unit_from_name("USD", &c).unwrap();
        assert_eq!(usd.factor.to_f64(), 0.8);
        assert_eq!(usd.expos[7], 1.0);
        assert_eq!(unit_from_name("$", &c).unwrap().factor, usd.factor);
    }
}
