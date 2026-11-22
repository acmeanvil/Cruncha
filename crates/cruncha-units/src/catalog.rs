// SPDX-License-Identifier: GPL-3.0-or-later

//! The units a calc can use, for the app's Units palette: Hurmet's unit table grouped by
//! quantity (length, force, pressure, …), with each unit's aliases (`ft`, `foot`, `feet`) folded
//! into one entry and its size in SI. Built from [`crate::table`], so it never disagrees with
//! what [`crate::unit_from_name`] accepts (tests/catalog.rs checks every name).

use crate::table::UNIT_TABLE;
use crate::units::SI_PREFIX_LIST;

/// One unit and its aliases.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogUnit {
    /// The name to insert: the shortest alias that starts with a letter (`ft`, `lb`, `N`).
    pub name: &'static str,
    /// The other names for the same unit (`foot`, `feet`).
    pub aliases: Vec<&'static str>,
    /// A [`QUANTITIES`] name (`"Length"`).
    pub quantity: &'static str,
    /// One of this unit in the quantity's SI unit (`ft` → 0.3048, in m).
    pub si_factor: f64,
    /// An offset scale (°C, °F, psig): the value is `(v + gauge) × factor`.
    pub gauge: f64,
    /// SI prefixes apply: `"symbol"` (km, MPa), `"word"` (kilometer), or none.
    pub prefixes: Option<&'static str>,
    /// A 3-letter currency code: needs exchange rates.
    pub currency: Option<&'static str>,
}

/// A quantity: its name, its base-unit exponents (length, mass, time, current, temperature,
/// amount, luminous intensity, money), and the SI unit sizes are shown in.
pub struct Quantity {
    pub name: &'static str,
    pub dims: [i8; 8],
    pub si: &'static str,
}

const fn q(name: &'static str, dims: [i8; 8], si: &'static str) -> Quantity {
    Quantity { name, dims, si }
}

/// The quantities the palette groups by, in the order it shows them. Units of any other
/// dimension go under "Other".
pub const QUANTITIES: &[Quantity] = &[
    q("Length", [1, 0, 0, 0, 0, 0, 0, 0], "m"),
    q("Area", [2, 0, 0, 0, 0, 0, 0, 0], "m²"),
    q("Volume", [3, 0, 0, 0, 0, 0, 0, 0], "m³"),
    q("Mass", [0, 1, 0, 0, 0, 0, 0, 0], "kg"),
    q("Force", [1, 1, -2, 0, 0, 0, 0, 0], "N"),
    q("Force per length", [0, 1, -2, 0, 0, 0, 0, 0], "N/m"),
    q("Pressure and stress", [-1, 1, -2, 0, 0, 0, 0, 0], "Pa"),
    q("Force per volume", [-2, 1, -2, 0, 0, 0, 0, 0], "N/m³"),
    q("Density", [-3, 1, 0, 0, 0, 0, 0, 0], "kg/m³"),
    q("Area density", [-2, 1, 0, 0, 0, 0, 0, 0], "kg/m²"),
    q("Energy, work and torque", [2, 1, -2, 0, 0, 0, 0, 0], "J"),
    q("Power", [2, 1, -3, 0, 0, 0, 0, 0], "W"),
    q("Time", [0, 0, 1, 0, 0, 0, 0, 0], "s"),
    q("Frequency and rate", [0, 0, -1, 0, 0, 0, 0, 0], "Hz"),
    q("Speed", [1, 0, -1, 0, 0, 0, 0, 0], "m/s"),
    q("Acceleration", [1, 0, -2, 0, 0, 0, 0, 0], "m/s²"),
    q("Flow rate", [3, 0, -1, 0, 0, 0, 0, 0], "m³/s"),
    q("Mass flow rate", [0, 1, -1, 0, 0, 0, 0, 0], "kg/s"),
    q("Pressure rate", [-1, 1, -3, 0, 0, 0, 0, 0], "Pa/s"),
    q("Per length", [-1, 0, 0, 0, 0, 0, 0, 0], "m⁻¹"),
    q("Fuel economy", [-2, 0, 0, 0, 0, 0, 0, 0], "m⁻²"),
    q("Viscosity", [-1, 1, -1, 0, 0, 0, 0, 0], "Pa·s"),
    q("Kinematic viscosity", [2, 0, -1, 0, 0, 0, 0, 0], "m²/s"),
    q("Radiation dose", [2, 0, -2, 0, 0, 0, 0, 0], "Gy"),
    q("Temperature", [0, 0, 0, 0, 1, 0, 0, 0], "K"),
    q("Amount of substance", [0, 0, 0, 0, 0, 1, 0, 0], "mol"),
    q("Catalytic activity", [0, 0, -1, 0, 0, 1, 0, 0], "kat"),
    q("Current", [0, 0, 0, 1, 0, 0, 0, 0], "A"),
    q("Charge", [0, 0, 1, 1, 0, 0, 0, 0], "C"),
    q("Exposure (ionizing)", [0, -1, 1, 1, 0, 0, 0, 0], "C/kg"),
    q("Voltage", [2, 1, -3, -1, 0, 0, 0, 0], "V"),
    q("Resistance", [2, 1, -3, -2, 0, 0, 0, 0], "Ω"),
    q("Conductance", [-2, -1, 3, 2, 0, 0, 0, 0], "S"),
    q("Capacitance", [-2, -1, 4, 2, 0, 0, 0, 0], "F"),
    q("Inductance", [2, 1, -2, -2, 0, 0, 0, 0], "H"),
    q("Magnetic flux", [2, 1, -2, -1, 0, 0, 0, 0], "Wb"),
    q("Magnetic flux density", [0, 1, -2, -1, 0, 0, 0, 0], "T"),
    q("Luminous intensity and flux", [0, 0, 0, 0, 0, 0, 1, 0], "cd"),
    q("Illuminance and luminance", [-2, 0, 0, 0, 0, 0, 1, 0], "lx"),
    q("Angle, ratio and count", [0, 0, 0, 0, 0, 0, 0, 0], "1"),
    q("Currency", [0, 0, 0, 0, 0, 0, 0, 1], "currency"),
];

