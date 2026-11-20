// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! The lexer (Hurmet's `lexer.js`). [`lex`] returns the next token of a string.
//!
//! Hurmet's token tables carry a third column, the "calc output", which `parser.js` never reads,
//! so it is not ported.

use crate::Formats;
use crate::dataframe::quick_display;
use crate::date::date_tex;
use crate::number::{NUMBER_RE, tex_from_num_parts};
use crate::utils::{add_text_escapes, first_char, skip_utf16, unit_tex_from_string, utf16_len};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Token types. The discriminants match Hurmet's `tt`, because the parser indexes its
/// precedence tables with them and compares some of them numerically.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Tt {
    /// Unary TeX function, e.g. `\sqrt`.
    Unary = 0,
    /// Binary TeX function, e.g. `\frac` or `root`.
    Binary = 1,
    Sub = 2,
    Sup = 3,
    Accent = 4,
    UnaryMinus = 5,
    /// Stacked division: `/` `//` `\atop`.
    Div = 6,
    Prime = 7,
    Currency = 8,
    Ord = 9,
    /// A one-letter variable name.
    Var = 10,
    Num = 11,
    Space = 12,
    LongVar = 13,
    LeftBracket = 14,
    RightBracket = 15,
    BigOperator = 16,
    /// `|` or `‖`.
    LeftRight = 17,
    String = 18,
    Unit = 19,
    /// Binary infix operators that render but don't calculate, e.g. `±`.
    Bin = 20,
    Add = 21,
    Mult = 22,
    Rel = 23,
    Logic = 24,
    Sep = 25,
    Function = 26,
    Accessor = 28,
    Environment = 29,
    Factorial = 30,
    SupChar = 31,
    Angle = 32,
    Range = 33,
    Keyword = 34,
    Property = 36,
    Comment = 37,
    Return = 38,
    To = 39,
    DataFrame = 40,
    RichText = 41,
    Boolean = 42,
    Date = 44,
    /// The type of the sentinel "previous token" before the first token (Hurmet uses 50).
    Start = 50,
    /// Hurmet's table gives `⋆` the type `"tt.BIN"`, a string, by mistake. It behaves as an
    /// unknown type: the parser's default branch, and `NaN` in numeric comparisons.
    Bogus = 98,
    /// Hurmet's table entry for `⇒` is missing a column, so its type is `""`. It behaves as an
    /// unknown type in `switch`, but as `0` in `ttype < 5`.
    Empty = 99,
}

impl Tt {
    /// Hurmet's `prevToken.ttype < 5`, with JavaScript coercions.
    pub(crate) fn lt5(self) -> bool {
        match self {
            Tt::Bogus => false,
            Tt::Empty => true,
            t => (t as u8) < 5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub input: String,
    pub output: String,
    pub ttype: Tt,
    pub close_delim: String,
}

impl Token {
    pub(crate) fn new(input: impl Into<String>, output: impl Into<String>, ttype: Tt) -> Token {
        Token {
            input: input.into(),
            output: output.into(),
            ttype,
            close_delim: String::new(),
        }
    }
    fn with_close(mut self, close: &str) -> Token {
        self.close_delim = close.to_string();
        self
    }
}

fn is_unary(prev: &Token) -> bool {
    !matches!(
        prev.ttype,
        Tt::Num
            | Tt::Ord
            | Tt::Var
            | Tt::Date
            | Tt::RightBracket
            | Tt::LongVar
            | Tt::Unit
            | Tt::Currency
            | Tt::SupChar
            | Tt::Prime
            | Tt::Factorial
    )
}

// utils.js isValidIdentifier, without the leading ^ and the trailing ′*$.
const IDENTIFIER_START: &str = r"(?:[A-Za-zıȷ\x{0391}-\x{03C9}\x{03D5}\x{210B}\x{210F}\x{2110}\x{2112}\x{2113}\x{211B}\x{212C}\x{2130}\x{2131}\x{2133}]|[\x{1D400}-\x{1D433}\x{1D49C}-\x{1D4B5}])";
const IDENTIFIER_BODY: &str = r"(?:[A-Za-z0-9\x{0391}-\x{03C9}\x{03D5}]+|[\x{0300}-\x{0308}\x{030A}\x{030C}\x{0332}\x{20d0}\x{20d1}\x{20d6}\x{20d7}\x{20e1}])?(?:(?:_(?:[A-Za-z0-9\x{0391}-\x{03C9}\x{03D5}]+|\([A-Za-z0-9\x{0391}-\x{03C9}\x{03D5},]+\))){1,2}|[₀-₉]+)?";

static WORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^(?:{IDENTIFIER_START}{IDENTIFIER_BODY}|!in|-->|->|left\.|right\.)"
    ))
    .unwrap()
});

static UNIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^(?:'[^']+'|[°\u{2126}\u{212B}\u{212A}])").unwrap());
static DATE_LITERAL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^'\d{4}-\d{1,2}-\d{1,2}'").unwrap());
static TEXT_CONTENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(?:[^"]|"")*""#).unwrap());
static TEX_FUNCTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\\[A-Za-z]+\.?|\\([:.! \x{220F}-\x{2211}\x{222B}-\x{2230}]|'+))").unwrap()
});
static SUPER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("^⁻?[²³¹⁰⁴⁵⁶⁷⁸⁹]+").unwrap());

/// Support a unit name written without a space after a number, as in `5m`.
pub(crate) fn unit_start(s: &str) -> bool {
    s.starts_with("ʹʹ")
        || s.starts_with(|c: char| {
            c.is_ascii_alphabetic() || "°′″$£¥₨₪€ÅΩ℃℉\u{2126}\u{212A}\u{212B}".contains(c)
        })
}

static UNIT_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    let unit_and_expo = "(?:(?:[A-Za-z][A-Za-zōö]*\\$?|[°′″$£¥₨₪€ÅΩ℃℉\u{2126}\u{212A}\u{212B}]|ʹʹ)(?:(?:(?:(?:\\^|\\^-)[1-9][0-9]*)|(?:\\^\\(-?[1-9][0-9]*\\))|⁻?[¹²³\u{2074}-\u{2079}][⁰¹²³\u{2074}-\u{2079}]*))?)";
    let product = format!("(?:{unit_and_expo}(?:[*·.•×]{unit_and_expo})*)");
    Regex::new(&format!("^{product}(?:/{product})?")).unwrap()
});

