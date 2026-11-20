// SPDX-License-Identifier: GPL-3.0-or-later

//! Throughput of `prepare` and `evaluate` over the golden statements.
//!
//! ```sh
//! cargo run -p cruncha-eval --release --example eval_bench -- 200
//! ```

use cruncha_eval::{Currencies, Options, Vars, calculate, evaluate, prepare};
use std::sync::Arc;
use std::time::Instant;

fn load(name: &str) -> Vec<serde_json::Value> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    v.as_array().unwrap().clone()
}

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(100);
    let mut c = Currencies::new();
    c.insert("USD".into(), "1.1014".into());
    c.insert("CAD".into(), "1.5734".into());
    let opts = Options {
        currencies: Arc::new(c),
        ..Options::default()
    };
    let mut vars = Vars::new();
    vars.set_format("h15");
    for case in load("assignments.json") {
        calculate(case[0].as_str().unwrap(), &mut vars, &opts);
    }
    for f in load("functions.json") {
        calculate(f.as_str().unwrap(), &mut vars, &opts);
    }
    // The calculations, minus those that (re)define variables, so every round is the same.
    let inputs: Vec<String> = load("calculations.json")
        .iter()
        .map(|c| c[0].as_str().unwrap().to_string())
        .filter(|s| !s.starts_with("divByZero"))
        .collect();
    let bytes: usize = inputs.iter().map(String::len).sum();

    let t = Instant::now();
    let mut stmts = Vec::new();
    for _ in 0..rounds {
        stmts = inputs.iter().map(|s| prepare(s, &opts).unwrap()).collect();
    }
    let prep = t.elapsed();

    let t = Instant::now();
    let mut n = 0usize;
    for _ in 0..rounds {
        for s in &stmts {
            let out = evaluate(s, &vars, &opts);
            n += out.result.len();
        }
    }
    let eval = t.elapsed();
    let count = (rounds * inputs.len()) as f64;
    // The slowest statements.
    let mut per: Vec<(f64, &str)> = stmts
        .iter()
        .zip(&inputs)
        .map(|(s, src)| {
            let t = Instant::now();
            for _ in 0..rounds.min(50) {
                evaluate(s, &vars, &opts);
            }
            (
                t.elapsed().as_secs_f64() * 1e6 / rounds.min(50) as f64,
                src.as_str(),
            )
        })
        .collect();
    per.sort_by(|a, b| b.0.total_cmp(&a.0));
    let median = per[per.len() / 2].0;
    println!("median evaluate: {median:.1} µs; slowest:");
    for (us, src) in per.iter().take(6) {
        println!("  {us:8.1} µs  {src}");
    }
    println!(
        "{} statements ({:.1} bytes on average), {rounds} rounds",
        inputs.len(),
        bytes as f64 / inputs.len() as f64
    );
    println!(
        "prepare:  {:.1} µs/statement ({:.0} statements/s)",
        prep.as_secs_f64() * 1e6 / count,
        count / prep.as_secs_f64()
    );
    println!(
        "evaluate: {:.1} µs/statement ({:.0} statements/s)  [{n}]",
        eval.as_secs_f64() * 1e6 / count,
        count / eval.as_secs_f64()
    );
}
