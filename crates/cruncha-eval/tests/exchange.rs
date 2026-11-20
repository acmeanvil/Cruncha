// SPDX-License-Identifier: GPL-3.0-or-later

//! The value exchange bridge: `Value::to_plain` / `Value::from_plain` round trips, values
//! brought back from "Python" used by later statements, the SI fallback, and errors.

use cruncha_eval::{
    ColumnData, Kind, Num, Options, Plain, PlainColumn, UnitAware, UnitView, Value, Vars,
    calculate, evaluate, prepare,
};

fn always() -> Options {
    Options {
        unit_aware: UnitAware::Always,
        ..Options::default()
    }
}

/// Assign with `calculate` and return the stored value.
fn value_of(lines: &[&str], name: &str, opts: &Options) -> Value {
    let mut vars = Vars::new();
    for l in lines {
        calculate(l, &mut vars, opts);
    }
    vars.get(name)
        .unwrap_or_else(|| panic!("{name} not defined"))
        .clone()
}

/// to_plain → from_plain → to_plain gives the same plain value. Returns the plain value.
fn round_trip(v: &Value) -> Plain {
    let p = v.to_plain();
    let back = Value::from_plain(&p).unwrap_or_else(|e| panic!("{p:?}: {e}"));
    assert_eq!(back.to_plain(), p, "round trip");
    // And through JSON.
    let json = serde_json::to_string(&p).unwrap();
    let parsed: Plain = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, p, "json: {json}");
    p
}

fn exact(num: &str, den: &str) -> Num {
    Num::Exact {
        num: num.into(),
        den: den.into(),
    }
}

fn unit(name: Option<&str>, factor: f64, gauge: f64, dims: [f64; 8]) -> UnitView {
    UnitView {
        name: name.map(str::to_string),
        factor,
        gauge,
        dims,
    }
}