/// Lex a unit name that directly follows a number.
pub(crate) fn lex_unit_name(s: &str) -> Token {
    let m = UNIT_NAME_RE.find(s).map_or("", |m| m.as_str());
    Token::new(m, unit_tex_from_string(m), Tt::Unit)
}

/// `words` in lexer.js: (TeX output, type).
fn word(w: &str) -> Option<(&'static str, Tt)> {
    use Tt::*;
    Some(match w {
        "true" => ("\\mathord{\\text{true}}", Boolean),
        "false" => ("\\mathord{\\text{false}}", Boolean),
        "cos" => ("\\cos", Function),
        "cosd" => ("\\operatorname{\\cos_d}", Function),
        "if" => ("\\mathrel{\\mathrm{if}}", Logic),
        "else" => ("\\mathrel{\\mathrm{else}}", Logic),
        "elseif" => ("\\mathrel{\\mathrm{elseif}}", Logic),
        "and" => ("\\mathrel{\\mathrm{and}}", Logic),
        "or" => ("\\mathrel{\\mathrm{or}}", Logic),
        "for" => ("\\mathrel{\\mathrm{for}}", Keyword),
        "while" => ("\\mathrel{\\mathrm{while}}", Keyword),
        "in" => ("\\mathrel{\\mathrm{in}}", Rel),
        "!in" => ("\\mathrel{\\mathrm{!in}}", Rel),
        "break" => ("\\mathrel{\\mathrm{break}}", Keyword),
        "to" => ("\\mathbin{\\mathrm{to}}", To),
        "throw" => ("\\mathrel{\\mathrm{throw}}", Unary),
        "print" => ("\\mathrel{\\mathrm{print}}", Unary),
        "return" => ("\\mathrel{\\mathrm{return}}", Return),
        "sqrt" => ("\\sqrt", Unary),
        "otherwise" => ("\\mathrel{\\mathrm{otherwise}}", Logic),
        "root" => ("\\sqrt", Binary),
        "modulo" => ("\\mathbin{modulo}", Mult),
        "sin" => ("\\sin", Function),
        "sind" => ("\\operatorname{\\sin_d}", Function),
        "tan" => ("\\tan", Function),
        "tand" => ("\\operatorname{\\tan_d}", Function),
        "cotd" => ("\\operatorname{\\cot_d}", Function),
        "secd" => ("\\operatorname{\\sec_d}", Function),
        "cscd" => ("\\operatorname{\\csc_d}", Function),
        "log" => ("\\log", Function),
        "ln" => ("\\ln", Function),
        "log10" => ("\\log_{10}", Function),
        "log2" => ("\\log_{2}", Function),
        "log!" => ("\\operatorname{log!}", Function),
        "pi" => ("\\mathrm{pi}", Ord),
        "π" => ("π", Ord),
        "ℓ" => ("ℓ", Var),
        "->" => ("\u{2192}", Rel),
        "-->" => ("\\xrightarrow", Unary),
        "<-->" => ("\\xrightleftarrows", Unary),
        _ => return None,
    })
}

const MISC_CHARS: &str = "/÷\u{2215}_:,;\t^+\\-–−*∗×∘⊗⦼⊙⋆√∛∜·.%|╏‖¦><=≈≟≠≡≤≥≅∈∉∋∌⊂⊄⊆⊈⊃⊇⊉!¡‼¬∧∨⊻~#?⇒⟶⟵→←&@′″∀∃([{⟨⌊⎿⌈⎾〖〗⏋⌉⏌⌋⟩}])˽∣ℂℕℚℝℤℓℏ∠¨ˆˉ˙˜▪✓\u{00A0}\u{20D7}$£¥€₨₩₪";

