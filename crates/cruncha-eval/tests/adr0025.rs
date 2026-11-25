// SPDX-License-Identifier: GPL-3.0-or-later
//! ADR-0025: engine gaps found converting the Stress and failure templates. Each case as the
//! app shows it (unit-aware).

use cruncha_eval::{Options, UnitAware, Vars, evaluate, prepare};

/// Evaluate `lines` in order; the last one's outcome: its result text (after the last
/// " = ") and its check.
fn run(lines: &[&str]) -> (String, Option<bool>) {
    let opts = Options { unit_aware: UnitAware::Always, ..Options::default() };
    let mut vars = Vars::new();
    let mut last = (String::new(), None);
    for l in lines {
        let stmt = prepare(l, &opts).unwrap_or_else(|e| panic!("{l}: {e}"));
        let out = evaluate(&stmt, &vars, &opts);
        last = (out.result.rsplit(" = ").next().unwrap_or_default().to_string(), out.check);
        for (n, v) in out.values {
            vars.insert(n, v);
        }
    }
    last
}

fn value(lines: &[&str]) -> String {
    run(lines).0
}

#[test]
fn zero_to_a_fractional_power() {
    assert_eq!(value(&["d = sqrt(0) = ??"]), "0");
    assert_eq!(value(&["c = 0^0.5 = ??"]), "0");
    assert_eq!(value(&["c = 0^(3/2) = ??"]), "0");
    assert_eq!(value(&["a = 0 ksi", "b = sqrt(a^2) = ?? ksi"]), "0 ksi");
    assert_eq!(value(&["c = 0^(-0.5) = ??"]), "Error. Divide by zero.");
    assert_eq!(value(&["c = 4^0.5 = ??"]), "2");
}

#[test]
fn compound_tests_pass_or_fail() {
    assert_eq!(run(&["g = 5", "@test 0.1 ≤ g ≤ 40"]), ("0.1 ≤ g ≤ 40, ok ✓".into(), Some(true)));
    assert_eq!(run(&["g = 50", "@test 0.1 ≤ g ≤ 40"]), ("0.1 ≤ g ≤ 40, n.g.".into(), Some(false)));
    assert_eq!(run(&["g = 5", "@test (0.1 ≤ g) and (g ≤ 40)"]).1, Some(true));
    assert_eq!(run(&["g = 50", "@test (0.1 ≤ g) and (g ≤ 40)"]).1, Some(false));
    assert_eq!(run(&["D = 20 mm", "@test 0.1 'mm' ≤ D ≤ 40 'mm'"]).1, Some(true));
    assert_eq!(run(&["g = 5", "@test g ≤ 4"]), ("g !≤ 4, n.g.".into(), Some(false)), "single comparisons unchanged");
}

#[test]
fn chained_comparisons_in_conditions() {
    assert_eq!(value(&["g = 5", "k = {1 if 0.1 ≤ g ≤ 40; 2 otherwise} = ??"]), "1");
    assert_eq!(value(&["g = 50", "k = {1 if 0.1 ≤ g ≤ 40; 2 otherwise} = ??"]), "2");
    assert_eq!(value(&["g = 5", "h = 0.1 ≤ g ≤ 40 = ??"]), "true");
}

#[test]
fn function_locals_with_subscripts() {
    let f = "function kt(c)\n    k_1 = c + 1\n    k_2, k_3 = 2, 3\n    return k_1 + k_2 k_3\nend";
    assert_eq!(value(&[f, "y = kt(2) = ??"]), "9");
}

#[test]
fn one_entry_vectors() {
    assert_eq!(value(&["w = [403] MPa", "n = length(w) = ??"]), "1");
    assert_eq!(value(&["w = [403]", "s = sum(w) = ??"]), "403");
    assert_eq!(value(&["w = [403]", "x = w[1] = ??"]), "403");
    assert_eq!(value(&["v = vcat([1; 2], [3]) = ??"]), "[1; 2; 3]");
    assert_eq!(value(&["v = vcat([1; 2], 3) = ??"]), "[1; 2; 3]");
    assert_eq!(value(&["v = vcat([1; 2], [3; 4]) = ??"]), "[1; 2; 3; 4]");
}

