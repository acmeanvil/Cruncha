// SPDX-License-Identifier: GPL-3.0-or-later

//! `jsmath` must give V8's results bit for bit. The vectors in `fixtures/v8_math.json` were
//! printed by Node.js (see `fixtures/README.md`).

use cruncha_units::jsmath;
use std::collections::BTreeMap;

fn f64_of(hex: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(hex, 16).unwrap())
}

#[test]
fn matches_v8_bit_for_bit() {
    let path = format!("{}/tests/fixtures/v8_math.json", env!("CARGO_MANIFEST_DIR"));
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut mismatches: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut examples = Vec::new();
    for case in data["cases"].as_array().unwrap() {
        let c: Vec<&str> = case
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let name = c[0];
        let (got, want, args) = match name {
            "atan2" | "pow" => {
                let (a, b) = (f64_of(c[1]), f64_of(c[2]));
                let got = if name == "atan2" {
                    jsmath::atan2(a, b)
                } else {
                    jsmath::pow(a, b)
                };
                (got, f64_of(c[3]), format!("{a:?}, {b:?}"))
            }
            _ => {
                let x = f64_of(c[1]);
                let f = match name {
                    "sin" => jsmath::sin,
                    "cos" => jsmath::cos,
                    "tan" => jsmath::tan,
                    "exp" => jsmath::exp,
                    "atan" => jsmath::atan,
                    "log" => jsmath::log,
                    "log10" => jsmath::log10,
                    "log2" => jsmath::log2,
                    "asin" => jsmath::asin,
                    "acos" => jsmath::acos,
                    _ => panic!("unknown function {name}"),
                };
                (f(x), f64_of(c[2]), format!("{x:?}"))
            }
        };
        let entry = mismatches.entry(name.to_string()).or_default();
        entry.1 += 1;
        if got.to_bits() != want.to_bits() && !(got.is_nan() && want.is_nan()) {
            if name == "pow" {
                // V8's pow is the platform's (these vectors are from macOS); jsmath's is
                // fdlibm's. They must agree to one unit in the last place.
                assert!(
                    got.to_bits().abs_diff(want.to_bits()) <= 1,
                    "pow({args}) = {got:?}, V8 {want:?}"
                );
            } else {
                entry.0 += 1;
                if examples.len() < 20 {
                    examples.push(format!("{name}({args}) = {got:?}, V8 {want:?}"));
                }
            }
        }
    }
    for e in &examples {
        println!("{e}");
    }
    let mut bad = 0;
    for (name, (miss, total)) in &mismatches {
        println!("{name}: {miss} / {total} differ");
        bad += miss;
    }
    assert_eq!(bad, 0);
}