/// `miscSymbols` in lexer.js: (input, TeX output, type, close delimiter).
fn misc_symbol(s: &str) -> Option<(&'static str, &'static str, Tt, &'static str)> {
    use Tt::*;
    let none = "";
    Some(match s {
        "#" => ("#", "#", Comment, none),
        "/" => ("/", "\\dfrac{", Div, none),
        "//" => ("//", "\\tfrac{", Div, none),
        "///" => ("///", "/", Mult, none),
        "\u{2215}" => ("\u{2215}", "\u{2215}", Mult, none),
        "÷" => ("÷", "÷", Mult, none),
        "./" => ("./", "\\mathbin{.'}", Mult, none),
        "_" => ("_", "_", Sub, none),
        "^" => ("^", "^", Sup, none),
        ".^" => (".^", "\\mathbin{.^}", Sup, none),
        "+" => ("+", "+", Add, none),
        "-" | "–" | "−" => ("-", "-", Add, none),
        ".+" => (".+", "\\mathbin{.+}", Add, none),
        ".-" => (".-", "\\mathbin{.-}", Add, none),
        "*" => ("*", "*", Mult, none),
        "∗" => ("∗", "∗", Mult, none),
        "×" => ("×", "×", Mult, none),
        "∘" => ("∘", "∘", Mult, none),
        "⊗" => ("⊗", "⊗", Mult, none),
        ".*" => (".*", "\\mathbin{.*}", Mult, none),
        "√" => ("√", "\\sqrt", Unary, none),
        "\u{221B}" => ("\u{221B}", "\\sqrt[3]", Unary, none),
        "\u{221C}" => ("\u{221C}", "\\sqrt[4]", Unary, none),
        "+-" => ("+-", "\u{00B1}", Bin, none),
        "**" => ("**", "\\star", Bin, none),
        "⋆" => ("⋆", "⋆", Bogus, none),
        "·" => ("·", "\u{22C5}", Mult, none),
        "..." => ("...", "\\dots", Range, none),
        "\\dots" => ("\\dots", "\\dots", Range, none),
        "%" => ("%", "\\%", Factorial, none),
        "-:" => ("-:", "÷", Mult, none),
        "=" => ("=", "=", Rel, none),
        "≈" => ("≈", "≈", Rel, none),
        "==" => ("==", "⩵", Rel, none),
        "≡" => ("≡", "≡", Rel, none),
        ">" => (">", "\\gt", Rel, none),
        "\u{226f}" => ("\u{226f}", "\\ngtr", Rel, none),
        "<" => ("<", "\\lt", Rel, none),
        "\u{226e}" => ("\u{226e}", "\\nless", Rel, none),
        "?=" => ("?=", "\u{225F}", Rel, none),
        "≟" => ("≟", "\u{225F}", Rel, none),
        "≠" => ("≠", "≠", Rel, none),
        "!=" => ("!=", "≠", Rel, none),
        "<>" => ("<>", "≠", Rel, none),
        ":=" => (":=", "\u{2254}", Rel, none),
        "<=" => ("<=", "≤", Rel, none),
        "≤" => ("≤", "≤", Rel, none),
        ">=" => (">=", "\u{2265}", Rel, none),
        "≥" => ("≥", "≥", Rel, none),
        "-=" => ("-=", "\u{2261}", Rel, none),
        "~=" => ("~=", "\u{2245}", Rel, none),
        "≅" => ("≅", "≅", Rel, none),
        "~~" => ("~~", "\u{2248}", Rel, none),
        "~" => ("~", "\\sim", Rel, none),
        "=>" => ("=>", "\u{21D2}", Rel, none),
        "⇒" => ("⇒", "\u{21D2}", Empty, none),
        "⟶" => ("⟶", "\\xrightarrow", Unary, none),
        "⟵" => ("⟵", "\\xleftarrow", Unary, none),
        "<=>" => ("<=>", "\u{21D4}", Rel, none),
        "-<" => ("-<", "\u{227A}", Rel, none),
        ">-" => (">-", "\u{227B}", Rel, none),
        "-<=" => ("-<=", "\u{2AAF}", Rel, none),
        ">-=" => (">-=", "\u{2AB0}", Rel, none),
        "_|_" => ("_|_", "\\bot", Rel, none),
        "|--" => ("|--", "\u{22A2}", Rel, none),
        "|==" => ("|==", "\\models", Rel, none),
        "∈" => ("∈", "∈", Rel, none),
        "∉" => ("∉", "∉", Rel, none),
        "∋" => ("∋", "∋", Rel, none),
        "∌" => ("∌", "∌", Rel, none),
        "⊂" => ("⊂", "⊂", Rel, none),
        "⊃" => ("⊃", "⊃", Rel, none),
        "⊄" => ("⊄", "⊄", Rel, none),
        "⊅" => ("⊅", "⊅", Rel, none),
        "⊆" => ("⊆", "⊆", Rel, none),
        "⊈" => ("⊈", "⊈", Rel, none),
        "⊇" => ("⊇", "⊇", Rel, none),
        "⊉" => ("⊉", "⊉", Rel, none),
        "▪" => ("▪", "\\mathrel{▪}", Rel, none),
        "!" => ("!", "!", Factorial, none),
        "‼" => ("‼", "!!", Factorial, none),
        "!!" => ("!!", "!!", Factorial, none),
        "¡" => ("¡", "¡", Factorial, none),
        "&" => ("&", "\\mathbin{\\&}", Add, none),
        "&_" => ("&_", "\\mathbin{\\underline{\\&}}", Add, none),
        "′" => ("′", "'", Prime, none),
        "″" => ("″", "''", Prime, none),
        "′′" => ("′′", "''", Prime, none),
        "′′′" => ("′′′", "'''", Prime, none),
        "∀" => ("∀", "∀", Logic, none),
        "∃" => ("∃", "∃", Logic, none),
        "∧" => ("∧", "∧", Logic, none),
        "∨" => ("∨", "∨", Logic, none),
        "⊻" => ("⊻", "⊻", Logic, none),
        "¬" => ("¬", "¬", Unary, none),
        "&&" => ("&&", "{\\;\\&\\&\\;}", Logic, none),
        "(" => ("(", "(", LeftBracket, ")"),
        "[" => ("[", "[", LeftBracket, "]"),
        "{" => ("{", "\\{", LeftBracket, "\\}"),
        "{:" => ("{:", "{", LeftBracket, "}"),
        "⟨" => ("⟨", "⟨", LeftBracket, "⟩"),
        ")" => (")", ")", RightBracket, none),
        "]" => ("]", "]", RightBracket, none),
        "}" => ("}", "\\}", RightBracket, none),
        "⟩" => ("⟩", "⟩", RightBracket, none),
        ":}" => (":}", "}", RightBracket, none),
        "|" => ("|", "|", LeftRight, none),
        "||" => ("||", "\\mathbin{||}", Bin, none),
        "\\|" => ("\\|", "‖", LeftRight, none),
        "‖" => ("‖", "‖", LeftRight, none),
        "<<" => ("<<", "\u{27E8}", LeftBracket, "\u{27E9}"),
        ">>" => (">>", "\u{27E9}", RightBracket, none),
        "\u{230A}" => ("\u{230A}", "\\lfloor ", LeftBracket, "\u{230B}"),
        "\u{23BF}" => ("\u{23BF}", "\\lfloor ", LeftBracket, "\u{230B}"),
        "\u{230B}" => ("\u{230B}", "\\rfloor ", RightBracket, none),
        "\u{23CC}" => ("\u{23CC}", "\\rfloor ", RightBracket, none),
        "\u{2308}" => ("\u{2308}", "\\lceil ", LeftBracket, "\u{2309}"),
        "\u{23BE}" => ("\u{23BE}", "\\lceil ", LeftBracket, "\u{2309}"),
        "\u{2309}" => ("\u{2309}", "\\rceil ", RightBracket, none),
        "\u{23CB}" => ("\u{23CB}", "\\rceil ", RightBracket, none),
        "\u{3016}" => ("\u{3016}", "{", LeftBracket, "\u{3017}"),
        "\u{3017}" => ("\u{3017}", "}", RightBracket, "\u{3017}"),
        "¦" => ("¦", "\\mid ", Rel, none),
        "ℂ" => ("ℂ", "\u{2102}", Ord, none),
        "ℕ" => ("ℕ", "\u{2115}", Ord, none),
        "ℚ" => ("ℚ", "\u{211A}", Ord, none),
        "ℝ" => ("ℝ", "\u{211D}", Ord, none),
        "ℤ" => ("ℤ", "\u{2124}", Ord, none),
        "ℏ" => ("ℏ", "ℏ", Ord, none),
        "\u{2192}" => ("\u{2192}", "\u{2192}", Rel, none),
        "\u{2190}" => ("\u{2190}", "\u{2190}", Rel, none),
        ">->" => (">->", "\u{21a3}", Rel, none),
        "->>" => ("->>", "\u{21a0}", Rel, none),
        "|->" => ("|->", "\u{21a6}", Rel, none),
        "<--" => ("<--", "\\xleftarrow", Unary, none),
        "==>" => ("==>", "\\xRightarrow", Unary, none),
        "<==" => ("<==", "\\xLeftarrow", Unary, none),
        "<-->" => ("<-->", "\\xleftrightarrow", Unary, none),
        "<==>" => ("<==>", "\\xLeftrightarrow", Unary, none),
        "\u{2220}" => ("\u{2220}", "\u{2220}", Angle, none),
        "✓" => ("✓", "✓", Ord, none),
        "˽" => ("˽", "~", Space, none),
        "\\;" => ("\\;", ";\\:", Sep, none),
        "…" => ("…", "…", Ord, none),
        ":" => (":", "{:}", Range, none),
        "," => (",", ",\\:", Sep, none),
        "\t" => ("\t", " & ", Sep, none),
        ";" => (";", " \\\\ ", Sep, none),
        "\\\\" => ("\\\\", " \\\\ ", Sep, none),
        "\\cr" => ("\\cr", " \\\\ ", Sep, none),
        "$" => ("$", "\\$", Currency, none),
        "£" => ("£", "£", Currency, none),
        "¥" => ("¥", "¥", Currency, none),
        "€" => ("€", "€", Currency, none),
        "₨" => ("₨", "₨", Currency, none),
        "₩" => ("₩", "₩", Currency, none),
        "₪" => ("₪", "₪", Currency, none),
        _ => return None,
    })
}

