// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! The parser (Hurmet's `parser.js`): a single pass over the token stream that writes TeX and,
//! for calculations, RPN, using two operator-precedence stacks (one per output).
//!
//! The structure follows Hurmet closely so that the output matches exactly, including its
//! quirks. Positions in the TeX and RPN strings are byte offsets; wherever Hurmet steps by
//! UTF-16 code units, the port steps by chars.

use crate::lexer::{Token, Tt, lex, lex_unit_name, unit_start};
use crate::number::rational_rpn;
use crate::utils::{
    add_text_escapes, check_for_numeric_subscript, drop_last_char, first_char, is_js_space,
    is_leading_space, js_trim, next_char_boundary, numeral_from_superscript, skip_utf16,
    trim_leading_space, utf16_len, verbatim_arg,
};
use crate::{Error, Formats};
use regex::Regex;
use std::sync::LazyLock;

pub const BUILT_IN_FUNCTIONS: &[&str] = &[
    "Char",
    "Int",
    "abs",
    "acos",
    "acosd",
    "acosh",
    "acot",
    "acotd",
    "acoth",
    "acsc",
    "acscd",
    "acsch",
    "angle",
    "asec",
    "asecd",
    "asech",
    "asin",
    "asind",
    "asinh",
    "atan",
    "atan2",
    "atand",
    "atanh",
    "binomial",
    "ceil",
    "conj",
    "cos",
    "cosd",
    "cosh",
    "cot",
    "cotd",
    "coth",
    "count",
    "csc",
    "cscd",
    "csch",
    "exp",
    "factorial",
    "fetch",
    "findmax",
    "floor",
    "gamma",
    "gcd",
    "hcat",
    "imag",
    "isnan",
    "length",
    "lerp",
    "ln",
    "log",
    "log10",
    "log2",
    "lfact",
    "lgamma",
    "logn",
    "mod",
    "number",
    "ones",
    "real",
    "rem",
    "round",
    "roundSig",
    "roundn",
    "savedate",
    "sec",
    "secd",
    "sech",
    "sign",
    "sin",
    "sind",
    "sinh",
    "startSvg",
    "string",
    "tan",
    "tand",
    "tanh",
    "today",
    "trace",
    "transpose",
    "vcat",
    "zeros",
    "Γ",
];

pub const BUILT_IN_REDUCER_FUNCTIONS: &[&str] = &[
    "accumulate",
    "beamDiagram",
    "dataframe",
    "findfirst",
    "hypot",
    "matrix2table",
    "max",
    "mean",
    "median",
    "min",
    "product",
    "rand",
    "range",
    "rms",
    "stddev",
    "sum",
    "variance",
];

const TRIG_FUNCTIONS: &[&str] = &[
    "cos", "cosd", "cot", "cotd", "csc", "cscd", "sec", "secd", "sin", "sind", "tand", "tan",
];

/// TeX functions whose argument is copied verbatim.
const VERBATIMS: &[&str] = &[
    "\\alignat",
    "\\alignedat",
    "\\array",
    "\\darray",
    "\\ce",
    "\\color",
    "\\colorbox",
    "\\hskip",
    "\\hspace",
    "\\label",
    "\\kern",
    "\\mathrm",
    "\\mkern",
    "\\mskin",
    "\\pu",
    "\\subarray",
    "\\tag",
    "\\text",
    "\\textcolor",
];

/// An RPN token that marks the start of the argument of `isnan()`.
const ISNAN_MARKER: &str = "⁅isnan";
const NBSP: &str = "\u{a0}";
const E_RPN: &str = "®27182818284590452353602874713527/10000000000000000000000000000000";

#[rustfmt::skip]
const RPN_PREC_FROM_TYPE: [i32; 45] = [
    13, 13, 16, 14, 17, 10,
     8, 10, 13, -1, -1,
    -1,  1, -1,  0,  0,
     7,  0, -1, 15,  0,
     6,  8,  5,  4,  1,
    13, 17, 16, -1, 15,
    14, 10,  3,  2, 11,
    -1, -1,  4,  3, -1,
    -1, -1, -1, -1,
];

#[rustfmt::skip]
const TEX_PREC_FROM_TYPE: [i32; 45] = [
    12, 12, 15, 13, 16, 10,
     2, 10, 12,  2,  2,
     2,  1,  2,  2,  0,
     2,  1,  2, 14,  1,
     2,  2,  1,  1,  1,
     2, -1, 15,  2, 14,
    13,  9, -1,  1, -1,
    15, -1,  1, -1,  2,
     2,  2,  2,  2,
];

/// `rpnPrecFromType[t]`; an undefined entry behaves like -1 (no pop).
fn rpn_prec(t: Tt) -> i32 {
    RPN_PREC_FROM_TYPE.get(t as usize).copied().unwrap_or(-1)
}

fn tex_prec(t: Tt) -> i32 {
    TEX_PREC_FROM_TYPE
        .get(t as usize)
        .copied()
        .unwrap_or(i32::MAX)
}

/* Operator precedence
TeX  RPN
  0    0    ( [ {        delimiters
  1    1    , ;  :       separators for arguments, elements, rows, and ranges
  1    2    for in while loop keywords
  1    3    :            range separator
  1    4    if ∧ ∨       logical operators, return
  1    5    < > ≤ ≥      comparisons
  2    6    + -          addition and subtraction
  2    7    ∑            big operators
  2    8    * (x)(y) /   multiplication, division
  9   10    ∠            separator for complex numbers in polar notation
 10   11    -            unary minus
 12   13    sqrt sin     unary functions, math functions, and binary functions (e.g. root 3 x)
 13   14    ^            superscript, i.e. exponent
 14   15    ! % ‰ °      factorial, percent, permil, degree
 15   16    _ ' .        subscript, prime, dot notation property accessor
 16   17    hat bb       accent and font
*/

// Hurmet declares but never assigns dVECTORFROMRANGE and dDICTIONARY; kept for parity.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DelimType {
    Nothing,
    Paren,
    Function,
    Accessor,
    Matrix,
    VectorFromRange,
    Dictionary,
    Cases,
    Binomial,
    Subscript,
    Distrib,
    Environment,
}

#[derive(Clone, Debug)]
struct Delim {
    name: String,
    delim_type: DelimType,
    is_tall: bool,
    open: String,
    close: String,
    num_args: i64,
    num_rows: i64,
    rpn_pos: usize,
    is_preceded_by_div: bool,
    is_control_word_paren: bool,
}

impl Delim {
    fn base() -> Delim {
        Delim {
            name: String::new(),
            delim_type: DelimType::Nothing,
            is_tall: false,
            open: String::new(),
            close: String::new(),
            num_args: 0,
            num_rows: 0,
            rpn_pos: 0,
            is_preceded_by_div: false,
            is_control_word_paren: false,
        }
    }
}

/// An entry of the TeX operator stack, and Hurmet's `op` (the last popped or noted operator).
#[derive(Clone, Debug)]
struct Op {
    prec: i32,
    pos: usize,
    ttype: Tt,
    close_delim: String,
}

#[derive(Clone, Debug)]
struct RpnOp {
    prec: i32,
    symbol: String,
}

fn start_token() -> Token {
    Token::new("", "", Tt::Start)
}

static SUM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("\\\\∑_¿([^\u{a0}]+)([^=]+)\u{a0}=(\u{a0}[^\\^]+)\u{a0}\\^\u{a0}([^∑]+)\u{a0}∑")
        .unwrap()
});
static VAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"〖[^ ().\\,;]+").unwrap());
static SPREADSHEET_CELL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z](\d+|_end)$").unwrap());

/// parser.js `accentRegEx`: one char (or one astral char), a combining accent, then `_`.
fn accent_then_underscore(s: &str) -> bool {
    let mut it = s.chars();
    let first = it.next();
    let second = it.next();
    let third = it.next();
    first.is_some_and(|c| c != '\n' && c != '\r' && c != '\u{2028}' && c != '\u{2029}')
        && second.is_some_and(is_combining)
        && third == Some('_')
}

fn is_combining(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'
            ..='\u{0308}'
                | '\u{030A}'
                | '\u{030C}'
                | '\u{0332}'
                | '\u{20d0}'
                | '\u{20d1}'
                | '\u{20d6}'
                | '\u{20d7}'
                | '\u{20e1}'
    )
}

fn factors_after_space(c: char) -> bool {
    c.is_ascii_alphabetic()
        || matches!(c, 'ı' | 'ȷ' | '\u{0391}'..='\u{03C9}' | '\u{03D5}' | '\u{210B}' | '\u{210F}' | '\u{2110}' | '\u{2112}' | '\u{2113}' | '\u{211B}' | '\u{212C}' | '\u{2130}' | '\u{2131}' | '\u{2133}' | '\u{1D400}'..='\u{1D7FF}')
}

/// `/^(?:(?:if|and|atop|or|else|elseif|modulo|otherwise|not|for|in|while|end)\b|mod )/`
fn starts_with_word_op(st: &str) -> bool {
    const WORDS: [&str; 13] = [
        "if",
        "and",
        "atop",
        "or",
        "else",
        "elseif",
        "modulo",
        "otherwise",
        "not",
        "for",
        "in",
        "while",
        "end",
    ];
    let boundary = |rest: &str| !rest.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_');
    WORDS
        .iter()
        .any(|w| st.strip_prefix(w).is_some_and(boundary))
        || st.starts_with("mod ")
}

fn strip_latex_space(s: &str) -> &str {
    let mut s = s;
    loop {
        if let Some(r) = s.strip_prefix('˽') {
            s = r;
        } else if let Some(r) = s.strip_prefix("\\quad") {
            s = r;
        } else if let Some(r) = s.strip_prefix("\\qquad") {
            s = r;
        } else {
            return s;
        }
    }
}

fn is_multiplicand(t: Tt) -> bool {
    matches!(
        t,
        Tt::Ord
            | Tt::Var
            | Tt::Num
            | Tt::LongVar
            | Tt::RightBracket
            | Tt::Currency
            | Tt::SupChar
            | Tt::BigOperator
    )
}

