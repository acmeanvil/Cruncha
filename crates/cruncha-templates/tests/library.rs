// SPDX-License-Identifier: GPL-3.0-or-later
//! Every built-in template (`templates/**/*.cruncha.md`) parses, validates, and passes its
//! worked examples (ADR-0014 §6). Code cells run on the real Python worker when the
//! environment is set up (see `support/py.rs`); without it, those templates are skipped.

use std::path::{Path, PathBuf};

mod support {
    pub mod py;
}

fn templates(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            templates(&p, out);
        } else if p.to_string_lossy().ends_with(".cruncha.md") {
            out.push(p);
        }
    }
}

#[test]
fn every_built_in_template_is_valid_and_passes_its_examples() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    let mut files = Vec::new();
    templates(&root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "no templates under {}", root.display());
    let mut failures = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut py = support::py::PyRunner::start();
    if py.is_none() {
        eprintln!("no Python environment: templates with code cells are skipped");
    }
    for f in &files {
        let name = f.strip_prefix(&root).unwrap().display().to_string();
        let t = match cruncha_templates::parse(&std::fs::read_to_string(f).unwrap()) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("{name}: {e}"));
                continue;
            }
        };
        if !name.ends_with(&format!("{}.cruncha.md", t.manifest.id)) {
            failures.push(format!("{name}: the file is named after its id, {}.cruncha.md", t.manifest.id));
        }
        if !ids.insert(t.manifest.id.clone()) {
            failures.push(format!("{name}: id {} is used twice", t.manifest.id));
        }
        for p in cruncha_templates::validate(&t) {
            failures.push(format!("{name}: {p}"));
        }
        if t.manifest.tests.is_empty() {
            failures.push(format!("{name}: a built-in template carries at least one worked example"));
        }
        let runner = py.as_mut().map(|r| r as &mut dyn cruncha_templates::CodeRunner);
        for r in cruncha_templates::run_tests_with(&t, runner) {
            if r.skipped.is_none() && !r.passed {
                failures.push(format!("{name}: test {:?}:\n    {}", r.name, r.lines.join("\n    ")));
            }
            println!("{name} · {} · {}", r.name, if r.passed { "passed" } else if r.skipped.is_some() { "skipped" } else { "FAILED" });
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn every_built_in_template_writes_back_as_itself() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    let entries = cruncha_templates::load_dir(&root);
    assert!(entries.len() >= 13, "load_dir found {}", entries.len());
    for e in entries {
        let t = e.template.unwrap_or_else(|err| panic!("{}: {err}", e.path));
        let text = cruncha_templates::write(&t).unwrap();
        let back = cruncha_templates::parse(&text).unwrap_or_else(|err| panic!("{}: {err}\n{text}", e.path));
        assert_eq!(back, t, "{} doesn't write back as itself:\n{text}", e.path);
    }
}