/// `texFunctions` in lexer.js: (TeX output, type, close delimiter).
fn tex_function(s: &str) -> Option<(&'static str, Tt, &'static str)> {
    use Tt::*;
    let none = "";
    Some(match s {
        "\\aleph" => ("\u{2135}", Var, none),
        "\\beth" => ("\u{2136}", Var, none),
        "\\gimel" => ("\u{2137}", Var, none),
        "\\daleth" => ("\u{2138}", Var, none),
        "\\ast" => ("∗", Mult, none),
        "\\div" => ("÷", Mult, none),
        "\\times" => ("×", Mult, none),
        "\\pm" => ("±", Bin, none),
        "\\bmod" => ("\\bmod", Mult, none),
        "\\circ" => ("∘", Mult, none),
        "\\nabla" => ("∇", Ord, none),
        "\\otimes" => ("⊗", Mult, none),
        "\\ne" => ("≠", Rel, none),
        "\\cdot" => ("\u{22C5}", Mult, none),
        "\\le" | "\\leq" => ("≤", Rel, none),
        "\\ge" | "\\geq" => ("≥", Rel, none),
        "\\equiv" => ("\u{2261}", Rel, none),
        "\\cong" => ("≅", Rel, none),
        "\\approx" => ("\u{2248}", Rel, none),
        "\\Rightarrow" => ("\u{21D2}", Rel, none),
        "\\forall" => ("∀", Logic, none),
        "\\exists" => ("∃", Logic, none),
        "\\infty" => ("∞", Ord, none),
        "\\sqrt" => ("\\sqrt", Unary, none),
        "\\atop" => ("\\atop{", Div, none),
        "\\cdots" => ("\u{22ef}", Bin, none),
        "\\vdots" => ("\u{22ee}", Bin, none),
        "\\ddots" => ("\u{22f1}", Bin, none),
        "\\iff" => ("\\iff", Logic, none),
        "\\land" => ("\\land", Bin, none),
        "\\lor" => ("\\lor", Bin, none),
        "\\ngtr" => ("\\ngtr", Rel, none),
        "\\nless" => ("\\nless", Rel, none),
        "\\nleq" => ("\\nleq", Rel, none),
        "\\ngeq" => ("\\ngeq", Rel, none),
        "\\in" => ("∈", Rel, none),
        "\\notin" => ("∉", Rel, none),
        "\\subset" => ("⊂", Rel, none),
        "\\subseteq" => ("⊆", Rel, none),
        "\\nsubset" => ("⊄", Rel, none),
        "\\nsubseteq" => ("⊈", Rel, none),
        "\\supset" => ("⊃", Rel, none),
        "\\left." => ("\\left.", LeftBracket, "\\right."),
        "\\right." => ("\\right.", RightBracket, none),
        "\\mod" => ("\\mod", Bin, none),
        "\\lim" => ("\\lim", Ord, none),
        "\\diamond" => ("\\diamond", Ord, none),
        "\\square" => ("\\square", Ord, none),
        "\\star" => ("⋆", Bin, none),
        "\\int" => ("∫", BigOperator, none),
        "\\iint" => ("∬", BigOperator, none),
        "\\iiint" => ("∭", BigOperator, none),
        "\\oint" => ("∮", BigOperator, none),
        "\\oiint" => ("∯", BigOperator, none),
        "\\oiiint" => ("∰", BigOperator, none),
        "\\∫" => ("\\displaystyle∫", BigOperator, none),
        "\\∬" => ("\\displaystyle∬", BigOperator, none),
        "\\∭" => ("\\displaystyle∭", BigOperator, none),
        "\\∮" => ("\\displaystyle∮", BigOperator, none),
        "\\∯" => ("\\displaystyle∯", BigOperator, none),
        "\\∰" => ("\\displaystyle∰", BigOperator, none),
        "\\over" => ("\\dfrac{", Div, none),
        "\\sum" => ("\\displaystyle∑", BigOperator, none),
        "\\∑" => ("\\displaystyle∑", BigOperator, none),
        "\\prod" => ("∏", BigOperator, none),
        "\\∏" => ("\\displaystyle∏", BigOperator, none),
        "\\quad" => ("\\quad", Space, none),
        "\\qquad" => ("\\qquad", Space, none),
        "\\align" => ("\\begin{align}", Environment, "\\end{align}"),
        "\\aligned" => ("\\begin{aligned}", Environment, "\\end{aligned}"),
        "\\alignat" => ("\\begin{alignat}", Environment, "\\end{alignat}"),
        "\\alignedat" => ("\\begin{alignedat}", Environment, "\\end{alignedat}"),
        "\\array" => ("\\begin{array}", Environment, "\\end{array}"),
        "\\darray" => ("\\begin{darray}", Environment, "\\end{darray}"),
        "\\subarray" => ("\\begin{subarray}", Environment, "\\end{subarray}"),
        "\\cases" => ("\\begin{cases}", Environment, "\\end{cases}"),
        "\\dcases" => ("\\begin{dcases}", Environment, "\\end{dcases}"),
        "\\rcases" => ("\\begin{rcases}", Environment, "\\end{rcases}"),
        "\\drcases" => ("\\begin{drcases}", Environment, "\\end{drcases}"),
        "\\smallmatrix" => ("\\begin{smallmatrix}", Environment, "\\end{smallmatrix}"),
        "\\bordermatrix" => ("\\bordermatrix", Environment, "}"),
        "\\equation" => ("\\begin{equation}", Environment, "\\end{equation}"),
        "\\split" => ("\\begin{split}", Environment, "\\end{split}"),
        "\\gather" => ("\\begin{gather}", Environment, "\\end{gather}"),
        "\\gathered" => ("\\begin{gathered}", Environment, "\\end{gathered}"),
        "\\CD" => ("\\begin{CD}", Environment, "\\end{CD}"),
        "\\multline" => ("\\begin{multline}", Environment, "\\end{multline}"),
        _ => return None,
    })
}