#[test]
fn ratios_are_plain_numbers_in_a_matrix() {
    let pre = ["t = 2 mm", "r = 1 mm", "x = t / r = ??"];
    assert_eq!(value(&[pre[0], pre[1], pre[2], "v = [1; sqrt(x); x] = ??"]), "[1; 1.4142135623731; 2]");
    assert_eq!(value(&[pre[0], pre[1], "v = [1; t] = ??"]), "Error. Matrix elements cannot contain a unit. Write the unit outside the matrix.");
}

#[test]
fn units_added() {
    assert_eq!(value(&["S = 60 kpsi", "x = S = ?? MPa"]), "413.685437590102 MPa");
    assert_eq!(value(&["x = 30 'Mpsi' = ?? GPa"]), "206.842718795051 GPa");
    assert_eq!(value(&["dT = 10 Δ°C", "x = dT = ?? K"]), "10 K");
    assert_eq!(value(&["x = 18 'Δ°F' = ?? Δ°C"]), "10 Δ°C");
}

#[test]
fn a_unit_after_a_power_says_how_to_write_it() {
    assert!(value(&["s = 2", "a = (3 - s)^2 'in' = ?? in"]).starts_with("Error. An exponent can't have a unit."));
    assert_eq!(value(&["s = 2", "a = (3 - s)^2 × 1 'in' = ?? in"]), "1 in");
}

#[test]
fn functions_call_functions() {
    let inner = "function inner(x)\n    return 2 x\nend";
    let outer = "function outer(x)\n    return inner(x) + 1\nend";
    assert_eq!(value(&[inner, outer, "y = outer(3) = ??"]), "7");
    let opts = Options { unit_aware: UnitAware::Always, ..Options::default() };
    assert_eq!(prepare(outer, &opts).unwrap().reads, vec!["inner".to_string()]);
    // A function still sees only its arguments, not the document's other values.
    let g = "function h(x)\n    return x + k\nend";
    assert!(value(&["k = 1", g, "y = h(3) = ??"]).starts_with("Error"));
}

#[test]
fn half_integer_unit_powers() {
    let k = ["σ = 100 MPa", "c = 1 mm"];
    assert_eq!(value(&[k[0], k[1], "K = σ sqrt(π c) = ?? MPa·√m"]), "5.60499121639793 MPa·√m");
    assert_eq!(value(&[k[0], k[1], "K = σ sqrt(π c) = ?? MPa·m^0.5"]), "5.60499121639793 MPa·m^0.5");
    assert_eq!(value(&[k[0], k[1], "K = σ sqrt(π c) = ?? ksi·√in"]), "5.10080939319207 ksi·√in");
    assert_eq!(value(&["K_Ic = 50 MPa·√m", "x = K_Ic = ?? ksi·√in"]), "45.5023852514628 ksi·√in");
    assert_eq!(value(&["x = 50 'MPa·m^(1/2)' = ?? MPa·√m"]), "50 MPa·√m");
    assert!(value(&["σ = 100 MPa", "x = σ = ?? MPa·√m"]).starts_with("Error"), "dimensions are checked");
    assert!(value(&["x = 2 'm^0.3' = ??"]).starts_with("Error"), "only halves");
}

