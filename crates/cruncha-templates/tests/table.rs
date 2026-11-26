// SPDX-License-Identifier: GPL-3.0-or-later
//! Table templates (ADR-0027): the `[table]` section, validation, and the cells a row inserts.

use std::path::Path;

use cruncha_eval::{Num, Options, Plain, UnitAware, Vars};
use cruncha_templates::{Cell, parse, table_cells, validate, write};

const SCREWS: &str = r#"+++
cruncha-template = 1
id = "test.screws"
version = "1.0.0"
title = "Screws"

[table]
unit = "mm"
part = "Socket head cap screw"
source = "ISO 4762 Table 1"
rows = [
  ["M6", 6, 10.22, 9.78, "8.8"],
  ["M8", 8, 13.27, "", "12.9"],
]

[[table.columns]]
symbol = "d"
name = "Nominal diameter"
unit = "mm"
var = "d"

[[table.columns]]
symbol = "dk"
name = "Head diameter"
limit = "max"
unit = "mm"
var = "d_(k,max)"

[[table.columns]]
symbol = "dk"
name = "Head diameter"
limit = "min"
unit = "mm"
var = "d_(k,min)"

[[table.columns]]
symbol = "Class"
name = "Property class"
kind = "text"
var = "class"

[table.row_notes]
"M8" = "dk min not given"
+++

```calc {description="Head to shank"}
r = d_(k,max) / d = ??
```
"#;

fn calcs(cells: &[Cell]) -> Vec<(String, Option<String>)> {
    cells
        .iter()
        .filter_map(|c| match c {
            Cell::Calc { source, description } => Some((source.clone(), description.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn a_row_inserts_a_line_its_values_and_the_body() {
    let t = parse(SCREWS).unwrap();
    assert_eq!(validate(&t), Vec::<String>::new());
    let cells = table_cells(&t, "M8", None).unwrap();
    let Cell::Prose { source } = &cells[0] else { panic!("a prose line first") };
    assert!(source.contains("Socket head cap screw M8 · ISO 4762 Table 1. Note: dk min not given."), "{source}");
    assert_eq!(
        calcs(&cells),
        vec![
            ("d = 8 mm".into(), Some("Nominal diameter".into())),
            ("d_(k,max) = 13.27 mm".into(), Some("Head diameter, max".into())),
            // M8's dk min is empty: nothing inserted.
            ("class = \"12.9\"".into(), Some("Property class".into())),
            ("r = d_(k,max) / d = ??".into(), Some("Head to shank".into())),
        ]
    );
    // Only the columns asked for (the body still comes).
    let some = table_cells(&t, "M6", Some(&["d".into(), "d_(k,max)".into()])).unwrap();
    assert_eq!(calcs(&some).len(), 3);
    assert!(table_cells(&t, "M5", None).unwrap_err().contains("has no size \"M5\""));
    assert!(table_cells(&t, "M6", Some(&["x".into()])).unwrap_err().contains("no column x"));
}

#[test]
fn a_table_writes_back_as_itself() {
    let t = parse(SCREWS).unwrap();
    assert_eq!(parse(&write(&t).unwrap()).unwrap(), t);
}

#[test]
fn validation_finds_a_bad_table() {
    let bad = SCREWS
        .replace(r#"["M8", 8, 13.27, "", "12.9"]"#, r#"["M8", 8, 13.27, 14, 12.9], ["M6", 6, 1, 1, "x"]"#)
        .replace(r#"var = "d_(k,min)""#, r#"var = "d""#);
    let problems = validate(&parse(&bad).unwrap()).join("\n");
    for want in [
        "var d is used twice",
        "row \"M6\" is listed twice",
        "row \"M8\", column \"Class\": 12.9 isn't text",
        "row \"M8\": dk max 13.27 is less than its min 14",
    ] {
        assert!(problems.contains(want), "{want} not in:\n{problems}");
    }
}

/// The SI value of `name` after evaluating `cells`' calcs in order.
fn evaluate(cells: &[Cell]) -> Vars {
    let o = Options { unit_aware: UnitAware::Always, ..Options::default() };
    let mut vars = Vars::new();
    for (src, _) in calcs(cells) {
        let stmt = cruncha_eval::prepare(&src, &o).unwrap();
        let out = cruncha_eval::evaluate(&stmt, &vars, &o);
        assert!(out.error.is_none(), "{src}: {:?}", out.error);
        for (n, v) in out.values {
            vars.insert(n, v);
        }
    }
    vars
}

fn si(vars: &Vars, name: &str) -> f64 {
    match vars.get(name).unwrap().to_plain() {
        Plain::Number { value, unit } => {
            let x = match value {
                Num::Float { value } => value,
                Num::Exact { num, den } => num.parse::<f64>().unwrap() / den.parse::<f64>().unwrap(),
            };
            unit.map_or(x, |u| x * u.factor)
        }
        p => panic!("{name}: {p:?}"),
    }
}

#[test]
fn the_built_in_pipe_table_gives_a_2_in_schedule_40_section() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/mechanical/tables/pipe/mechanical.tables.pipe.steel.cruncha.md");
    let t = parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    let vars = evaluate(&table_cells(&t, "2 × 0.154 in (40 / STD)", None).unwrap());
    let inch = 0.0254;
    // Machinery's / B36.10M: ID 2.067 in, metal area 1.075 in², I 0.666 in⁴, S 0.561 in³.
    assert!((si(&vars, "d") / inch - 2.067).abs() < 1e-9);
    assert!((si(&vars, "A") / inch.powi(2) - 1.075).abs() < 0.001);
    assert!((si(&vars, "I") / inch.powi(4) - 0.666).abs() < 0.001);
    assert!((si(&vars, "S") / inch.powi(3) - 0.561).abs() < 0.001);
}

fn built_in(rel: &str) -> cruncha_templates::Template {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates").join(rel);
    parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn the_shape_tables_insert_values_as_their_sources_print_them() {
    let inch: f64 = 0.0254;
    // AISC Shapes Database v16.0: W14X22.
    let w = built_in("structural/shapes/aisc/structural.tables.aisc.w.cruncha.md");
    let vars = evaluate(&table_cells(&w, "W14X22", None).unwrap());
    assert!((si(&vars, "A") / inch.powi(2) - 6.49).abs() < 1e-9);
    assert!((si(&vars, "I_x") / inch.powi(4) - 199.0).abs() < 1e-9);
    assert!((si(&vars, "W") - 22.0 * 4.4482216152605 / 0.3048).abs() < 1e-6, "W in lbf/ft");
    // ArcelorMittal: IPE 200's Iw is tabulated as 12.98 × 10³ cm⁶.
    let ipe = built_in("structural/shapes/en/structural.tables.en.ipe.cruncha.md");
    let vars = evaluate(&table_cells(&ipe, "IPE 200", None).unwrap());
    assert!((si(&vars, "I_w") / 0.01f64.powi(6) - 12980.0).abs() < 1e-6);
    assert!((si(&vars, "I_y") / 0.01f64.powi(4) - 1943.0).abs() < 1e-9);
}