fn set(words: &[&'static str]) -> HashSet<&'static str> {
    words.iter().copied().collect()
}

static ACCENTS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set(&[
        "Bbb",
        "Overrightarrow",
        "acute",
        "bar",
        "bm",
        "bold",
        "boldsymbol",
        "breve",
        "check",
        "ddot",
        "dot",
        "frak",
        "grave",
        "hat",
        "mathring",
        "overbrace",
        "overbracket",
        "overgroup",
        "overleftarrow",
        "overleftharpoon",
        "overleftrightarrow",
        "overline",
        "overrightarrow",
        "overrightharpoon",
        "tilde",
        "underbrace",
        "underbracket",
        "undergroup",
        "underleftarrow",
        "underleftrightarrow",
        "underline",
        "underrightarrow",
        "utilde",
        "vec",
        "widecheck",
        "widehat",
        "widetilde",
    ])
});

/// Functions that are already math operators, so they don't need `\operatorname`.
static MATH_OPERATORS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set(&[
        "arccos", "arcsin", "arctan", "arctg", "arcctg", "cos", "cosec", "cosh", "cot", "cotg",
        "coth", "csc", "ctg", "cth", "det", "dim", "exp", "gcd", "lg", "ln", "log", "max", "min",
        "sec", "sin", "sinh", "sh", "sqrt", "sup", "tan", "tanh", "tg", "th",
    ])
});

static COLORS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set(&[
        "blue",
        "firebrick",
        "gray",
        "green",
        "orange",
        "pink",
        "purple",
        "red",
    ])
});

static UNARIES: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set(&[
        "bcancel",
        "boxed",
        "Bra",
        "bra",
        "braket",
        "cancel",
        "ce",
        "clap",
        "color",
        "hphantom",
        "hskip",
        "hspace",
        "kern",
        "Ket",
        "ket",
        "label",
        "llap",
        "longdiv",
        "mathclap",
        "mathbb",
        "mathbf",
        "mathcal",
        "mathfrak",
        "mathit",
        "mathnormal",
        "mathrm",
        "mathscr",
        "mathsf",
        "mathtt",
        "mkern",
        "mskip",
        "not",
        "operatorname",
        "phantom",
        "phase",
        "pu",
        "reflectbox",
        "rlap",
        "sout",
        "tag",
        "text",
        "textbf",
        "textcircled",
        "textit",
        "textmd",
        "textnormal",
        "textrm",
        "textsc",
        "textsf",
        "texttt",
        "textup",
        "vcenter",
        "vphantom",
        "xLeftarrow",
        "xLeftrightarrow",
        "xRightarrow",
        "xcancel",
        "xleftarrow",
        "xleftrightarrow",
        "xleftharpoondown",
        "xleftharpoons",
        "xleftharpoonup",
        "xlongequal",
        "xmapsto",
        "xrightarrow",
        "xrightharpoondown",
        "xrightharpoonup",
        "xrightleftarrows",
        "xrightleftharpoons",
        "xtofrom",
        "xtwoheadleftarrow",
        "xtwoheadrightarrow",
    ])
});

fn greek(s: &str) -> Option<&'static str> {
    Some(match s {
        "alpha" => "α",
        "beta" => "β",
        "chi" => "χ",
        "delta" => "δ",
        "Delta" => "Δ",
        "epsilon" => "ε",
        "varepsilon" => "\u{025B}",
        "eta" => "\u{03B7}",
        "gamma" => "γ",
        "Gamma" => "Γ",
        "iota" => "\u{03B9}",
        "kappa" => "\u{03BA}",
        "lambda" => "λ",
        "Lambda" => "Λ",
        "mu" => "μ",
        "nu" => "\u{03BD}",
        "omega" => "ω",
        "Omega" => "Ω",
        "phi" => "\u{03D5}",
        "varphi" => "\u{03C6}",
        "Phi" => "\u{03A6}",
        "pi" => "π",
        "Pi" => "Π",
        "psi" => "ψ",
        "Psi" => "Ψ",
        "rho" => "ρ",
        "sigma" => "σ",
        "Sigma" => "Σ",
        "tau" => "τ",
        "theta" => "θ",
        "vartheta" => "\u{03D1}",
        "Theta" => "Θ",
        "upsilon" => "\u{03C5}",
        "xi" => "\u{03BE}",
        "Xi" => "\u{039E}",
        "zeta" => "\u{03B6}",
        _ => return None,
    })
}

static BINARIES: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set(&[
        "cancelto",
        "colorbox",
        "dfrac",
        "frac",
        "lower",
        "overset",
        "raisebox",
        "stackrel",
        "textcolor",
        "tfrac",
        "underset",
    ])
});

