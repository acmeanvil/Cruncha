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

/// Unit names Hurmet couldn't read, and units whose dimensions it had wrong (ADR-0022).
#[test]
fn unit_names_and_dimensions() {
    let cases = [
        ("a = 5 'kilogram' = ?? kg", "5 kg"),
        ("a = 3 'kilograms' = ?? kg", "3 kg"),
        ("a = 20 'Celsius' = ?? K", "293.15 K"),
        ("a = 1 'gauss' = ?? T", "0.0001 T"),
        // Hurmet's kilobyte is 1024 bytes (kB and KiB are separate units).
        ("a = 2 'kilobyte' = ?? byte", "2,048 byte"),
        ("a = 2 'T' × 3 'm^2' = ?? Wb", "6 Wb"),
        ("a = 4 'At' = ?? A", "4 A"),
        ("a = 3600 'mas' = ?? arcsecond", "3.6 arcsecond"),
    ];
    let mut failures = Vec::new();
    for (line, want) in cases {
        let got = calc(&[line]);
        if got != want {
            failures.push(format!("{line}: got `{got}`, expected `{want}`"));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// ADR-0022 item 10: a fixed format kept the digits of a value between −1 and 0 but dropped
/// its sign (Hurmet printed −0.25 as 0.25), in results, `roundn`, `round` and `string`.
#[test]
fn fixed_format_keeps_the_sign() {
    let fixed = |lines: &[&str]| calc(lines);
    assert_eq!(fixed(&["format = \"f2\"", "a = -0.25 = ?"]), "-0.25");
    assert_eq!(fixed(&["format = \"f2\"", "a = -0.004 = ?"]), "0.00");
    assert_eq!(fixed(&["format = \"f2\"", "a = -0.006 = ?"]), "-0.01");
    assert_eq!(fixed(&["format = \"f1\"", "a = -0.96 = ?"]), "-1.0");
    assert_eq!(fixed(&["format = \"f2\"", "a = -1.25 = ?"]), "-1.25");
    assert_eq!(fixed(&["format = \"f0\"", "a = -0.7 = ?"]), "-1");
    check(&[
        ("roundn(-0.256, \"f2\")", "-0.26"),
        ("round(-0.7)", "-1"),
        ("round(-0.4)", "0"),
        ("round(-1.5)", "-2"),
        ("round(2.5)", "3"),
    ]);
}

/// ADR-0022 items 11–14: vectors, matrices and data frames.
#[test]
fn vectors_matrices_and_frames() {
    check(&[
        // Item 11: the cross product, reducers over a matrix, the outer product, |A|.
        ("[1, 0, 0] × [0, 1, 0]", "[0, 0, 1]"),
        ("[1; 2; 3] × [4; 5; 6]", "[-3; 6; -3]"),
        ("sum([1, 2; 3, 4])", "10"),
        ("product([1, 2; 3, 4])", "24"),
        ("mean([1, 2; 3, 4])", "2.5"),
        ("min([1, 2; 3, 4])", "1"),
        ("max([1, 2; 3, 4])", "4"),
        ("range([1, 2; 3, 4])", "3"),
        ("median([1, 2; 3, 4])", "2.5"),
        ("sum([1, 2; 3, 4], 1)", "[4, 6]"),
        ("[2; 3] [4, 5, 6]", "(8, 10, 12; 12, 15, 18)"),
        ("|[1, 2; 3, 4]|", "-2"),
        ("|[0, 1; 1, 0]|", "-1"),
        ("|[1, 2; 2, 4]|", "0"),
        ("|[2, 0, 1; 1, 3, 2; 1, 1, 2]|", "6"),
        ("|[2, 0; 0, 3] 'm'|", "6 m²"),
        // Item 12: units of product, offset units on vectors, zeros/ones columns, powers.
        ("product(2 'm', 3 'm')", "6 m²"),
        ("product([2; 3] 'm')", "6 m²"),
        ("zeros(3, 1)", "[0; 0; 0]"),
        ("ones(1, 2)", "[1, 1]"),
        ("[2, 3]^2", "[4, 9]"),
        ("[2; 3]^2", "[4; 9]"),
        // Item 13: one unit for two-argument functions.
        ("atan2(1 'm', 1 's')", "Error. Unit mis-match between arguments to function atan2."),
        ("gcd(12 'm', 18 's')", "Error. Incorrect unit for input to function gcd."),
        ("gcd(12, 18)", "6"),
        ("sum(1 'm', 2 's')", "Error. Unit mis-match between arguments to function sum."),
    ]);
    assert_eq!(calc(&["v = [20; 30] '°C'", "a = v = ?? °F"]), "[68; 86] °F");
    // Item 14: column vectors side by side inside an expression; stepped columns.
    assert_eq!(calc(&["x = [1; 2]", "y = [3; 4]", "a = 2 + sum([x, y]) = ?"]), "12");
    assert_eq!(calc(&["M = [1, 2, 3, 4; 5, 6, 7, 8]", "a = M[1:2, 1:2:4] = ?"]), "(1, 3; 5, 7)");
    // Item 14: data-frame selections.
    let df = "df = ``#name\tA\tB\tC\nr1\t1\t2\t3\nr2\t4\t5\t6\nr3\t7\t8\t9``";
    assert_eq!(calc(&[df, "a = df[\"r2\", \"B\"] = ?"]), "5");
    assert_eq!(calc(&[df, "s = df[[\"C\", \"A\"]] = !", "a = s.C = ?"]), "[3; 6; 9]");
    assert_eq!(calc(&[df, "a = df[\"A\", \"B\"] = ?"]).replace('\n', "/"), "``A\tB/1\t2/4\t5/7\t8``");
    let plain = "p = ``A\tB\tC\n1\t2\t3\n4\t5\t6``";
    assert_eq!(calc(&[plain, "a = p[1, 2:3] = ?"]).replace('\n', "/"), "``B\tC/2\t3``");
    assert_eq!(calc(&[plain, "a = p[\"x\"] = ?"]), "Error. Data frame does not have a column named x.");
}
