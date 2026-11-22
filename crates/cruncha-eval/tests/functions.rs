// SPDX-License-Identifier: GPL-3.0-or-later
//! Built-in functions that ADR-0022 fixed (Hurmet's versions were wrong or missing):
//! each case's result as the app shows it (unit-aware).

use cruncha_eval::{Options, UnitAware, Vars, calculate};

fn calc(lines: &[&str]) -> String {
    let opts = Options { unit_aware: UnitAware::Always, ..Options::default() };
    let mut vars = Vars::new();
    let mut last = String::new();
    for l in lines {
        last = calculate(l, &mut vars, &opts);
    }
    last.rsplit(" = ").next().unwrap_or_default().to_string()
}

fn check(cases: &[(&str, &str)]) {
    let mut failures = Vec::new();
    for (expr, want) in cases {
        let got = calc(&[&format!("a = {expr} = ?")]);
        if got != *want {
            failures.push(format!("{expr}: got `{got}`, expected `{want}`"));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn modulo_and_remainder() {
    check(&[
        ("mod(-7, 3)", "2"),
        ("mod(7, -3)", "-2"),
        ("mod(7.5, 2)", "1.5"),
        ("mod(-7.5, 2)", "0.5"),
        ("mod(6, 3)", "0"),
        ("rem(-7, 3)", "-1"),
        ("rem(7.5, 2)", "1.5"),
        ("rem(-7.5, 2)", "-1.5"),
        ("5 mod 3", "2"),
        ("-7 mod 3", "2"),
        ("mod(7.5 'm', 2 'm')", "1.5 m"),
        ("mod(5, 0)", "Error. Divide by zero."),
        ("rem(5, 0)", "Error. Divide by zero."),
        ("mod(5 'm', 2 's')", "Error. Unit mis-match between arguments to function mod."),
    ]);
}

#[test]
fn hypot_and_rms() {
    check(&[
        ("hypot(-3, -4)", "5"),
        ("hypot(-3, 4)", "5"),
        ("hypot(0, 0)", "0"),
        ("hypot(3, 4, 12)", "13"),
        ("hypot(3 'm', 4 'm')", "5 m"),
        ("rms(3, 4)", "3.53553390593274"),
        ("rms(1, 2, 3, 4)", "2.73861278752583"),
        ("rms(-2, 2)", "2"),
        ("rms([3; 4])", "3.53553390593274"),
    ]);
}

#[test]
fn statistics() {
    check(&[
        ("median(1, 2, 10)", "2"),
        ("median(10, 1, 2, 3)", "2.5"),
        ("median([1; 2; 10])", "2"),
        ("median(3 'm', 1 'm', 2 'm')", "2 m"),
        ("median(4)", "4"),
        ("variance(1, 2, 3, 4)", "1.66666666666667"),
        ("stddev(1, 2, 3, 4)", "1.29099444873581"),
        ("stddev([2; 4; 4; 4; 5; 5; 7; 9])", "2.1380899352994"),
        ("stddev(1 'm', 2 'm', 3 'm')", "1 m"),
        ("variance(1)", "Error. variance needs at least two values."),
    ]);
    assert_eq!(calc(&["a = variance(1 'm', 2 'm', 3 'm') = ?? m^2"]), "1 m^2");
    assert_eq!(calc(&["a = rms(3 'V', 4 'V') = ?? V"]), "3.53553390593274 V");
}

#[test]
fn inverse_trigonometry() {
    check(&[
        ("asec(2)", "1.0471975511966"),
        ("asec(-2)", "2.0943951023932"),
        ("acsc(2)", "0.523598775598299"),
        ("acot(2)", "0.463647609000806"),
        ("acot(0)", "1.5707963267949"),
        ("acsch(2)", "0.481211825059603"),
        ("asecd(2)", "60"),
        ("acscd(2)", "30"),
        ("acotd(1)", "45"),
        ("asec(0.5)", "Error. Absolute value of input to asec must be ≥ 1"),
        ("acsc(0.5)", "Error. Absolute value of input to acsc must be ≥ 1"),
    ]);
}

#[test]
fn trace_char_and_e() {
    check(&[
        ("trace([1, 2; 3, 4])", "5"),
        ("trace([1, 2, 3])", "Error. trace needs a square matrix."),
        ("Char(65)", "A"),
        ("roundSig(3.14159, 3)", "3.14"),
        ("roundSig(123456, 2)", "120,000"),
        ("roundSig(3.14159, 0)", "Error. The number of significant digits must be a whole number from 1 to 21."),
        ("e", "2.71828182845905"),
        ("2 e", "5.43656365691809"),
        ("e^2", "7.38905609893065"),
    ]);
    // A document's own `e` (an eccentricity, say) wins over Euler's number.
    assert_eq!(calc(&["e = 0.5", "a = 2 e = ?"]), "1");
}