fn next_char_is_factor(s: &str, token_type: Tt, is_followed_by_space: bool) -> bool {
    let st = strip_latex_space(s);
    let Some(fc) = first_char(st) else {
        return false;
    };
    if fc == '|' || fc == '‖' {
        return false; // TODO in Hurmet: work out left/right
    }
    if matches!(fc, '(' | '{' | '[' | '√' | '∛' | '∜' | '∑' | '0'..='9')
        && is_multiplicand(token_type)
    {
        return true;
    }
    if matches!(fc, '[' | '(' | '{' | '√' | '∛' | '∜')
        || (is_followed_by_space && factors_after_space(fc))
    {
        return !starts_with_word_op(st);
    }
    false
}

fn ends_of_ord(t: Tt) -> bool {
    matches!(
        t,
        Tt::Ord | Tt::Var | Tt::Num | Tt::Date | Tt::LongVar | Tt::RightBracket | Tt::SupChar
    )
}

fn check_for_unary_minus(token: Token, prev: &Token) -> Token {
    if matches!(
        prev.ttype,
        Tt::Num
            | Tt::Ord
            | Tt::Var
            | Tt::Date
            | Tt::RightBracket
            | Tt::LongVar
            | Tt::Property
            | Tt::Unit
            | Tt::SupChar
            | Tt::Prime
            | Tt::Factorial
    ) {
        return token;
    }
    if token.output == "-" {
        Token::new("~", "\\text{-}", Tt::UnaryMinus)
    } else {
        Token::new("+", "~+", Tt::UnaryMinus)
    }
}

fn num_from_sup_chars(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '⁻' => "-".to_string(),
            '²' => "2".to_string(),
            '³' => "3".to_string(),
            '¹' => "1".to_string(),
            '⁰' => "0".to_string(),
            '⁴'..='⁹' => ((c as u32 - 0x2074 + '4' as u32) as u8 as char).to_string(),
            _ => "undefined".to_string(),
        })
        .collect()
}

/// JS: `rpn.replace(/^.*\xa0/, "").replace(/§$/, "\xa0")`
fn last_expression(rpn: &str) -> String {
    // `.` does not match line terminators, so the greedy match stays on the first line.
    let line_end = rpn
        .find(['\n', '\r', '\u{2028}', '\u{2029}'])
        .unwrap_or(rpn.len());
    let mut expr = match rpn[..line_end].rfind(NBSP) {
        Some(p) => rpn[p + NBSP.len()..].to_string(),
        None => rpn.to_string(),
    };
    if expr.ends_with('§') {
        expr.pop();
        expr.push_str(NBSP);
    }
    expr
}

/// The Hurmet CASES expression acts lazily: move each sub-expression onto a stack; at the
/// closing brace they are placed after the conditions.
fn set_up_if(
    rpn: String,
    token_input: &str,
    expr_stack: &mut Vec<String>,
    delim: &mut Delim,
) -> String {
    let expression = last_expression(&rpn);
    let mut rpn = if rpn.len() == expression.len() && utf16_len(&rpn) == utf16_len(&expression) {
        String::new()
    } else {
        // JS slices by length; the expression is a suffix whose last char may differ (§ → nbsp).
        let n = utf16_len(&rpn) - utf16_len(&expression);
        let mut units = 0;
        let mut end = rpn.len();
        for (i, c) in rpn.char_indices() {
            if units >= n {
                end = i;
                break;
            }
            units += c.len_utf16();
        }
        rpn[..end].to_string()
    };
    expr_stack.push(expression);
    delim.num_args += 1;
    if token_input == "otherwise" {
        rpn.push_str("true");
    }
    rpn
}

fn js_number(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_string()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
    } else if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// The output of one parse.
pub(crate) struct Output {
    pub tex: String,
    pub rpn: String,
    pub dependencies: Vec<String>,
}

const MAX_DEPTH: usize = 32;

struct Parser<'a> {
    depth: usize,
    formats: &'a Formats,
    is_calc: bool,
    in_real_time: bool,
    sheet_name: &'a str,
    s: &'a str,
    tex: String,
    rpn: String,
    token: Token,
    prev_token: Token,
    dependencies: Vec<String>,
    must_align: bool,
    pos_of_prev_run: usize,
    is_preceded_by_space: bool,
    is_followed_by_space: bool,
    is_followed_by_space_or_newline: bool,
    followed_by_factor: bool,
    op: Op,
    tex_stack: Vec<Op>,
    rpn_stack: Vec<RpnOp>,
    delims: Vec<Delim>,
    ok_to_append: bool,
    pending_function_name: String,
    token_sep: &'static str,
    expr_stack: Vec<String>,
    num_free_commas: usize,
    pos_arrow: usize,
}

fn err(what: &str) -> Error {
    Error::Malformed(what.to_string())
}

impl<'a> Parser<'a> {
    fn top_delim(&mut self) -> Result<&mut Delim, Error> {
        self.delims
            .last_mut()
            .ok_or_else(|| err("unbalanced delimiters"))
    }

    fn pop_rpn(&mut self) -> Result<RpnOp, Error> {
        self.rpn_stack
            .pop()
            .ok_or_else(|| err("RPN operator stack underflow"))
    }

    fn push_rpn(&mut self, prec: i32, symbol: impl Into<String>) {
        self.rpn_stack.push(RpnOp {
            prec,
            symbol: symbol.into(),
        });
    }

    fn pop_rpn_tokens(&mut self, rpn_prec: i32) {
        if !(self.is_calc && rpn_prec >= 0) {
            return;
        }
        let exponent_prec = rpn_prec_of_sup();
        while let Some(top) = self.rpn_stack.last() {
            let top_prec = top.prec;
            // Exponents associate from right to left.
            if top_prec < rpn_prec || (top_prec == exponent_prec && rpn_prec == exponent_prec) {
                break;
            }
            let symbol = self.rpn_stack.pop().map(|o| o.symbol).unwrap_or_default();
            if symbol == "→" {
                let a = next_char_boundary(&self.rpn, self.pos_arrow);
                let mut body = self.rpn[a.min(self.rpn.len())..].to_string();
                drop_last_char(&mut body);
                self.rpn = format!(
                    "{}\"{}\"{}",
                    &self.rpn[..a.min(self.rpn.len())],
                    body.replace(NBSP, "§"),
                    self.token_sep
                );
                self.pos_arrow = 0;
            }
            self.rpn.push_str(&symbol);
            self.rpn.push_str(self.token_sep);
        }
    }

    fn note_op_at_prev_run(&mut self) {
        self.op = Op {
            prec: i32::MIN,
            pos: self.pos_of_prev_run,
            ttype: self.prev_token.ttype,
            close_delim: String::new(),
        };
    }

    /// Pop operators off the TeX stack and append their closing delimiters. When necessary,
    /// insert an opening brace before a fraction numerator.
    fn pop_tex_tokens(&mut self, tex_prec: i32, ok_to_append: bool) -> Result<(), Error> {
        if !ok_to_append {
            return Ok(());
        }
        let prev_is_right = matches!(self.prev_token.ttype, Tt::RightBracket | Tt::LeftRight);
        let Some(top) = self.tex_stack.last() else {
            if !prev_is_right {
                // Let an upcoming division know where to insert "\frac{".
                self.note_op_at_prev_run();
            }
            return Ok(());
        };
        if matches!(tex_prec, 2 | 12 | 14 | 15) && !prev_is_right && top.prec < tex_prec {
            self.note_op_at_prev_run();
            return Ok(());
        }

        let mut delim_idx: Option<usize> = None;
        loop {
            let Some(top) = self.tex_stack.last() else {
                return Ok(());
            };
            if !(top.prec >= tex_prec && !(top.prec == 13 && tex_prec == 13)) {
                return Ok(());
            }
            self.op = self
                .tex_stack
                .pop()
                .ok_or_else(|| err("TeX stack underflow"))?;

            // Before we append braces, check if we must hide a pair of parens.
            if self.op.prec == 0 {
                delim_idx = Some(
                    self.delims
                        .len()
                        .checked_sub(1)
                        .ok_or_else(|| err("unbalanced delimiters"))?,
                );
                if matches!(self.op.ttype, Tt::LeftBracket | Tt::LeftRight)
                    && !self.op.close_delim.is_empty()
                {
                    if let Some(next) = self.tex_stack.last()
                        && self.op.ttype == Tt::LeftRight
                        && self.token.output == ")"
                        && next.close_delim == ")"
                    {
                        // op is a middle |, as in P(A|B). Check if it's tall.
                        let di = delim_idx.unwrap_or(0);
                        if self.delims[di].is_tall {
                            self.tex.insert_str(self.op.pos, "\\middle");
                            self.delims[di].is_tall = true;
                        }
                        self.op = self
                            .tex_stack
                            .pop()
                            .ok_or_else(|| err("TeX stack underflow"))?;
                        self.delims.pop();
                        delim_idx = Some(
                            self.delims
                                .len()
                                .checked_sub(1)
                                .ok_or_else(|| err("unbalanced delimiters"))?,
                        );
                    }
                    let delim = self.delims[delim_idx.unwrap_or(0)].clone();
                    let pos = self.op.pos.min(self.tex.len());
                    let inc = if self.tex[pos..].starts_with('\\') {
                        2
                    } else {
                        crate::utils::first_char_len(&self.tex[pos..])
                    };
                    if delim.delim_type == DelimType::Dictionary && utf16_len(&delim.open) > 3 {
                        let end = (pos + 2).min(self.tex.len());
                        self.tex =
                            format!("{}{}{}", &self.tex[..pos], delim.open, &self.tex[end..]);
                        self.op.close_delim = delim.close.clone();
                    } else if delim.delim_type == DelimType::Matrix && utf16_len(&delim.open) > 1 {
                        let end = (pos + inc).min(self.tex.len());
                        self.tex =
                            format!("{}{}{}", &self.tex[..pos], delim.open, &self.tex[end..]);
                        self.op.close_delim = delim.close.clone();
                    } else if delim.delim_type == DelimType::Accessor {
                        let add_tall = delim.is_tall && !delim.open.contains("\\begin");
                        let end = (pos + inc).min(self.tex.len());
                        self.tex = format!(
                            "{}{}{}{}",
                            &self.tex[..pos],
                            if add_tall { "\\left" } else { "" },
                            delim.open,
                            &self.tex[end..]
                        );
                        self.op.close_delim =
                            format!("{}{}", if add_tall { "\\right" } else { "" }, delim.close);
                    } else if delim.delim_type == DelimType::Cases {
                        let end = (pos + 2).min(self.tex.len());
                        self.tex =
                            format!("{}{}{}", &self.tex[..pos], delim.open, &self.tex[end..]);
                        self.op.close_delim = delim.close.clone();
                    } else if delim.delim_type == DelimType::Paren
                        && delim.name == "("
                        && (self.s.starts_with('/') || starts_with_atop_space(self.s))
                    {
                        // The parens surround a numerator. Delete them.
                        self.tex
                            .replace_range(pos..next_char_boundary(&self.tex, pos), "");
                        self.op.close_delim = String::new();
                    } else if delim.is_preceded_by_div
                        && delim.delim_type == DelimType::Paren
                        && delim.name == "("
                        && (self.s.is_empty()
                            || !self.s.starts_with([
                                '^', '_', '!', '%', '°', '⁻', '²', '³', '¹', '⁰', '⁴', '⁵', '⁶',
                                '⁷', '⁸', '⁹',
                            ]))
                    {
                        // The parens surround a denominator. Delete them.
                        self.tex
                            .replace_range(pos..next_char_boundary(&self.tex, pos), "");
                        self.op.close_delim = String::new();
                    } else if delim.is_tall {
                        // Make the delims tall.
                        if !self.tex[pos..].starts_with("\\left") {
                            self.tex.insert_str(pos, "\\left");
                        }
                        if !self.op.close_delim.contains("\\right") {
                            self.op.close_delim = format!("\\right{}", self.token.output);
                        }
                    }
                }
            }
            let _ = delim_idx;

            // Remove an implicit multiplication space.
            let trimmed = self.tex.trim_end_matches(' ');
            if trimmed.ends_with("\\,") {
                let n = trimmed.len() - 2;
                self.tex.truncate(n);
            }
            self.tex.push_str(&self.op.close_delim);

            if self.op.close_delim.ends_with('{') {
                // We just closed the first part of a binary function, root()(),
                // or a function exponent (sin^2 θ) or function subscript (log_10).
                if self.op.ttype == Tt::Binary {
                    self.tex_stack.push(Op {
                        prec: 12,
                        pos: self.op.pos,
                        ttype: Tt::Unary,
                        close_delim: "}".into(),
                    });
                    if self.is_calc {
                        self.rpn.push_str(self.token_sep);
                        if let Some(top) = self.rpn_stack.last_mut()
                            && top.symbol == "\\sqrt"
                        {
                            top.symbol = "root".into();
                        }
                    }
                }
                self.op.ttype = Tt::Unary;
                self.prev_token = Token::new("", "", Tt::Unary);
                return Ok(());
            }

            if self.tex_stack.is_empty() || self.op.prec == 0 {
                return Ok(());
            }
        }
    }

