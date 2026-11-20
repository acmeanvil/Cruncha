// SPDX-License-Identifier: GPL-3.0-or-later

//! Fragments of every golden input (each prefix and each suffix, cut at every character) must
//! prepare and evaluate without panicking, in both unit-awareness modes, against the
//! variables that the golden run defines.

use cruncha_eval::{Currencies, Options, UnitAware, Vars, calculate, evaluate, prepare};
use std::sync::Arc;

fn load(name: &str) -> Vec<serde_json::Value> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    v.as_array().unwrap().clone()
}

#[test]
fn fragments_do_not_panic() {
    let mut c = Currencies::new();
    c.insert("USD".into(), "1.1014".into());
    c.insert("CAD".into(), "1.5734".into());
    let base = Options {
        currencies: Arc::new(c),
        ..Options::default()
    };
    let mut vars = Vars::new();
    vars.set_format("h15");
    let mut inputs: Vec<String> = Vec::new();
    for case in load("assignments.json") {
        let (entry, template) = (case[0].as_str().unwrap(), case[1].as_str().unwrap());
        calculate(entry, &mut vars, &base);
        inputs.push(entry.into());
        inputs.push(template.into());
    }
    for f in load("functions.json") {
        let s = f.as_str().unwrap();
        calculate(s, &mut vars, &base);
        inputs.push(s.into());
    }
    for case in load("calculations.json") {
        inputs.push(case[0].as_str().unwrap().into());
    }
    let mut count = 0;
    for mode in [UnitAware::Hurmet, UnitAware::Always] {
        let opts = Options {
            unit_aware: mode,
            ..base.clone()
        };
        for input in &inputs {
            let cuts: Vec<usize> = input
                .char_indices()
                .map(|(i, _)| i)
                .chain([input.len()])
                .collect();
            for &k in &cuts {
                for fragment in [&input[..k], &input[k..]] {
                    count += 1;
                    if let Ok(stmt) = prepare(fragment, &opts) {
                        let _ = evaluate(&stmt, &vars, &opts);
                    }
                }
            }
        }
    }
    println!("{count} fragments evaluated");
}

/// Inputs that would make JavaScript hang, overflow or throw.
#[test]
fn adversarial_inputs_return() {
    let opts = Options::default();
    let mut vars = Vars::new();
    let inputs = [
        "x = 1e400 = ?",
        "x = 2^100000 = @",
        "x = 2^1e9 = @",
        "x = (-8)^(1/3) = ?",
        "x = [1:0.0000001:100000...] = ?",
        "x = 100000! = ?",
        "x = 99999!! = ?",
        "x = ones(1e9, 1e9) = ?",
        "x = sin(1e300) = ?",
        "x = '9999-99-99' + 1 = @@",
        "x = 1 'm^99999' = @@ m",
        "x = 0/0 = ?",
        "x = log(0) = ?",
        "x = log(-1) = ?",
        "x = sqrt(-4) = ?",
        "x = acos(2) = ?",
        "x = \"abc\"[1:2:1e18] = ?",
        "x = [1; 2; 3][1:1e15] = ?",
        "x = rand(1e12) = ?",
        "x = ∑_(n=1)^(1e12) n = ?",
        "function loop(n)\n  while true\n    n = n + 1\n  end\n  return n\nend",
        "y = loop(1) = ?",
        "z = (((((((((((((((((((((((((((((((1))))))))))))))))))))))))))))))) = ?",
    ];
    for input in inputs {
        let started = std::time::Instant::now();
        let out = calculate(input, &mut vars, &opts);
        let elapsed = started.elapsed();
        println!("{input:?} -> {out:?} ({elapsed:.2?})");
        assert!(elapsed.as_secs() < 30, "{input} took {elapsed:?}");
    }
}
