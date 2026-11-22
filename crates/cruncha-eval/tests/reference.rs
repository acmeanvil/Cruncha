// SPDX-License-Identifier: GPL-3.0-or-later
//! The function reference (`reference/functions.json`, shown in the app): every example
//! evaluates, unit-aware as in the app, to what the entry says, and every built-in function the
//! parser knows is listed.

use cruncha_eval::{Options, UnitAware, Vars, calculate};
use serde_json::Value;

fn reference() -> Value {
    serde_json::from_str(include_str!("../reference/functions.json")).expect("functions.json parses")
}

#[test]
fn every_example_gives_its_result() {
    let r = reference();
    let opts = Options { unit_aware: UnitAware::Always, ..Options::default() };
    let categories: Vec<&str> = r["categories"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
    let mut failures = Vec::new();
    for e in r["entries"].as_array().unwrap() {
        let name = e["name"].as_str().unwrap();
        assert!(categories.contains(&e["category"].as_str().unwrap()), "{name}: unknown category");
        assert!(e["summary"].as_str().is_some_and(|s| !s.is_empty()), "{name}: no summary");
        let available = e["available"].as_bool().unwrap_or(true);
        if !available {
            assert!(e["note"].is_string(), "{name}: unavailable without a note");
            assert!(e["example"].is_null(), "{name}: unavailable with an example");
            continue;
        }
        let Some(lines) = e["example"].as_array() else { continue };
        let want = e["result"].as_str().unwrap_or_else(|| panic!("{name}: an example needs a result"));
        let mut vars = Vars::new();
        let mut last = String::new();
        for l in lines {
            last = calculate(l.as_str().unwrap(), &mut vars, &opts);
        }
        let got = last.rsplit(" = ").next().unwrap_or_default();
        if std::env::var_os("CRUNCHA_PRINT_REFERENCE").is_some() {
            println!("{name:>14}: {last}");
        }
        if !got.contains(want) || last.contains("Error") {
            failures.push(format!("{name}: `{}` gave `{last}`, expected `{want}`", lines.last().unwrap()));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn every_built_in_is_listed() {
    let r = reference();
    let names: Vec<&str> = r["entries"].as_array().unwrap().iter().map(|e| e["name"].as_str().unwrap()).collect();
    let missing: Vec<_> = cruncha_lang::hurmet::BUILT_IN_FUNCTIONS
        .iter()
        .chain(cruncha_lang::hurmet::BUILT_IN_REDUCER_FUNCTIONS)
        .filter(|f| !names.contains(f))
        .collect();
    assert!(missing.is_empty(), "not in reference/functions.json: {missing:?}");
    let mut seen = std::collections::HashSet::new();
    for n in &names {
        assert!(seen.insert(n), "{n} is listed twice");
    }
}
