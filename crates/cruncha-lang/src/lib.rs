// SPDX-License-Identifier: GPL-3.0-or-later

//! The Cruncha calc language: a port of Hurmet's calculation syntax.
//!
//! This crate is spike S1 (see `docs/spikes/S1-calc-parser.md`): Hurmet's lexer and parser,
//! ported so that they produce exactly Hurmet's output.
//!
//! - [`parse_tex`] renders an entry as TeX, like `hurmet.parse(entry)`.
//! - [`parse_calc`] also compiles it to Hurmet's RPN, like `hurmet.parse(entry, formats, true)`.
//!   In that mode the TeX is the "blue echo" template, with `〖name〗` around variables whose
//!   values are plugged in later.
//!
//! ```
//! use cruncha_lang::{parse_calc, parse_tex, Options};
//! let opts = Options::default();
//! assert_eq!(parse_tex("M = (w L^2)/8", &opts).unwrap(), "M = \\dfrac{w L^{2}}{8}");
//! let calc = parse_calc("2^3^4", &opts).unwrap();
//! assert_eq!(calc.rpn, "®2/1\u{a0}®3/1\u{a0}®4/1\u{a0}^\u{a0}^");
//! ```

mod dataframe;
mod date;
mod lexer;
mod number;
mod parser;
mod utils;

pub use lexer::{Token, Tt};

/// Hurmet helpers that the evaluator (`cruncha-eval`) shares with the parser, so that both
/// produce identical text. Not a stable API.
#[doc(hidden)]
pub mod hurmet {
    pub use crate::dataframe::quick_display;
    pub use crate::date::seconds_from_iso_string;
    pub use crate::lexer::{IDENTIFIER_BODY, IDENTIFIER_START};
    pub use crate::number::{formatted_decimal, formatted_integer};
    pub use crate::parser::{BUILT_IN_FUNCTIONS, BUILT_IN_REDUCER_FUNCTIONS};
    pub use crate::utils::{
        add_text_escapes, check_for_numeric_subscript, is_js_space, js_trim,
        numeral_from_superscript, tabless_trim, unit_tex_from_string, utf16_len,
    };
}

/// Number and date display formats (Hurmet's `formats`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formats {
    /// For example `"1,000,000."`, `"1.000.000,"`, `"1 000 000."`, `"1,00,000."`.
    pub decimal_format: String,
    /// For example `"yyyy-mm-dd"`. `None` displays ISO dates.
    pub date_format: Option<String>,
}

impl Default for Formats {
    fn default() -> Self {
        // Hurmet's defaults, including its "yyy-mm-dd" spelling.
        Formats {
            decimal_format: "1,000,000.".into(),
            date_format: Some("yyy-mm-dd".into()),
        }
    }
}

/// Parse options.
#[derive(Clone, Debug, Default)]
pub struct Options {
    pub formats: Formats,
    /// True when re-rendering on every keystroke (Hurmet's `inRealTime`). Accepted for API
    /// parity; the port always uses the quick data frame display.
    pub in_real_time: bool,
    /// For spreadsheet cells, whose RPN refers to cells through the sheet (`sheetName`).
    pub sheet_name: String,
}

/// A parse in calculation mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calc {
    /// The echo template: TeX with `〖name〗` marking variables to plug in.
    pub tex: String,
    /// Space-free RPN; tokens are separated by U+00A0 (or `§` inside lazily evaluated parts).
    pub rpn: String,
    /// Variable names the expression reads.
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The input is malformed in a way that makes Hurmet's parser throw (for example an
    /// unmatched closing bracket).
    Malformed(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Malformed(m) => write!(f, "malformed calculation: {m}"),
        }
    }
}

impl std::error::Error for Error {}

/// Render an entry as TeX.
pub fn parse_tex(input: &str, opts: &Options) -> Result<String, Error> {
    Ok(parser::parse(
        input,
        &opts.formats,
        false,
        opts.in_real_time,
        &opts.sheet_name,
    )?
    .tex)
}

/// Compile an entry to its echo template and RPN.
pub fn parse_calc(input: &str, opts: &Options) -> Result<Calc, Error> {
    let out = parser::parse(
        input,
        &opts.formats,
        true,
        opts.in_real_time,
        &opts.sheet_name,
    )?;
    Ok(Calc {
        tex: out.tex,
        rpn: out.rpn,
        dependencies: out.dependencies,
    })
}
