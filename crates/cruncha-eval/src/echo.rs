// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! The "blue echo" (Hurmet's `echo.js`, `plugValsIntoEcho`): the parser marks each variable in
//! the TeX template as `〖name〗`; here each mark is replaced by the TeX of the variable's value.

use crate::error::err;
use crate::eval::{Ctx, Scope, from_assignment, property_from_dot_accessor};
use crate::literal::tex;
use crate::value::{Operand, dt, is_matrix};
use regex::Regex;
use std::sync::LazyLock;

static OPEN_PAREN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:[(\[{|‖]|[^\\][,;:](?:\\:)?)$").unwrap());
static CLOSE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:[)|‖\]},;:]|\\right)").unwrap());

fn js_trim(s: &str) -> &str {
    cruncha_lang::hurmet::js_trim(s)
}

/// Replace each `〖name〗` in the TeX template with the value of `name`.
pub(crate) fn plug_vals_into_echo(
    template: &str,
    scope: &Scope,
    unit_aware: bool,
    spec: &str,
    ctx: &Ctx,
) -> Result<String, Operand> {
    let mut s = template.to_string();
    let mut guard = 0;
    while let Some(open) = s.find('〖') {
        guard += 1;
        if guard > 10_000 {
            break;
        }
        let Some(close_rel) = s[open..].find('〗') else {
            break;
        };
        let close = open + close_rel + '〗'.len_utf8();
        let raw = &s[open..close];
        let var_name: String = js_trim(&raw.replace(['〖', '〗', '(', ')'], "")).replace('\'', "′");
        let pos = open;
        let match_len = close - open;

        let (dtype, resultdisplay, is_df_or_map, name_for_df);
        if var_name.contains('.') {
            let names: Vec<&str> = var_name.split('.').collect();
            let parent = names[0];
            let Some(cell) = scope.get(parent) else {
                return Err(err("V_NAME", parent));
            };
            if cell.dtype == dt::DATAFRAME && names.len() == 2 {
                // Hurmet writes the names only (and drops a closing brace here).
                let display = format!(
                    "\\mathrm{{{}{{.}}\\mathrm{{{}}}",
                    cell.name.clone().unwrap_or_default(),
                    names[1]
                );
                return Ok(format!("{}{display}{}", &s[..pos], &s[pos + match_len..]));
            }
            let mut hvar = from_assignment(cell, unit_aware);
            let mut rd = String::new();
            for n in &names[1..] {
                let prop = js_trim(&n.replace('}', "").replace("\\mathrm{", "")).to_string();
                let index = Operand::string(prop.clone());
                hvar = property_from_dot_accessor(&hvar, &index, unit_aware, ctx);
                if hvar.is_error() {
                    return Err(err("V_NAME", &prop));
                }
                rd = crate::result::operand_tex(&hvar, spec, ctx);
            }
            dtype = hvar.dtype;
            resultdisplay = rd;
            is_df_or_map = false;
            name_for_df = String::new();
        } else if var_name == "T" && scope.get("T").is_none() {
            dtype = dt::RATIONAL;
            resultdisplay = "\\text{T}".to_string();
            is_df_or_map = false;
            name_for_df = String::new();
        } else if var_name == "e"
            && s.get(pos + 3..)
                .is_some_and(|t| js_trim(t).starts_with('^'))
        {
            s = format!("{}e{}", &s[..pos], &s[pos + match_len..]);
            continue;
        } else if var_name == "e" && scope.get("e").is_none() {
            // Euler's number (ADR-0022): echoed as e.
            s = format!("{}e{}", &s[..pos], &s[pos + match_len..]);
            continue;
        } else if var_name == "j" {
            s = format!("{}j{}", &s[..pos], &s[pos + match_len..]);
            continue;
        } else {
            let Some(cell) = scope.get(&var_name) else {
                return Err(err("V_NAME", &var_name));
            };
            if cell.error {
                return Err(err("NULL", &var_name));
            }
            let mut rd = cell.resultdisplay.clone();
            if rd.starts_with("\\colorbox{aqua}{$") && rd.len() >= 19 {
                rd = js_trim(&rd[17..rd.len() - 2]).to_string();
            }
            dtype = cell.dtype;
            resultdisplay = rd;
            is_df_or_map = cell.dtype == dt::DATAFRAME || cell.dtype & dt::MAP != 0;
            name_for_df = cell.name.clone().unwrap_or_default();
        }
        if resultdisplay.is_empty() {
            return Err(err(
                "NULL",
                if var_name.is_empty() { "?" } else { &var_name },
            ));
        }

        let mut needs_parens = !(is_matrix(dtype) || dtype & dt::DATAFRAME != 0);
        if unit_aware && dtype & dt::QUANTITY != 0 {
            needs_parens = true;
        }
        let mut is_parened = false;
        if pos > 0 {
            let p_str = js_trim(&s[..pos]);
            if OPEN_PAREN_RE.is_match(p_str) {
                // Hurmet slices at pos + varName.length + 2 (in UTF-16 units).
                let skip = cruncha_lang::hurmet::utf16_len(&var_name) + 2;
                let after = skip_utf16(&s[pos..], skip);
                let f_str = js_trim(after);
                is_parened = !f_str.is_empty() && CLOSE_RE.is_match(f_str);
            } else if resultdisplay.starts_with("\\begin{bmatrix}")
                || resultdisplay.starts_with("\\begin{pmatrix}")
            {
                is_parened = resultdisplay.ends_with("\\end{bmatrix}")
                    || resultdisplay.ends_with("\\end{pmatrix}");
            }
        }
        needs_parens = needs_parens && !is_parened;

        let display = if is_df_or_map || resultdisplay.starts_with('!') {
            format!("\\mathrm{{{}}}", tex(&name_for_df, ctx))
        } else {
            let mut d = resultdisplay.clone();
            if !unit_aware && let Some(pu) = d.rfind("{\\text{") {
                let t = js_trim(&d[..pu]).to_string();
                let t = t
                    .strip_suffix("\\;")
                    .map(|x| x.trim_end_matches(' '))
                    .unwrap_or(&t)
                    .to_string();
                d = js_trim(&t).to_string();
            }
            if needs_parens {
                if dtype > 256 {
                    format!("\\left({d}\\right)")
                } else {
                    format!("({d})")
                }
            } else {
                d
            }
        };
        s = format!("{}{display}{}", &s[..pos], &s[pos + match_len..]);
    }
    Ok(s)
}

fn skip_utf16(s: &str, n: usize) -> &str {
    let mut units = 0;
    for (i, c) in s.char_indices() {
        if units >= n {
            return &s[i..];
        }
        units += c.len_utf16();
    }
    ""
}
