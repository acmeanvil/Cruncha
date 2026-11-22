// SPDX-License-Identifier: GPL-3.0-or-later
//! The Units palette's catalog: every name in it is a unit `unit_from_name` reads, with the
//! size and dimension the catalog claims, and every row of the table is in it.

use cruncha_units::catalog::{OTHER, QUANTITIES, catalog, prefixes};
use cruncha_units::{Currencies, unit_from_name};

#[test]
fn every_name_reads_as_the_unit_listed() {
    let cat = catalog();
    let none = Currencies::new();
    let mut failures = Vec::new();
    for u in &cat {
        if u.currency.is_some() {
            continue; // needs exchange rates, which the app doesn't supply yet
        }
        let q = QUANTITIES.iter().find(|q| q.name == u.quantity);
        for name in std::iter::once(&u.name).chain(&u.aliases) {
            match unit_from_name(name, &none) {
                Ok(unit) => {
                    let f = unit.factor.to_f64();
                    if (f - u.si_factor).abs() > 1e-12 * u.si_factor.abs().max(1.0) {
                        failures.push(format!("{name}: factor {f}, catalog {}", u.si_factor));
                    }
                    if let Some(q) = q {
                        let dims: Vec<f64> = q.dims.iter().map(|&d| f64::from(d)).collect();
                        if unit.expos.to_vec() != dims {
                            failures.push(format!("{name}: dims {:?}, catalog {}", unit.expos, q.name));
                        }
                    }
                }
                Err(e) => failures.push(format!("{name}: {e:?}")),
            }
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn grouped_sensibly() {
    let cat = catalog();
    let find = |n: &str| cat.iter().find(|u| u.name == n || u.aliases.contains(&n)).unwrap_or_else(|| panic!("{n}"));
    assert_eq!(find("feet").name, "ft");
    assert!(find("ft").aliases.contains(&"foot"));
    assert_eq!(find("ft").quantity, "Length");
    assert_eq!(find("kip").quantity, "Force");
    assert_eq!(find("psi").quantity, "Pressure and stress");
    assert_eq!(find("#").name, "lb");
    assert_eq!(find("N").prefixes, Some("symbol"));
    assert!(find("Fahrenheit").gauge != 0.0);
    assert_eq!(find("USD").currency, Some("USD"));
    let names: Vec<&str> = cat.iter().map(|u| u.name).collect();
    let unique: std::collections::HashSet<_> = names.iter().collect();
    assert_eq!(names.len(), unique.len(), "a name is the primary of two entries");
    let other: Vec<&str> = cat.iter().filter(|u| u.quantity == OTHER).map(|u| u.name).collect();
    println!("{} units, {} under Other: {other:?}", cat.len(), other.len());
    assert!(other.is_empty(), "units without a quantity: {other:?}");
    assert_eq!(find("tesla").quantity, "Magnetic flux density");
    assert_eq!(find("ampere turn").quantity, "Current");
    assert!(prefixes().iter().any(|p| p.0 == "k" && p.2 == 1e3));
}
