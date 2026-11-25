// SPDX-License-Identifier: GPL-3.0-or-later

//! Hurmet's evaluator golden tests (`test/test.cjs`: `assignmentTests`, the function
//! definitions, and `calcTests`), run in order against one set of variables, exactly as the
//! Hurmet harness does. See `tests/fixtures/README.md` for provenance.

use cruncha_eval::{Currencies, Options, Vars, calculate};
use std::sync::Arc;

/// Inputs whose expected output the port does not reproduce, with the reason. Deliberate
/// corrections of Hurmet (ADR-0022) are listed here too, so the rest stay pinned.
/// Cases whose RPN differs from Hurmet's on purpose, with the same result. (input, reason)
const RPN_CORRECTIONS: &[(&str, &str)] = &[(
    "e^z1 = @",
    "ADR-0025: `e` before `^` stays a variable (Euler's number when the document doesn't define e)",
)];

const KNOWN_GAPS: &[(&str, &str)] = &[(
    "|matrix| = @",
    "ADR-0022 item 11: |A| is the determinant, 184.05; Hurmet gave its reciprocal, 0.0054333",
)];

fn load(name: &str) -> serde_json::Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn options() -> Options {
    // Hurmet's harness reads site/currencyRates.json; only the ratio of two rates matters in
    // the tests, so two of its rates are enough here.
    let mut c = Currencies::new();
    c.insert("USD".into(), "1.1014".into());
    c.insert("CAD".into(), "1.5734".into());
    c.insert("EUR".into(), "1.0".into());
    Options {
        currencies: Arc::new(c),
        ..Options::default()
    }
}

struct Run {
    vars: Vars,
    opts: Options,
    failures: Vec<(String, String, String)>,
    passed: usize,
    total: usize,
}

fn run_all() -> Run {
    let opts = options();
    let mut vars = Vars::new();
    vars.set_format("h15");
    let mut run = Run {
        vars,
        opts,
        failures: Vec::new(),
        passed: 0,
        total: 0,
    };

    let assignments = load("assignments.json");
    for case in assignments.as_array().unwrap() {
        let entry = case[0].as_str().unwrap();
        let template = case[1].as_str().unwrap();
        let expected = case[2].as_str().unwrap();
        calculate(entry, &mut run.vars, &run.opts);
        let value = calculate(template, &mut run.vars, &run.opts);
        if expected.is_empty() {
            continue;
        }
        run.total += 1;
        if value == expected {
            run.passed += 1;
        } else {
            run.failures
                .push((entry.to_string(), expected.to_string(), value));
        }
    }

    let functions = load("functions.json");
    for f in functions.as_array().unwrap() {
        calculate(f.as_str().unwrap(), &mut run.vars, &run.opts);
    }

    let lang_opts = cruncha_lang::Options {
        formats: cruncha_lang::Formats {
            decimal_format: "1,000,000.".into(),
            date_format: Some("yyy-mm-dd".into()),
        },
        ..Default::default()
    };
    let calcs = load("calculations.json");
    for case in calcs.as_array().unwrap() {
        let input = case[0].as_str().unwrap();
        let expected_rpn = case[1].as_str().unwrap();
        let expected = case[2].as_str().unwrap();
        run.total += 1;
        // The harness also checks the RPN of the expression before the last "=".
        let s = match input.rfind('=') {
            Some(p) => input[..p].trim(),
            None => input,
        };
        let s = s
            .strip_prefix("@@test ")
            .or_else(|| s.strip_prefix("@test "))
            .unwrap_or(s);
        let rpn = cruncha_lang::parse_calc(s, &lang_opts)
            .map(|c| c.rpn)
            .unwrap_or_default();
        let output = calculate(input, &mut run.vars, &run.opts);
        // An RPN Cruncha corrects on purpose (cruncha-lang's golden CORRECTIONS) still has to
        // give Hurmet's result.
        let rpn_corrected = RPN_CORRECTIONS.iter().any(|(i, _)| *i == input);
        if output == expected && (rpn == expected_rpn || rpn_corrected) {
            run.passed += 1;
        } else {
            run.failures
                .push((input.to_string(), expected.to_string(), output));
        }
    }
    run
}

#[test]
fn hurmet_golden() {
    let run = run_all();
    let mut unexpected = 0;
    for (input, expected, actual) in &run.failures {
        let gap = KNOWN_GAPS.iter().find(|(i, _)| i == input);
        match gap {
            Some((_, reason)) => println!(
                "KNOWN GAP {input:?}\n   expected {expected:?}\n   actual   {actual:?}\n   reason   {reason}"
            ),
            None => {
                unexpected += 1;
                println!("FAIL {input:?}\n   expected {expected:?}\n   actual   {actual:?}");
            }
        }
    }
    for (input, _) in KNOWN_GAPS {
        if !run.failures.iter().any(|(i, _, _)| i == input) {
            println!("note: known gap now passes: {input:?}");
        }
    }
    println!(
        "{} / {} golden cases pass ({} known gaps)",
        run.passed,
        run.total,
        run.failures.len() - unexpected
    );
    assert_eq!(unexpected, 0, "{unexpected} unexpected failures");
}