static TEX_REL: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set(&[
        "Bumpeq",
        "Colonapprox",
        "Coloneq",
        "Coloneqq",
        "Colonsim",
        "Darr",
        "Doteq",
        "Downarrow",
        "Eqcolon",
        "Eqqcolon",
        "Harr",
        "Larr",
        "Leftarrow",
        "Leftrightarrow",
        "Lleftarrow",
        "Longleftarrow",
        "Longleftrightarrow",
        "Longrightarrow",
        "Lrarr",
        "Lsh",
        "Rarr",
        "Rrightarrow",
        "Rsh",
        "Supset",
        "Subset",
        "Uarr",
        "Uparrow",
        "Updownarrow",
        "Vdash",
        "Vvdash",
        "approxeq",
        "asymp",
        "backepsilon",
        "backsim",
        "backsimeq",
        "between",
        "bowtie",
        "bumpeq",
        "circeq",
        "circlearrowleft",
        "circlearrowright",
        "colonapprox",
        "coloneq",
        "coloneqq",
        "colonsim",
        "curlyeqprec",
        "curlyeqsucc",
        "curvearrowleft",
        "curvearrowright",
        "dArr",
        "darr",
        "dashleftarrow",
        "dashrightarrow",
        "dashv",
        "dblcolon",
        "doteq",
        "doteqdot",
        "downarrow",
        "downdownarrows",
        "downharpoonleft",
        "downharpoonright",
        "eqcirc",
        "eqcolon",
        "eqqcolon",
        "eqsim",
        "eqslantgtr",
        "eqslantless",
        "fallingdotseq",
        "frown",
        "geqq",
        "geqslant",
        "gets",
        "gg",
        "ggg",
        "gggtr",
        "gnapprox",
        "gneq",
        "gneqq",
        "gnsim",
        "gt",
        "gtrapprox",
        "gtreqless",
        "gtreqqless",
        "gtrless",
        "gtrsim",
        "gvertneqq",
        "hArr",
        "harr",
        "hookleftarrow",
        "hookrightarrow",
        "impliedby",
        "implies",
        "isin",
        "Join",
        "lArr",
        "larr",
        "leadsto",
        "leftarrow",
        "leftarrowtail",
        "leftharpoondown",
        "leftharpoonup",
        "leftleftarrows",
        "leftrightarrow",
        "leftrightarrows",
        "leftrightharpoons",
        "leftrightsquigarrow",
        "leqq",
        "leqslant",
        "lessapprox",
        "lesseqgtr",
        "lesseqqgtr",
        "lessgtr",
        "lesssim",
        "ll",
        "lll",
        "llless",
        "lnapprox",
        "lneq",
        "lneqq",
        "lnsim",
        "longleftarrow",
        "longleftrightarrow",
        "longmapsto",
        "longrightarrow",
        "looparrowleft",
        "looparrowright",
        "lrArr",
        "lrarr",
        "lt",
        "lvertneqq",
        "mapsto",
        "mid",
        "models",
        "multimap",
        "nLeftarrow",
        "nLeftrightarrow",
        "nRightarrow",
        "nVDash",
        "nVdash",
        "ncong",
        "nearrow",
        "neq",
        "nexists",
        "ngeq",
        "ngeqq",
        "ngeqslant",
        "ngtr",
        "ni",
        "nleftarrow",
        "nleftrightarrow",
        "nleq",
        "nleqq",
        "nleqslant",
        "nless",
        "nmid",
        "notni",
        "nparallel",
        "nprec",
        "npreceq",
        "nrightarrow",
        "nshortmid",
        "nshortparallel",
        "nsim",
        "nsubseteq",
        "nsubseteqq",
        "nsucc",
        "nsucceq",
        "nsupseteq",
        "nsupseteqq",
        "ntriangleleft",
        "ntrianglelefteq",
        "ntriangleright",
        "ntrianglerighteq",
        "nvDash",
        "nvdash",
        "nwarrow",
        "owns",
        "parallel",
        "perp",
        "pitchfork",
        "prec",
        "precapprox",
        "preccurlyeq",
        "preceq",
        "precnapprox",
        "precneqq",
        "precnsim",
        "precsim",
        "propto",
        "rArr",
        "rarr",
        "restriction",
        "rightarrow",
        "rightarrowtail",
        "rightharpoondown",
        "rightharpoonup",
        "rightleftarrows",
        "rightleftharpoons",
        "rightrightarrows",
        "rightsquigarrow",
        "risingdotseq",
        "searrow",
        "shortmid",
        "shortparallel",
        "sim",
        "simeq",
        "smallfrown",
        "smallsmile",
        "smile",
        "sqsubset",
        "sqsubseteq",
        "sqsupset",
        "sqsupseteq",
        "sub",
        "sube",
        "subseteqq",
        "subsetneq",
        "subsetneqq",
        "succ",
        "succapprox",
        "succcurlyeq",
        "succeq",
        "succnapprox",
        "succneqq",
        "succnsim",
        "succsim",
        "supe",
        "supset",
        "supseteq",
        "supseteqq",
        "supsetneq",
        "supsetneqq",
        "swarrow",
        "thickapprox",
        "thicksim",
        "to",
        "trianglelefteq",
        "triangleq",
        "trianglerighteq",
        "twoheadleftarrow",
        "twoheadrightarrow",
        "uArr",
        "uarr",
        "uparrow",
        "updownarrow",
        "upharpoonleft",
        "upharpoonright",
        "upuparrows",
        "varpropto",
        "varsubsetneq",
        "varsubsetneqq",
        "varsupsetneq",
        "varsupsetneqq",
        "vartriangle",
        "vartriangleleft",
        "vartriangleright",
        "vcentcolon",
        "vdash",
        "vDash",
    ])
});

fn is_combining_accent(c: char) -> bool {
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

fn accent_from_char(c: char) -> &'static str {
    match c {
        '\u{0300}' => "\\grave",
        '\u{0301}' => "\\acute",
        '\u{0302}' => "\\hat",
        '\u{0303}' => "\\tilde",
        '\u{0304}' | '\u{0305}' => "\\bar",
        '\u{0306}' => "\\breve",
        '\u{0307}' => "\\dot",
        '\u{0308}' => "\\ddot",
        '\u{030A}' => "\\mathring",
        '\u{030C}' => "\\check",
        '\u{0332}' => "\\underline",
        '\u{20d0}' => "\\overleftharpoon",
        '\u{20d1}' => "\\overrightharpoon",
        '\u{20d6}' => "\\overleftarrow",
        '\u{20d7}' => "\\vec",
        '\u{20e1}' => "\\overleftrightarrow",
        _ => "undefined",
    }
}

