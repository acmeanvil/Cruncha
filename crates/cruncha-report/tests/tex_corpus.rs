// SPDX-License-Identifier: GPL-3.0-or-later

//! Every TeX string the calc engine produces for Hurmet's golden tests (the parser's TeX
//! goldens, and the whole display line of every evaluated assignment and calc, with echoes
//! and errors) must convert to Typst math that compiles.

use cruncha_eval::{Currencies, Options, Vars, evaluate, prepare};
use cruncha_report::{check_math, check_math_batch, tex::to_typst};
use std::sync::Arc;

fn load(krate: &str, name: &str) -> serde_json::Value {
    let path = format!("{}/../{krate}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The corpus: (where it came from, TeX).
fn corpus() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for case in load("cruncha-lang", "parser_tex.json").as_array().unwrap() {
        out.push((format!("parser {}", case[0]), case[1].as_str().unwrap().to_string()));
    }
    let mut c = Currencies::new();
    c.insert("USD".into(), "1.1014".into());
    c.insert("CAD".into(), "1.5734".into());
    c.insert("EUR".into(), "1.0".into());
    let opts = Options { currencies: Arc::new(c), ..Options::default() };
    let mut vars = Vars::new();
    vars.set_format("h15");
    let mut run = |src: &str, out: &mut Vec<(String, String)>| {
        let Ok(stmt) = prepare(src, &opts) else { return };
        let o = evaluate(&stmt, &vars, &opts);
        for (name, value) in o.values {
            vars.insert(name, value);
        }
        if !o.tex.is_empty() {
            out.push((format!("calc {src:?}"), o.tex));
        }
    };
    for case in load("cruncha-eval", "assignments.json").as_array().unwrap() {
        run(case[0].as_str().unwrap(), &mut out);
        run(case[1].as_str().unwrap(), &mut out);
    }
    for f in load("cruncha-eval", "functions.json").as_array().unwrap() {
        run(f.as_str().unwrap(), &mut out);
    }
    for case in load("cruncha-eval", "calculations.json").as_array().unwrap() {
        run(case[0].as_str().unwrap(), &mut out);
    }
    out
}

#[test]
fn every_engine_tex_string_converts_and_compiles() {
    let corpus = corpus();
    let mut converted = Vec::new();
    let mut failures = Vec::new();
    for (from, tex) in &corpus {
        match to_typst(tex) {
            Ok(m) => converted.push((from, tex, m)),
            Err(e) => failures.push(format!("CONVERT {from}\n   tex   {tex}\n   error {e}")),
        }
    }
    let maths: Vec<String> = converted.iter().map(|c| c.2.clone()).collect();
    let t = std::time::Instant::now();
    if check_math_batch(&maths).is_err() {
        for (from, tex, m) in &converted {
            if let Err(e) = check_math(m) {
                failures.push(format!("COMPILE {from}\n   tex   {tex}\n   typst {m}\n   error {e}"));
            }
        }
    }
    println!("compiled {} snippets in {:.0} ms", maths.len(), t.elapsed().as_secs_f64() * 1e3);
    for f in &failures {
        println!("{f}");
    }
    println!("{} / {} TeX strings convert and compile", corpus.len() - failures.len(), corpus.len());
    assert!(failures.is_empty(), "{} failures", failures.len());
}
