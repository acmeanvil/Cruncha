// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Statement preparation (Hurmet's `compile.js`): split `name = expression = ?? unit` into its
//! parts, compile the expression to RPN and the echo template, read a literal value, and
//! assemble the display templates. Nothing here depends on other statements' values.

use crate::attrs::{Attrs, Names};
use crate::dataframe::is_valid_identifier;
use crate::error::err;
use crate::eval::Ctx;
use crate::literal::{lang_opts, tex, value_from_literal};
use crate::module::{DRAW_RE, FUNCTION_RE, MODULE_RE, scan_module};
use crate::value::{OUnit, V, dt};
use cruncha_lang::hurmet::{check_for_numeric_subscript, js_trim};
use regex::Regex;
use std::sync::{Arc, LazyLock};

static CONTAINS_OPERATOR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[+\-×·*∘⌧/^%‰&√!¡|‖&=<>≟≠≤≥∈∉⋐∧∨⊻¬]|\x{a0}(function|mod|\\atop|root|sum|abs|cos|sin|tan|acos|asin|atan|sec|csc|cot|asec|acsc|acot|exp|log|ln|log10|log2|cosh|sinh|tanh|sech|csch|coth|acosh|asinh|atanh|asech|acsch|acoth|gamma|Γ|lgamma|logΓ|lfact|cosd|sind|tand|acosd|asind|atand|secd|cscd|cotd|asecd|acscd|acotd|real|imag|angle|Char|round|sqrt|sign|\?\{\}|%|⎾⏋|⎿⏌|\[\]|\(\))\x{a0}").unwrap()
});
static MUST_DO_CALCULATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:``.+``|(?:[£¥\x{20A0}-\x{20CF}]|(?:[ACR]|HK|US)?\$)?(?:\?{1,2}|@{1,2}|%{1,3}|!{1,2})[^=!(?@!{})]*)$").unwrap()
});
static PERCENT_OPERATOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"((?:\?{1,2}|@{1,2}|%%|!{1,2}) *)%").unwrap());
static ASSIGN_DATA_FRAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^[^=]+=\s*``.+``\s*$").unwrap());
static MATRIX_OF_NAMES_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^[(\[]{}{}′*[,;].+[)\]]$",
        cruncha_lang::hurmet::IDENTIFIER_START,
        cruncha_lang::hurmet::IDENTIFIER_BODY
    ))
    .unwrap()
});
static IS_KEYWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(π|pi|ℏ|true|false|root|if|in|else|elseif|and|or|otherwise|mod|modulo|for|while|end|break|return|throw)$").unwrap()
});
static LAST_UNIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^ ?!@%]+)$").unwrap());
static TEXT_SELECTOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\text\{(\?\??|%%?)\}").unwrap());
static SPACED_SELECTOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([?%]) ([?%])").unwrap());

/// What `compile` produced, beyond the attributes.
pub(crate) struct Compiled {
    pub attrs: Attrs,
    /// The display unit after the selector, as written.
    pub unit_str: Option<String>,
    /// The selector characters (e.g. `??`, `@`), or "" for statements without one.
    pub selector: String,
    pub is_calc: bool,
    pub is_test: bool,
}

fn shortcut(s: &str, ctx: &Ctx) -> Compiled {
    let t = tex(s, ctx);
    Compiled {
        attrs: Attrs {
            entry: s.to_string(),
            tex: t,
            alt: s.to_string(),
            is_static: true,
            ..Default::default()
        },
        unit_str: None,
        selector: String::new(),
        is_calc: false,
        is_test: false,
    }
}

fn is_identifier(s: &str) -> bool {
    is_valid_identifier(s) && !IS_KEYWORD_RE.is_match(s)
}

fn selector_of(trail: &str) -> String {
    trail
        .chars()
        .filter(|c| matches!(c, '?' | '@' | '%' | '!'))
        .collect()
}

