// SPDX-License-Identifier: GPL-3.0-or-later

//! Check template files while writing them: validation problems, then each worked example
//! with what came out.
//!
//! ```sh
//! cargo run -p cruncha-templates --example check -- templates/…/<id>.cruncha.md …
//! ```
//!
//! Code cells run on the Python worker when the environment is set up.

#[path = "../tests/support/py.rs"]
mod py;

fn main() {
    let mut failed = false;
    let mut runner = py::PyRunner::start();
    for path in std::env::args().skip(1) {
        println!("== {path}");
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                println!("  can't read: {e}");
                failed = true;
                continue;
            }
        };
        let t = match cruncha_templates::parse(&text) {
            Ok(t) => t,
            Err(e) => {
                println!("  {e}");
                failed = true;
                continue;
            }
        };
        for p in cruncha_templates::validate(&t) {
            println!("  invalid: {p}");
            failed = true;
        }
        let r = runner.as_mut().map(|r| r as &mut dyn cruncha_templates::CodeRunner);
        for r in cruncha_templates::run_tests_with(&t, r) {
            let verdict = match (&r.skipped, r.passed) {
                (Some(why), _) => format!("skipped ({why})"),
                (None, true) => "passed".to_owned(),
                (None, false) => "FAILED".to_owned(),
            };
            println!("  {} · {verdict}", r.name);
            for l in &r.lines {
                println!("      {l}");
            }
            failed |= r.skipped.is_none() && !r.passed;
        }
    }
    std::process::exit(i32::from(failed));
}