    fn test_for_implicit_mult(&self) -> bool {
        if let Some(top) = self.tex_stack.last()
            && matches!(top.ttype, Tt::Unary | Tt::Binary)
        {
            return false;
        }
        let prev = &self.prev_token;
        let mut preceded_by_factor = false;
        if !prev.output.is_empty() {
            // Hurmet reads `prevToken.output.charAt(prevToken.length - 1)`. A token has no
            // `length`, so that is charAt(NaN), the first character.
            let pc = first_char(&prev.output).unwrap_or(' ');
            if matches!(pc, ')' | ']' | '}') {
                if (pc == ')' || pc == ']') && self.s.starts_with(['(', '[']) {
                    return false; // handled by the RIGHTBRACKET case
                }
                preceded_by_factor = true;
            } else {
                preceded_by_factor = matches!(
                    prev.ttype,
                    Tt::Ord
                        | Tt::Num
                        | Tt::Var
                        | Tt::LongVar
                        | Tt::Prime
                        | Tt::Sup
                        | Tt::SupChar
                        | Tt::Sub
                        | Tt::Property
                        | Tt::Unit
                        | Tt::RightBracket
                        | Tt::Factorial
                );
            }
        }
        preceded_by_factor && next_char_is_factor(self.s, prev.ttype, self.is_followed_by_space)
    }

    fn sub_parse(&self, s: &str, formats: &Formats, is_calc: bool) -> Result<Output, Error> {
        if self.depth >= MAX_DEPTH {
            // Hurmet recurses without bound here, e.g. on an unclosed rich-text backtick,
            // and fails with a JavaScript stack overflow.
            return Err(err("recursion too deep"));
        }
        parse_at_depth(s, formats, is_calc, false, "", self.depth + 1)
    }

    /// As in sin²(), sin^2 x, sin^(-1) x.
    fn exponent_of_function(&self) -> Result<(String, String, String), Error> {
        let s = self.s;
        let sub_formats = Formats {
            decimal_format: self.formats.decimal_format.clone(),
            date_format: None,
        };
        let expo_input: String = if !s.starts_with('^') {
            let mut out = String::new();
            let mut it = s.chars().peekable();
            if it.peek() == Some(&'⁻') {
                out.push('⁻');
                it.next();
            }
            for c in it {
                if matches!(c, '⁰' | '¹' | '²' | '³' | '\u{2074}'..='\u{2079}' | '⁻') {
                    out.push(c);
                } else {
                    break;
                }
            }
            out.chars().map(numeral_from_superscript).collect()
        } else if !s[1..].trim_start_matches(' ').starts_with('(') {
            lex(&s[1..], &sub_formats, &start_token(), false).input
        } else {
            // The exponent is in parens. Find its extent.
            let chars: Vec<char> = s.chars().collect();
            let mut expo = String::from("(");
            let mut level = 1;
            let mut i = 2;
            while i < chars.len() {
                let ch = chars[i];
                expo.push(ch);
                if matches!(ch, '"' | '\'' | '`') {
                    match chars[i + 1..].iter().position(|&c| c == ch) {
                        Some(off) => {
                            let pos = i + 1 + off;
                            expo.extend(&chars[i + 1..=pos]);
                            i = pos;
                        }
                        None => break,
                    }
                } else if "([{⟨\u{2308}\u{23BF}\u{23BE}\u{3016}".contains(ch) {
                    level += 1;
                } else if ")]}⟩\u{2309}\u{230B}\u{23CC}\u{3017}".contains(ch) {
                    level -= 1;
                }
                if level == 0 {
                    break;
                }
                i += 1;
            }
            expo
        };

        let parse_input = if let Some(inner) = expo_input.strip_prefix('(') {
            let mut inner = inner.to_string();
            drop_last_char(&mut inner);
            js_trim(&inner).to_string()
        } else {
            expo_input.clone()
        };
        let out = self.sub_parse(&parse_input, &sub_formats, self.is_calc)?;
        let rpn = if self.is_calc { out.rpn } else { String::new() };
        Ok((expo_input, format!("{{{}}}", out.tex), rpn))
    }

    fn run(&mut self) -> Result<(), Error> {
        while !self.s.is_empty() {
            if let Some(rest) = self.s.strip_prefix('\n') {
                self.s = rest;
                let prev_input = &self.prev_token.input;
                let prev_char_ok = match prev_input.chars().last() {
                    None => false, // "".indexOf("") is 0, i.e. found
                    Some(c) => !"{[(,;+-".contains(c),
                };
                let next_closes = self.s.trim_start_matches(' ').starts_with([')', '}', ']']);
                if self.prev_token.ttype == Tt::Comment || (prev_char_ok && !next_closes) {
                    self.pop_tex_tokens(0, true)?;
                    self.tex.push_str("\\\\ ");
                    let spaces = self.s.len() - self.s.trim_start_matches(' ').len();
                    self.s = self.s.trim_start_matches(' ');
                    if self.s.starts_with('=') && self.tex.contains('=') {
                        self.must_align = true; // Use {aligned} to align = signs.
                        self.tex.push('&');
                    } else if spaces > 0 {
                        self.tex.push_str(&"\\quad ".repeat(spaces - 1));
                    }
                }
                self.s = js_trim(self.s);
            }

            let mut must_lex = true;

            let is_implicit_mult =
                self.is_preceded_by_space && self.ok_to_append && self.test_for_implicit_mult();
            if self.is_calc
                && self.prev_token.input == "⌧"
                && self.rpn_stack.len() > 1
                && self.rpn_stack[self.rpn_stack.len() - 2].symbol == "∑"
                && self.rpn.chars().rev().nth(1) == Some('^')
            {
                // The space after a ∑_(i=0)^n symbol is not an implicit multiplication.
                self.rpn_stack.pop();
            }

            if is_implicit_mult {
                let out = if matches!(self.prev_token.ttype, Tt::LongVar | Tt::Num | Tt::Unit) {
                    "\\,"
                } else {
                    ""
                };
                self.token = Token::new("⌧", out, Tt::Mult);
                self.is_followed_by_space = false;
                self.is_followed_by_space_or_newline = false;
                must_lex = false;
            }

            if must_lex {
                self.token = if self.prev_token.ttype == Tt::Num
                    && !self.is_followed_by_space
                    && unit_start(self.s)
                {
                    lex_unit_name(self.s) // something like the "m" in "5m"
                } else {
                    lex(self.s, self.formats, &self.prev_token, self.in_real_time)
                };
                self.s = skip_utf16(self.s, utf16_len(&self.token.input));
                self.is_followed_by_space = self.s.starts_with(is_leading_space)
                    || self.s.starts_with('˽')
                    || self.s.starts_with("\\quad")
                    || self.s.starts_with("\\qquad");
                self.is_followed_by_space_or_newline = self.s.starts_with([' ', '\n']);
                self.s = trim_leading_space(self.s);
                self.followed_by_factor =
                    next_char_is_factor(self.s, self.token.ttype, self.is_followed_by_space);
            }

            if self.token.input == "!"
                && (self.is_preceded_by_space || !ends_of_ord(self.prev_token.ttype))
            {
                // "!" is logical not in some contexts, to match Julia syntax.
                self.token.ttype = Tt::Unary;
                self.token.input = "¬".into();
            }

            self.step()?;

            self.prev_token = self.token.clone();
            self.is_preceded_by_space = self.is_followed_by_space || self.token.input == "⌧";
        }
        Ok(())
    }

