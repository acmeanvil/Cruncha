// SPDX-License-Identifier: GPL-3.0-or-later

//! TeX math, as Cruncha's calc engine writes it (Hurmet's subset), to Typst math code.
//!
//! The engine's TeX is generated, so the subset is known: fractions, scripts, `\text`,
//! `\textcolor` (the blue echo), spacing, matrices and cases, accents, `\left…\right`, and a
//! symbol table. Anything else is an error, never silently dropped, so a report can't show a
//! calc differently from the screen without saying so.
//!
//! The output is safe to `eval` in math mode: every literal character Typst treats specially
//! is escaped, letters are separated (Typst reads `ab` as one identifier), and text goes in
//! quoted strings.

use std::fmt::Write as _;

/// Why a TeX string could not be converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexError(pub String);

impl std::fmt::Display for TexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TexError {}

type Result<T> = std::result::Result<T, TexError>;

/// Convert TeX math to Typst math code (the inside of `$…$`).
pub fn to_typst(tex: &str) -> Result<String> {
    let toks = lex(tex)?;
    let mut p = Parser { toks, pos: 0 };
    let nodes = p.list(&[])?;
    if p.pos < p.toks.len() {
        return Err(TexError(format!("unexpected {:?}", p.toks[p.pos])));
    }
    let mut out = String::new();
    emit_list(&nodes, &mut out)?;
    Ok(out.split_whitespace().collect::<Vec<_>>().join(" "))
}

// ---------------------------------------------------------------------------------------
// Lexer

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    /// `\name` or `\,` (a single non-letter).
    Cmd(String),
    Open,
    Close,
    Sup,
    Sub,
    Amp,
    /// `\\`, a row break.
    Row,
    /// Any other character.
    Char(char),
    Space,
}

fn lex(s: &str) -> Result<Vec<Tok>> {
    let mut out = Vec::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\\' => match it.next() {
                Some('\\') => out.push(Tok::Row),
                Some(c) if c.is_ascii_alphabetic() => {
                    let mut name = String::from(c);
                    while let Some(&d) = it.peek() {
                        if d.is_ascii_alphabetic() {
                            name.push(d);
                            it.next();
                        } else {
                            break;
                        }
                    }
                    out.push(Tok::Cmd(name));
                }
                Some(c) => out.push(Tok::Cmd(c.to_string())),
                None => return Err(TexError("a trailing backslash".into())),
            },
            '{' => out.push(Tok::Open),
            '}' => out.push(Tok::Close),
            '^' => out.push(Tok::Sup),
            '_' => out.push(Tok::Sub),
            '&' => out.push(Tok::Amp),
            '%' => break, // a TeX comment; the engine escapes literal percent signs
            c if c.is_whitespace() => {
                if out.last() != Some(&Tok::Space) {
                    out.push(Tok::Space)
                }
            }
            c => out.push(Tok::Char(c)),
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------
// Parser: a small tree, so fractions, groups and environments can be emitted as calls.

