// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! User-defined functions and modules (Hurmet's `module.js`: `scanModule`, `scanFunction`).
//!
//! A function is a block `function name(params) … end` of statements, one per line, with
//! `if`/`elseif`/`else`/`while`/`for`/`break`/`return`/`throw`/`print` and `end`. Each line is
//! compiled to RPN once, when the function is defined.

use crate::dataframe::{is_valid_identifier, tsv_trim};
use crate::error::{err, err_msg};
use crate::eval::Ctx;
use crate::literal::{lang_opts, value_from_literal};
use crate::value::{FnStatement, Operand, Parameter, Udf, V, Value, dt};
use cruncha_lang::hurmet::check_for_numeric_subscript;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static KEYWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(if|elseif|else|return|throw|while|for|break|print|end)(\x{2002}|(?-u:\b))")
        .unwrap()
});
static ONE_LINER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^( *)if ([^\n`]+) +(return|throw|print|break)(?-u:\b)([^\n]+)?(?: end)? *\n")
        .unwrap()
});
pub(crate) static FUNCTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^function {}{}′*\(",
        cruncha_lang::hurmet::IDENTIFIER_START,
        cruncha_lang::hurmet::IDENTIFIER_BODY
    ))
    .unwrap()
});
pub(crate) static MODULE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^module ([A-Za-z][A-Za-z0-9]*)").unwrap());
pub(crate) static DRAW_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^draw\(").unwrap());
static LEX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""(?:[^"]|"")*"|``.*|`[^`]*`|'[^']*'|#|[^"`'#]+"#).unwrap());
static CONTINUES_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[{(\[,;]$").unwrap());
static NEXT_CONTINUES_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*[+\-)\]}]").unwrap());

fn js_trim(s: &str) -> &str {
    cruncha_lang::hurmet::js_trim(s)
}

fn test_for_statement(s: &str) -> bool {
    let Some(pos) = s.find('=') else { return false };
    let lead = js_trim(s[..pos].trim_start_matches([' ', '\t']));
    if is_valid_identifier(lead) {
        return true;
    }
    if !lead.contains(',') {
        return false;
    }
    lead.split(',').all(|e| is_valid_identifier(js_trim(e)))
}

fn strip_comment(s: &str) -> &str {
    for m in LEX_RE.find_iter(s) {
        if m.as_str() == "#" {
            return js_trim(&s[..m.start()]);
        }
    }
    js_trim(s)
}

fn compile_rpn(expression: &str, ctx: &Ctx) -> String {
    match cruncha_lang::parse_calc(expression, &lang_opts(ctx)) {
        Ok(c) => c.rpn,
        // Hurmet would throw; an RPN that names an unknown variable reports an error later.
        Err(_) => "¿undefined\u{a0}¿undefined".into(),
    }
}

fn handle_tsv(mut expression: String, lines: &[&str], start: usize) -> (String, usize) {
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        let line = tsv_trim(line);
        if line.is_empty() {
            continue;
        }
        expression.push('\n');
        expression.push_str(line);
        if line.ends_with("``") {
            return (expression, i);
        }
    }
    (expression, lines.len().saturating_sub(1))
}

/// `scanFunction(lines, formats, startLineNum)`: (function, index of its last line).
fn scan_function(lines: &[&str], start: usize, ctx: &Ctx) -> Result<(Udf, usize), Operand> {
    let line1 = strip_comment(lines[start]);
    let pos_paren = line1.find('(').unwrap_or(line1.len());
    let pos_fn = line1.find("function").unwrap_or(0);
    let name = js_trim(line1.get(pos_fn + 8..pos_paren).unwrap_or("")).to_string();
    let param_str = js_trim(
        line1
            .get(pos_paren + 1..line1.len().saturating_sub(1))
            .unwrap_or(""),
    );
    let mut parameters = Vec::new();
    if !param_str.is_empty() {
        static SPLIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" *[,;] *").unwrap());
        static EQ_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" *= *").unwrap());
        for p in SPLIT_RE.split(param_str) {
            let parts: Vec<&str> = EQ_RE.split(p).collect();
            // Named as the body reads it (`u_1` is `u₁`), as locals are (ADR-0025).
            let pname = check_for_numeric_subscript(parts[0]);
            let default = match parts.get(1).filter(|s| !s.is_empty()) {
                Some(lit) => {
                    let (value, unit, dtype, display) = value_from_literal(lit, "", ctx);
                    Value {
                        name: Some(pname.clone()),
                        value,
                        unit,
                        dtype,
                        resultdisplay: display,
                        ..Default::default()
                    }
                }
                None => Value {
                    name: Some(pname.clone()),
                    value: V::Null,
                    ..Default::default()
                },
            };
            parameters.push(Parameter {
                name: pname,
                default: Some(default),
            });
        }
    }
    let mut udf = Udf {
        name: name.clone(),
        parameters,
        statements: Vec::new(),
    };
    let mut stack_of_ctrls: Vec<usize> = Vec::new();
    let mut prev_continued = false;
    let mut prev_line = String::new();
    let mut i = start + 1;
    while i < lines.len() {
        let mut line = strip_comment(lines[i]).to_string();
        if line.is_empty() {
            i += 1;
            continue;
        }
        if prev_continued {
            let last = prev_line.chars().last();
            line = if last == Some(';') && line.starts_with(['}', ')', ']']) {
                format!("{}{line}", js_trim(&prev_line[..prev_line.len() - 1]))
            } else if last == Some(';') || last == Some(',') {
                format!("{prev_line} {line}")
            } else {
                format!("{prev_line}{line}")
            };
        }
        if CONTINUES_RE.is_match(&line)
            || (i + 1 < lines.len() && NEXT_CONTINUES_RE.is_match(lines[i + 1]))
        {
            prev_continued = true;
            prev_line = line;
            i += 1;
            continue;
        }
        let mut is_statement = false;
        let (stype, names, mut expression);
        if let Some(k) = KEYWORD_RE.captures(&line) {
            let kw = k[1].to_string();
            expression = js_trim(&line[kw.len()..]).to_string();
            if expression.starts_with("``") {
                let (e, last) = handle_tsv(expression, lines, i);
                expression = e;
                i = last;
            }
            stype = kw;
            names = Vec::new();
        } else if test_for_statement(&line) {
            let pos_eq = line.find('=').unwrap_or(0);
            // Hurmet slices to `posEq - 1`, so `x=1` would lose the name's last character.
            let name_part = js_trim(line.get(..pos_eq.saturating_sub(1)).unwrap_or("")).to_string();
            expression = js_trim(&line[pos_eq + 1..]).to_string();
            if expression.starts_with("``") {
                let (e, last) = handle_tsv(expression, lines, i);
                expression = e;
                i = last;
            }
            is_statement = true;
            stype = "statement".to_string();
            // A local is named as the calc reads it (`k_1` is `k₁`), as top-level names are
            // (ADR-0025 row 4).
            names = if name_part.contains([',', ';']) {
                name_part
                    .split([',', ';'])
                    .map(|e| check_for_numeric_subscript(js_trim(e)))
                    .collect()
            } else {
                vec![check_for_numeric_subscript(&name_part)]
            };
        } else {
            return Err(err("FUNC_LINE", &format!("{name}, line {}\n{line}", i + 1)));
        }
        let mut rpn = String::new();
        if !expression.is_empty() {
            rpn = compile_rpn(&expression, ctx);
            if stype == "for" {
                rpn = rpn.replacen("\u{a0}in\u{a0}", "\u{a0}", 1);
                if let Some(r) = rpn.strip_suffix("\u{a0}in") {
                    rpn = r.to_string();
                }
            }
        }
        let _ = is_statement;
        udf.statements.push(FnStatement {
            name: names,
            rpn,
            stype: stype.clone(),
            end_of_block: 0,
        });
        let idx = udf.statements.len() - 1;
        if stype == "if" || stype == "while" || stype == "for" {
            stack_of_ctrls.push(idx);
        } else if stype == "end" {
            match stack_of_ctrls.pop() {
                None => return Ok((udf, i)),
                Some(c) => udf.statements[c].end_of_block = idx,
            }
        }
        prev_continued = false;
        prev_line.clear();
        i += 1;
    }
    Err(err("END_MISS", &name))
}