fn wide_accent_from_char(c: char) -> &'static str {
    match c {
        '\u{0302}' => "\\widehat",
        '\u{0303}' => "\\widetilde",
        '\u{0304}' | '\u{0305}' => "\\overline",
        '\u{20d7}' => "\\overrightarrow",
        // Hurmet's wide table has no breve; JavaScript yields "undefined".
        '\u{0306}' => "undefined",
        c => accent_from_char(c),
    }
}

fn group_subscript(word: &str) -> String {
    match word.find('_') {
        None => word.to_string(),
        Some(pos) => format!("{}{{{}}}", &word[..=pos], &word[pos + 1..]),
    }
}

fn check_for_trailing_accent(word: &str) -> String {
    match word.chars().last() {
        Some(ch) if is_combining_accent(ch) => {
            let base = &word[..word.len() - ch.len_utf8()];
            if base == "i" {
                format!("{}{{ı}}", accent_from_char(ch))
            } else if base == "j" {
                format!("{}{{ȷ}}", accent_from_char(ch))
            } else if utf16_len(base) == 1 {
                format!("{}{{{}}}", accent_from_char(ch), base)
            } else {
                format!("{}{{{}}}", wide_accent_from_char(ch), base)
            }
        }
        _ => word.to_string(),
    }
}

fn is_sub_or_accent(c: char) -> bool {
    c == '_' || ('₀'..='₉').contains(&c) || is_combining_accent(c)
}

fn is_sub(c: char) -> bool {
    c == '_' || ('₀'..='₉').contains(&c)
}

fn is_math_alphanumeric(c: char) -> bool {
    ('\u{1D400}'..='\u{1D7FF}').contains(&c)
}

fn lex_one_word(s: &str, prev: &Token) -> Option<Token> {
    let m = WORD_RE.find(s)?;
    let mut matched = m.as_str().trim_end_matches('_').to_string();
    // The immediately following character.
    let fc = s[matched.len()..].chars().next();

    if let Some((tex, tt)) = word(&matched)
        && fc != Some('′')
    {
        return Some(Token::new(matched, tex, tt));
    }
    if fc == Some('(') {
        // A word followed by an open paren is a function name.
        return Some(if prev.ttype == Tt::Accent {
            let out = format!("{matched}}}{{");
            Token::new(matched, out, Tt::Function)
        } else if matched == "sqrt" {
            Token::new(matched, "\\sqrt", Tt::Unary)
        } else if matched == "f" {
            Token::new(matched, "f", Tt::Function)
        } else if MATH_OPERATORS.contains(matched.as_str()) {
            let out = format!("\\{matched}");
            Token::new(matched, out, Tt::Function)
        } else {
            let out = format!("\\operatorname{{{}}}", group_subscript(&matched));
            Token::new(matched, out, Tt::Function)
        });
    }
    if prev.ttype == Tt::Accessor {
        return Some(Token::new(matched.clone(), matched, Tt::Property));
    }
    if matched.chars().any(is_sub_or_accent) {
        if !matched.chars().any(is_sub) {
            let identifier = check_for_trailing_accent(&matched);
            let tt = if utf16_len(&matched) > 2 {
                Tt::LongVar
            } else {
                Tt::Var
            };
            return Some(Token::new(matched, identifier, tt));
        }
        let sub_pos = matched.find(is_sub).unwrap_or(0);
        let base = check_for_trailing_accent(&matched[..sub_pos]);
        let mut subscript = matched[sub_pos..].to_string();
        if let Some(rest) = subscript.strip_prefix('_') {
            let rest = rest.replace('(', "{").replace(')', "}");
            if let Some(pos_us) = rest.find('_') {
                // Double subscript.
                subscript = format!(
                    "_{{\\text{{{}}}_\\text{{{}}}}}",
                    &rest[..pos_us],
                    &rest[pos_us + 1..]
                );
            } else {
                subscript = format!("_\\text{{{rest}}}");
                if fc == Some('_') {
                    matched.push('_'); // Double subscript. The second one is empty.
                }
            }
        }
        let mut identifier = format!("{base}{subscript}");
        let after = &s[matched.len().min(s.len())..];
        let primes = after.chars().take_while(|&c| c == '′').count();
        for _ in 0..primes {
            matched.push('′');
            identifier.push('\'');
        }
        let tt = if utf16_len(&base) > 1 {
            Tt::LongVar
        } else {
            Tt::Var
        };
        return Some(Token::new(matched, identifier, tt));
    }
    let mut chars = matched.chars();
    let first = chars.next()?;
    if is_math_alphanumeric(first) && chars.next().is_none() {
        return Some(Token::new(matched.clone(), matched, Tt::Var));
    }
    if utf16_len(&matched) > 1 {
        if matched == "mod" {
            return Some(Token::new(matched, "\\bmod", Tt::Bin));
        }
        return Some(Token::new(matched.clone(), matched, Tt::LongVar));
    }
    Some(Token::new(first.to_string(), first.to_string(), Tt::Var))
}

/// Remove the first match of `\r?\n` followed by the rest of that line (JS `/\r?\n.*/`).
fn cut_at_newline_line(s: &str) -> String {
    match s.find('\n') {
        None => s.to_string(),
        Some(nl) => {
            let start = if nl > 0 && s.as_bytes()[nl - 1] == b'\r' {
                nl - 1
            } else {
                nl
            };
            let after = &s[nl + 1..];
            let end_rel = after
                .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                .unwrap_or(after.len());
            format!("{}{}", &s[..start], &after[end_rel..])
        }
    }
}

