// SPDX-License-Identifier: GPL-3.0-or-later
//! The `.cruncha.md` format (ADR-0014 §1), validation (§2, §6) and the test runner.

use cruncha_templates::{Cell, parse, run_tests, validate};

const BEAM: &str = r#"+++
cruncha-template = 1
id = "test.beam"
version = "1.0.0"
title = "Midspan moment"
inputs = ["w", "L"]
outputs = ["M"]

[[tests]]
name = "w L² / 8"
inputs = { w = "2 kip/ft", L = "20 ft" }
expect = { M = "100 kip·ft" }
+++

A simply supported beam under **uniform** load.

```calc {description="Uniform load"}
w = 1 kip/ft_
```

```calc {description="Span"}
L = 10 ft_
```

```calc {description="Midspan moment"}
M = w L^2 / 8 = ?? kip·ft
```

```python
x = 1
```
"#;

#[test]
fn a_template_parses_into_manifest_and_cells() {
    let t = parse(BEAM).unwrap();
    assert_eq!(t.manifest.id, "test.beam");
    assert_eq!(t.manifest.tests[0].expect["M"], "100 kip·ft");
    assert!(matches!(&t.cells[0], Cell::Prose { source } if source.contains("\"strong\"")));
    assert_eq!(t.cells[1], Cell::Calc { source: "w = 1 kip/ft_".into(), description: Some("Uniform load".into()) });
    assert!(matches!(&t.cells[4], Cell::Code { lang, .. } if lang == "python"));
}

fn calc_only(src: &str) -> String {
    src.split("```python").next().unwrap().to_owned()
}

#[test]
fn worked_examples_pass_and_fail() {
    let t = parse(&calc_only(BEAM)).unwrap();
    assert!(validate(&t).is_empty(), "{:?}", validate(&t));
    let r = run_tests(&t);
    assert!(r[0].passed, "{:?}", r[0].lines);
    // A wrong expectation fails, with the value it got in the expected unit.
    let wrong = parse(&calc_only(BEAM).replace("100 kip·ft", "90 kip·ft")).unwrap();
    let r = run_tests(&wrong);
    assert!(!r[0].passed);
    assert!(r[0].lines[0].starts_with("FAIL M: expected 90 kip·ft, got 100"), "{:?}", r[0].lines);
    // Wrong dimensions fail too.
    let dims = parse(&calc_only(BEAM).replace("100 kip·ft", "100 kip")).unwrap();
    assert!(run_tests(&dims)[0].lines[0].contains("different dimensions"));
    // With a code cell the runner leaves it to the app.
    assert!(run_tests(&parse(BEAM).unwrap())[0].skipped.is_some());
}

#[test]
fn validation_explains_what_is_wrong() {
    let bad = calc_only(BEAM)
        .replace("id = \"test.beam\"", "id = \"test beam\"")
        .replace("version = \"1.0.0\"", "version = \"1.0\"")
        .replace("w = 1 kip/ft_", "w = 1 kip/ft")
        .replace("outputs = [\"M\"]", "outputs = [\"M\", \"V\"]")
        .replace("inputs = { w = \"2 kip/ft\", L = \"20 ft\" }", "inputs = { w = \"2 kip/ft\", q = \"1\" }");
    let problems = validate(&parse(&bad).unwrap()).join("\n");
    for want in ["id \"test beam\"", "version \"1.0\"", "input w is defined without a placeholder", "output V isn't defined", "q isn't one of the template's inputs"] {
        assert!(problems.contains(want), "missing {want:?} in:\n{problems}");
    }
}

#[test]
fn front_matter_errors_say_where() {
    assert!(parse("no front matter").unwrap_err().contains("+++"));
    assert!(parse("+++\nid = 1\n").unwrap_err().contains("ends with a +++"));
    assert!(parse("+++\ncruncha-template = 1\nid = \"a\"\nversion = \"1.0.0\"\ntitle = \"t\"\ncolour = \"red\"\n+++\n").unwrap_err().contains("colour"));
}

#[test]
fn a_written_snippet_reads_back_with_its_inputs() {
    let cells = vec![
        Cell::Prose { source: r#"{"type":"paragraph","content":[{"type":"text","text":"Load \"case\" A"}]}"#.into() },
        Cell::Calc { source: "w = 2 kN/m_".into(), description: Some(r#"Uniform "dead" load \ per `metre`"#.into()) },
        Cell::Calc { source: "L = 6 m_".into(), description: None },
        Cell::Calc { source: "M = w L^2 / 8 = ?? kN·m".into(), description: None },
        Cell::Code { lang: "python".into(), source: "s = \"```\"\nprint(s)".into(), description: None },
    ];
    assert_eq!(cruncha_templates::placeholder_inputs(&cells), ["w", "L"]);
    let manifest = parse("+++\ncruncha-template = 1\nid = \"user.load\"\nversion = \"1.0.0\"\ntitle = \"Load\"\n+++\n").unwrap().manifest;
    let t = cruncha_templates::Template { manifest, cells };
    let text = cruncha_templates::write(&t).unwrap();
    assert!(!text.contains("inputs ="), "empty lists aren't written:\n{text}");
    assert_eq!(parse(&text).unwrap(), t, "{text}");
}
