// SPDX-License-Identifier: GPL-3.0-or-later

//! Print the outcome (TeX and plain text) of a few statements in both unit-awareness modes.
//!
//! ```sh
//! cargo run -p cruncha-eval --example eval_demo
//! ```

use cruncha_eval::{Options, UnitAware, Vars, evaluate, prepare};

fn main() {
    let lines = [
        "w = 1.5 kip/ft",
        "L = 24 ft",
        "M = w L^2 / 8 = ?? kip·ft",
        "M_2 = (w L^2) / 8 = ? kip·ft",
        "2 L = ?",
        "A = L^2 = %",
        "x = 1/0 = ?",
        "@@test M > 100 'kip·ft'",
    ];
    for mode in [UnitAware::Hurmet, UnitAware::Always] {
        println!("== {mode:?}");
        let opts = Options {
            unit_aware: mode,
            ..Options::default()
        };
        let mut vars = Vars::new();
        for l in lines {
            let stmt = match prepare(l, &opts) {
                Ok(s) => s,
                Err(e) => {
                    println!("{l}\n  prepare error: {e}");
                    continue;
                }
            };
            let out = evaluate(&stmt, &vars, &opts);
            println!(
                "{l}\n  display: {:?}  defines: {:?}  reads: {:?}",
                stmt.display, stmt.defines, stmt.reads
            );
            println!("  result: {}\n  tex:    {}", out.result, out.tex);
            if let Some(e) = &out.error {
                println!("  error:  {e}");
            }
            for (n, v) in out.values {
                vars.insert(n, v);
            }
        }
    }
}