    fn step(&mut self) -> Result<(), Error> {
        let is_calc = self.is_calc;
        match self.token.ttype {
            Tt::Space | Tt::Bin | Tt::Add | Tt::Mult | Tt::Rel | Tt::BigOperator => {
                if !self.token.output.is_empty() && "- +".contains(self.token.output.as_str()) {
                    self.token = check_for_unary_minus(self.token.clone(), &self.prev_token);
                }
                if is_calc && self.token.output == "→" {
                    // The arrow of an anonymous function, e.g., x → cos x.
                    if let Some(p) = self.rpn.rfind('¿') {
                        let name = &self.rpn[p + '¿'.len_utf8()..];
                        if !name.is_empty() && !name.contains(NBSP) {
                            self.rpn = format!("{}\"{}\"", &self.rpn[..p], name);
                        }
                    }
                    self.pos_arrow = self.rpn.len();
                    if let Some(pb) = self.tex.rfind('〖') {
                        self.tex.replace_range(pb..pb + '〖'.len_utf8(), "");
                    } else {
                        // JS: tex.slice(0, -1) + tex.slice(0)
                        let mut head = self.tex.clone();
                        drop_last_char(&mut head);
                        self.tex = head + &self.tex;
                    }
                }
                if is_calc && self.token.ttype != Tt::Space {
                    if self.token.output != "\\text{-}" && self.token.ttype != Tt::BigOperator {
                        self.rpn.push_str(self.token_sep);
                    }
                    let t = if self.token.ttype == Tt::BigOperator {
                        Tt::Var
                    } else {
                        self.token.ttype
                    };
                    self.pop_rpn_tokens(rpn_prec(t));
                }
                self.pop_tex_tokens(tex_prec(self.token.ttype), self.ok_to_append)?;
                self.tex.push_str(&self.token.output);
                self.tex.push(' ');
                self.pos_of_prev_run = self.tex.len();
                if self.token.ttype == Tt::BigOperator && self.delims.len() > 1 {
                    self.top_delim()?.is_tall = true;
                }
                if is_calc {
                    if self.token.input == "∑" || self.token.input == "\\sum" {
                        self.rpn.push_str("\\∑");
                    }
                    let symbol = if self.token.input == "mod" {
                        "modulo".to_string()
                    } else {
                        self.token.input.clone()
                    };
                    self.push_rpn(rpn_prec(self.token.ttype), symbol);
                }
                self.ok_to_append = true;
            }

            Tt::Accessor | Tt::Angle => {
                self.token = check_for_unary_minus(self.token.clone(), &self.prev_token);
                if is_calc {
                    self.rpn.push_str(self.token_sep);
                    let p = rpn_prec(self.token.ttype);
                    self.pop_rpn_tokens(p);
                    self.push_rpn(p, self.token.input.clone());
                }
                self.pop_tex_tokens(tex_prec(self.token.ttype), self.ok_to_append)?;
                if is_calc {
                    self.tex.push_str(&self.token.input);
                } else {
                    self.tex.push_str(&self.token.output);
                    self.tex.push(' ');
                }
                self.ok_to_append = true;
            }

            Tt::Boolean => {
                self.pop_tex_tokens(2, self.ok_to_append)?;
                if is_calc {
                    self.pop_rpn_tokens(-1);
                    self.rpn.push_str(&self.token.input);
                }
                if self.is_preceded_by_space {
                    self.pos_of_prev_run = self.tex.len();
                }
                self.tex.push_str(&self.token.output);
                self.ok_to_append = true;
            }

            Tt::Num | Tt::Ord | Tt::Date => {
                self.pop_tex_tokens(2, self.ok_to_append)?;
                if is_calc {
                    self.pop_rpn_tokens(-1);
                    let r = match self.token.ttype {
                        Tt::Num => rational_rpn(&self.token.input),
                        Tt::Date => crate::date::date_rpn(&self.token.input),
                        _ => self.token.input.clone(),
                    };
                    self.rpn.push_str(&r);
                }
                if self.is_preceded_by_space {
                    self.pos_of_prev_run = self.tex.len();
                }
                if is_calc
                    && (self.prev_token.ttype == Tt::Mult
                        || (self.followed_by_factor && self.prev_token.ttype != Tt::Div))
                {
                    self.token.output = format!("({})", self.token.output);
                }
                self.tex.push_str(&self.token.output);
                self.tex.push(' ');
                self.ok_to_append = true;

                if !self.is_followed_by_space && self.followed_by_factor {
                    // Something like "2(a)".
                    self.pop_tex_tokens(2, self.ok_to_append)?;
                    if is_calc {
                        let p = rpn_prec(Tt::Mult);
                        self.rpn.push_str(self.token_sep);
                        self.pop_rpn_tokens(p);
                        self.push_rpn(p, "⌧");
                    }
                }
            }

            Tt::String => {
                self.pop_tex_tokens(2, self.ok_to_append)?;
                let ch = first_char(&self.token.input).unwrap_or('"');
                if is_calc {
                    self.rpn.push(ch);
                    self.rpn.push_str(&self.token.output);
                    self.rpn.push(ch);
                }
                if self.is_preceded_by_space {
                    self.pos_of_prev_run = self.tex.len();
                }
                let mut out = add_text_escapes(&self.token.output);
                // Prevent loss of a trailing space.
                let t = out.trim_end_matches(' ').len();
                if t < out.len() {
                    out.truncate(t);
                    out.push_str("\\,");
                }
                self.token.output = out;
                self.tex.push_str("\\text{");
                self.tex.push_str(&self.token.output);
                self.tex.push('}');
                self.ok_to_append = true;
            }

            Tt::RichText => {
                self.pop_tex_tokens(2, self.ok_to_append)?;
                let ch = first_char(&self.token.input).unwrap_or('`');
                if is_calc {
                    self.rpn.push(ch);
                    self.rpn.push_str(&self.token.output);
                    self.rpn.push(ch);
                }
                if self.is_preceded_by_space {
                    self.pos_of_prev_run = self.tex.len();
                }
                if self.token.output != "`" {
                    self.token.output = self
                        .sub_parse(&self.token.output.clone(), self.formats, false)?
                        .tex;
                }
                self.tex.push('{');
                self.tex.push_str(&self.token.output);
                self.tex.push('}');
                self.ok_to_append = true;
            }

            Tt::DataFrame => {
                self.pop_tex_tokens(2, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                self.tex.push_str(&self.token.output);
                if is_calc {
                    self.rpn.push_str(&self.token.input);
                    // String interpolation, ${varName}
                    let input = self.token.input.clone();
                    let mut rest = input.as_str();
                    while let Some(p) = rest.find("${") {
                        let after = &rest[p + 2..];
                        let n = after
                            .find(|c: char| c == '}' || is_js_space(c))
                            .unwrap_or(after.len());
                        if n > 0 && after[n..].starts_with('}') {
                            self.dependencies.push(after[..n].to_string());
                            rest = &after[n + 1..];
                        } else {
                            rest = &rest[p + 1..];
                        }
                    }
                }
                self.ok_to_append = true;
            }

            Tt::Var | Tt::LongVar => {
                if self.token.ttype == Tt::LongVar && self.prev_token.input == "⌧" {
                    self.tex.push_str("\\,"); // A space before a long variable name.
                }
                self.pop_tex_tokens(7, self.ok_to_append)?;
                if self.is_preceded_by_space {
                    self.pos_of_prev_run = self.tex.len();
                }
                if !is_calc {
                    if self.token.ttype == Tt::LongVar && !accent_then_underscore(&self.token.input)
                    {
                        self.token.output = format!("\\mathrm{{{}}}", self.token.output);
                    }
                } else if self.prev_token.input == "for" {
                    self.rpn.push('"');
                    self.rpn.push_str(&self.token.input);
                    self.rpn.push('"'); // a loop index variable name
                } else {
                    // The echo of a calculation.
                    let indexed = self.s.starts_with('[')
                        || (self.s.starts_with('.')
                            && self.s[1..].chars().next().is_some_and(|c| c != '.'));
                    if indexed {
                        // varName[index] renders the name of the variable, not its value.
                        if self.token.ttype == Tt::LongVar {
                            self.token.output = format!("\\mathrm{{{}}}", self.token.output);
                        }
                    } else {
                        self.token.input = check_for_numeric_subscript(&self.token.input);
                        self.token.output = format!(
                            "{}{}",
                            if self.pos_arrow > 0 { "" } else { "〖" },
                            self.token.input
                        );
                    }
                    if !self.sheet_name.is_empty()
                        && SPREADSHEET_CELL_RE.is_match(&self.token.input)
                    {
                        self.rpn.push('¿');
                        self.rpn.push_str(self.sheet_name);
                        self.rpn.push_str(self.token_sep);
                        self.rpn.push_str(&format!("\"{}\"", self.token.input));
                        self.rpn.push_str(self.token_sep);
                        self.rpn.push('.');
                    } else {
                        self.rpn.push('¿');
                        self.rpn.push_str(&self.token.input);
                    }
                    self.dependencies.push(self.token.input.clone());
                }
                self.tex.push_str(&self.token.output);
                if !self.s.starts_with('.') {
                    self.tex.push(' ');
                }
                if is_calc {
                    // The variable's value may be tall. We don't know.
                    self.top_delim()?.is_tall = true;
                }
                self.ok_to_append = true;
            }

            Tt::Unit => {
                if self.delims.len() > 1 && self.top_delim()?.delim_type == DelimType::Matrix {
                    self.token.output = "\\text{Error. Write a unit name outside a matrix, not inside. Apply one unit to the entire matrix.}".into();
                }
                self.pop_tex_tokens(14, true)?;
                self.tex_stack.push(Op {
                    prec: 14,
                    pos: self.op.pos,
                    ttype: Tt::Unit,
                    close_delim: String::new(),
                });
                if is_calc {
                    self.pop_rpn_tokens(rpn_prec(Tt::Unit));
                    self.rpn.push_str(self.token_sep);
                    self.rpn.push_str("applyUnit");
                    self.rpn.push_str(self.token_sep);
                    self.rpn.push_str(&self.token.input.replace('\'', ""));
                }
                let inp = self.token.input.as_str();
                if !(inp == "°" || inp == "'°" || inp == "°'" || inp == "'°'") {
                    self.tex.push_str("\\;");
                }
                self.tex.push_str(&self.token.output);
                self.ok_to_append = true;
            }

            Tt::Property => {
                // A property name after a dot accessor.
                self.pop_tex_tokens(15, self.ok_to_append)?;
                let pos = self.token.input.find('_');
                if is_calc {
                    self.rpn.push_str(&format!("\"{}\"", self.token.output));
                    self.tex
                        .push_str(&format!("\\mathrm{{{}}}", self.token.output));
                    if !self.s.starts_with('.') {
                        self.tex.push(' ');
                    }
                } else if let Some(p) = pos {
                    let inp = &self.token.input;
                    self.tex
                        .push_str(&format!("{}_\\mathrm{{{}}}", &inp[..p], &inp[p + 1..]));
                } else {
                    let mut out = add_text_escapes(&self.token.output);
                    let t = out.trim_end_matches(' ').len();
                    if t < out.len() {
                        out.truncate(t);
                        out.push_str("\\,");
                    }
                    self.tex.push_str(&format!("\\text{{{out}}}"));
                }
                self.ok_to_append = true;
            }

            Tt::To => {
                // A probability distribution defined by its low and high values: (2 to 3).
                self.top_delim()?.delim_type = DelimType::Distrib;
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                self.tex.push_str(&self.token.output);
                if is_calc {
                    self.rpn.push_str(self.token_sep);
                    self.pop_rpn_tokens(3);
                    // Hurmet reads `delims[…].symbol`, which is never set, so this is always
                    // "lognormal".
                    self.push_rpn(3, "lognormal");
                }
            }

            Tt::Range => {
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                if is_calc && self.token.input != "..." {
                    self.rpn.push_str(self.token_sep);
                    self.pop_rpn_tokens(3);
                    self.push_rpn(3, ":");
                    if (self.prev_token.input == "[" || self.prev_token.input == ",")
                        && self.s.starts_with([',', ']'])
                    {
                        // A bare colon in an accessor, e.g., M[:, 2]
                        self.rpn.push_str("®1/1");
                        self.rpn.push_str(self.token_sep);
                        self.rpn.push_str("\"∞\"");
                    } else if self.s.starts_with("end,") || self.s.starts_with("end]") {
                        self.rpn.push_str("\"∞\""); // identifier[n:end]
                        self.s = &self.s[3..];
                    }
                }
                // Hurmet checks `topDelim.symbol === "{"` for "\colon", but `symbol` is never set.
                self.tex.push_str(&self.token.output);
            }

            Tt::Div => {
                if is_calc {
                    self.rpn.push_str(self.token_sep);
                }
                self.pop_tex_tokens(2, true)?;
                let p = rpn_prec(Tt::Div);
                self.pop_rpn_tokens(p);
                let pos = self.op.pos.min(self.tex.len());
                if self.token.input == "//" {
                    self.tex_stack.push(Op {
                        prec: 2,
                        pos,
                        ttype: Tt::Div,
                        close_delim: "}".into(),
                    });
                    self.tex = format!("{}\\tfrac{{{}}}{{", &self.tex[..pos], &self.tex[pos..]);
                } else if self.token.input == "/" || self.token.input == "\\over" {
                    self.tex_stack.push(Op {
                        prec: 2,
                        pos,
                        ttype: Tt::Div,
                        close_delim: "}".into(),
                    });
                    // Inner fractions become \frac.
                    let inner = self.tex[pos..].replace("\\dfrac{", "\\frac{");
                    self.tex = format!("{}\\dfrac{{{}}}{{", &self.tex[..pos], inner);
                } else {
                    // \atop, for binomials
                    self.tex_stack.push(Op {
                        prec: 2,
                        pos,
                        ttype: Tt::Div,
                        close_delim: "}}".into(),
                    });
                    self.tex = format!("{}{{{{{}}}\\atop{{", &self.tex[..pos], &self.tex[pos..]);
                    let d = self.top_delim()?;
                    if d.name == "(" {
                        d.delim_type = DelimType::Binomial;
                    }
                }
                if is_calc {
                    if self.token.input == "\\atop" {
                        if self.top_delim()?.delim_type == DelimType::Binomial {
                            self.push_rpn(p, "()");
                        }
                    } else {
                        self.push_rpn(p, self.token.input.clone());
                    }
                }
                self.top_delim()?.is_tall = true;
                self.pos_of_prev_run = self.tex.len();
                self.ok_to_append = false;
            }

            Tt::Sub => {
                self.pop_tex_tokens(15, true)?;
                let close = if self.prev_token.ttype == Tt::Function {
                    "}{"
                } else {
                    "}"
                };
                self.tex_stack.push(Op {
                    prec: 15,
                    pos: self.op.pos,
                    ttype: Tt::Sub,
                    close_delim: close.into(),
                });
                self.tex.push_str("_{");
                if is_calc {
                    self.rpn.push('_');
                }
                self.ok_to_append = false;
            }

            Tt::Sup | Tt::SupChar => {
                let is_char = self.token.ttype == Tt::SupChar;
                if is_calc {
                    if self.rpn.ends_with("¿e") {
                        // e^3: replace e with its value.
                        self.rpn.truncate(self.rpn.len() - "¿e".len());
                        self.rpn.push_str(E_RPN);
                    }
                    self.rpn.push_str(self.token_sep);
                    self.pop_rpn_tokens(rpn_prec(self.token.ttype));
                }
                self.pop_tex_tokens(13, true)?;
                let pos = if self.prev_token.ttype == Tt::RightBracket {
                    self.op.pos
                } else {
                    self.pos_of_prev_run
                };
                self.tex_stack.push(Op {
                    prec: 13,
                    pos,
                    ttype: Tt::Sup,
                    close_delim: "}".into(),
                });
                if is_char {
                    let sup_num = num_from_sup_chars(&self.token.output);
                    self.tex.push_str("^{");
                    self.tex.push_str(&sup_num);
                    if is_calc {
                        self.push_rpn(rpn_prec(Tt::SupChar), "^");
                        self.rpn.push_str(&rational_rpn(&sup_num));
                    }
                    self.ok_to_append = true;
                } else {
                    if is_calc {
                        self.push_rpn(rpn_prec(Tt::Sup), "^");
                    }
                    if !self.delims.is_empty() && self.s.starts_with('(') {
                        self.top_delim()?.is_tall = true;
                    }
                    self.tex.push_str("^{");
                    self.ok_to_append = false;
                }
            }

            Tt::Function => {
                self.pop_tex_tokens(2, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                // Is there an exponent on the function name?
                if self.s.starts_with([
                    '^', '⁻', '⁰', '¹', '²', '³', '\u{2074}', '\u{2075}', '\u{2076}', '\u{2077}',
                    '\u{2078}', '\u{2079}',
                ]) {
                    let (expo_input, expo_tex, expo_rpn) = self.exponent_of_function()?;
                    if is_calc
                        && expo_rpn == "®-1/1"
                        && TRIG_FUNCTIONS.contains(&self.token.input.as_str())
                    {
                        // Inverse trig function.
                        self.token.input = format!("a{}", self.token.input);
                        self.token.output = format!("\\a{}", &self.token.output[1..]);
                    } else {
                        if is_calc {
                            self.token.input = format!(
                                "{}{}{}{}^",
                                self.token.input, self.token_sep, expo_rpn, self.token_sep
                            );
                        }
                        self.token.output = format!("{}^{}", self.token.output, expo_tex);
                    }
                    let l = utf16_len(&expo_input) + usize::from(self.s.starts_with('^'));
                    self.s = js_trim(skip_utf16(self.s, l));
                }
                if is_calc {
                    self.push_rpn(rpn_prec(Tt::Function), self.token.input.clone());
                    if self.prev_token.input == "⌧" {
                        self.tex.push('×');
                    }
                }
                let paren = self.s.starts_with('(');
                self.tex_stack.push(Op {
                    prec: 12,
                    pos: self.tex.len(),
                    ttype: Tt::Function,
                    close_delim: if paren { String::new() } else { "}".into() },
                });
                self.tex.push_str(&self.token.output);
                if !paren {
                    self.tex.push('{');
                }
                self.pending_function_name = self.token.input.clone();
                self.ok_to_append = false;
            }

            Tt::Accent => {
                if is_calc {
                    self.rpn.push_str(self.token_sep);
                    self.pop_rpn_tokens(rpn_prec(Tt::Accent));
                }
                self.pop_tex_tokens(1, self.ok_to_append)?;
                if is_calc {
                    self.tex_stack.push(Op {
                        prec: 16,
                        pos: self.tex.len(),
                        ttype: Tt::Accent,
                        close_delim: "〗".into(),
                    });
                    if self.pos_arrow == 0 {
                        self.tex.push('〖');
                    }
                    self.tex.push_str(&self.token.input);
                    self.rpn.push('¿');
                    self.rpn.push_str(&self.token.input);
                    self.dependencies.push(self.token.input.clone());
                } else {
                    self.tex_stack.push(Op {
                        prec: 16,
                        pos: self.tex.len(),
                        ttype: Tt::Accent,
                        close_delim: "}".into(),
                    });
                    self.tex.push_str(&self.token.output);
                    self.tex.push('{');
                }
                self.top_delim()?.is_tall = true;
                self.ok_to_append = false;
            }

            Tt::Prime => {
                self.pop_tex_tokens(15, true)?;
                if is_calc {
                    self.rpn.push_str(&self.token.input);
                    self.dependencies
                        .push(format!("{}{}", self.prev_token.input, self.token.input));
                }
                self.tex = format!("{}{} ", js_trim(&self.tex), self.token.output);
                self.ok_to_append = true;
            }

            Tt::Binary => {
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                if self.token.input == "root" {
                    self.tex_stack.push(Op {
                        prec: 12,
                        pos: self.tex.len(),
                        ttype: Tt::Binary,
                        close_delim: "]{".into(),
                    });
                    self.tex.push_str("\\sqrt[");
                } else if VERBATIMS.contains(&self.token.input.as_str()) {
                    self.tex_stack.push(Op {
                        prec: 0,
                        pos: self.tex.len(),
                        ttype: Tt::Unary,
                        close_delim: "}".into(),
                    });
                    let arg = verbatim_arg(self.s);
                    self.tex
                        .push_str(&format!("{}{{{}}}{{", self.token.output, arg));
                    self.s = skip_utf16(self.s, utf16_len(&arg) + 2);
                    self.s = trim_leading_space(self.s);
                    self.s = skip_utf16(self.s, 1);
                    self.s = trim_leading_space(self.s);
                } else {
                    self.tex_stack.push(Op {
                        prec: 12,
                        pos: self.tex.len(),
                        ttype: Tt::Binary,
                        close_delim: "}{".into(),
                    });
                    self.tex.push_str(&self.token.output);
                    self.tex.push('{');
                }
                if is_calc && self.token.input == "root" {
                    self.push_rpn(rpn_prec(Tt::Binary), self.token.output.clone());
                }
                self.top_delim()?.is_tall = true;
                self.ok_to_append = false;
            }

            Tt::Currency => {
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                self.tex_stack.push(Op {
                    prec: 12,
                    pos: self.tex.len(),
                    ttype: Tt::Currency,
                    close_delim: String::new(),
                });
                if is_calc {
                    let symbol = format!("applyUnit{}{}", self.token_sep, self.token.input);
                    self.push_rpn(rpn_prec(Tt::Currency), symbol);
                    if self.prev_token.input == "⌧" {
                        self.tex.push('×');
                    }
                }
                self.tex.push_str(&self.token.output);
                self.ok_to_append = false;
            }

            Tt::Environment => {
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                self.tex.push_str(&self.token.output);
                if VERBATIMS.contains(&self.token.input.as_str())
                    || (self.token.input == "\\bordermatrix" && self.s.starts_with('['))
                {
                    let arg = verbatim_arg(self.s);
                    if self.token.output == "\\bordermatrix" {
                        self.tex.push_str(&format!("[{arg}]"));
                    } else {
                        self.tex.push_str(&format!("{{{arg}}}"));
                    }
                    self.s = skip_utf16(self.s, utf16_len(&arg) + 2);
                }
                if self.token.output == "\\bordermatrix" {
                    self.tex.push('{');
                }
                self.s = skip_utf16(self.s, 1);
                self.s = trim_leading_space(self.s);
                self.tex_stack.push(Op {
                    prec: 0,
                    pos: self.tex.len(),
                    ttype: Tt::Environment,
                    close_delim: self.token.close_delim.clone(),
                });
                self.delims.push(Delim {
                    name: self.token.input.clone(),
                    delim_type: DelimType::Environment,
                    is_tall: true,
                    open: self.token.output.clone(),
                    close: self.token.close_delim.clone(),
                    num_args: 1,
                    num_rows: 1,
                    rpn_pos: 0,
                    is_preceded_by_div: self.prev_token.ttype == Tt::Div,
                    is_control_word_paren: true,
                });
                self.ok_to_append = true;
            }

            Tt::Unary => {
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                if VERBATIMS.contains(&self.token.input.as_str()) {
                    let arg = verbatim_arg(self.s);
                    self.tex
                        .push_str(&format!("{}{{{}}}", self.token.output, arg));
                    self.s = skip_utf16(self.s, utf16_len(&arg) + 2);
                    self.s = trim_leading_space(self.s);
                    self.token.ttype = Tt::RightBracket;
                    self.ok_to_append = true;
                } else {
                    self.tex_stack.push(Op {
                        prec: 12,
                        pos: self.tex.len(),
                        ttype: Tt::Unary,
                        close_delim: "}".into(),
                    });
                    if is_calc {
                        self.push_rpn(rpn_prec(Tt::Unary), self.token.input.clone());
                        if self.prev_token.input == "⌧" {
                            self.tex.push('×');
                        }
                    }
                    self.tex.push_str(&self.token.output);
                    if (self.token.input.contains("det") || self.token.input.contains("inf"))
                        && self.s.starts_with('_')
                    {
                        self.tex_stack.push(Op {
                            prec: 15,
                            pos: self.tex.len(),
                            ttype: Tt::Sub,
                            close_delim: "}".into(),
                        });
                        self.token = Token::new("_", "_", Tt::Sub);
                        self.tex.push_str("_{");
                        self.s = self.s[1..].trim_start_matches(is_js_space);
                    } else {
                        self.tex.push('{');
                    }
                    self.top_delim()?.is_tall = true;
                    self.ok_to_append = false;
                }
            }

            Tt::Factorial => {
                self.pop_tex_tokens(12, true)?;
                self.tex_stack.push(Op {
                    prec: 12,
                    pos: self.op.pos,
                    ttype: Tt::Factorial,
                    close_delim: String::new(),
                });
                if is_calc {
                    self.pop_rpn_tokens(rpn_prec(Tt::Factorial));
                    self.rpn.push_str(self.token_sep);
                    self.rpn.push_str(&self.token.output);
                }
                self.tex.push_str(&self.token.output);
                self.ok_to_append = true;
            }

            Tt::Return => {
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.pos_of_prev_run = self.tex.len();
                if is_calc {
                    self.pop_rpn_tokens(4);
                    self.push_rpn(4, "return");
                }
                self.tex.push_str(&self.token.output);
                self.tex.push(' ');
            }

            Tt::Keyword => {
                self.pop_tex_tokens(1, true)?;
                self.pos_of_prev_run = self.tex.len();
                if is_calc {
                    self.pop_rpn_tokens(2);
                }
                self.tex.push_str(&self.token.output);
                self.tex.push(' ');
            }

            Tt::Logic => {
                self.pop_tex_tokens(1, self.ok_to_append)?;
                if is_calc {
                    self.rpn.push_str(self.token_sep);
                }
                self.pop_rpn_tokens(4);
                let input = self.token.input.clone();
                let is_if_or_otherwise = input == "if" || input == "otherwise";
                let token_sep = self.token_sep;
                let di = self
                    .delims
                    .len()
                    .checked_sub(1)
                    .ok_or_else(|| err("unbalanced delimiters"))?;
                if is_if_or_otherwise
                    && self.delims[di].delim_type == DelimType::Paren
                    && self.delims[di].name == "{"
                {
                    // The enclosing braces are a CASES expression.
                    let d = &mut self.delims[di];
                    d.delim_type = DelimType::Cases;
                    d.close = "\\end{cases}".into();
                    d.open = "\\begin{cases}".into();
                    // For lazy evaluation, the expressions move after the conditions.
                    // Temporarily change the token separator.
                    if is_calc && token_sep == NBSP && input == "if" {
                        let rp = d.rpn_pos.min(self.rpn.len());
                        self.rpn =
                            format!("{}{}", &self.rpn[..rp], self.rpn[rp..].replace(NBSP, "§"));
                    }
                }
                let is_cases = self.delims[di].delim_type == DelimType::Cases;
                if is_cases && is_if_or_otherwise {
                    self.tex.push_str(&format!("& \\text{{{input}}}~ "));
                } else {
                    self.tex.push_str(&self.token.output);
                }
                if is_calc {
                    if is_cases && is_if_or_otherwise {
                        // The end of an expression in an If expression.
                        let rpn = std::mem::take(&mut self.rpn);
                        self.rpn =
                            set_up_if(rpn, &input, &mut self.expr_stack, &mut self.delims[di]);
                        self.token_sep = NBSP;
                    } else {
                        self.push_rpn(4, input);
                    }
                }
                self.pos_of_prev_run = self.tex.len();
                self.ok_to_append = true;
            }

            Tt::LeftBracket => self.left_bracket()?,
            Tt::Sep => self.separator()?,
            Tt::RightBracket => self.right_bracket()?,
            Tt::LeftRight => self.left_right()?,

            Tt::Comment => {
                self.pop_tex_tokens(0, true)?;
                self.tex.push_str(&self.token.output);
                self.tex.push(' ');
            }

            _ => {
                if is_calc {
                    self.rpn.push_str(self.token_sep);
                    self.pop_rpn_tokens(rpn_prec(Tt::Ord));
                }
                self.pop_tex_tokens(1, self.ok_to_append)?;
                self.tex_stack.push(Op {
                    prec: 1,
                    pos: self.tex.len(),
                    ttype: Tt::Ord,
                    close_delim: String::new(),
                });
                if is_calc {
                    self.push_rpn(rpn_prec(Tt::Ord), self.token.output.clone());
                }
                self.tex.push_str(&self.token.output);
                self.tex.push(' ');
                self.pos_of_prev_run = self.tex.len();
                self.ok_to_append = true;
            }
        }
        Ok(())
    }

    fn left_bracket(&mut self) -> Result<(), Error> {
        self.pop_tex_tokens(2, self.ok_to_append)?;
        let is_preceded_by_div = self.prev_token.ttype == Tt::Div;
        let mut is_func_paren = false;
        let input = self.token.input.clone();
        let mut item = Op {
            prec: 0,
            pos: self.tex.len(),
            ttype: Tt::LeftBracket,
            close_delim: self.token.close_delim.clone(),
        };

        if (input == "(" || input == "[") && self.prev_token.ttype.lt5() {
            // The delimiters delimit a TeX function's extent. Make them invisible.
            item.close_delim = String::new();
        } else if input == "(" && self.op.ttype == Tt::Binary {
            item.close_delim = String::new();
        } else {
            is_func_paren = (input == "(" || input == "[") && self.prev_token.ttype == Tt::Function;
            self.tex.push_str(&self.token.output);
        }
        let close = item.close_delim.clone();
        self.tex_stack.push(item);

        if self.is_calc {
            while self.rpn_stack.last().is_some_and(|o| o.symbol == ".") {
                let sym = self.pop_rpn()?.symbol;
                self.rpn.push_str(self.token_sep);
                self.rpn.push_str(&sym);
            }
            self.push_rpn(0, js_trim(&self.token.output).to_string());
        }

        let num_args = if self
            .s
            .trim_start_matches(is_js_space)
            .starts_with([')', '}', ']'])
        {
            0
        } else {
            1
        };

        let mut delim = Delim {
            name: input.clone(),
            delim_type: DelimType::Paren,
            is_tall: false,
            open: self.token.output.clone(),
            close,
            num_args,
            num_rows: num_args,
            rpn_pos: self.rpn.len(),
            is_preceded_by_div,
            is_control_word_paren: self.prev_token.ttype.lt5(),
        };

        if is_func_paren {
            delim.delim_type = DelimType::Function;
            delim.name = self.pending_function_name.clone();
        } else if self.prev_token.ttype == Tt::Sub {
            delim.delim_type = DelimType::Subscript;
            delim.name = "(".into();
        } else if input == "{" {
            // This may change to a CASES.
            delim.delim_type = DelimType::Paren;
        } else if input == "["
            && (matches!(
                self.prev_token.ttype,
                Tt::Var | Tt::LongVar | Tt::String | Tt::Property
            ) || self.prev_token.input == "]"
                || (self.prev_token.input == ")" && !self.is_preceded_by_space))
        {
            self.rpn.push_str(self.token_sep);
            delim.delim_type = DelimType::Accessor;
        } else {
            // This may change to a MATRIX, but for now it's a paren.
            delim.delim_type = DelimType::Paren;
        }
        self.delims.push(delim);

        self.pending_function_name = String::new();
        self.pos_of_prev_run = self.tex.len();
        self.ok_to_append = false;
        Ok(())
    }

    fn separator(&mut self) -> Result<(), Error> {
        // A comma, a tab or a semicolon. Colons are handled elsewhere.
        self.pop_tex_tokens(1, self.ok_to_append)?;
        self.pos_of_prev_run = self.tex.len();
        let input = self.token.input.clone();
        let n_delims = self.delims.len();
        let follow_nl = self.is_followed_by_space_or_newline;
        {
            let delim = self.top_delim()?;
            if matches!(delim.delim_type, DelimType::Paren | DelimType::Accessor)
                && (input == ";" || input == "\t" || (input == "," && follow_nl))
            {
                delim.delim_type = if delim.delim_type == DelimType::Paren {
                    DelimType::Matrix
                } else {
                    DelimType::Accessor
                };
                let ch = match delim.name.as_str() {
                    "[" => "b",
                    "(" => "p",
                    "{:" => "",
                    "|" => "v",
                    "‖" => "V",
                    _ => "B",
                };
                delim.open = format!("\\begin{{{ch}matrix}}");
                delim.close = format!("\\end{{{ch}matrix}}");
                for i in (1..n_delims).rev() {
                    self.delims[i].is_tall = true;
                }
            }
        }
        let delim_type = self.top_delim()?.delim_type;
        if self.is_calc {
            if self.prev_token.ttype == Tt::LeftBracket && delim_type == DelimType::Accessor {
                self.rpn.push_str("®0/1");
            }
            self.rpn.push_str(self.token_sep);
            self.pop_rpn_tokens(1);
        }
        if matches!(
            delim_type,
            DelimType::Matrix | DelimType::Cases | DelimType::Environment
        ) && input == ","
        {
            self.token.output = " & ".into();
        }
        if input == "," && delim_type == DelimType::Subscript {
            self.tex.push(',');
        } else {
            self.tex.push_str(&self.token.output);
            self.tex.push(' ');
        }

        if self.is_calc {
            if self.delims.len() == 1 {
                if input == "," {
                    self.num_free_commas += 1; // an item in a tuple
                }
            } else {
                let token_sep = self.token_sep;
                let next_is_close_bracket = self.s.starts_with(']');
                let di = self.delims.len() - 1;
                if input == ";" {
                    self.delims[di].num_rows += 1;
                    if self.delims[di].delim_type == DelimType::Cases {
                        // We're about to begin an expression inside an If expression.
                        self.token_sep = "§";
                    }
                }
                if self.delims[di].num_rows == 1 {
                    if (input == "," || input == "\t") && next_is_close_bracket {
                        self.rpn.push_str("®0/1");
                    }
                    if input == ","
                        && self.delims[di].delim_type == DelimType::Function
                        && self.delims[di].num_args == 2
                        && self.delims[di].name == "plot"
                    {
                        // The literal function of a plot() statement inside draw().
                        let a = char_offset(&self.rpn, self.delims[di].rpn_pos, 5);
                        let mut body = self.rpn[a..].to_string();
                        drop_last_char(&mut body);
                        self.rpn = format!(
                            "{}\"{}\"{}",
                            &self.rpn[..a],
                            body.replace(NBSP, "§"),
                            token_sep
                        );
                    }
                }
                self.delims[di].num_args += 1;
            }
        }
        self.ok_to_append = true;
        Ok(())
    }

    fn right_bracket(&mut self) -> Result<(), Error> {
        self.pop_tex_tokens(0, true)?;
        let mut top = self
            .delims
            .pop()
            .ok_or_else(|| err("unbalanced delimiters"))?;

        if top.delim_type == DelimType::Paren
            && !top.is_control_word_paren
            && top.close != self.token.output
        {
            // Enable unmatched delims, such as (1.2] or |ϕ⟩
            let n = utf16_len(&top.close);
            self.tex = if n == 0 {
                // JS: tex.slice(0, -0) is the empty string.
                self.token.output.clone()
            } else {
                let keep = utf16_len(&self.tex).saturating_sub(n);
                let mut t: String = take_utf16(&self.tex, keep).to_string();
                t.push_str(&self.token.output);
                t
            };
        }

        if top.is_tall && self.delims.len() > 1 {
            self.top_delim()?.is_tall = true;
        }

        if self.is_calc {
            while self.rpn_stack.last().is_some_and(|o| o.prec > 0) {
                let sym = self.pop_rpn()?.symbol;
                self.rpn.push_str(self.token_sep);
                self.rpn.push_str(&sym);
            }
            // Only when the last row has no condition of its own (`{a if c; b}`; each row counts its
            // value and its condition): with one
            // (`{a if c; b if d}`) Hurmet added an `otherwise` anyway and lost the last
            // comparison (ADR-0025).
            if top.delim_type == DelimType::Cases
                && self.prev_token.input != "otherwise"
                && top.num_args < 2 * top.num_rows
            {
                // "otherwise" is optional. Run setUpIf as if it were present, with the
                // separator a Logic token pushes first (Hurmet didn't, so the value lost its
                // last character: `{1 if c; 25}` gave `®25/`).
                self.rpn.push_str(self.token_sep);
                let rpn = std::mem::take(&mut self.rpn);
                self.rpn = set_up_if(rpn, "otherwise", &mut self.expr_stack, &mut top);
                self.token_sep = NBSP;
            }
            let rpn_op = self.rpn_stack.pop();
            let num_args = top.num_args;
            let num_rows = top.num_rows;
            let num_cols = js_number(num_args as f64 / num_rows as f64);
            let sep = self.token_sep;
            let first_sep = if num_args == 0 { "" } else { sep };

            'sw: {
                match top.delim_type {
                    DelimType::Function => {
                        let mut symbol = self.pop_rpn()?.symbol;
                        let ends_bang = self.rpn.ends_with(&format!("{sep}!"));
                        if num_args == 2 {
                            match symbol.as_str() {
                                "log" => symbol = "logn".into(),
                                "round" => symbol = "roundn".into(),
                                "atan" => symbol = "atan2".into(),
                                "plot" => {
                                    let a = char_offset(&self.rpn, 0, 6);
                                    self.rpn = format!(
                                        "{}\"{}\"",
                                        &self.rpn[..a],
                                        self.rpn[a..].replace(NBSP, "§")
                                    );
                                }
                                _ => {}
                            }
                        } else if symbol == "log" && ends_bang {
                            drop_last_char(&mut self.rpn);
                            self.rpn.push_str("lfact");
                            break 'sw;
                        } else if symbol == "isnan" && num_args == 1 {
                            // Mark the start of the argument so the evaluator can catch its error.
                            let rp = top.rpn_pos.min(self.rpn.len());
                            let arg = self.rpn[rp..].to_string();
                            let marked = if arg.starts_with(sep) {
                                format!("{sep}{ISNAN_MARKER}{arg}")
                            } else {
                                format!("{ISNAN_MARKER}{sep}{arg}")
                            };
                            self.rpn.truncate(rp);
                            self.rpn.push_str(&marked);
                        }
                        let tail = if symbol.ends_with('^')
                            || BUILT_IN_FUNCTIONS.contains(&symbol.as_str())
                        {
                            format!("{first_sep}{symbol}")
                        } else if BUILT_IN_REDUCER_FUNCTIONS.contains(&symbol.as_str()) {
                            format!("{first_sep}{symbol}{sep}{num_args}")
                        } else {
                            format!("{first_sep}function{sep}{symbol}{sep}{num_args}")
                        };
                        self.rpn.push_str(&tail);
                    }
                    DelimType::Accessor => {
                        self.rpn.push_str(&format!("{first_sep}[]{sep}{num_args}"));
                    }
                    DelimType::Matrix => {
                        self.rpn
                            .push_str(&format!("{first_sep}matrix{sep}{num_rows}{sep}{num_cols}"));
                    }
                    DelimType::Cases => {
                        self.token_sep = NBSP;
                        let sep = NBSP;
                        self.rpn
                            .push_str(&format!("{sep}cases{sep}{num_rows}{sep}"));
                        for e in self.expr_stack.drain(..) {
                            self.rpn.push_str(&e);
                        }
                        drop_last_char(&mut self.rpn);
                    }
                    DelimType::VectorFromRange => {
                        self.rpn.push_str(&format!("{sep}matrix{sep}1{sep}1"));
                    }
                    DelimType::Distrib => {}
                    _ => {
                        if num_args == 0 && top.open == "[" {
                            // An empty matrix.
                            self.rpn.push_str(&format!("matrix{sep}0{sep}0"));
                        } else if num_args == 1 && top.open == "[" {
                            self.rpn.push_str(&format!("{sep}matrix{sep}1{sep}1"));
                        }
                        let rpn_op = rpn_op.ok_or_else(|| err("RPN operator stack underflow"))?;
                        if rpn_op.symbol == "\\lfloor" {
                            self.rpn.push_str(&format!("{sep}⎿⏌"));
                        }
                        if rpn_op.symbol == "\\lceil" {
                            self.rpn.push_str(&format!("{sep}⎾⏋"));
                        }
                    }
                }
            }
            let implicit = (self.token.input == ")"
                && !(top.delim_type == DelimType::Function
                    && self.s.starts_with('[')
                    && !self.is_followed_by_space)
                && next_char_is_factor(self.s, Tt::RightBracket, self.is_followed_by_space))
                || (self.token.input == "]" && self.s.starts_with('('))
                || (top.delim_type == DelimType::Matrix && self.s.starts_with('['));
            if implicit {
                // Implicit multiplication between parens, as in (2)(3).
                self.rpn.push_str(self.token_sep);
                self.pop_rpn_tokens(rpn_prec(Tt::Mult));
                self.push_rpn(rpn_prec(Tt::Mult), "⌧");
                self.is_followed_by_space = false;
                self.token = Token::new("⌧", "⌧", Tt::Mult);
            }
        }
        self.pos_of_prev_run = self.tex.len();
        self.ok_to_append = self.op.ttype != Tt::Binary;
        Ok(())
    }

    fn left_right(&mut self) -> Result<(), Error> {
        // "|" or "‖", as in |x|, ‖M‖, P(A|B), {x|x ∈ℝ}, |ϕ⟩
        self.pop_tex_tokens(1, self.ok_to_append)?;
        let bar = if self.token.input == "|" { "|" } else { "‖" };
        let is_right_delim = self.tex_stack.last().is_some_and(|t| {
            t.ttype == Tt::LeftRight || t.close_delim == "\u{27E9}" || t.close_delim == "\\right."
        });
        let (top_type, top_name) = {
            let d = self.top_delim()?;
            (d.delim_type, d.name.clone())
        };
        if is_right_delim {
            if top_type != DelimType::Matrix {
                self.top_delim()?.close = bar.into();
                if let Some(t) = self.tex_stack.last_mut() {
                    t.close_delim = bar.into();
                }
            }
            self.pop_tex_tokens(0, self.ok_to_append)?;
            self.delims.pop();
            if self.is_calc {
                while self.rpn_stack.last().is_some_and(|o| o.prec > 0) {
                    let sym = self.pop_rpn()?.symbol;
                    self.rpn.push_str(self.token_sep);
                    self.rpn.push_str(&sym);
                }
                let sym = self.pop_rpn()?.symbol;
                self.rpn.push_str(self.token_sep);
                self.rpn.push_str(&sym);
            }
            self.ok_to_append = self.op.ttype != Tt::Binary;
        } else if top_type == DelimType::Paren && top_name == "{" {
            self.tex.push_str("\\mid ");
            self.pos_of_prev_run = self.tex.len();
            self.ok_to_append = true;
        } else {
            // A left delimiter.
            self.tex_stack.push(Op {
                prec: 0,
                pos: self.tex.len(),
                ttype: Tt::LeftRight,
                close_delim: bar.into(),
            });
            self.delims.push(Delim {
                name: self.token.input.clone(),
                delim_type: DelimType::Paren,
                is_tall: false,
                open: bar.into(),
                close: bar.into(),
                num_args: 1,
                num_rows: 1,
                rpn_pos: self.rpn.len(),
                is_preceded_by_div: self.prev_token.ttype == Tt::Div,
                is_control_word_paren: false,
            });
            if self.is_calc {
                self.push_rpn(0, self.token.output.clone());
            }
            self.tex.push_str(bar);
            self.pos_of_prev_run = self.tex.len();
            self.ok_to_append = false;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Output, Error> {
        self.pop_tex_tokens(0, true)?; // Pop all the remaining close delimiters.

        let mut index_variable = String::new();
        if self.is_calc {
            while let Some(op) = self.rpn_stack.pop() {
                self.rpn.push_str(self.token_sep);
                self.rpn.push_str(&op.symbol);
            }
            while let Some(c) = SUM_RE.captures(&self.rpn) {
                // A ∑_(i=0)^n … term. Edit the index variable and the local RPN.
                let whole = c.get(0).map_or(0..0, |m| m.range());
                index_variable = c[1].to_string();
                let replacement = format!(
                    "\"{}\"{}{}{}\"{}\"\u{a0}∑",
                    &c[1],
                    &c[2],
                    &c[3],
                    self.token_sep,
                    c[4].replace(NBSP, "§")
                );
                self.rpn.replace_range(whole, &replacement);
            }
            if self.num_free_commas > 0 {
                let sep = self.token_sep;
                self.rpn
                    .push_str(&format!("{sep}tuple{sep}{}", self.num_free_commas + 1));
            }
            // Close each 〖 variable marker with 〗, unless it is a method call.
            let mut start = 0;
            while let Some(m) = VAR_RE.find_at(&self.tex, start) {
                let second = m.as_str()['〖'.len_utf8()..].chars().next().unwrap_or(' ');
                let (m_start, m_end) = (m.start(), m.end());
                if "¨ˆˉ˙˜".contains(second) {
                    start = m_end;
                    continue;
                }
                if self.tex[m_end..].starts_with('(') {
                    // A method, not a data index. Delete the 〖.
                    self.tex
                        .replace_range(m_start..m_start + '〖'.len_utf8(), "");
                    // JS keeps lastIndex, which now points one char past the match end.
                    let new_end = m_end - '〖'.len_utf8();
                    start = next_char_boundary(&self.tex, new_end);
                } else {
                    self.tex.insert(m_end, '〗');
                    start = m_end;
                }
            }
        }

        let mut tex = collapse_spaces(&self.tex);
        tex = delete_spaces_before_right_delims(&tex);
        tex.truncate(tex.trim_end_matches(is_js_space).len());
        if !index_variable.is_empty() {
            tex = tex.replace(&format!("〖{index_variable}〗"), &index_variable);
        }
        if self.must_align {
            let pos = tex.find('=').unwrap_or(0);
            tex = format!(
                "\\begin{{aligned}}{}&{}\\end{{aligned}}",
                &tex[..pos],
                &tex[pos..]
            );
        }
        Ok(Output {
            tex,
            rpn: self.rpn,
            dependencies: self.dependencies,
        })
    }
}

fn rpn_prec_of_sup() -> i32 {
    rpn_prec(Tt::Sup)
}

fn starts_with_atop_space(s: &str) -> bool {
    s.strip_prefix("\\atop")
        .is_some_and(|r| r.starts_with(is_js_space))
}

/// Byte offset of `base + n` chars (JS `rpnPos + n`).
fn char_offset(s: &str, base: usize, n: usize) -> usize {
    let mut p = base.min(s.len());
    for _ in 0..n {
        p = next_char_boundary(s, p);
    }
    p
}

fn take_utf16(s: &str, n: usize) -> &str {
    let rest = skip_utf16(s, n);
    &s[..s.len() - rest.len()]
}

/// `/ {2,}/g` → " "
fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

/// `/\s+(?=[_^'!)}\]〗])/g` → ""
fn delete_spaces_before_right_delims(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending = String::new();
    for c in s.chars() {
        if is_js_space(c) {
            pending.push(c);
        } else {
            if !matches!(c, '_' | '^' | '\'' | '!' | ')' | '}' | ']' | '〗') {
                out.push_str(&pending);
            }
            pending.clear();
            out.push(c);
        }
    }
    out.push_str(&pending);
    out
}

pub(crate) fn parse(
    input: &str,
    formats: &Formats,
    is_calc: bool,
    in_real_time: bool,
    sheet_name: &str,
) -> Result<Output, Error> {
    parse_at_depth(input, formats, is_calc, in_real_time, sheet_name, 0)
}

fn parse_at_depth(
    input: &str,
    formats: &Formats,
    is_calc: bool,
    in_real_time: bool,
    sheet_name: &str,
    depth: usize,
) -> Result<Output, Error> {
    // Trim leading white space (except \n, \t, nbsp) and trailing white space.
    let s = trim_leading_space(input);
    let s = s.trim_end_matches(is_js_space);
    let mut p = Parser {
        depth,
        formats,
        is_calc,
        in_real_time,
        sheet_name,
        s,
        tex: String::new(),
        rpn: String::new(),
        token: start_token(),
        prev_token: start_token(),
        dependencies: Vec::new(),
        must_align: false,
        pos_of_prev_run: 0,
        is_preceded_by_space: false,
        is_followed_by_space: false,
        is_followed_by_space_or_newline: false,
        followed_by_factor: false,
        op: Op {
            prec: i32::MIN,
            pos: 0,
            ttype: Tt::Start,
            close_delim: String::new(),
        },
        tex_stack: Vec::new(),
        rpn_stack: Vec::new(),
        delims: vec![Delim::base()],
        ok_to_append: true,
        pending_function_name: String::new(),
        token_sep: NBSP,
        expr_stack: Vec::new(),
        num_free_commas: 0,
        pos_arrow: 0,
    };
    p.run()?;
    p.finish()
}
