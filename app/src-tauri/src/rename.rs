// SPDX-License-Identifier: GPL-3.0-or-later

//! Renaming names in cells (ADR-0024: whole cells pasted, or copied by an agent, into a
//! document that already defines some of their names; ADR-0014's insert later).
//!
//! Each language renames with its own tools: calcs lexically here (quoted units and strings
//! skipped), Python with Python's tokenizer in the worker, Rust with `proc_macro2` tokens
//! (`cruncha_kernel_rs::rename`). Every result is checked: scanned again, its definitions and
//! reads must be the old ones with the names replaced. If not, the rename is refused (with the
//! cell's first line) rather than leaving a cell that means something else.

use std::collections::{BTreeMap, BTreeSet};

use cruncha_store::CellKind;

/// A calc source with these names renamed. Names inside quotes ('unit', "text", `frame`) are
/// left alone, as are a property after `.` and a unit written right after a number
/// (`L = 20 ft`) unless the cell reads that name (`y = 2 M`).
pub fn rename_calc(source: &str, names: &BTreeMap<String, String>, reads: &BTreeSet<String>) -> String {
    let chars: Vec<char> = source.chars().collect();
    let is_start = |c: char| c.is_alphabetic();
    let is_part = |c: char| c.is_alphanumeric() || c == '_' || c == '′' || ('\u{300}'..='\u{36f}').contains(&c);
    let mut out = String::with_capacity(source.len());
    let mut quote: Option<char> = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            out.push(c);
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if c == '\'' || c == '"' || c == '`' {
            quote = Some(c);
            out.push(c);
            i += 1;
            continue;
        }
        if is_start(c) {
            let start = i;
            while i < chars.len() && (i == start || is_part(chars[i])) {
                i += 1;
            }
            // A parenthesized subscript is part of the name: `X_(CO2,dry)`.
            if chars[i - 1] == '_' && chars.get(i) == Some(&'(')
                && let Some(close) = chars[i..].iter().position(|&c| c == ')')
            {
                i += close + 1;
            }
            let word: String = chars[start..i].iter().collect();
            let before = chars[..start].iter().rev().find(|c| !c.is_whitespace()).copied();
            let after_number = before.is_some_and(|b| b.is_ascii_digit());
            let after_dot = start > 0 && chars[start - 1] == '.';
            // A word glued to digits before it is part of a number-and-unit (5m), not a name.
            let glued = start > 0 && chars[start - 1].is_ascii_digit();
            match names.get(&word) {
                Some(new) if !after_dot && !glued && (!after_number || reads.contains(&word)) => out.push_str(new),
                _ => out.push_str(&word),
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// The names a cell defines and reads, as the rename check compares them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellNames {
    pub defines: BTreeSet<String>,
    pub reads: BTreeSet<String>,
}

impl CellNames {
    pub fn new(defines: &[String], reads: &[String]) -> CellNames {
        CellNames { defines: defines.iter().cloned().collect(), reads: reads.iter().cloned().collect() }
    }

    fn renamed(&self, names: &BTreeMap<String, String>) -> CellNames {
        let map = |s: &BTreeSet<String>| s.iter().map(|n| names.get(n).cloned().unwrap_or_else(|| n.clone())).collect();
        CellNames { defines: map(&self.defines), reads: map(&self.reads) }
    }
}

/// Check a rename: `after` must be `before` with `names` applied.
pub fn check(kind: CellKind, source: &str, before: &CellNames, after: &CellNames, names: &BTreeMap<String, String>) -> Result<(), String> {
    let first = source.lines().next().unwrap_or_default();
    // A new name the cell already uses (and keeps) would merge two names into one.
    let kept: BTreeSet<&String> = before.defines.iter().chain(&before.reads).filter(|n| !names.contains_key(*n)).collect();
    if let Some(new) = names.values().find(|n| kept.contains(n)) {
        return Err(format!(
            "can't rename to {new} in the {} cell “{first}”: it already uses {new}",
            kind.as_str()
        ));
    }
    if &before.renamed(names) == after {
        return Ok(());
    }
    Err(format!(
        "couldn't rename safely in the {} cell “{}”: it would define {:?} and read {:?}; rename it by hand",
        kind.as_str(),
        first,
        after.defines,
        after.reads
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }
    fn set(xs: &[&str]) -> BTreeSet<String> {
        xs.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn calc_rename_skips_quotes_units_and_properties() {
        let names = map(&[("M", "M_2"), ("m", "m_2"), ("area", "area_2")]);
        assert_eq!(
            rename_calc("M = w L^2 / 8 = ?? kip·ft", &names, &set(&["w", "L"])),
            "M_2 = w L^2 / 8 = ?? kip·ft"
        );
        assert_eq!(rename_calc("y = 2 M + 'm' = ?", &names, &set(&["M"])), "y = 2 M_2 + 'm' = ?");
        // A literal's unit after its number stays a unit.
        assert_eq!(rename_calc("L = 20 m", &names, &set(&[])), "L = 20 m");
        assert_eq!(rename_calc("L = 20m", &names, &set(&[])), "L = 20m");
        assert_eq!(rename_calc("A = beam.area = ?", &names, &set(&["beam"])), "A = beam.area = ?");
        assert_eq!(rename_calc("s = \"M\" & string(M) = ?", &names, &set(&["M"])), "s = \"M\" & string(M_2) = ?");
        assert_eq!(rename_calc("Mσ = M + σ = ?", &names, &set(&["M", "σ"])), "Mσ = M_2 + σ = ?");
        let sub = map(&[("X_(CO2,dry)", "X2_(CO2,dry)"), ("X", "X2")]);
        assert_eq!(rename_calc("y = X_(CO2,dry) + X = ?", &sub, &set(&["X_(CO2,dry)", "X"])), "y = X2_(CO2,dry) + X2 = ?");
    }

    #[test]
    fn the_check_compares_names_after_renaming() {
        let names = map(&[("M", "M_2")]);
        let before = CellNames::new(&["M".into()], &["w".into()]);
        assert!(check(CellKind::Calc, "M = w", &before, &CellNames::new(&["M_2".into()], &["w".into()]), &names).is_ok());
        let wrong = CellNames::new(&["M".into()], &["w".into()]);
        assert!(check(CellKind::Calc, "M = w", &before, &wrong, &names).unwrap_err().contains("rename it by hand"));
    }
}