/// `scanModule(str, formats)`: the functions and assignments in a block of code.
pub(crate) fn scan_module(input: &str, ctx: &Ctx) -> Result<HashMap<String, Value>, Operand> {
    let expanded = ONE_LINER_RE.replace_all(
        input,
        "${1}if\u{2002}${2}\n${1}    ${3}\u{2002}${4}\n${1}end\n",
    );
    let lines: Vec<&str> = expanded
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let mut module = HashMap::new();
    let mut i = 0;
    while i < lines.len() {
        let line = strip_comment(lines[i]);
        if line.is_empty() {
            i += 1;
            continue;
        }
        if FUNCTION_RE.is_match(line) || DRAW_RE.is_match(line) {
            if DRAW_RE.is_match(line) {
                return Err(err("NOT_SUPPORTED", "draw"));
            }
            let (udf, end) = scan_function(&lines, i, ctx)?;
            let name = udf.name.clone();
            module.insert(
                name.clone(),
                Value {
                    name: Some(name),
                    value: V::Udf(std::sync::Arc::new(udf)),
                    dtype: dt::MODULE,
                    ..Default::default()
                },
            );
            i = end;
        } else if test_for_statement(line) {
            // A plain assignment in a module (continuation lines are joined).
            let mut s = line.to_string();
            let mut end = i;
            let mut j = i;
            loop {
                let continues = CONTINUES_RE.is_match(&s)
                    || (j + 1 < lines.len() && NEXT_CONTINUES_RE.is_match(lines[j + 1]));
                if !continues || j + 1 >= lines.len() {
                    end = j.max(end);
                    break;
                }
                j += 1;
                let next = strip_comment(lines[j]);
                s = if s.ends_with(';') && next.starts_with(['}', ')', ']']) {
                    format!("{}{next}", js_trim(&s[..s.len() - 1]))
                } else {
                    format!("{s}{next}")
                };
            }
            let pos = s.find('=').unwrap_or(0);
            let name = check_for_numeric_subscript(js_trim(&s[..pos]));
            let trail = js_trim(&s[pos + 1..]);
            let (value, unit, dtype, display) = value_from_literal(trail, &name, ctx);
            module.insert(
                name.clone(),
                Value {
                    name: Some(name),
                    value,
                    unit,
                    dtype,
                    resultdisplay: display,
                    ..Default::default()
                },
            );
            i = end;
        }
        i += 1;
    }
    Ok(module)
}

#[allow(dead_code)]
pub(crate) fn not_supported(what: &str) -> Operand {
    err_msg(format!("Error. Cruncha does not support {what} yet."))
}