/// The group for a dimension none of [`QUANTITIES`] has.
pub const OTHER: &str = "Other";

/// An SI prefix: symbol, word, factor.
pub type Prefix = (&'static str, &'static str, f64);

/// The SI (and binary) prefixes, largest first.
pub fn prefixes() -> &'static [Prefix] {
    SI_PREFIX_LIST
}

fn quantity_of(dims: &[i8; 8]) -> &'static str {
    QUANTITIES.iter().find(|q| &q.dims == dims).map_or(OTHER, |q| q.name)
}

/// Every unit in the table, aliases folded, ordered by [`QUANTITIES`] then by size.
pub fn catalog() -> Vec<CatalogUnit> {
    // Rows that agree on factor, gauge, prefixes and dimensions are one unit under several
    // names. A currency's rows differ (the code row carries the rate), so they group by code.
    let mut groups: Vec<(String, Vec<&'static str>, usize)> = Vec::new();
    for (i, (name, num, den, gauge, misc, dims)) in UNIT_TABLE.iter().enumerate() {
        let key = if misc.len() == 3 && misc.chars().all(|c| c.is_ascii_uppercase()) {
            format!("currency {misc}")
        } else {
            format!("{num}/{den}+{gauge} {misc} {dims:?}")
        };
        match groups.iter_mut().find(|g| g.0 == key) {
            Some(g) => g.1.push(name),
            None => groups.push((key, vec![name], i)),
        }
    }
    let mut out: Vec<CatalogUnit> = groups
        .into_iter()
        .map(|(_, names, i)| {
            let (_, num, den, gauge, misc, dims) = UNIT_TABLE[i];
            let name = primary(&names, misc);
            let mut aliases: Vec<&'static str> = names.into_iter().filter(|n| *n != name).collect();
            aliases.sort_by_key(|a| (a.chars().count(), *a));
            let currency = (misc.len() == 3 && misc.chars().all(|c| c.is_ascii_uppercase())).then_some(misc);
            CatalogUnit {
                name,
                aliases,
                quantity: quantity_of(&dims),
                si_factor: if currency.is_some() { 1.0 } else { parse(num) / parse(den) },
                gauge: parse(gauge),
                prefixes: match misc {
                    "siSymbol" => Some("symbol"),
                    "siWord" => Some("word"),
                    _ => None,
                },
                currency,
            }
        })
        .collect();
    let rank = |q: &str| QUANTITIES.iter().position(|x| x.name == q).unwrap_or(QUANTITIES.len());
    out.sort_by(|a, b| {
        rank(a.quantity)
            .cmp(&rank(b.quantity))
            .then(a.si_factor.total_cmp(&b.si_factor))
            .then(a.name.cmp(b.name))
    });
    out
}

/// The name to show and insert: a currency's code, else the shortest name that starts with a
/// letter (so `lb`, not `#`), ties broken alphabetically.
fn primary(names: &[&'static str], misc: &'static str) -> &'static str {
    if names.contains(&misc) {
        return misc;
    }
    let letter = |n: &&str| n.chars().next().is_some_and(char::is_alphabetic);
    let pool: Vec<&'static str> = if names.iter().any(letter) {
        names.iter().copied().filter(|n| letter(n)).collect()
    } else {
        names.to_vec()
    };
    pool.into_iter().min_by_key(|n| (n.chars().count(), *n)).unwrap_or(names[0])
}

fn parse(s: &str) -> f64 {
    s.parse().unwrap_or(f64::NAN)
}
