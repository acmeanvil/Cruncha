// SPDX-License-Identifier: GPL-3.0-or-later

//! Hurmet's golden tests (see `tests/fixtures/README.md`): TeX output of `parse` and RPN output
//! of `parse(…, true)`. All failures are collected and printed before asserting.

use cruncha_lang::{Formats, Options, parse_calc, parse_tex};

/// Cases that need code outside the lexer and parser. (suite, input, reason)
const KNOWN_GAPS: &[(&str, &str, &str)] = &[(
    "tex",
    "α = ``B\tC\tD\n7.0\t9.5\t11.5``[\"C\"]",
    "data frame literal: Hurmet renders it with DataFrame.display(dataFrameFromTSV(…)), which needs the evaluator's data frame model and number formatting",
)];

/// Goldens that encode a Hurmet bug Cruncha corrects on purpose: (suite, input, Cruncha's
/// output, reason). The differences list (`docs/design/03-hurmet-differences.md`) has each.
const CORRECTIONS: &[(&str, &str, &str, &str)] = &[(
    "rpn",
    "e^z1",
    "¿e\u{a0}¿z1\u{a0}^",
    "ADR-0025: `e` before `^` stays a variable, so a document's own e (an eccentricity) is used; the evaluator gives Euler's number when e isn't defined",
)];

fn load(name: &str) -> Vec<Vec<String>> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap()
}

fn show(s: &str) -> String {
    s.replace('\u{a0}', "␣")
}

struct Suite {
    name: &'static str,
    passed: usize,
    total: usize,
    gaps: usize,
    failures: Vec<String>,
    unexpected_passes: Vec<String>,
}

impl Suite {
    fn new(name: &'static str) -> Suite {
        Suite {
            name,
            passed: 0,
            total: 0,
            gaps: 0,
            failures: vec![],
            unexpected_passes: vec![],
        }
    }

    fn check(&mut self, input: &str, expected: &str, actual: Result<String, String>) {
        self.total += 1;
        let expected = CORRECTIONS
            .iter()
            .find(|(s, i, _, _)| *s == self.name && *i == input)
            .map_or(expected, |(_, _, ours, _)| *ours);
        let gap = KNOWN_GAPS
            .iter()
            .find(|(s, i, _)| *s == self.name && *i == input);
        let ok = actual.as_deref() == Ok(expected);
        match (ok, gap) {
            (true, None) => self.passed += 1,
            (true, Some(_)) => {
                self.passed += 1;
                self.unexpected_passes.push(input.to_string());
            }
            (false, Some(_)) => self.gaps += 1,
            (false, None) => self.failures.push(format!(
                "  input:    {}\n  expected: {}\n  actual:   {}\n",
                show(input),
                show(expected),
                match &actual {
                    Ok(a) => show(a),
                    Err(e) => format!("ERROR {e}"),
                }
            )),
        }
    }

    fn report(&self) {
        println!(
            "{}: {}/{} passed, {} known gaps, {} failures",
            self.name,
            self.passed,
            self.total,
            self.gaps,
            self.failures.len()
        );
        for f in &self.failures {
            println!("{f}");
        }
        for u in &self.unexpected_passes {
            println!(
                "  known gap now passes (remove it from KNOWN_GAPS): {}",
                show(u)
            );
        }
    }
}

#[test]
fn hurmet_golden() {
    let mut tex = Suite::new("tex");
    let opts = Options::default();
    for case in load("parser_tex.json") {
        let actual = parse_tex(&case[0], &opts).map_err(|e| e.to_string());
        tex.check(&case[0], &case[1], actual);
    }

    let mut rpn = Suite::new("rpn");
    let calc_opts = Options {
        formats: Formats {
            decimal_format: "1,000,000.".into(),
            date_format: Some("yyy-mm-dd".into()),
        },
        ..Options::default()
    };
    for case in load("calc_rpn.json") {
        let actual = parse_calc(&case[0], &calc_opts)
            .map(|c| c.rpn)
            .map_err(|e| e.to_string());
        rpn.check(&case[0], &case[1], actual);
    }

    tex.report();
    rpn.report();
    for (suite, input, reason) in KNOWN_GAPS {
        println!("known gap [{suite}] {}: {reason}", show(input));
    }
    assert!(
        tex.failures.is_empty() && rpn.failures.is_empty(),
        "golden failures; see output above"
    );
    assert!(tex.unexpected_passes.is_empty() && rpn.unexpected_passes.is_empty());
}

/// Every prefix and suffix of every fixture input, in both modes, must parse or return an
/// error, never panic. (The port slices strings by byte offsets that Hurmet computes in UTF-16
/// units, so this guards the char-boundary handling.)
#[test]
fn no_panics_on_fragments() {
    let opts = Options::default();
    let mut inputs: Vec<String> = load("parser_tex.json")
        .into_iter()
        .map(|c| c[0].clone())
        .collect();
    inputs.extend(load("calc_rpn.json").into_iter().map(|c| c[2].clone()));
    let mut n = 0;
    for input in &inputs {
        let bounds: Vec<usize> = input
            .char_indices()
            .map(|(i, _)| i)
            .chain([input.len()])
            .collect();
        for &b in &bounds {
            for frag in [&input[..b], &input[b..]] {
                let _ = parse_tex(frag, &opts);
                let _ = parse_calc(frag, &opts);
                n += 1;
            }
        }
    }
    assert!(n > 1000);
}
