// SPDX-License-Identifier: GPL-3.0-or-later

//! Rename names in a Rust cell's source (ADR-0024: cells pasted or copied with renames).
//!
//! The source is tokenized (`proc_macro2`, with span locations) and identifiers are replaced
//! in place, so formatting and comments stay. Not renamed: fields and methods (`x.M`), path
//! segments after `::`, and identifiers followed by `!` (macros) or `::`. Inside the
//! format-string macros the scanner reads (`println!`, `format!`, …), `{M}` and `{M:…}` in the
//! format string are renamed too. The app checks the result by scanning it.

use std::collections::BTreeMap;
use std::str::FromStr;

use proc_macro2::{Delimiter, LineColumn, Spacing, TokenStream, TokenTree};

use crate::scan::EXPR_MACROS;

/// `source` with each name in `names` (old → new) renamed, or why it couldn't be.
pub fn rename(source: &str, names: &BTreeMap<String, String>) -> Result<String, String> {
    let stream = TokenStream::from_str(source).map_err(|e| format!("not valid Rust: {e}"))?;
    let mut edits: Vec<(LineColumn, LineColumn, String)> = Vec::new();
    walk(stream, names, false, &mut edits);
    Ok(apply(source, edits))
}

fn walk(stream: TokenStream, names: &BTreeMap<String, String>, fmt_macro: bool, edits: &mut Vec<(LineColumn, LineColumn, String)>) {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    for (i, t) in tokens.iter().enumerate() {
        let prev = i.checked_sub(1).map(|j| &tokens[j]);
        let prev2 = i.checked_sub(2).map(|j| &tokens[j]);
        let next = tokens.get(i + 1);
        match t {
            TokenTree::Group(g) => {
                // The arguments of `println!(…)` and friends: `ident ! (group)`.
                let is_fmt = matches!((prev2, prev), (Some(TokenTree::Ident(m)), Some(TokenTree::Punct(p)))
                    if p.as_char() == '!' && EXPR_MACROS.contains(&m.to_string().as_str()))
                    && g.delimiter() != Delimiter::None;
                walk(g.stream(), names, is_fmt, edits);
            }
            TokenTree::Ident(id) => {
                let Some(new) = names.get(&id.to_string()) else { continue };
                let after_dot = matches!(prev, Some(TokenTree::Punct(p)) if p.as_char() == '.');
                let after_path = matches!((prev2, prev), (Some(TokenTree::Punct(a)), Some(TokenTree::Punct(b)))
                    if a.as_char() == ':' && a.spacing() == Spacing::Joint && b.as_char() == ':');
                let before_bang_or_path = matches!(next, Some(TokenTree::Punct(p)) if p.as_char() == '!')
                    || matches!(next, Some(TokenTree::Punct(p)) if p.as_char() == ':' && p.spacing() == Spacing::Joint);
                if after_dot || after_path || before_bang_or_path {
                    continue;
                }
                let s = id.span();
                edits.push((s.start(), s.end(), new.clone()));
            }
            TokenTree::Literal(l) if fmt_macro => {
                let text = l.to_string();
                if !text.starts_with('"') {
                    continue;
                }
                let mut out = text.clone();
                for (old, new) in names {
                    out = out.replace(&format!("{{{old}}}"), &format!("{{{new}}}"));
                    out = out.replace(&format!("{{{old}:"), &format!("{{{new}:"));
                }
                if out != text {
                    let s = l.span();
                    edits.push((s.start(), s.end(), out));
                }
            }
            _ => {}
        }
    }
}

/// Apply replacements given by (line, column) spans: lines are 1-based, columns in chars.
fn apply(source: &str, mut edits: Vec<(LineColumn, LineColumn, String)>) -> String {
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(source.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let offset = |lc: LineColumn| -> usize {
        let start = line_starts.get(lc.line - 1).copied().unwrap_or(source.len());
        source[start..]
            .char_indices()
            .nth(lc.column)
            .map_or(source.len(), |(i, _)| start + i)
    };
    edits.sort_by_key(|(s, _, _)| std::cmp::Reverse((s.line, s.column)));
    let mut out = source.to_owned();
    for (s, e, text) in edits {
        let (a, b) = (offset(s), offset(e));
        out.replace_range(a..b, &text);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_fields_paths_macros_and_format_strings() {
        let src = "let M = w * L * L / 8.0; // M here\nlet s = format!(\"M is {M:.2} and {M}\", );\nlet f = p.M + M;\nlet pi = std::f64::consts::PI;\nprintln!(\"{}\", M);\n";
        let names = BTreeMap::from([("M".to_string(), "M_2".to_string()), ("PI".to_string(), "PI_2".to_string())]);
        let out = rename(src, &names).unwrap();
        assert_eq!(
            out,
            "let M_2 = w * L * L / 8.0; // M here\nlet s = format!(\"M is {M_2:.2} and {M_2}\", );\nlet f = p.M + M_2;\nlet pi = std::f64::consts::PI;\nprintln!(\"{}\", M_2);\n"
        );
        let scanned = crate::scan::scan(&out);
        assert!(scanned.defines.contains(&"M_2".to_string()), "{scanned:?}");
        assert!(!scanned.reads.contains(&"M".to_string()));
    }

    #[test]
    fn unicode_columns_are_counted_in_characters() {
        let out = rename("let σ = 2.0; let x = σ * M;", &BTreeMap::from([("M".into(), "M_2".into())])).unwrap();
        assert_eq!(out, "let σ = 2.0; let x = σ * M_2;");
    }
}