const FORCE: [f64; 8] = [1.0, 1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const MOMENT: [f64; 8] = [2.0, 1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const LENGTH: [f64; 8] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const TEMP: [f64; 8] = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];

// ----------------------------------------------------------------------------------------
// Round trips
// ----------------------------------------------------------------------------------------

#[test]
fn exact_rational() {
    let v = value_of(&["r = 1/3 = ?"], "r", &Options::default());
    let p = round_trip(&v);
    assert_eq!(
        p,
        Plain::Number {
            value: exact("1", "3"),
            unit: None
        }
    );
    let back = Value::from_plain(&p).unwrap();
    assert_eq!(back.as_rational(), v.as_rational());
}

#[test]
fn float_literal_and_float_input() {
    let v = value_of(&["g = 9.80665"], "g", &Options::default());
    let p = round_trip(&v);
    assert_eq!(
        p,
        Plain::Number {
            value: exact("980665", "100000"),
            unit: None
        }
    );
    // A float from Python becomes a rational the way Hurmet's Rnl.fromNumber does it.
    let f = Value::from_plain(&Plain::Number {
        value: Num::Float { value: 0.1 },
        unit: None,
    })
    .unwrap();
    assert_eq!(
        f.to_plain(),
        Plain::Number {
            value: exact("1", "10"),
            unit: None
        }
    );
    assert_eq!(f.tex(), "0.1");
    round_trip(&f);
}

#[test]
fn quantity() {
    let v = value_of(&["w = 1.5 kip/ft"], "w", &Options::default());
    let p = round_trip(&v);
    let Plain::Number {
        value,
        unit: Some(u),
    } = &p
    else {
        panic!("{p:?}")
    };
    assert_eq!(*value, exact("15", "10"));
    assert_eq!(u.name.as_deref(), Some("kip/ft"));
    assert_eq!(u.dims, [0.0, 1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    assert!((u.factor - 4448.2216152605 / 0.3048).abs() < 1e-9);
    assert_eq!(u.gauge, 0.0);
    let back = Value::from_plain(&p).unwrap();
    assert_eq!(back.unit_name(), Some("kip/ft"));
    assert_eq!(back.as_f64(), Some(1.5));
    assert!((back.as_base_f64().unwrap() - v.as_base_f64().unwrap()).abs() < 1e-9);
    assert_eq!(back.tex(), "1.5 \\; {\\text{kip/ft}}");
}

#[test]
fn calc_result_quantity() {
    let v = value_of(
        &["w = 1.5 kip/ft", "L = 24 ft", "M = w L^2 / 8 = ?? kip·ft"],
        "M",
        &Options::default(),
    );
    let p = round_trip(&v);
    let Plain::Number { unit: Some(u), .. } = &p else {
        panic!("{p:?}")
    };
    assert_eq!(u.name.as_deref(), Some("kip·ft"));
    assert_eq!(u.dims, MOMENT);
    // The echo TeX matches the original calc result's.
    assert_eq!(Value::from_plain(&p).unwrap().tex(), v.tex());
}

#[test]
fn temperatures_with_gauge() {
    let c = value_of(&["T = 20 °C"], "T", &Options::default());
    let p = round_trip(&c);
    let Plain::Number { unit: Some(u), .. } = &p else {
        panic!("{p:?}")
    };
    assert_eq!(u.name.as_deref(), Some("°C"));
    assert_eq!((u.factor, u.gauge, u.dims), (1.0, 273.15, TEMP));
    let back = Value::from_plain(&p).unwrap();
    assert!((back.as_base_f64().unwrap() - 293.15).abs() < 1e-12);

    let f = value_of(&["T = 68 °F"], "T", &Options::default());
    let p = round_trip(&f);
    let Plain::Number { unit: Some(u), .. } = &p else {
        panic!("{p:?}")
    };
    assert_eq!(u.name.as_deref(), Some("°F"));
    assert!((u.factor - 5.0 / 9.0).abs() < 1e-15);
    // Hurmet's unit table has 459 (not 459.67) as the gauge of °F; the port keeps its data.
    assert!(u.gauge > 459.0 - 1e-9 && u.gauge < 460.0);
    let back = Value::from_plain(&p).unwrap();
    assert_eq!(back.as_base_f64(), f.as_base_f64());
    assert!((back.as_base_f64().unwrap() - (68.0 + u.gauge) * u.factor).abs() < 1e-9);
}

#[test]
fn vectors_and_matrices() {
    let opts = Options::default();
    let row = value_of(&["v = [1, 2.5, 3]"], "v", &opts);
    assert_eq!(
        round_trip(&row),
        Plain::Array {
            shape: vec![3],
            orientation: "row".into(),
            values: vec![1.0, 2.5, 3.0],
            unit: None
        }
    );
    let col = value_of(&["v = [1; 2; 3]"], "v", &opts);
    let p = round_trip(&col);
    let Plain::Array { orientation, .. } = &p else {
        panic!("{p:?}")
    };
    assert_eq!(orientation, "column");

    let m = value_of(&["A = [1, 2; 3, 4]"], "A", &opts);
    assert_eq!(
        round_trip(&m),
        Plain::Array {
            shape: vec![2, 2],
            orientation: "matrix".into(),
            values: vec![1.0, 2.0, 3.0, 4.0],
            unit: None
        }
    );
    let back = Value::from_plain(&m.to_plain()).unwrap();
    assert_eq!(back.kind(), Kind::Matrix);
    // Shown as a calc result shows a matrix.
    assert_eq!(back.tex(), "\\begin{pmatrix}1 &2  \\\\ 3 &4\\end{pmatrix}");

    let vm = value_of(&["v = [1, 2, 3] m"], "v", &opts);
    let p = round_trip(&vm);
    let Plain::Array { unit: Some(u), .. } = &p else {
        panic!("{p:?}")
    };
    assert_eq!(u.name.as_deref(), Some("m"));
    assert_eq!(u.dims, LENGTH);

    let mk = value_of(&["K = [1, 2; 3, 4] kN"], "K", &opts);
    let p = round_trip(&mk);
    let Plain::Array {
        unit: Some(u),
        shape,
        ..
    } = &p
    else {
        panic!("{p:?}")
    };
    assert_eq!(shape, &vec![2, 2]);
    assert_eq!(u.name.as_deref(), Some("kN"));
    assert_eq!(u.dims, FORCE);
}

#[test]
fn bool_and_text() {
    let opts = Options::default();
    let b = value_of(&["ok = true"], "ok", &opts);
    assert_eq!(round_trip(&b), Plain::Bool { value: true });
    let s = value_of(&[r#"s = "steel""#], "s", &opts);
    assert_eq!(
        round_trip(&s),
        Plain::Text {
            value: "steel".into()
        }
    );
    assert_eq!(
        Value::from_plain(&s.to_plain()).unwrap().as_str(),
        Some("steel")
    );
}

#[test]
fn data_frame_with_units() {
    let opts = Options::default();
    let df = value_of(
        &[
            "rebar = ``#name\tdiameter\tarea\nunit\tin\tin²\n#3\t0.375\t0.11\n#4\t0.5\t0.2\n#5\t0.625\t0.31\n#6\t0.75\t0.44``",
        ],
        "rebar",
        &opts,
    );
    let p = round_trip(&df);
    let Plain::Frame { columns, row_names } = &p else {
        panic!("{p:?}")
    };
    assert!(*row_names);
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0].name, "name");
    assert_eq!(
        columns[0].data,
        ColumnData::Text {
            values: ["#3", "#4", "#5", "#6"]
                .map(|s| Some(s.to_string()))
                .to_vec()
        }
    );
    assert_eq!(
        columns[1].unit.as_ref().unwrap().name.as_deref(),
        Some("in")
    );
    assert_eq!(
        columns[2].data,
        ColumnData::Number {
            values: vec![Some(0.11), Some(0.2), Some(0.31), Some(0.44)]
        }
    );
    assert_eq!(columns[2].unit.as_ref().unwrap().dims[0], 2.0);
    let back = Value::from_plain(&p).unwrap();
    assert_eq!(back.kind(), Kind::DataFrame);
    assert_eq!(back.tex(), df.tex());

    // The frame works in later statements: a unit-aware lookup by row name and column.
    let mut vars = Vars::new();
    vars.insert("bars", back);
    assert_eq!(
        calculate("A = bars[\"#5\"].area = ?? in²", &mut vars, &opts),
        "A = bars[\"#5\"].area = 0.31 in²"
    );
    assert_eq!(
        calculate("A = bars[\"#5\"].area = ?? mm²", &mut vars, &opts),
        "A = bars[\"#5\"].area = 199.9996 mm²"
    );
}

#[test]
fn complex() {
    let z = value_of(&["z = 2 + j3"], "z", &Options::default());
    let p = round_trip(&z);
    assert_eq!(p, Plain::Complex { re: 2.0, im: 3.0 });
    assert_eq!(Value::from_plain(&p).unwrap().kind(), Kind::Complex);
}

#[test]
fn missing_and_unsupported() {
    assert_eq!(
        Value::from_plain(&Plain::Missing).unwrap().kind(),
        Kind::Missing
    );
    let f = value_of(
        &["function f(x)\n  return 2 x\nend"],
        "f",
        &Options::default(),
    );
    let p = f.to_plain();
    assert!(
        matches!(&p, Plain::Unsupported { kind, .. } if kind == "function"),
        "{p:?}"
    );
    assert!(Value::from_plain(&p).is_err());
    let d = value_of(&["d = '2026-11-20'"], "d", &Options::default());
    assert_eq!(
        d.to_plain(),
        Plain::Unsupported {
            kind: "date".into(),
            summary: "2026-11-20".into()
        }
    );
}

// ----------------------------------------------------------------------------------------
// Values brought back are used by later statements
// ----------------------------------------------------------------------------------------

fn eval_with(vars: &Vars, line: &str, opts: &Options) -> cruncha_eval::Outcome {
    let out = evaluate(&prepare(line, opts).unwrap(), vars, opts);
    assert!(out.error.is_none(), "{line}: {:?}", out.error);
    out
}

#[test]
fn use_a_calc_result_after_the_bridge() {
    for opts in [Options::default(), always()] {
        let mut vars = Vars::new();
        calculate("w = 1.5 kip/ft", &mut vars, &opts);
        calculate("L = 24 ft", &mut vars, &opts);
        let out = eval_with(&vars, "M = w L^2 / 8 = ?? kip·ft", &opts);
        let m = out.values[0].1.clone();
        let back = Value::from_plain(&m.to_plain()).unwrap();
        vars.insert("x", back);
        let out = eval_with(&vars, "y = 2 x = ?? kip·ft", &opts);
        assert_eq!(out.result, "y = 2 x = 216 kip·ft");
        assert!(out.tex.contains("108"), "{}", out.tex);
        assert!(out.tex.contains("\\textcolor{#0000ff}"), "{}", out.tex);
        // Unit-aware conversion works.
        let out = eval_with(&vars, "y = 2 x = ?? kN·m", &opts);
        let expect = 216.0 * 4448.2216152605 * 0.3048 / 1000.0;
        let got: f64 = out
            .result
            .split(" = ")
            .nth(2)
            .unwrap()
            .split(' ')
            .next()
            .unwrap()
            .replace(',', "")
            .parse()
            .unwrap();
        assert!((got - expect).abs() < 1e-6 * expect, "{}", out.result);
    }
}

#[test]
fn use_a_python_quantity() {
    // As a Python worker would send it: a float in metres.
    let p = Plain::Number {
        value: Num::Float { value: 2.5 },
        unit: Some(unit(Some("m"), 1.0, 0.0, LENGTH)),
    };
    let mut vars = Vars::new();
    vars.insert("L", Value::from_plain(&p).unwrap());
    let opts = always();
    let out = eval_with(&vars, "A = L^2 = ?? m²", &opts);
    assert_eq!(out.result, "A = L^2 = 6.25 m²");
    assert!(out.tex.contains("2.5"), "{}", out.tex);
    let out = eval_with(&vars, "x = 2 L = ?? ft", &opts);
    assert!(
        out.result.starts_with("x = 2 L = 16.404199475"),
        "{}",
        out.result
    );
}

#[test]
fn unparsable_unit_name_falls_back_to_si() {
    let factor = 4.4482216152605 * 0.3048;
    let mut vars = Vars::new();

    // `lbf*foot` parses in Hurmet (as `lbf·foot`) with these dimensions, so Hurmet's unit
    // is used and the value stays as written.
    let p = Plain::Number {
        value: Num::Float { value: 100.0 },
        unit: Some(unit(Some("lbf*foot"), factor, 0.0, MOMENT)),
    };
    let v = Value::from_plain(&p).unwrap();
    assert_eq!(v.unit_name(), Some("lbf·foot"));
    assert_eq!(v.as_f64(), Some(100.0));
    vars.insert("T0", v);
    for opts in [Options::default(), always()] {
        let out = eval_with(&vars, "M = 2 T0 = ?? lbf·ft", &opts);
        assert_eq!(out.result, "M = 2 T0 = 200 lbf·ft", "{:?}", opts.unit_aware);
    }

    // `lbf_ft` does not parse: SI fallback.
    assert!(cruncha_units::unit_from_name("lbf_ft", &Default::default()).is_err());
    let p = Plain::Number {
        value: Num::Float { value: 100.0 },
        unit: Some(unit(Some("lbf_ft"), factor, 0.0, MOMENT)),
    };
    let v = Value::from_plain(&p).unwrap();
    let si = v.unit_name().unwrap().to_string();
    assert_eq!(si, "kg·m²·s⁻²");
    assert!(cruncha_units::unit_from_name(&si, &Default::default()).is_ok());
    assert!((v.as_f64().unwrap() - 100.0 * factor).abs() < 1e-9);
    assert!((v.as_base_f64().unwrap() - 100.0 * factor).abs() < 1e-9);
    vars.insert("T", v);
    for opts in [Options::default(), always()] {
        let out = eval_with(&vars, "M = 2 T = ?? lbf·ft", &opts);
        assert_eq!(out.result, "M = 2 T = 200 lbf·ft", "{:?}", opts.unit_aware);
    }

    // A name that parses but with other dimensions also falls back.
    let p = Plain::Number {
        value: Num::Float { value: 3.0 },
        unit: Some(unit(Some("ft"), 4.4482216152605, 0.0, FORCE)),
    };
    let v = Value::from_plain(&p).unwrap();
    assert_eq!(v.unit_name(), Some("kg·m·s⁻²"));
    assert!((v.as_f64().unwrap() - 3.0 * 4.4482216152605).abs() < 1e-12);

    // A gauge unit without a usable name: (value + gauge) * factor, shown in kelvin.
    let p = Plain::Number {
        value: Num::Float { value: 20.0 },
        unit: Some(unit(Some("degC"), 1.0, 273.15, TEMP)),
    };
    let v = Value::from_plain(&p).unwrap();
    assert_eq!(v.unit_name(), Some("K"));
    assert!((v.as_f64().unwrap() - 293.15).abs() < 1e-12);

    // Dimensionless with a factor (percent as a ratio) becomes a plain number.
    let p = Plain::Number {
        value: Num::Float { value: 5.0 },
        unit: Some(unit(None, 0.01, 0.0, [0.0; 8])),
    };
    let v = Value::from_plain(&p).unwrap();
    assert_eq!(v.unit_name(), None);
    assert_eq!(v.as_f64(), Some(0.05));
}

#[test]
fn python_frame_and_array() {
    let frame = Plain::Frame {
        columns: vec![
            PlainColumn {
                name: "span".into(),
                unit: Some(unit(Some("ft"), 0.3048, 0.0, LENGTH)),
                data: ColumnData::Number {
                    values: vec![Some(10.0), Some(20.0), None],
                },
            },
            PlainColumn {
                name: "label".into(),
                unit: None,
                data: ColumnData::Text {
                    values: vec![Some("a".into()), None, Some("c".into())],
                },
            },
        ],
        row_names: false,
    };
    let v = Value::from_plain(&frame).unwrap();
    assert_eq!(v.to_plain(), frame);

    let arr = Plain::Array {
        shape: vec![2, 3],
        orientation: "matrix".into(),
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        unit: Some(unit(Some("kN"), 1000.0, 0.0, FORCE)),
    };
    let v = Value::from_plain(&arr).unwrap();
    assert_eq!(v.to_plain().clone(), {
        let Plain::Array { unit: Some(u), .. } = v.to_plain() else {
            panic!()
        };
        Plain::Array {
            shape: vec![2, 3],
            orientation: "matrix".into(),
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            unit: Some(u),
        }
    });
    let mut vars = Vars::new();
    vars.insert("P", v);
    let out = eval_with(&vars, "Q = 2 P = ?? kN", &always());
    assert!(out.result.contains("12"), "{}", out.result);
}

// ----------------------------------------------------------------------------------------
// Errors
// ----------------------------------------------------------------------------------------

#[test]
fn errors() {
    let bad_len = Plain::Array {
        shape: vec![2, 2],
        orientation: "matrix".into(),
        values: vec![1.0, 2.0, 3.0],
        unit: None,
    };
    assert!(Value::from_plain(&bad_len).is_err());
    let bad_orientation = Plain::Array {
        shape: vec![3],
        orientation: "matrix".into(),
        values: vec![1.0, 2.0, 3.0],
        unit: None,
    };
    assert!(Value::from_plain(&bad_orientation).is_err());
    let zero_den = Plain::Number {
        value: exact("1", "0"),
        unit: None,
    };
    assert!(Value::from_plain(&zero_den).is_err());
    let bad_num = Plain::Number {
        value: exact("one", "2"),
        unit: None,
    };
    assert!(Value::from_plain(&bad_num).is_err());
    let nan = Plain::Number {
        value: Num::Float { value: f64::NAN },
        unit: None,
    };
    assert!(Value::from_plain(&nan).is_err());
    let ragged = Plain::Frame {
        columns: vec![
            PlainColumn {
                name: "a".into(),
                unit: None,
                data: ColumnData::Number {
                    values: vec![Some(1.0)],
                },
            },
            PlainColumn {
                name: "b".into(),
                unit: None,
                data: ColumnData::Number { values: vec![] },
            },
        ],
        row_names: false,
    };
    assert!(Value::from_plain(&ragged).is_err());
    let unsupported = Plain::Unsupported {
        kind: "function".into(),
        summary: "f".into(),
    };
    assert!(Value::from_plain(&unsupported).is_err());
}

#[test]
fn json_shape() {
    let p = Plain::Number {
        value: exact("3", "2"),
        unit: Some(unit(Some("m"), 1.0, 0.0, LENGTH)),
    };
    let json = serde_json::to_value(&p).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "type": "number",
            "value": {"type": "exact", "num": "3", "den": "2"},
            "unit": {"name": "m", "factor": 1.0, "gauge": 0.0, "dims": [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}
        })
    );
}