#[derive(Debug, Clone)]
enum Node {
    /// Already Typst code (symbols, spacing, functions).
    Code(String),
    /// One literal character, escaped on output.
    Char(char),
    /// Literal text, emitted as a quoted string.
    Text(String),
    Group(Vec<Node>),
    /// `base^sup_sub`.
    Script {
        base: Box<Node>,
        sup: Option<Box<Node>>,
        sub: Option<Box<Node>>,
    },
    /// A Typst function applied to arguments: `frac(a, b)`.
    Call(String, Vec<Vec<Node>>),
    /// `#text(fill: …)[$…$]` and friends: Typst code around nested math.
    Wrap(String, Vec<Node>),
    /// `lr(left body right)`.
    Lr(String, Vec<Node>, String),
    /// Rows of cells: matrices, arrays, cases, aligned.
    Grid {
        kind: GridKind,
        rows: Vec<Vec<Vec<Node>>>,
        hlines: Vec<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GridKind {
    Mat(&'static str),
    Cases { reverse: bool },
    Aligned,
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn skip_space(&mut self) {
        while self.peek() == Some(&Tok::Space) {
            self.pos += 1;
        }
    }

    /// Nodes until `}` (not consumed), the end, or a stop command (`\right`, `\end`, `\\`, `&`).
    fn list(&mut self, stops: &[&str]) -> Result<Vec<Node>> {
        let mut out: Vec<Node> = Vec::new();
        loop {
            match self.peek() {
                None | Some(Tok::Close) => break,
                Some(Tok::Row) | Some(Tok::Amp) if stops.contains(&"grid") => break,
                Some(Tok::Cmd(c)) if stops.contains(&c.as_str()) => break,
                Some(Tok::Cmd(c)) if c == "over" || c == "atop" => {
                    // Infix: everything so far over everything after.
                    let bar = c == "over";
                    self.pos += 1;
                    let num = std::mem::take(&mut out);
                    let den = self.list(stops)?;
                    out.push(if bar {
                        Node::Call("frac".into(), vec![num, den])
                    } else {
                        Node::Grid {
                            kind: GridKind::Mat("#none"),
                            rows: vec![vec![num], vec![den]],
                            hlines: vec![],
                        }
                    });
                    break;
                }
                Some(Tok::Sup) | Some(Tok::Sub) => {
                    let base = out.pop().unwrap_or(Node::Group(vec![]));
                    let node = self.scripts(base)?;
                    out.push(node);
                }
                Some(Tok::Space) => {
                    self.pos += 1;
                }
                _ => {
                    if let Some(n) = self.atom()? {
                        out.push(n);
                    }
                }
            }
        }
        Ok(out)
    }

    fn scripts(&mut self, base: Node) -> Result<Node> {
        let (mut sup, mut sub) = (None, None);
        loop {
            match self.peek() {
                Some(Tok::Sup) if sup.is_none() => {
                    self.pos += 1;
                    sup = Some(Box::new(self.arg()?));
                }
                Some(Tok::Sub) if sub.is_none() => {
                    self.pos += 1;
                    sub = Some(Box::new(self.arg()?));
                }
                Some(Tok::Char('\'')) if sup.is_none() => {
                    break;
                }
                _ => break,
            }
        }
        Ok(Node::Script { base: Box::new(base), sup, sub })
    }

    /// One argument: a `{group}`, or a single token (`x^2`, `x_\text{2}`).
    fn arg(&mut self) -> Result<Node> {
        self.skip_space();
        match self.peek() {
            Some(Tok::Open) => {
                self.pos += 1;
                let inner = self.list(&[])?;
                self.expect_close()?;
                Ok(Node::Group(inner))
            }
            None => Err(TexError("a missing argument".into())),
            _ => Ok(self.atom()?.unwrap_or(Node::Group(vec![]))),
        }
    }

    fn arg_list(&mut self) -> Result<Vec<Node>> {
        Ok(match self.arg()? {
            Node::Group(v) => v,
            n => vec![n],
        })
    }

    /// A raw `{…}` argument as text (colors, environment names, column specs).
    fn raw_arg(&mut self) -> Result<String> {
        self.skip_space();
        if self.next() != Some(Tok::Open) {
            return Err(TexError("expected {".into()));
        }
        let mut depth = 0;
        let mut s = String::new();
        loop {
            match self.next() {
                None => return Err(TexError("an unclosed {".into())),
                Some(Tok::Close) if depth == 0 => break,
                Some(Tok::Close) => {
                    depth -= 1;
                    s.push('}');
                }
                Some(Tok::Open) => {
                    depth += 1;
                    s.push('{');
                }
                Some(Tok::Char(c)) => s.push(c),
                Some(Tok::Space) => s.push(' '),
                Some(Tok::Cmd(c)) => {
                    s.push('\\');
                    s.push_str(&c);
                }
                Some(Tok::Sup) => s.push('^'),
                Some(Tok::Sub) => s.push('_'),
                Some(Tok::Amp) => s.push('&'),
                Some(Tok::Row) => s.push_str("\\\\"),
            }
        }
        Ok(s)
    }

    /// Text inside `\text{…}`: characters and escaped symbols, kept verbatim.
    fn text_arg(&mut self) -> Result<String> {
        let raw = self.raw_arg()?;
        let mut s = String::new();
        let mut it = raw.chars().peekable();
        while let Some(c) = it.next() {
            if c == '\\' {
                let mut name = String::new();
                while let Some(&d) = it.peek() {
                    if d.is_ascii_alphabetic() {
                        name.push(d);
                        it.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    if let Some(d) = it.next() {
                        match d {
                            ',' | ';' | ':' | ' ' | '!' => s.push(' '),
                            d => s.push(d),
                        }
                    }
                } else {
                    match name.as_str() {
                        "textasciicircum" => s.push('^'),
                        "textasciitilde" => s.push('~'),
                        "quad" | "qquad" => s.push(' '),
                        _ => {} // formatting commands inside text are dropped
                    }
                    if it.peek() == Some(&' ') {
                        it.next();
                    }
                }
            } else if c != '{' && c != '}' {
                s.push(c);
            }
        }
        Ok(s)
    }

    fn expect_close(&mut self) -> Result<()> {
        match self.next() {
            Some(Tok::Close) => Ok(()),
            t => Err(TexError(format!("expected }}, found {t:?}"))),
        }
    }

    /// A delimiter after `\left`, `\right` or `\middle`.
    fn delim(&mut self) -> Result<String> {
        self.skip_space();
        Ok(match self.next() {
            Some(Tok::Char('.')) => String::new(),
            Some(Tok::Char('(')) => "paren.l".into(),
            Some(Tok::Char(')')) => "paren.r".into(),
            Some(Tok::Char('[')) => "bracket.l".into(),
            Some(Tok::Char(']')) => "bracket.r".into(),
            Some(Tok::Char('|')) => "bar.v".into(),
            Some(Tok::Char('/')) => "slash".into(),
            // Unicode delimiters (⟨ ⟩ ‖ ⌊ ⌋ ⌈ ⌉) are Typst symbols as they are.
            Some(Tok::Char(c)) if "⟨⟩‖⌊⌋⌈⌉".contains(c) => c.to_string(),
            Some(Tok::Cmd(c)) => match c.as_str() {
                "{" | "lbrace" => "brace.l".into(),
                "}" | "rbrace" => "brace.r".into(),
                "|" | "Vert" => "bar.v.double".into(),
                "vert" => "bar.v".into(),
                "langle" => "angle.l".into(),
                "rangle" => "angle.r".into(),
                "lfloor" => "floor.l".into(),
                "rfloor" => "floor.r".into(),
                "lceil" => "ceil.l".into(),
                "rceil" => "ceil.r".into(),
                c => return Err(TexError(format!("unknown delimiter \\{c}"))),
            },
            t => return Err(TexError(format!("unknown delimiter {t:?}"))),
        })
    }

    fn atom(&mut self) -> Result<Option<Node>> {
        let t = match self.next() {
            Some(t) => t,
            None => return Ok(None),
        };
        Ok(Some(match t {
            Tok::Open => {
                let inner = self.list(&[])?;
                self.expect_close()?;
                Node::Group(inner)
            }
            Tok::Char(c) => Node::Char(c),
            Tok::Cmd(c) => return self.command(&c),
            Tok::Close | Tok::Sup | Tok::Sub | Tok::Space => return Ok(None),
            Tok::Amp => Node::Code("&".into()),
            Tok::Row => Node::Code("\\".into()),
        }))
    }

    fn command(&mut self, c: &str) -> Result<Option<Node>> {
        let code = |s: &str| Ok(Some(Node::Code(s.into())));
        let call1 = |p: &mut Parser, f: &str| -> Result<Option<Node>> {
            let a = p.arg_list()?;
            Ok(Some(Node::Call(f.into(), vec![a])))
        };
        if let Some(sym) = symbol(c) {
            return code(sym);
        }
        match c {
            // Spacing.
            "," | "thinspace" => code("thin"),
            ":" | ">" | "medspace" => code("med"),
            ";" | "thickspace" => code("thick"),
            " " => code("space"),
            "!" => code("#h(-0.1667em)"),
            "quad" => code("quad"),
            "qquad" => code("wide"),
            "mkern" | "kern" | "hskip" | "mskip" => {
                let mut amount = String::new();
                while let Some(Tok::Char(ch)) = self.peek() {
                    if ch.is_ascii_digit() || *ch == '.' || *ch == '-' {
                        amount.push(*ch);
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let mut unit = String::new();
                while let Some(Tok::Char(ch)) = self.peek() {
                    if ch.is_ascii_alphabetic() && unit.len() < 2 {
                        unit.push(*ch);
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let n: f64 = amount.parse().unwrap_or(0.0);
                let em = match unit.as_str() {
                    "mu" => n / 18.0,
                    "em" => n,
                    "ex" => n * 0.43,
                    "pt" => n / 10.0,
                    _ => 0.0,
                };
                Ok(Some(Node::Code(format!("#h({:.4}em)", em))))
            }
            "hspace" => {
                let _ = self.raw_arg()?;
                code("thin")
            }
            // Characters the engine escapes.
            "{" | "lbrace" => Ok(Some(Node::Char('{'))),
            "}" | "rbrace" => Ok(Some(Node::Char('}'))),
            "%" => Ok(Some(Node::Char('%'))),
            "$" => Ok(Some(Node::Char('$'))),
            "#" => Ok(Some(Node::Char('#'))),
            "&" => Ok(Some(Node::Char('&'))),
            "_" => Ok(Some(Node::Char('_'))),
            "|" => code("bar.v.double"),
            // Fractions and roots.
            "frac" | "dfrac" | "tfrac" | "sfrac" | "cfrac" => {
                let a = self.arg_list()?;
                let b = self.arg_list()?;
                let f = Node::Call("frac".into(), vec![a, b]);
                Ok(Some(match c {
                    "dfrac" | "cfrac" => Node::Call("display".into(), vec![vec![f]]),
                    "tfrac" | "sfrac" => Node::Call("inline".into(), vec![vec![f]]),
                    _ => f,
                }))
            }
            "sqrt" => {
                self.skip_space();
                if self.peek() == Some(&Tok::Char('[')) {
                    self.pos += 1;
                    let mut idx = Vec::new();
                    while let Some(t) = self.peek() {
                        if *t == Tok::Char(']') {
                            break;
                        }
                        if let Some(n) = self.atom()? {
                            idx.push(n);
                        }
                    }
                    self.pos += 1;
                    let body = self.arg_list()?;
                    Ok(Some(Node::Call("root".into(), vec![idx, body])))
                } else {
                    call1(self, "sqrt")
                }
            }
            // Text and fonts.
            "text" | "textrm" | "textnormal" | "mbox" | "textit" | "textbf" | "texttt" => {
                let t = self.text_arg()?;
                Ok(Some(match c {
                    "textbf" => Node::Call("bold".into(), vec![vec![Node::Text(t)]]),
                    "texttt" => Node::Call("mono".into(), vec![vec![Node::Text(t)]]),
                    "textit" => Node::Call("italic".into(), vec![vec![Node::Text(t)]]),
                    _ => Node::Text(t),
                }))
            }
            "mathrm" | "operatorname" | "mathup" => {
                let t = self.text_arg()?;
                Ok(Some(if c == "operatorname" {
                    Node::Code(format!("op({})", quote(&t)))
                } else {
                    Node::Call("upright".into(), vec![vec![Node::Text(t)]])
                }))
            }
            "mathbf" | "boldsymbol" | "bm" => call1(self, "bold"),
            "mathit" => call1(self, "italic"),
            "mathcal" => call1(self, "cal"),
            "mathbb" => call1(self, "bb"),
            "mathfrak" => call1(self, "frak"),
            "mathsf" => call1(self, "sans"),
            "mathtt" => call1(self, "mono"),
            "mathbin" | "mathrel" | "mathord" | "mathop" => {
                let class = match c {
                    "mathbin" => "binary",
                    "mathrel" => "relation",
                    "mathop" => "large",
                    _ => "normal",
                };
                let a = self.arg_list()?;
                Ok(Some(Node::Call(format!("class(\"{class}\", "), vec![a])))
            }
            "displaystyle" => {
                let rest = self.list(&["right", "end", "grid"])?;
                Ok(Some(Node::Call("display".into(), vec![rest])))
            }
            "textstyle" => {
                let rest = self.list(&["right", "end", "grid"])?;
                Ok(Some(Node::Call("inline".into(), vec![rest])))
            }
            // Color.
            "textcolor" | "color" | "colorbox" => {
                let color = typst_color(&self.raw_arg()?)?;
                let body = match c {
                    "color" => self.list(&["right", "end", "grid"])?,
                    // \colorbox's argument is text mode, with math in $…$.
                    "colorbox" => {
                        let raw = self.raw_arg()?;
                        let mut nodes = Vec::new();
                        for (i, part) in raw.split('$').enumerate() {
                            if part.is_empty() {
                                continue;
                            }
                            if i % 2 == 0 {
                                nodes.push(Node::Text(part.to_string()));
                            } else {
                                nodes.push(Node::Code(to_typst(part)?));
                            }
                        }
                        nodes
                    }
                    _ => self.arg_list()?,
                };
                let wrap = if c == "colorbox" {
                    format!("box(fill: {color}, inset: (x: 2pt), outset: (y: 2pt))")
                } else {
                    format!("text(fill: {color})")
                };
                Ok(Some(Node::Wrap(wrap, body)))
            }
            "phantom" => {
                let a = self.arg_list()?;
                Ok(Some(Node::Wrap("hide".into(), a)))
            }
            "class" => {
                let _ = self.raw_arg()?;
                Ok(Some(Node::Group(self.arg_list()?)))
            }
            "pu" | "ce" => Ok(Some(Node::Text(self.text_arg()?))),
            "label" | "tag" => {
                let _ = self.raw_arg()?;
                Ok(None)
            }
            "renewcommand" => {
                let _ = self.arg()?;
                let _ = self.raw_arg()?;
                Ok(None)
            }
            "arraystretch" | "arraycolsep" | "hline" | "nonumber" | "notag" => Ok(None),
            // Accents.
            "hat" | "widehat" => call1(self, "hat"),
            "bar" => call1(self, "macron"),
            "overline" => call1(self, "overline"),
            "underline" => call1(self, "underline"),
            "vec" | "overrightarrow" => call1(self, "arrow"),
            "overleftarrow" => {
                let a = self.arg_list()?;
                Ok(Some(Node::Call("accent".into(), vec![a, vec![Node::Code("arrow.l".into())]])))
            }
            "overleftrightarrow" => {
                let a = self.arg_list()?;
                Ok(Some(Node::Call("accent".into(), vec![a, vec![Node::Code("arrow.l.r".into())]])))
            }
            "overrightharpoon" => {
                let a = self.arg_list()?;
                Ok(Some(Node::Call("accent".into(), vec![a, vec![Node::Code("harpoon.rt".into())]])))
            }
            "overleftharpoon" => {
                let a = self.arg_list()?;
                Ok(Some(Node::Call("accent".into(), vec![a, vec![Node::Code("harpoon.lt".into())]])))
            }
            "dot" => call1(self, "dot"),
            "ddot" => call1(self, "dot.double"),
            "tilde" | "widetilde" => call1(self, "tilde"),
            "acute" => call1(self, "acute"),
            "grave" => call1(self, "grave"),
            "breve" => call1(self, "breve"),
            "check" => call1(self, "caron"),
            "mathring" => call1(self, "circle"),
            "underbrace" | "overbrace" => {
                let body = self.arg_list()?;
                self.skip_space();
                let note = match (c, self.peek()) {
                    ("underbrace", Some(Tok::Sub)) | ("overbrace", Some(Tok::Sup)) => {
                        self.pos += 1;
                        Some(self.arg_list()?)
                    }
                    _ => None,
                };
                let mut args = vec![body];
                args.extend(note);
                Ok(Some(Node::Call(c.into(), args)))
            }
            "xrightarrow" | "xleftarrow" | "xRightarrow" | "xLeftarrow" | "xleftrightarrow"
            | "xLeftrightarrow" | "xrightleftarrows" => {
                let arrow = match c {
                    "xrightarrow" => "arrow.r.long",
                    "xleftarrow" => "arrow.l.long",
                    "xRightarrow" => "arrow.r.double.long",
                    "xLeftarrow" => "arrow.l.double.long",
                    "xleftrightarrow" => "arrow.l.r.long",
                    "xLeftrightarrow" => "arrow.l.r.double.long",
                    _ => "arrows.rl",
                };
                let over = self.arg_list()?;
                Ok(Some(Node::Script {
                    base: Box::new(Node::Code(format!("limits({arrow})"))),
                    sup: Some(Box::new(Node::Group(over))),
                    sub: None,
                }))
            }
            // Delimiters.
            "left" => {
                let l = self.delim()?;
                let body = self.list(&["right"])?;
                match self.next() {
                    Some(Tok::Cmd(r)) if r == "right" => {}
                    t => return Err(TexError(format!("\\left without \\right ({t:?})"))),
                }
                let r = self.delim()?;
                Ok(Some(Node::Lr(l, body, r)))
            }
            "middle" => {
                let d = self.delim()?;
                Ok(Some(Node::Code(format!("mid({})", if d.is_empty() { "bar.v" } else { &d }))))
            }
            "right" => Err(TexError("\\right without \\left".into())),
            "begin" => self.environment().map(Some),
            "end" => Err(TexError("\\end without \\begin".into())),
            c => Err(TexError(format!("unsupported command \\{c}"))),
        }
    }

    fn environment(&mut self) -> Result<Node> {
        let name = self.raw_arg()?;
        let kind = match name.as_str() {
            "pmatrix" | "pmatrix*" => GridKind::Mat("\"(\""),
            "bmatrix" | "bmatrix*" => GridKind::Mat("\"[\""),
            "Bmatrix" => GridKind::Mat("\"{\""),
            "vmatrix" => GridKind::Mat("\"|\""),
            "Vmatrix" => GridKind::Mat("\"||\""),
            "matrix" | "smallmatrix" | "array" | "darray" | "subarray" => GridKind::Mat("#none"),
            "cases" | "dcases" => GridKind::Cases { reverse: false },
            "rcases" | "drcases" => GridKind::Cases { reverse: true },
            "aligned" | "align" | "align*" | "alignat" | "alignat*" | "alignedat" | "gather"
            | "gather*" | "gathered" | "split" | "equation" | "equation*" | "multline" => {
                GridKind::Aligned
            }
            n => return Err(TexError(format!("unsupported environment {n}"))),
        };
        // Column specs and alignat's column count.
        if matches!(name.as_str(), "array" | "darray" | "subarray" | "alignat" | "alignat*" | "alignedat") {
            let _ = self.raw_arg()?;
        }
        let mut rows: Vec<Vec<Vec<Node>>> = Vec::new();
        let mut row: Vec<Vec<Node>> = Vec::new();
        let mut hlines = Vec::new();
        loop {
            self.skip_space();
            if let Some(Tok::Cmd(c)) = self.peek()
                && c == "hline"
            {
                self.pos += 1;
                hlines.push(rows.len());
                continue;
            }
            let cell = self.list(&["end", "grid"])?;
            row.push(cell);
            match self.next() {
                Some(Tok::Amp) => {}
                Some(Tok::Row) => {
                    rows.push(std::mem::take(&mut row));
                }
                Some(Tok::Cmd(c)) if c == "end" => {
                    let end = self.raw_arg()?;
                    if end != name {
                        return Err(TexError(format!("\\begin{{{name}}} ended by \\end{{{end}}}")));
                    }
                    if !(row.len() == 1 && row[0].is_empty()) || rows.is_empty() {
                        rows.push(row);
                    }
                    break;
                }
                Some(Tok::Close) => return Err(TexError(format!("{name} was not ended"))),
                t => return Err(TexError(format!("unexpected {t:?} in {name}"))),
            }
        }
        // An \hline after the last row draws nothing in a matrix (augment needs an inner line).
        hlines.retain(|&h| h > 0 && h < rows.len());
        Ok(Node::Grid { kind, rows, hlines })
    }
}

/// `\cdot`, `\alpha`, `\sin`, …: commands that are a single Typst symbol or operator.
fn symbol(c: &str) -> Option<&'static str> {
    Some(match c {
        // Operators and relations.
        "cdot" => "dot.op",
        "times" => "times",
        "div" => "div",
        "pm" => "plus.minus",
        "mp" => "minus.plus",
        "ast" => "ast",
        "star" => "star",
        "circ" => "compose",
        "bullet" => "bullet",
        "diamond" => "diamond.stroked.small",
        "otimes" => "times.circle",
        "oplus" => "plus.circle",
        "square" => "square.stroked",
        "le" | "leq" => "lt.eq",
        "ge" | "geq" => "gt.eq",
        "lt" => "lt",
        "gt" => "gt",
        "ne" | "neq" => "eq.not",
        "nleq" => "lt.eq.not",
        "ngeq" => "gt.eq.not",
        "nless" => "lt.not",
        "ngtr" => "gt.not",
        "approx" => "approx",
        "equiv" => "equiv",
        "sim" => "tilde.op",
        "cong" => "tilde.equiv",
        "propto" => "prop",
        "in" => "in",
        "notin" => "in.not",
        "ni" => "in.rev",
        "subset" => "subset",
        "subseteq" => "subset.eq",
        "supset" => "supset",
        "supseteq" => "supset.eq",
        "nsubset" => "subset.not",
        "nsubseteq" => "subset.eq.not",
        "cup" => "union",
        "cap" => "sect",
        "emptyset" | "varnothing" => "emptyset",
        "forall" => "forall",
        "exists" => "exists",
        "neg" | "lnot" => "not",
        "land" | "wedge" => "and",
        "lor" | "vee" => "or",
        "iff" => "arrow.l.r.double.long",
        "implies" => "arrow.r.double.long",
        "to" | "rightarrow" => "arrow.r",
        "leftarrow" | "gets" => "arrow.l",
        "Rightarrow" => "arrow.r.double",
        "Leftarrow" => "arrow.l.double",
        "leftrightarrow" => "arrow.l.r",
        "Leftrightarrow" => "arrow.l.r.double",
        "uparrow" | "uarr" => "arrow.t",
        "downarrow" | "darr" => "arrow.b",
        "mapsto" => "arrow.r.bar",
        "models" => "models",
        "mid" => "divides",
        "parallel" => "parallel",
        "perp" | "bot" => "perp",
        "top" => "top",
        "angle" => "angle",
        "degree" => "degree",
        "prime" => "prime",
        "partial" => "partial",
        "nabla" => "nabla",
        "infty" => "infinity",
        "aleph" => "aleph",
        "beth" => "beth",
        "gimel" => "gimel",
        "daleth" => "daleth",
        "hbar" => "planck.reduce",
        "ell" => "ell",
        "Re" => "Re",
        "Im" => "Im",
        "dots" | "ldots" => "dots.h",
        "cdots" => "dots.h.c",
        "vdots" => "dots.v",
        "ddots" => "dots.down",
        "lfloor" => "floor.l",
        "rfloor" => "floor.r",
        "lceil" => "ceil.l",
        "rceil" => "ceil.r",
        "langle" => "angle.l",
        "rangle" => "angle.r",
        "vert" => "bar.v",
        "Vert" => "bar.v.double",
        "backslash" => "backslash",
        "checkmark" => "checkmark",
        // Big operators.
        "sum" => "sum",
        "prod" => "product",
        "coprod" => "product.co",
        "int" => "integral",
        "iint" => "integral.double",
        "iiint" => "integral.triple",
        "oint" => "integral.cont",
        "oiint" => "integral.surf",
        "oiiint" => "integral.vol",
        "bigcup" => "union.big",
        "bigcap" => "sect.big",
        // Functions (upright operators).
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "sec" => "sec",
        "csc" => "csc",
        "cot" => "cot",
        "sinh" => "sinh",
        "cosh" => "cosh",
        "tanh" => "tanh",
        "coth" => "coth",
        "arcsin" => "arcsin",
        "arccos" => "arccos",
        "arctan" => "arctan",
        "log" => "log",
        "ln" => "ln",
        "lg" => "lg",
        "exp" => "exp",
        "lim" => "lim",
        "max" => "max",
        "min" => "min",
        "sup" => "sup",
        "inf" => "inf",
        "det" => "det",
        "gcd" => "gcd",
        "deg" => "deg",
        "arg" => "arg",
        "dim" => "dim",
        "ker" => "ker",
        "mod" => "mod",
        "bmod" => "mod",
        // Greek.
        "alpha" => "alpha",
        "beta" => "beta",
        "gamma" => "gamma",
        "delta" => "delta",
        "epsilon" => "epsilon.alt",
        "varepsilon" => "epsilon",
        "zeta" => "zeta",
        "eta" => "eta",
        "theta" => "theta",
        "vartheta" => "theta.alt",
        "iota" => "iota",
        "kappa" => "kappa",
        "lambda" => "lambda",
        "mu" => "mu",
        "nu" => "nu",
        "xi" => "xi",
        "pi" => "pi",
        "varpi" => "pi.alt",
        "rho" => "rho",
        "varrho" => "rho.alt",
        "sigma" => "sigma",
        "varsigma" => "sigma.alt",
        "tau" => "tau",
        "upsilon" => "upsilon",
        "phi" => "phi.alt",
        "varphi" => "phi",
        "chi" => "chi",
        "psi" => "psi",
        "omega" => "omega",
        "Gamma" => "Gamma",
        "Delta" => "Delta",
        "Theta" => "Theta",
        "Lambda" => "Lambda",
        "Xi" => "Xi",
        "Pi" => "Pi",
        "Sigma" => "Sigma",
        "Upsilon" => "Upsilon",
        "Phi" => "Phi",
        "Psi" => "Psi",
        "Omega" => "Omega",
        _ => return None,
    })
}

/// A TeX color (`#0000ff`, `blue`, `firebrick`) as a Typst color expression.
fn typst_color(c: &str) -> Result<String> {
    let c = c.trim();
    if let Some(hex) = c.strip_prefix('#')
        && (hex.len() == 3 || hex.len() == 6)
        && hex.chars().all(|h| h.is_ascii_hexdigit())
    {
        return Ok(format!("rgb(\"#{hex}\")"));
    }
    let hex = match c.to_ascii_lowercase().as_str() {
        "aqua" | "cyan" => "00ffff",
        "salmon" => "fa8072",
        "yellow" => "ffff00",
        "blue" => "0000ff",
        "red" => "ff0000",
        "green" => "008000",
        "black" => "000000",
        "gray" | "grey" => "808080",
        "orange" => "ffa500",
        "purple" => "800080",
        "pink" => "ffc0cb",
        "firebrick" => "b22222",
        "white" => "ffffff",
        c => return Err(TexError(format!("unknown color {c}"))),
    };
    Ok(format!("rgb(\"#{hex}\")"))
}

/// A Typst string literal.
fn quote(s: &str) -> String {
    let mut q = String::with_capacity(s.len() + 2);
    q.push('"');
    for c in s.chars() {
        match c {
            '"' => q.push_str("\\\""),
            '\\' => q.push_str("\\\\"),
            '\n' => q.push_str("\\n"),
            c => q.push(c),
        }
    }
    q.push('"');
    q
}

// ---------------------------------------------------------------------------------------
// Emitter

fn emit_list(nodes: &[Node], out: &mut String) -> Result<()> {
    let mut i = 0;
    while i < nodes.len() {
        // A number with thousands separators: keep the commas tight.
        if let Node::Char(c) = &nodes[i]
            && c.is_ascii_digit()
        {
            let mut j = i;
            let mut num = String::new();
            while j < nodes.len() {
                match &nodes[j] {
                    Node::Char(d) if d.is_ascii_digit() || *d == '.' => num.push(*d),
                    Node::Char(',')
                        if matches!(nodes.get(j + 1), Some(Node::Char(d)) if d.is_ascii_digit())
                            && !num.is_empty() =>
                    {
                        num.push(',')
                    }
                    _ => break,
                }
                j += 1;
            }
            out.push(' ');
            for part in num.split(',').enumerate() {
                if part.0 > 0 {
                    out.push_str("class(\"normal\", \\,)");
                }
                out.push_str(part.1);
            }
            out.push(' ');
            i = j;
            continue;
        }
        emit(&nodes[i], out)?;
        i += 1;
    }
    Ok(())
}

fn emit(n: &Node, out: &mut String) -> Result<()> {
    match n {
        Node::Code(s) => {
            out.push(' ');
            out.push_str(s);
            out.push(' ');
        }
        Node::Char(c) => {
            out.push(' ');
            emit_char(*c, out);
            out.push(' ');
        }
        Node::Text(t) => {
            out.push(' ');
            out.push_str(&quote(t));
            out.push(' ');
        }
        Node::Group(v) => {
            // Grouping is invisible in TeX. A group that is a script's base is emitted as a
            // box (see Script); elsewhere its contents are inlined.
            emit_list(v, out)?;
        }
        Node::Script { base, sup, sub } => {
            match base.as_ref() {
                Node::Group(v) if needs_box(v) => {
                    out.push_str(" #box($");
                    emit_list(v, out)?;
                    out.push_str("$)");
                }
                Node::Group(v) if v.is_empty() => out.push_str(" \"\""),
                b => emit(b, out)?,
            }
            if let Some(s) = sup {
                out.push_str("^(");
                emit_arg(s, out)?;
                out.push(')');
            }
            if let Some(s) = sub {
                out.push_str("_(");
                emit_arg(s, out)?;
                out.push(')');
            }
            out.push(' ');
        }
        Node::Call(f, args) => {
            out.push(' ');
            out.push_str(f);
            if !f.ends_with(' ') {
                out.push('(');
            }
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                if a.is_empty() {
                    out.push_str("\"\"");
                } else {
                    emit_list(a, out)?;
                }
            }
            out.push_str(") ");
        }
        Node::Wrap(w, body) => {
            out.push_str(" #");
            out.push_str(w);
            out.push_str("[$");
            emit_list(body, out)?;
            out.push_str("$] ");
        }
        Node::Lr(l, body, r) => {
            out.push_str(" lr(");
            out.push_str(l);
            out.push(' ');
            emit_list(body, out)?;
            out.push(' ');
            out.push_str(r);
            out.push_str(") ");
        }
        Node::Grid { kind, rows, hlines } => emit_grid(*kind, rows, hlines, out)?,
    }
    Ok(())
}

/// Does a script's base group need a box (it has more than one visible item)?
fn needs_box(v: &[Node]) -> bool {
    v.len() > 1
}

fn emit_arg(n: &Node, out: &mut String) -> Result<()> {
    match n {
        Node::Group(v) if v.is_empty() => {
            out.push_str("\"\"");
            Ok(())
        }
        Node::Group(v) => emit_list(v, out),
        n => emit(n, out),
    }
}

fn emit_char(c: char, out: &mut String) {
    match c {
        // Typst math syntax: escape it.
        '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '/' | '_' | '^' | '&' | '#' | '$' | '"'
        | '\\' | '@' | '*' | '<' | '>' | '=' | '-' | '+' | '!' | ':' | '|' | '.' | '~' | '`'
        | '\'' | '?' | '%' => {
            if c == '\'' {
                out.push('\'');
            } else if c == '-' {
                out.push('-');
            } else if c == '+' || c == '=' || c == '<' || c == '>' || c == '!' {
                // Plain operators: spaced, so no shorthand (`<=`, `!=`, `->`) can form.
                out.push(c);
            } else if c == '.' || c == '%' || c == '?' {
                out.push(c);
            } else if c == '~' {
                out.push_str("space.nobreak");
            } else {
                out.push('\\');
                out.push(c);
            }
        }
        c => out.push(c),
    }
}

fn emit_grid(kind: GridKind, rows: &[Vec<Vec<Node>>], hlines: &[usize], out: &mut String) -> Result<()> {
    match kind {
        GridKind::Mat(delim) => {
            write!(out, " mat(delim: {delim}, ").unwrap();
            if !hlines.is_empty() {
                let lines: Vec<String> = hlines.iter().map(|h| h.to_string()).collect();
                write!(out, "augment: #(hline: ({},)), ", lines.join(", ")).unwrap();
            }
            let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
            for (i, row) in rows.iter().enumerate() {
                if i > 0 {
                    out.push_str("; ");
                }
                for j in 0..width {
                    if j > 0 {
                        out.push_str(", ");
                    }
                    match row.get(j) {
                        Some(cell) if !cell.is_empty() => emit_list(cell, out)?,
                        _ => out.push_str("\"\""),
                    }
                }
            }
            out.push_str(") ");
        }
        GridKind::Cases { reverse } => {
            out.push_str(" cases(");
            if reverse {
                out.push_str("reverse: #true, ");
            }
            for (i, row) in rows.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                for (j, cell) in row.iter().enumerate() {
                    if j > 0 {
                        out.push_str(" & ");
                    }
                    emit_list(cell, out)?;
                }
            }
            out.push_str(") ");
        }
        GridKind::Aligned => {
            for (i, row) in rows.iter().enumerate() {
                if i > 0 {
                    out.push_str(" \\ ");
                }
                for (j, cell) in row.iter().enumerate() {
                    if j > 0 {
                        out.push_str(" & ");
                    }
                    emit_list(cell, out)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::to_typst;

    /// Compare without spaces: the emitter's spacing is not significant.
    fn t(tex: &str) -> String {
        to_typst(tex).unwrap().replace(' ', "")
    }

    #[test]
    fn calc_lines() {
        assert_eq!(t(r"L = 24\, {\text{ft}}"), r#"L=24thin"ft""#);
        let m = t(r"M = w \dfrac{L^{2}}{8} = \textcolor{#0000ff}{(1.5\, {\text{kip/ft}})} = 108 \; {\text{kip}\mkern1mu{\cdot}\mkern1mu\text{ft}}");
        assert!(m.contains("display(frac(L^(2),8))"), "{m}");
        assert!(m.contains(r##"#text(fill:rgb("#0000ff"))[$\(1.5thin"kip/ft"\)$]"##), "{m}");
        assert!(m.contains(r##"108thick"kip"#h(0.0556em)dot.op#h(0.0556em)"ft""##), "{m}");
    }

    #[test]
    fn letters_are_separate_and_specials_escaped() {
        assert_eq!(to_typst("ab").unwrap(), "a b");
        assert_eq!(t(r"a/b, c;"), r"a\/b\,c\;");
        assert_eq!(t(r"1,605.33"), r#"1class("normal",\,)605.33"#);
        assert_eq!(t(r#"\text{a "b"}"#), r#""a\"b\"""#);
    }

    #[test]
    fn matrices_cases_and_delimiters() {
        assert_eq!(t(r"\begin{pmatrix}1 & 2 \\ 3 & 4\end{pmatrix}"), r#"mat(delim:"(",1,2;3,4)"#);
        assert_eq!(t(r"\left(\dfrac{a}{b}\right)"), "lr(paren.ldisplay(frac(a,b))paren.r)");
        assert_eq!(t(r"\left⟨{{n}\atop{k}}\right⟩"), r#"lr(⟨mat(delim:#none,n;k)⟩)"#);
        assert!(t(r"\begin{cases}a & x \\ b & y\end{cases}").starts_with("cases("));
    }

    #[test]
    fn colors() {
        assert!(t(r"\colorbox{Salmon}{ n.g.}").starts_with(r##"#box(fill:rgb("#fa8072")"##));
        assert!(to_typst(r"\textcolor{chartreuse}{x}").is_err());
    }

    #[test]
    fn unknown_commands_are_errors() {
        assert!(to_typst(r"\frobnicate{x}").is_err());
        assert!(to_typst(r"\left( x").is_err());
    }
}