/// Get the next token in `s`.
pub(crate) fn lex(s: &str, formats: &Formats, prev: &Token, _in_real_time: bool) -> Token {
    if let Some(rest) = s.strip_prefix('"') {
        // A string between double quotation marks. A doubled quotation mark is an escape.
        if let Some(m) = TEXT_CONTENT_RE.find(rest)
            && !m.as_str().contains(['\r', '\n'])
        {
            let m = m.as_str();
            let st = m[..m.len() - 1].replace("\"\"", "\"");
            return Token::new(format!("\"{m}"), st, Tt::String);
        }
        // No closing quotation mark on this line.
        let line_end = s
            .find("\r\n")
            .into_iter()
            .chain(s.find('\n'))
            .min()
            .unwrap_or(s.len());
        let st = s[..line_end].replace("\"\"", "\"");
        return Token::new(s, st, Tt::String);
    }

    if s.starts_with('#') {
        let mut st = skip_utf16(s, 2);
        if let Some(pos) = st.find('\n') {
            st = &st[..pos];
        }
        return Token::new(
            format!("#{st}"),
            format!("\\text{{\\texttt{{ \\#{st}}}}}"),
            Tt::Comment,
        );
    }

    if let Some(body) = s.strip_prefix("``") {
        // A data frame literal: TSV between double back ticks.
        let from = if body.starts_with('`') { 3 } else { 2 };
        let pos = s[from..].find('`').map(|p| p + from);
        let input_str = match pos {
            Some(p) if p > 0 => &s[2..p],
            _ => &s[2..],
        };
        // Hurmet uses DataFrame.display(DataFrame.dataFrameFromTSV(…)) unless typing in real
        // time. That needs the evaluator's data frame model, so we always use the quick display.
        let tex = quick_display(crate::utils::tabless_trim(input_str));
        return Token::new(format!("``{input_str}``"), tex, Tt::DataFrame);
    }

    if let Some(rest) = s.strip_prefix('`') {
        // Rich text.
        return match rest.find('`') {
            Some(p) => {
                let st = cut_at_newline_line(&rest[..p]);
                Token::new(format!("`{st}`"), st, Tt::RichText)
            }
            None => Token::new(s, cut_at_newline_line(s), Tt::RichText),
        };
    }

    if UNIT_RE.is_match(s) {
        // Between single quotation marks: a unit or a date.
        let c1 = crate::utils::first_char_len(s);
        if let Some(p) = s[c1..].find('\'').map(|p| p + c1) {
            let st = &s[c1..p];
            let with_delims = format!("'{st}'");
            if DATE_LITERAL_RE.is_match(s) {
                let tex = date_tex(&with_delims, formats.date_format.as_deref());
                return Token::new(with_delims, tex, Tt::Date);
            }
            return Token::new(with_delims, unit_tex_from_string(st), Tt::Unit);
        }
        // One of the unambiguous unit symbols, like ° or Å.
        let c = &s[..c1];
        return Token::new(c, c, Tt::Unit);
    }

    if let Some(m) = TEX_FUNCTION_RE.find(s) {
        // A TeX control word, starting with a backslash.
        let m = m.as_str();
        let st = &m[1..];
        if ACCENTS.contains(st) {
            return Token::new(m, m, Tt::Accent);
        }
        if UNARIES.contains(st) {
            return Token::new(m, m, Tt::Unary);
        }
        if COLORS.contains(st) {
            return Token::new(m, format!("\\textcolor{{{st}}}"), Tt::Unary);
        }
        if BINARIES.contains(st) {
            return Token::new(m, m, Tt::Binary);
        }
        if TEX_REL.contains(st) {
            return Token::new(m, m, Tt::Rel);
        }
        if let Some((tex, tt, close)) = tex_function(m) {
            return Token::new(m, tex, tt).with_close(close);
        }
        if MATH_OPERATORS.contains(st) {
            return Token::new(m, m, Tt::Function);
        }
        if let Some(ch) = greek(st) {
            return Token::new(m, ch, Tt::Var);
        }
        return Token::new(m, m, Tt::Ord);
    }

    if s.starts_with('-') && !s[1..].starts_with(['-', '=', '<', '>', ':']) {
        if is_unary(prev)
            && let Some(parts) = NUMBER_RE.captures(s)
        {
            let tex = tex_from_num_parts(&parts, &formats.decimal_format);
            return Token::new(&parts[0], tex, Tt::Num);
        }
        return Token::new("-", "-", Tt::Add);
    }

    // (Every number starts with a digit; checking first avoids allocating captures.)
    if s.starts_with(|c: char| c.is_ascii_digit())
        && let Some(parts) = NUMBER_RE.captures(s)
    {
        let tex = tex_from_num_parts(&parts, &formats.decimal_format);
        return Token::new(&parts[0], tex, Tt::Num);
    }

    if s.starts_with("_(") || s.starts_with("_[") || s.starts_with("_{") {
        return Token::new("_", "_", Tt::Sub);
    }

    if let Some(w) = lex_one_word(s, prev) {
        return w;
    }

    if let Some(m) = SUPER_RE.find(s) {
        return Token::new(m.as_str(), m.as_str(), Tt::SupChar);
    }

    let c1 = first_char(s).unwrap_or(' ');
    if "∑∫∬∭⨌∮∯∰⨍⨚⨙∏∐".contains(c1) {
        return Token::new(c1.to_string(), c1.to_string(), Tt::BigOperator);
    }

    // The longest initial substring of s that is in the misc symbols.
    // The longest reachable key is 4 chars ("<==>"), so look no further. (Keys such as
    // "\\dots" and "\\cr" contain letters, which are not misc chars, so they are unreachable.)
    let run: Vec<(usize, char)> = s
        .char_indices()
        .take_while(|(_, c)| MISC_CHARS.contains(*c))
        .take(4)
        .collect();
    for n in (1..=run.len()).rev() {
        let end = run
            .get(n)
            .map_or_else(|| run[n - 1].0 + run[n - 1].1.len_utf8(), |x| x.0);
        if let Some((input, out, tt, close)) = misc_symbol(&s[..end]) {
            return Token::new(input, out, tt).with_close(close);
        }
    }

    // Nothing matched. Return one character.
    if c1 == '.'
        && (matches!(
            prev.ttype,
            Tt::Var | Tt::LongVar | Tt::String | Tt::Property
        ) || prev.input == "]"
            || prev.input == ")")
    {
        // The accessor dot, with its spacing suppressed.
        return Token::new(".", "{.}", Tt::Accessor);
    }
    Token::new(c1.to_string(), add_text_escapes(&c1.to_string()), Tt::Var)
}