#[test]
fn lints_for_silent_mistakes() {
    let opts = Options { unit_aware: UnitAware::Always, ..Options::default() };
    let lint = |src: &str| cruncha_eval::lints(src, &prepare(src, &opts).unwrap());
    assert!(lint("x = t / r")[0].starts_with("x isn't calculated"), "{:?}", lint("x = t / r"));
    assert!(lint("x = t / r = ??").is_empty());
    assert!(lint("x = 5 mm").is_empty(), "a literal assigns");
    assert!(lint("@test x ≤ 4").is_empty());
    assert!(lint("x = 5 mm_").is_empty());
    let w = lint("K_Ic,req = 2 K_I = ??");
    assert!(w.iter().any(|w| w.contains("K_(Ic,req)")), "{w:?}");
    assert!(lint("a, b = 1, 2").is_empty(), "a real multiple assignment");
}

#[test]
fn double_primes_define_what_they_read() {
    assert_eq!(value(&["δ_P = 2", "δ_P″ = δ_P + 1 = ??", "y = 2 δ_P″ = ??"]), "6");
    assert_eq!(value(&["a″ = 4", "y = a″ = ??"]), "4");
}

#[test]
fn a_double_prime_after_a_number_is_still_inches() {
    assert_eq!(value(&["x = 5″", "y = x = ?? mm"]), "127 mm");
}

#[test]
fn piecewise_without_otherwise() {
    assert_eq!(value(&["k = 1", "c = { 1 if k == 1; 2 if k == 2 } = ??"]), "1");
    assert_eq!(value(&["k = 2", "c = { 1 if k == 1; 2 if k == 2 } = ??"]), "2");
    assert_eq!(value(&["k = 2", "c = { 10 if k < 2; 20 if k ≥ 2 } = ??"]), "20");
    assert_eq!(value(&["k = 2", "c = { 1 if k == 1; 25 } = ??"]), "25");
    assert_eq!(value(&["k = 2", "c = { 1 if k == 1; 25 otherwise } = ??"]), "25");
    assert!(value(&["k = 3", "c = { 1 if k == 1; 2 if k == 2 } = ??"]).starts_with("Error"));
}

#[test]
fn display_units_with_parentheses() {
    assert_eq!(value(&["T = 2 N·m", "a = 0.5 / T = ?? 1/(N·m)"]), "0.25 1/(N·m)");
    assert_eq!(value(&["T = 2 N·m", "a = 0.5 / T = ?? (N·m)^-1"]), "0.25 (N·m)^-1");
}

#[test]
fn ohms_and_reciprocal_units() {
    assert_eq!(value(&["R = 120 Ω", "x = R = ?? ohm"]), "120 ohm");
    assert_eq!(value(&["R = 2.2 kΩ", "x = R = ?? Ω"]), "2,200 Ω");
    assert_eq!(value(&["a = 0.0000133 1/Δ°F", "x = a = ?? 1/Δ°C"]), "0.00002394 1/Δ°C");
    assert_eq!(value(&["k = 4 1/in", "x = k = ?? 1/mm"]), "0.15748031496063 1/mm");
}

#[test]
fn a_range_index_is_a_plain_number() {
    let f = "function f3(a)\n    s = 0\n    for k in 0:3\n        s = s + k + a\n    end\n    return s\nend";
    assert_eq!(value(&[f, "q = f3(2) = ??"]), "14");
    let g = "function g(a)\n    s = 0\n    for k in 1:2\n        s = s + sin(k)\n    end\n    return s\nend";
    assert!(value(&[g, "q = g(0) = ??"]).starts_with("1.75076"), "{}", value(&[g, "q = g(0) = ??"]));
}

#[test]
fn a_defined_e_is_used_before_a_power() {
    assert_eq!(value(&["e = 0.5", "b = e^2 = ??"]), "0.25");
    assert_eq!(value(&["e = 0.5", "b = (1 - e^2) / 2 = ??"]), "0.375");
    assert!(value(&["b = e^2 = ??"]).starts_with("7.389056"), "Euler's number when e isn't defined");
}

#[test]
fn function_parameters_with_subscripts() {
    let f = "function f(u_1, θ_0)\n    return u_1 + θ_0\nend";
    assert_eq!(value(&[f, "y = f(2, 3) = ??"]), "5");
}
