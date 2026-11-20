// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Data frame literals, ``` ``a\tb\n1\t2`` ```: only `DataFrame.quickDisplay` (dataframe.js),
//! the display Hurmet uses while typing. The full `DataFrame.display` needs the data frame
//! model and result formatting, which belong to the evaluator.

use crate::number::NUMBER_PATTERN;
use crate::utils::{add_text_escapes, js_trim, tabless_trim};
use regex::Regex;
use std::sync::LazyLock;

static NUMBER_CELL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^(?:=|{}$)", &NUMBER_PATTERN[1..])).unwrap());
static SEP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" *\t *").unwrap());

fn drop_last_10(s: &mut String) {
    for _ in 0..10 {
        s.pop();
    }
}

pub fn quick_display(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let mut s = js_trim(input);
    let array_format;
    if let Some(rest) = s.strip_prefix('#') {
        s = js_trim(rest);
        array_format = "l|cccccccccccccccccccccccc";
    } else {
        array_format = "c";
    }
    let s = add_text_escapes(s);
    let lines: Vec<&str> = s
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let mut tex;
    if lines.len() < 3 {
        tex = String::from("\\begin{matrix}\\text{");
        for line in &lines {
            tex.push_str(&SEP_RE.replace_all(tabless_trim(line), "} & \\text{"));
            tex.push_str("} \\\\ \\text{");
        }
        drop_last_10(&mut tex);
        tex.push_str("\\end{matrix}");
    } else {
        tex = format!("\\begin{{array}}{{{array_format}}}\\text{{");
        let cells: Vec<Vec<&str>> = lines
            .iter()
            .map(|l| SEP_RE.split(tabless_trim(l)).collect())
            .collect();
        let got_answer = cells[1].iter().any(|c| NUMBER_CELL_RE.is_match(c));
        let got_units = !got_answer && cells[2].iter().any(|c| NUMBER_CELL_RE.is_match(c));
        for (i, line) in lines.iter().enumerate() {
            tex.push_str(&SEP_RE.replace_all(tabless_trim(line), "} & \\text{"));
            tex.push_str(if (got_units && i == 1) || (!got_units && i == 0) {
                "} \\\\ \\hline \\text{"
            } else {
                "} \\\\ \\text{"
            });
        }
        drop_last_10(&mut tex);
        tex.push_str("\\end{array}");
    }
    tex.replace('·', "$·$")
}
