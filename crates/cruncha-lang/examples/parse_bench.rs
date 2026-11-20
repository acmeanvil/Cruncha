// SPDX-License-Identifier: GPL-3.0-or-later

//! Parse throughput over the golden fixture inputs (spike S1).
//!
//! ```sh
//! cargo run -p cruncha-lang --release --example parse_bench -- [rounds]
//! ```

use cruncha_lang::{Options, parse_calc, parse_tex};
use std::time::Instant;

fn load(name: &str) -> Vec<Vec<String>> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(2000);
    let opts = Options::default();
    let tex_inputs: Vec<String> = load("parser_tex.json")
        .into_iter()
        .map(|c| c[0].clone())
        .collect();
    let calc_inputs: Vec<String> = load("calc_rpn.json")
        .into_iter()
        .map(|c| c[0].clone())
        .collect();

    // Warm up the lazily compiled regexes.
    for s in tex_inputs.iter().chain(&calc_inputs) {
        let _ = parse_tex(s, &opts);
        let _ = parse_calc(s, &opts);
    }

    let bytes = |v: &[String]| v.iter().map(String::len).sum::<usize>();
    let report = |label: &str, inputs: &[String], calc: bool| {
        let start = Instant::now();
        let mut sink = 0usize;
        for _ in 0..rounds {
            for s in inputs {
                sink += if calc {
                    parse_calc(s, &opts).map(|c| c.rpn.len()).unwrap_or(0)
                } else {
                    parse_tex(s, &opts).map(|t| t.len()).unwrap_or(0)
                };
            }
        }
        let secs = start.elapsed().as_secs_f64();
        let n = (rounds * inputs.len()) as f64;
        println!(
            "{label:<24} {:>4} inputs × {rounds} rounds: {:>9.0} statements/s, {:>5.2} µs/statement, {:>6.1} MB/s (sink {sink})",
            inputs.len(),
            n / secs,
            secs / n * 1e6,
            (bytes(inputs) * rounds) as f64 / secs / 1e6,
        );
    };
    report("TeX (parserTests)", &tex_inputs, false);
    report("TeX + RPN (calcTests)", &calc_inputs, true);
}
