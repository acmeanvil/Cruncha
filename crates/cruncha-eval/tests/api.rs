// SPDX-License-Identifier: GPL-3.0-or-later

//! The public API: prepare/evaluate, the TeX output, and ADR-0007's always-unit-aware mode.

use cruncha_eval::{Display, Kind, Options, UnitAware, Vars, calculate, evaluate, prepare};

fn run(lines: &[&str], opts: &Options) -> (Vars, Vec<cruncha_eval::Outcome>) {
    let mut vars = Vars::new();
    let mut outs = Vec::new();
    for l in lines {
        let stmt = prepare(l, opts).unwrap();
        let out = evaluate(&stmt, &vars, opts);
        for (n, v) in &out.values {
            vars.insert(n.clone(), v.clone());
        }
        outs.push(out);
    }
    (vars, outs)
}

const BEAM: [&str; 3] = ["w = 1.5 kip/ft", "L = 24 ft", "M = w L^2 / 8 = ?? kip·ft"];

#[test]
fn beam_moment_hurmet_mode() {
    let opts = Options::default();
    let (vars, outs) = run(&BEAM, &opts);
    let m = &outs[2];
    println!("{}", m.tex);
    assert_eq!(m.result, "M = w L^2 / 8 = 108 kip·ft");
    assert!(m.error.is_none());
    assert_eq!(m.values.len(), 1);
    // Hurmet's `/` takes the run of factors just before it: w·(L²/8).
    assert_eq!(
        m.tex,
        "M = w \\dfrac{L^{2}}{8} = \\textcolor{#0000ff}{(1.5\\, {\\text{kip/ft}}) \\dfrac{(24\\, {\\text{ft}})^{2}}{8}} = 108 \\; {\\text{kip}\\mkern1mu{\\cdot}\\mkern1mu\\text{ft}}"
    );
    let v = vars.get("M").unwrap();
    assert_eq!(v.kind(), Kind::Number);
    assert_eq!(v.as_f64(), Some(108.0));
    assert_eq!(v.unit_name(), Some("kip·ft"));
    assert!((v.as_base_f64().unwrap() - 108.0 * 4448.2216152605 * 0.3048).abs() < 1e-6);
}

#[test]
fn beam_moment_always_mode() {
    let opts = Options {
        unit_aware: UnitAware::Always,
        ..Options::default()
    };
    let (_, outs) = run(&BEAM, &opts);
    println!("{}", outs[2].tex);
    assert_eq!(outs[2].result, "M = w L^2 / 8 = 108 kip·ft");
}

#[test]
fn single_selector_is_unit_aware_under_always() {
    // ADR-0007: with L = 3 ft, Hurmet gives 6 for `2 L = ?`; Cruncha gives 1.8288 m.
    let lines = ["L = 3 ft", "2 L = ?"];
    let (_, hurmet) = run(&lines, &Options::default());
    assert_eq!(hurmet[1].result, "2 L = 6");
    let always = Options {
        unit_aware: UnitAware::Always,
        ..Options::default()
    };
    let (_, outs) = run(&lines, &always);
    assert_eq!(outs[1].result, "2 L = 1.8288 m");
    // With a display unit, `?` converts.
    let (_, outs) = run(&["L = 3 ft", "2 L = ? inch"], &always);
    assert_eq!(outs[1].result, "2 L = 72 inch");
    // `??` without a unit is an error in Hurmet, SI base units under Always.
    let (_, h) = run(&["L = 3 ft", "x = 2 L = @@"], &Options::default());
    assert_eq!(h[1].result, "Error. No units specified for the result.");
    let (vars, outs) = run(&["L = 3 ft", "A = L^2 = @"], &always);
    assert_eq!(outs[1].result, "0.83612736 m²");
    assert_eq!(vars.get("A").unwrap().unit_name(), Some("m²"));
    // Dimensionless results are unchanged.
    let (_, outs) = run(&["L = 3 ft", "r = L / (1ft) = ?"], &always);
    assert_eq!(outs[1].result, "r = L / (1ft) = 3");
    // Incompatible display units are still an error.
    let (_, outs) = run(&["L = 3 ft", "2 L = ? kg"], &always);
    assert!(outs[1].error.is_some());
}

#[test]
fn statement_metadata() {
    let opts = Options::default();
    let s = prepare("M = w L^2 / 8 = ?? kip·ft", &opts).unwrap();
    assert_eq!(s.defines, vec!["M"]);
    assert_eq!(s.reads, vec!["w", "L"]);
    assert_eq!(s.display, Display::EchoUnitAware);
    assert_eq!(s.unit.as_deref(), Some("kip·ft"));
    let s = prepare("w = 1.5 kip/ft", &opts).unwrap();
    assert_eq!(s.display, Display::None);
    assert_eq!(s.defines, vec!["w"]);
    let s = prepare("2 + 2 = @", &opts).unwrap();
    assert!(s.defines.is_empty());
    assert_eq!(s.display, Display::ResultOnly);
    let s = prepare("a, b = [1, 2] = !", &opts).unwrap();
    assert_eq!(s.defines, vec!["a", "b"]);
    assert_eq!(s.display, Display::Suppress);
}

#[test]
fn errors_are_reported_not_panicked() {
    let opts = Options::default();
    let (_, outs) = run(&["x = 1/0 = ?", "y = x + 1 = ?"], &opts);
    assert_eq!(outs[0].error.as_deref(), Some("Error. Divide by zero."));
    assert!(
        outs[0]
            .tex
            .contains("\\textcolor{firebrick}{\\text{Error. Divide by zero.}}")
    );
    assert_eq!(
        outs[0].values.len(),
        1,
        "an errored assignment still defines x"
    );
    assert!(outs[1].error.as_deref().unwrap().contains("Missing value"));
    let (_, outs) = run(&["z = q + 1 = ?"], &opts);
    assert_eq!(outs[0].result, "z = q + 1 = Error. Variable $q$ not found.");
}

#[test]
fn multiple_assignment() {
    let opts = Options::default();
    let mut vars = Vars::new();
    calculate("a, b = [3, 4] = ?", &mut vars, &opts);
    assert_eq!(calculate("a + b = @", &mut vars, &opts), "7");
}

/// Hurmet behaviors that the port reproduces on purpose (docs/spikes/S2b, Finding 4).
#[test]
fn hurmet_quirks() {
    let opts = Options::default();
    let mut vars = Vars::new();
    let mut calc = |s: &str| calculate(s, &mut vars, &opts);
    assert_eq!(calc("[2, 3]^2 = @"), "4 + j 9");
    assert_eq!(calc("[2; 3] [4, 5] = @"), "(8, 10; 12, 15)");
    assert_eq!(
        calc("[2; 3] [4, 5, 6] = @"),
        "Error. Mis-matched number of elements"
    );
    assert_eq!(calc("round(-0.7) = @"), "1");
    assert_eq!(calc("(\"a\" ∉ \"xyz\") = @"), "false");
    assert_eq!(calc("|(2, 0; 0, 4)| = @"), "0.125");
}

#[test]
fn values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<cruncha_eval::Value>();
    assert_send_sync::<cruncha_eval::Statement>();
    assert_send_sync::<Vars>();
    assert_send_sync::<Options>();
    assert_send_sync::<cruncha_eval::Outcome>();
}