/// Hurmet's `compile(inputStr, formats)`.
pub(crate) fn compile(input: &str, ctx: &Ctx) -> Result<Compiled, String> {
    let parse_err = |e: cruncha_lang::Error| e.to_string();
    let opts = lang_opts(ctx);

    let is_module = MODULE_RE.is_match(input);
    let is_draw = DRAW_RE.is_match(input);
    if FUNCTION_RE.is_match(input) || is_draw || is_module {
        let name = if is_draw {
            "draw".to_string()
        } else if is_module {
            let c = MODULE_RE
                .captures(input)
                .map(|c| c[1].to_string())
                .unwrap_or_default();
            check_for_numeric_subscript(js_trim(&c))
        } else {
            let pos_fn = input.find("function").unwrap_or(0);
            let pos_paren = input.find('(').unwrap_or(input.len());
            check_for_numeric_subscript(js_trim(input.get(pos_fn + 8..pos_paren).unwrap_or("")))
        };
        let mut attrs = Attrs {
            entry: input.to_string(),
            name: Names::One(name.clone()),
            resultdisplay: format!("\\text{{{name}}}"),
            is_static: true,
            ..Default::default()
        };
        match scan_module(input, ctx) {
            Ok(module) => {
                attrs.dtype = dt::MODULE;
                attrs.value = if is_module {
                    V::Module(Arc::new(module))
                } else {
                    module
                        .get(&name)
                        .map(|v| v.value.clone())
                        .unwrap_or(V::Undef)
                };
                attrs.tex = attrs.resultdisplay.clone();
                attrs.alt = name.clone();
            }
            Err(e) => {
                attrs.dtype = dt::ERROR;
                attrs.error = true;
                attrs.value = V::Null;
                attrs.tex = format!(
                    "\\textcolor{{firebrick}}{{\\text{{{}}}}}",
                    e.error_message()
                );
                attrs.alt = e.error_message().to_string();
            }
        }
        return Ok(Compiled {
            attrs,
            unit_str: None,
            selector: String::new(),
            is_calc: false,
            is_test: false,
        });
    }

    let mut s = input.to_string();
    let test_sel = if input.starts_with("@@test ") {
        Some("@@")
    } else if input.starts_with("@test ") {
        Some("@")
    } else {
        None
    };
    if let Some(sel) = test_sel {
        s = js_trim(&s[sel.len() + 5..]).to_string();
        let calc = cruncha_lang::parse_calc(&s, &opts).map_err(parse_err)?;
        let attrs = Attrs {
            entry: input.to_string(),
            rpn: Some(calc.rpn),
            dependencies: calc.dependencies,
            resulttemplate: Some(sel.to_string()),
            altresulttemplate: sel.to_string(),
            ..Default::default()
        };
        return Ok(Compiled {
            attrs,
            unit_str: None,
            selector: sel.to_string(),
            is_calc: true,
            is_test: true,
        });
    }

    let is_df_assignment = ASSIGN_DATA_FRAME_RE.is_match(&s);
    let pos_of_last_equals = if is_df_assignment {
        s.find('=').map_or(0, |p| p + 1)
    } else {
        s.rfind('=').map_or(0, |p| p + 1)
    };
    let mut lead_str;
    let mut main_str: String;
    let mut trail_str: String;
    let mut is_calc = false;
    let (mut suppress, mut result_only, mut omit_echo, mut must_align) =
        (false, false, false, false);
    let mut expression = String::new();
    let mut name = Names::None;
    if pos_of_last_equals > 1 {
        main_str = s[..pos_of_last_equals - 1]
            .trim_end_matches(' ')
            .to_string();
        if let Some(stripped) = main_str
            .trim_end_matches(char::is_whitespace)
            .strip_suffix(';')
        {
            must_align = true;
            main_str = stripped.to_string();
        }
        main_str = js_trim(&main_str).to_string();
        trail_str = js_trim(&s[pos_of_last_equals..]).to_string();

        if MUST_DO_CALCULATION_RE.is_match(&trail_str) {
            is_calc = true;
            suppress = trail_str.contains('!');
            result_only = trail_str.contains('@');
            omit_echo = trail_str.contains('%');
            if let Some(p) = main_str.find('=') {
                lead_str = js_trim(&main_str[..p]).to_string();
                expression = js_trim(&main_str[p + 1..]).to_string();
                if MATRIX_OF_NAMES_RE.is_match(&lead_str) {
                    let inner: String = {
                        let mut c = lead_str.chars();
                        c.next();
                        c.next_back();
                        c.collect()
                    };
                    lead_str = js_trim(&inner).to_string();
                }
                // A single name first: `X_(CO2,dry)` has a comma in its subscript (Cruncha;
                // Hurmet took any comma for a multiple assignment and refused the statement).
                if is_identifier(&lead_str) {
                    name = Names::One(check_for_numeric_subscript(&lead_str));
                } else if lead_str.contains([',', ';']) {
                    let candidates: Vec<&str> = lead_str.split([',', ';']).collect();
                    for c in &candidates {
                        let c = js_trim(c);
                        if IS_KEYWORD_RE.is_match(c) || !is_valid_identifier(c) {
                            return Ok(shortcut(&s, ctx));
                        }
                    }
                    name = Names::Many(
                        candidates
                            .iter()
                            .map(|e| check_for_numeric_subscript(js_trim(e)))
                            .collect(),
                    );
                } else {
                    expression = main_str.clone();
                }
            } else {
                expression = main_str.clone();
            }
        } else if is_df_assignment {
            name = Names::One(check_for_numeric_subscript(&main_str));
            expression = trail_str.clone();
        } else if is_identifier(&main_str) {
            name = Names::One(check_for_numeric_subscript(&main_str));
            if trail_str.is_empty() {
                let mut c = shortcut(&s, ctx);
                c.attrs.name = Names::None;
                return Ok(c);
            }
        } else {
            return Ok(shortcut(&s, ctx));
        }
    } else {
        return Ok(shortcut(&s, ctx));
    }

    let mut echo = String::new();
    let mut rpn = String::new();
    let mut dependencies = Vec::new();
    if !expression.is_empty() {
        if js_trim(&expression).starts_with("fetch(") {
            echo.clear();
        } else {
            let calc = cruncha_lang::parse_calc(&expression, &opts).map_err(parse_err)?;
            echo = calc.tex;
            rpn = calc.rpn;
            dependencies = calc.dependencies;
            if suppress
                || result_only
                || !echo.contains('〖')
                || rpn.contains("\u{a0}for\u{a0}")
                || omit_echo
            {
                echo.clear();
            } else if CONTAINS_OPERATOR_RE.is_match(&format!("\u{a0}{rpn}\u{a0}")) {
                echo = format!("\\textcolor{{#0000ff}}{{{echo}}}");
            } else {
                echo.clear();
            }
        }
    }

    let mut unit_str: Option<String> = None;
    let leads_with_currency = trail_str.starts_with(['$', '£', '¥'])
        || trail_str.starts_with(|c: char| ('\u{20A0}'..='\u{20CF}').contains(&c));
    if leads_with_currency {
        unit_str = trail_str.chars().next().map(String::from);
    }
    trail_str = PERCENT_OPERATOR_RE
        .replace(&trail_str, "${1}\\⦂")
        .into_owned();
    trail_str = trail_str.replacen("\\%", "\\⦂", 1);

    let mut attrs = Attrs {
        entry: s.clone(),
        name: name.clone(),
        ..Default::default()
    };
    let mut result_display;
    let mut literal_unit = OUnit::Null;
    let mut has_value = false;
    if is_calc {
        if !leads_with_currency {
            let u: String = trail_str
                .chars()
                .filter(|c| !matches!(c, '?' | '@' | '%' | '!' | '\''))
                .collect();
            unit_str = Some(js_trim(&u).to_string());
        }
        if suppress {
            result_display = trail_str.clone();
        } else {
            let has_unit = unit_str.as_deref().is_some_and(|u| !u.is_empty());
            let source = if has_unit {
                LAST_UNIT_RE
                    .replace(js_trim(&trail_str), "'${1}'")
                    .into_owned()
            } else {
                trail_str.clone()
            };
            result_display = tex(&source, ctx)
                .replace("\\%", "%")
                .replacen("@ @", "@@", 1);
            result_display = TEXT_SELECTOR_RE
                .replace(&result_display, "${1}")
                .into_owned();
            result_display = SPACED_SELECTOR_RE
                .replace(&result_display, "${1}${2}")
                .into_owned();
        }
    } else {
        let mut is_placeholder = false;
        if trail_str.ends_with('_') {
            is_placeholder = true;
            trail_str = js_trim(&trail_str[..trail_str.len() - 1]).to_string();
        }
        let name_str = match &name {
            Names::One(n) => n.clone(),
            Names::Many(v) => v.join(","),
            Names::None => String::new(),
        };
        let (value, unit, dtype, display) = value_from_literal(&trail_str, &name_str, ctx);
        result_display = display;
        if is_placeholder {
            result_display = format!("\\colorbox{{aqua}}{{${result_display}$}}");
        }
        if dtype == dt::ERROR {
            if result_display.is_empty() {
                return Ok(shortcut(&s, ctx));
            }
            let msg = result_display.clone();
            let shown = format!("\\textcolor{{firebrick}}{{\\text{{{msg}}}}}");
            attrs.resultdisplay = shown.clone();
            attrs.alt_result_display = msg.clone();
            attrs.tex = format!("{main_str} = {shown}");
            attrs.alt = format!("{main_str} = {msg}");
            attrs.error = true;
            attrs.dtype = dt::ERROR;
            attrs.value = V::Null;
            attrs.is_static = true;
            attrs.name = Names::None;
            return Ok(Compiled {
                attrs,
                unit_str: None,
                selector: String::new(),
                is_calc: false,
                is_test: false,
            });
        }
        has_value = value.truthy() || dtype == dt::BOOLEAN;
        attrs.value = if has_value { value } else { V::Undef };
        attrs.dtype = dtype;
        literal_unit = unit;
        rpn.clear();
    }

    let mut eqn = String::new();
    let mut alt_eqn = String::new();
    if !result_only {
        eqn = tex(&main_str, ctx);
        if must_align {
            eqn = format!("\\begin{{aligned}}{eqn}");
            if let Some(p) = eqn.find('=') {
                eqn.insert(p, '&');
            }
        }
        let align_char = if must_align { "\\\\ &" } else { "" };
        alt_eqn = main_str.clone();
        if !echo.is_empty() && !omit_echo {
            eqn.push_str(&format!(" {align_char}= {echo}"));
        }
        if !suppress {
            eqn.push_str(&format!(
                " {}= {result_display}",
                if must_align { "\\\\&" } else { "" }
            ));
            alt_eqn.push_str(&format!(" = {trail_str}"));
        }
        if must_align {
            eqn.push_str("\\end{aligned}");
        }
    }

    attrs.template = eqn.clone();
    attrs.alt_template = alt_eqn.clone();
    attrs.resultdisplay = result_display.clone();
    if is_calc {
        attrs.resulttemplate = Some(result_display);
        attrs.altresulttemplate = trail_str.clone();
    } else {
        attrs.tex = eqn;
        attrs.alt = alt_eqn;
        attrs.is_static = true;
    }
    if !rpn.is_empty() {
        attrs.rpn = Some(rpn.clone());
    }
    attrs.dependencies = dependencies;
    let unit_given = unit_str.as_deref().filter(|u| !u.is_empty());
    if is_calc {
        if let Some(u) = unit_given
            && !rpn.is_empty()
        {
            attrs.unit = match ctx.unit(u) {
                Ok(unit) => OUnit::from_unit(&unit),
                // Hurmet stores the error object, which has none of a unit's fields.
                Err(_) => OUnit::Obj(Box::default()),
            };
        }
    } else if literal_unit.truthy() {
        attrs.unit = literal_unit;
    }
    let _ = has_value;
    let selector = if is_calc {
        selector_of(&attrs.altresulttemplate)
    } else {
        String::new()
    };
    let _ = err;
    Ok(Compiled {
        attrs,
        unit_str: unit_given.map(str::to_string),
        selector,
        is_calc,
        is_test: false,
    })
}
