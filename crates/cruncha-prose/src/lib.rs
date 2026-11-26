// SPDX-License-Identifier: GPL-3.0-or-later

//! Prose cells. A prose cell's `source` is the ProseMirror JSON of one top-level editor block
//! (app/src/lib/editor/schema.ts): a paragraph, a heading, a bulleted or numbered list, a
//! block quote or a horizontal rule. This crate reads that JSON into a [`Block`] tree (for
//! reports and the agents' text mirror), writes it back in exactly the form the editor writes
//! it (so a cell written here doesn't look edited when the editor opens it), and converts
//! Markdown both ways: agents propose prose as Markdown, the mirror shows it as Markdown,
//! pasted Markdown becomes cells, and templates (ADR-0014) are Markdown files.
//!
//! The tree is deliberately small: what the editor's schema can hold. Markdown it can't hold
//! degrades to text: an image becomes its alt text, raw HTML its source (except `<sub>`,
//! `<sup>` and `<br>`), a heading inside a list or quote bold text, and a fenced code block
//! that isn't a cell (not at the top level, or not `calc`, `python`, `rust` or `tex`) a
//! paragraph in code type.

use std::collections::BTreeMap;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde_json::Value;

/// One top-level prose block, or a block inside a list item or a quote.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Only at the top level (the editor allows headings nowhere else).
    Heading { level: u8, content: Vec<Inline> },
    Paragraph(Vec<Inline>),
    /// Each item is a paragraph followed by any other blocks.
    BulletList(Vec<Vec<Block>>),
    OrderedList { start: u64, items: Vec<Vec<Block>> },
    Quote(Vec<Block>),
    /// Only at the top level.
    Rule,
    /// A new page in reports (ADR-0028); only at the top level.
    PageBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text { text: String, marks: Marks },
    Break,
}

/// The marks on a run of text. `link` is an `http`, `https` or `mailto` URL.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Marks {
    pub strong: bool,
    pub em: bool,
    pub code: bool,
    pub sub: bool,
    pub sup: bool,
    pub link: Option<String>,
}

/// What a piece of Markdown becomes: prose blocks and the cells fenced in it.
#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    Prose(Block),
    /// A fenced `python`, `rust` or `tex` block: a code cell of that kind.
    Code { lang: String, source: String, attrs: BTreeMap<String, String> },
    /// One line of a fenced `calc` block: a calc cell. A fence's attributes go with its first
    /// calc. In a group (several calcs in one fence), a line starting with `>` is the
    /// description of the calc after it.
    Calc { source: String, attrs: BTreeMap<String, String> },
}

/// A calc fence's statements: one per line, except that a data frame literal (` `` `… ` `` `)
/// keeps its lines (tab-separated rows) until it closes, and `function …` keeps its lines to
/// its matching `end`. A `>` line (a description) is returned as it is.
fn calc_statements(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut open: Option<String> = None;
    let mut depth = 0usize;
    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some(mut cur) = open.take() {
            cur.push('\n');
            cur.push_str(if depth > 0 { line } else { raw });
            let in_literal = cur.matches("``").count() % 2 == 1;
            if depth > 0 {
                let word = line.split_whitespace().next().unwrap_or("");
                match word {
                    "if" | "while" | "for" => depth += 1,
                    "end" => depth -= 1,
                    _ => {}
                }
            }
            if in_literal || depth > 0 {
                open = Some(cur);
            } else {
                out.push(cur);
            }
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("function ") {
            depth = 1;
            open = Some(line.to_owned());
        } else if line.matches("``").count() % 2 == 1 {
            open = Some(line.to_owned());
        } else {
            out.push(line.to_owned());
        }
    }
    out.extend(open);
    out
}

/// A page break's Markdown line (ADR-0028), as pandoc and most Markdown-to-PDF tools read it.
pub const PAGE_BREAK: &str = "\\pagebreak";

/// A fence's info string: its language and `{key="value" key2=value2}` attributes
/// (```` ```calc {description="Radial depth"} ````, ADR-0014).
pub fn fence_info(info: &str) -> (String, BTreeMap<String, String>) {
    let info = info.trim();
    let (lang, rest) = match info.find(|c: char| c.is_whitespace() || c == '{') {
        Some(i) => (&info[..i], info[i..].trim()),
        None => (info, ""),
    };
    let mut attrs = BTreeMap::new();
    let body = rest.strip_prefix('{').and_then(|r| r.strip_suffix('}')).unwrap_or("");
    let mut chars = body.chars().peekable();
    loop {
        while chars.next_if(|c| c.is_whitespace() || *c == ',').is_some() {}
        let key: String = std::iter::from_fn(|| chars.next_if(|c| !c.is_whitespace() && *c != '=')).collect();
        if key.is_empty() {
            break;
        }
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        if chars.next_if_eq(&'=').is_none() {
            attrs.insert(key, String::new());
            continue;
        }
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        let value: String = if chars.next_if_eq(&'"').is_some() {
            // A quote closes the value only before the end or the next `key=`: CommonMark has
            // already undone a hand-typed `\"` in the info string, so a bare quote inside the
            // value is the author's (`{description="the "B" exposure"}`).
            let mut v = String::new();
            while let Some(c) = chars.next() {
                match c {
                    '"' if closes(chars.clone().collect::<String>().as_str()) => break,
                    '\\' => v.extend(chars.next()),
                    c => v.push(c),
                }
            }
            v
        } else {
            std::iter::from_fn(|| chars.next_if(|c| !c.is_whitespace() && *c != ',')).collect()
        };
        attrs.insert(key, value);
    }
    (lang.to_ascii_lowercase(), attrs)
}

/// Whether a quote followed by `rest` closes an attribute value: `rest` is empty, or the next
/// attribute (`, key=…`, ` key=…`, ` flag`) follows.
fn closes(rest: &str) -> bool {
    let r = rest.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
    if r.is_empty() {
        return true;
    }
    if r.len() == rest.len() {
        return false; // `"` directly followed by text: part of the value
    }
    let key: String = r.chars().take_while(|c| !c.is_whitespace() && *c != '=' && *c != '"').collect();
    !key.is_empty() && {
        let after = r[key.len()..].trim_start();
        after.is_empty() || after.starts_with('=') || r[key.len()..].starts_with(char::is_whitespace)
    }
}

/// A link target the editor and reports will follow: `http`, `https` or `mailto` only.
pub fn safe_href(href: &str) -> Option<String> {
    let h = href.trim();
    let lower = h.to_ascii_lowercase();
    (lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:"))
        .then(|| h.to_owned())
}

// ---------------------------------------------------------------------------------------
// Stored JSON

/// A prose cell's source as a tree. Never fails: text that isn't JSON is a plain paragraph,
/// and node types this version doesn't know keep their text.
pub fn parse_json(source: &str) -> Block {
    match serde_json::from_str::<Value>(source) {
        Ok(v) if v.is_object() => block_of(&v, true),
        _ => Block::Paragraph(plain(source)),
    }
}

fn plain(text: &str) -> Vec<Inline> {
    if text.is_empty() { vec![] } else { vec![Inline::Text { text: text.to_owned(), marks: Marks::default() }] }
}

fn children(v: &Value) -> impl Iterator<Item = &Value> {
    v["content"].as_array().into_iter().flatten()
}

fn block_of(v: &Value, top: bool) -> Block {
    match v["type"].as_str().unwrap_or_default() {
        "heading" if top => {
            let level = v["attrs"]["level"].as_u64().unwrap_or(1).clamp(1, 6) as u8;
            Block::Heading { level, content: inlines_of(v) }
        }
        "bullet_list" => Block::BulletList(children(v).map(item_of).collect()),
        "ordered_list" => Block::OrderedList {
            start: v["attrs"]["order"].as_u64().unwrap_or(1),
            items: children(v).map(item_of).collect(),
        },
        "blockquote" => Block::Quote(children(v).map(|c| block_of(c, false)).collect()),
        "horizontal_rule" if top => Block::Rule,
        "page_break" if top => Block::PageBreak,
        "paragraph" | "heading" => Block::Paragraph(inlines_of(v)),
        _ => Block::Paragraph(plain(&text_of(v))),
    }
}

fn item_of(v: &Value) -> Vec<Block> {
    children(v).map(|c| block_of(c, false)).collect()
}

fn text_of(v: &Value) -> String {
    match v["text"].as_str() {
        Some(t) => t.to_owned(),
        None => children(v).map(text_of).collect::<Vec<_>>().join(" "),
    }
}

fn inlines_of(v: &Value) -> Vec<Inline> {
    let mut out = Vec::new();
    for n in children(v) {
        match n["type"].as_str() {
            Some("text") => {
                let mut marks = Marks::default();
                for m in n["marks"].as_array().into_iter().flatten() {
                    match m["type"].as_str() {
                        Some("strong") => marks.strong = true,
                        Some("em") => marks.em = true,
                        Some("code") => marks.code = true,
                        Some("sub") => marks.sub = true,
                        Some("sup") => marks.sup = true,
                        Some("link") => marks.link = m["attrs"]["href"].as_str().and_then(safe_href),
                        _ => {}
                    }
                }
                push_text(&mut out, n["text"].as_str().unwrap_or_default(), &marks);
            }
            Some("hard_break") => out.push(Inline::Break),
            _ => {}
        }
    }
    out
}

/// Appends text, merged with the previous run when the marks agree (as ProseMirror does).
fn push_text(out: &mut Vec<Inline>, text: &str, marks: &Marks) {
    if text.is_empty() {
        return;
    }
    if let Some(Inline::Text { text: t, marks: m }) = out.last_mut()
        && m == marks
    {
        t.push_str(text);
        return;
    }
    out.push(Inline::Text { text: text.to_owned(), marks: marks.clone() });
}

/// The tree as the editor stores it: ProseMirror's `toJSON` with the cell attributes left out
/// (app/src/lib/editor/cells.ts `nodeToContent`), keys in its order, marks in schema order.
pub fn to_json(b: &Block) -> String {
    let mut s = String::new();
    write_block(&mut s, b);
    s
}

fn q(s: &str) -> String {
    serde_json::to_string(s).expect("a string serializes")
}

fn node(s: &mut String, ty: &str, attrs: Option<String>, content: impl FnOnce(&mut String) -> bool) {
    s.push_str(&format!("{{\"type\":{}", q(ty)));
    if let Some(a) = attrs {
        s.push_str(&format!(",\"attrs\":{a}"));
    }
    let start = s.len();
    s.push_str(",\"content\":[");
    if content(s) {
        s.push(']');
    } else {
        s.truncate(start);
    }
    s.push('}');
}

fn write_list(s: &mut String, items: &[Vec<Block>]) -> bool {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        node(s, "list_item", None, |s| write_blocks(s, item));
    }
    !items.is_empty()
}

fn write_blocks(s: &mut String, blocks: &[Block]) -> bool {
    for (i, b) in blocks.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        write_block(s, b);
    }
    !blocks.is_empty()
}

fn write_block(s: &mut String, b: &Block) {
    match b {
        Block::Heading { level, content } => {
            node(s, "heading", Some(format!("{{\"level\":{level}}}")), |s| write_inlines(s, content))
        }
        Block::Paragraph(content) => node(s, "paragraph", None, |s| write_inlines(s, content)),
        Block::BulletList(items) => node(s, "bullet_list", None, |s| write_list(s, items)),
        Block::OrderedList { start, items } => {
            node(s, "ordered_list", Some(format!("{{\"order\":{start}}}")), |s| write_list(s, items))
        }
        Block::Quote(blocks) => node(s, "blockquote", None, |s| write_blocks(s, blocks)),
        Block::Rule => node(s, "horizontal_rule", None, |_| false),
        Block::PageBreak => node(s, "page_break", None, |_| false),
    }
}

fn write_inlines(s: &mut String, content: &[Inline]) -> bool {
    let mut first = true;
    for n in content {
        match n {
            Inline::Text { text, .. } if text.is_empty() => continue,
            _ if !first => s.push(','),
            _ => {}
        }
        first = false;
        match n {
            Inline::Break => s.push_str("{\"type\":\"hard_break\"}"),
            Inline::Text { text, marks } => {
                s.push_str("{\"type\":\"text\"");
                let mut ms = Vec::new();
                for (on, name) in [(marks.strong, "strong"), (marks.em, "em"), (marks.code, "code"), (marks.sub, "sub"), (marks.sup, "sup")] {
                    if on {
                        ms.push(format!("{{\"type\":\"{name}\"}}"));
                    }
                }
                if let Some(h) = &marks.link {
                    ms.push(format!("{{\"type\":\"link\",\"attrs\":{{\"href\":{}}}}}", q(h)));
                }
                if !ms.is_empty() {
                    s.push_str(&format!(",\"marks\":[{}]", ms.join(",")));
                }
                s.push_str(&format!(",\"text\":{}}}", q(text)));
            }
        }
    }
    !first
}

// ---------------------------------------------------------------------------------------
// Markdown in

enum Frame {
    Root,
    Quote(Vec<Block>),
    List { start: Option<u64>, items: Vec<Vec<Block>> },
    Item(Vec<Block>),
    /// `implicit`: the text of a tight list item, which has no paragraph events.
    Para { content: Vec<Inline>, implicit: bool },
    Heading { level: u8, content: Vec<Inline> },
    Code { lang: String, attrs: BTreeMap<String, String>, text: String },
    Html(String),
}

struct Builder {
    stack: Vec<Frame>,
    pieces: Vec<Piece>,
    strong: usize,
    em: usize,
    sub: bool,
    sup: bool,
    links: Vec<Option<String>>,
}

impl Builder {
    fn marks(&self) -> Marks {
        Marks {
            strong: self.strong > 0,
            em: self.em > 0,
            code: false,
            sub: self.sub,
            sup: self.sup,
            link: self.links.last().cloned().flatten(),
        }
    }

    fn at_root(&self) -> bool {
        matches!(self.stack.last(), Some(Frame::Root))
    }

    fn close_implicit(&mut self) {
        if let Some(Frame::Para { implicit: true, .. }) = self.stack.last()
            && let Some(Frame::Para { content, .. }) = self.stack.pop()
        {
            self.push_block(Block::Paragraph(content));
        }
    }

    fn push_block(&mut self, b: Block) {
        self.close_implicit();
        match self.stack.last_mut() {
            Some(Frame::Root) | None => self.pieces.push(Piece::Prose(b)),
            Some(Frame::Quote(v)) | Some(Frame::Item(v)) => v.push(b),
            // Lists hold items only; anything else between items joins the last one.
            Some(Frame::List { items, .. }) => match items.last_mut() {
                Some(v) => v.push(b),
                None => items.push(vec![b]),
            },
            _ => {}
        }
    }

    fn inline(&mut self, n: Inline) {
        match self.stack.last_mut() {
            Some(Frame::Para { content, .. }) | Some(Frame::Heading { content, .. }) => match n {
                Inline::Text { text, marks } => push_text(content, &text, &marks),
                Inline::Break => content.push(Inline::Break),
            },
            Some(Frame::Code { text, .. }) | Some(Frame::Html(text)) => {
                if let Inline::Text { text: t, .. } = n {
                    text.push_str(&t);
                }
            }
            _ => {
                self.stack.push(Frame::Para { content: vec![], implicit: true });
                self.inline(n);
            }
        }
    }

    fn text(&mut self, t: &str) {
        let marks = self.marks();
        self.inline(Inline::Text { text: t.to_owned(), marks });
    }

    fn event(&mut self, e: Event) {
        match e {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) | Event::Html(t) => self.text(&t),
            Event::Code(t) => {
                let mut marks = self.marks();
                marks.code = true;
                self.inline(Inline::Text { text: t.to_string(), marks });
            }
            Event::InlineMath(t) | Event::DisplayMath(t) => self.text(&t),
            Event::InlineHtml(t) => match t.trim().to_ascii_lowercase().replace(' ', "").as_str() {
                "<sub>" => self.sub = true,
                "</sub>" => self.sub = false,
                "<sup>" => self.sup = true,
                "</sup>" => self.sup = false,
                "<br>" | "<br/>" => self.inline(Inline::Break),
                _ => self.text(&t),
            },
            Event::SoftBreak => self.text(" "),
            Event::HardBreak => {
                if matches!(self.stack.last(), Some(Frame::Heading { .. })) {
                    self.text(" ");
                } else {
                    self.inline(Inline::Break);
                }
            }
            Event::Rule => {
                self.close_implicit();
                if self.at_root() {
                    self.push_block(Block::Rule);
                }
            }
            Event::TaskListMarker(done) => self.text(if done { "[x] " } else { "[ ] " }),
            Event::FootnoteReference(t) => self.text(&format!("[^{t}]")),
        }
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                self.close_implicit();
                self.stack.push(Frame::Para { content: vec![], implicit: false });
            }
            Tag::Heading { level, .. } => {
                self.close_implicit();
                self.stack.push(Frame::Heading { level: level as u8, content: vec![] });
            }
            Tag::BlockQuote(_) => {
                self.close_implicit();
                self.stack.push(Frame::Quote(vec![]));
            }
            Tag::List(start) => {
                self.close_implicit();
                self.stack.push(Frame::List { start, items: vec![] });
            }
            Tag::Item => self.stack.push(Frame::Item(vec![])),
            Tag::CodeBlock(kind) => {
                self.close_implicit();
                let (lang, attrs) = match kind {
                    CodeBlockKind::Fenced(info) => fence_info(&info),
                    CodeBlockKind::Indented => (String::new(), BTreeMap::new()),
                };
                self.stack.push(Frame::Code { lang, attrs, text: String::new() });
            }
            Tag::HtmlBlock => {
                self.close_implicit();
                self.stack.push(Frame::Html(String::new()));
            }
            Tag::Emphasis => self.em += 1,
            Tag::Strong => self.strong += 1,
            Tag::Link { dest_url, .. } => self.links.push(safe_href(&dest_url)),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                if let Some(Frame::Para { content, .. }) = self.stack.pop() {
                    // A line holding only `\pagebreak` is a page break (ADR-0028).
                    let page = self.at_root()
                        && matches!(content.as_slice(), [Inline::Text { text, marks }] if text.trim() == PAGE_BREAK && *marks == Marks::default());
                    self.push_block(if page { Block::PageBreak } else { Block::Paragraph(content) });
                }
            }
            TagEnd::Heading(_) => {
                if let Some(Frame::Heading { level, content }) = self.stack.pop() {
                    if self.at_root() {
                        self.push_block(Block::Heading { level, content });
                    } else {
                        let bold = content
                            .into_iter()
                            .map(|n| match n {
                                Inline::Text { text, mut marks } => {
                                    marks.strong = true;
                                    Inline::Text { text, marks }
                                }
                                b => b,
                            })
                            .collect();
                        self.push_block(Block::Paragraph(bold));
                    }
                }
            }
            TagEnd::BlockQuote(_) => {
                self.close_implicit();
                if let Some(Frame::Quote(v)) = self.stack.pop() {
                    self.push_block(Block::Quote(if v.is_empty() { vec![Block::Paragraph(vec![])] } else { v }));
                }
            }
            TagEnd::List(_) => {
                self.close_implicit();
                if let Some(Frame::List { start, items }) = self.stack.pop() {
                    self.push_block(match start {
                        Some(start) => Block::OrderedList { start, items },
                        None => Block::BulletList(items),
                    });
                }
            }
            TagEnd::Item => {
                self.close_implicit();
                if let Some(Frame::Item(mut v)) = self.stack.pop() {
                    // A list item starts with a paragraph (the editor's schema).
                    if !matches!(v.first(), Some(Block::Paragraph(_))) {
                        v.insert(0, Block::Paragraph(vec![]));
                    }
                    if let Some(Frame::List { items, .. }) = self.stack.last_mut() {
                        items.push(v);
                    }
                }
            }
            TagEnd::CodeBlock => {
                if let Some(Frame::Code { lang, attrs, text }) = self.stack.pop() {
                    let text = text.strip_suffix('\n').unwrap_or(&text).to_owned();
                    let lang = match lang.as_str() {
                        "py" => "python",
                        "rs" => "rust",
                        l => l,
                    };
                    match lang {
                        "python" | "rust" | "tex" if self.at_root() => {
                            self.pieces.push(Piece::Code { lang: lang.to_owned(), source: text, attrs })
                        }
                        "calc" if self.at_root() => {
                            let mut attrs = Some(attrs);
                            let mut description: Option<String> = None;
                            for source in calc_statements(&text) {
                                if let Some(d) = source.strip_prefix('>') {
                                    description = Some(d.trim().to_owned());
                                    continue;
                                }
                                let mut attrs = attrs.take().unwrap_or_default();
                                if let Some(d) = description.take() {
                                    attrs.insert("description".to_owned(), d);
                                }
                                self.pieces.push(Piece::Calc { source, attrs });
                            }
                        }
                        _ => {
                            let marks = Marks { code: true, ..Marks::default() };
                            let block = Block::Paragraph(lines(&text, &marks));
                            self.push_block(block);
                        }
                    }
                }
            }
            TagEnd::HtmlBlock => {
                if let Some(Frame::Html(text)) = self.stack.pop() {
                    let text = text.trim();
                    if !text.is_empty() {
                        self.push_block(Block::Paragraph(lines(text, &Marks::default())));
                    }
                }
            }
            TagEnd::Emphasis => self.em = self.em.saturating_sub(1),
            TagEnd::Strong => self.strong = self.strong.saturating_sub(1),
            TagEnd::Link => {
                self.links.pop();
            }
            _ => {}
        }
    }
}

/// Lines of text as runs separated by hard breaks.
fn lines(text: &str, marks: &Marks) -> Vec<Inline> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push(Inline::Break);
        }
        push_text(&mut out, line, marks);
    }
    out
}

/// Markdown (CommonMark: a single line break is a space) as prose blocks and fenced cells.
pub fn from_markdown(md: &str) -> Vec<Piece> {
    let mut b = Builder { stack: vec![Frame::Root], pieces: vec![], strong: 0, em: 0, sub: false, sup: false, links: vec![] };
    for e in Parser::new_ext(md, Options::empty()) {
        b.event(e);
    }
    b.close_implicit();
    b.pieces
}

// ---------------------------------------------------------------------------------------
// Markdown out

/// A block as Markdown that [`from_markdown`] reads back as the same block.
pub fn to_markdown(b: &Block) -> String {
    match b {
        Block::Heading { level, content } => {
            let text: Vec<Inline> = content
                .iter()
                .map(|n| match n {
                    Inline::Break => Inline::Text { text: " ".into(), marks: Marks::default() },
                    t => t.clone(),
                })
                .collect();
            format!("{} {}", "#".repeat(*level as usize), inline_markdown(&text))
        }
        Block::Paragraph(content) => paragraph_markdown(content),
        Block::BulletList(items) => list_markdown(items, |_| "- ".to_owned()),
        Block::OrderedList { start, items } => list_markdown(items, |i| format!("{}. ", start + i as u64)),
        Block::Quote(blocks) => blocks_markdown(blocks)
            .lines()
            .map(|l| if l.is_empty() { ">".to_owned() } else { format!("> {l}") })
            .collect::<Vec<_>>()
            .join("\n"),
        Block::Rule => "---".to_owned(),
        Block::PageBreak => PAGE_BREAK.to_owned(),
    }
}

fn blocks_markdown(blocks: &[Block]) -> String {
    blocks.iter().map(to_markdown).collect::<Vec<_>>().join("\n\n")
}

fn list_markdown(items: &[Vec<Block>], marker: impl Fn(usize) -> String) -> String {
    let tight = items.iter().all(|i| i.len() <= 1);
    let out: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let m = marker(i);
            let pad = " ".repeat(m.len());
            let body = blocks_markdown(item);
            let mut s = String::new();
            for (j, line) in body.split('\n').enumerate() {
                if j == 0 {
                    s.push_str(&m);
                } else {
                    s.push('\n');
                    if !line.is_empty() {
                        s.push_str(&pad);
                    }
                }
                s.push_str(line);
            }
            s.trim_end().to_owned()
        })
        .collect();
    out.join(if tight { "\n" } else { "\n\n" })
}

fn paragraph_markdown(content: &[Inline]) -> String {
    let text = inline_markdown(content);
    // A line that would start a block (`# `, `- `, `1. `, `> `, a rule or an underline) is
    // escaped so it stays text.
    text.split('\n')
        .map(|line| {
            let t = line.trim_start();
            let lead = &line[..line.len() - t.len()];
            let bytes = t.as_bytes();
            let digits = bytes.iter().take_while(|c| c.is_ascii_digit()).count();
            let starts_block = (t.starts_with('#') && t.trim_start_matches('#').chars().next().is_none_or(char::is_whitespace))
                || t.starts_with('>')
                || ((t.starts_with("- ") || t.starts_with("+ ")) || t == "-" || t == "+")
                || (!t.is_empty() && t.chars().all(|c| c == '-' || c == '=' || c == ' '));
            if starts_block {
                format!("{lead}\\{t}")
            } else if digits > 0 && digits <= 9 && matches!(bytes.get(digits), Some(b'.') | Some(b')'))
                && bytes.get(digits + 1).is_none_or(|c| *c == b' ')
            {
                format!("{lead}{}\\{}", &t[..digits], &t[digits..])
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '*' | '_' | '`' | '[' | ']' | '<' | '&') {
            s.push('\\');
        }
        s.push(c);
    }
    s
}

fn code_span(text: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in text.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat(longest + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') || (text.starts_with(' ') && text.ends_with(' ') && !text.trim().is_empty()) {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{text}{pad}{fence}")
}

#[derive(Clone, PartialEq)]
enum Open {
    Link(String),
    Strong,
    Em,
    Sub,
    Sup,
}

fn opener(m: &Open) -> String {
    match m {
        Open::Link(_) => "[".into(),
        Open::Strong => "**".into(),
        Open::Em => "*".into(),
        Open::Sub => "<sub>".into(),
        Open::Sup => "<sup>".into(),
    }
}

fn closer(m: &Open) -> String {
    match m {
        Open::Link(h) => format!("]({})", h.replace(' ', "%20").replace('(', "%28").replace(')', "%29")),
        Open::Strong => "**".into(),
        Open::Em => "*".into(),
        Open::Sub => "</sub>".into(),
        Open::Sup => "</sup>".into(),
    }
}

fn wanted(m: &Marks) -> Vec<Open> {
    let mut v = Vec::new();
    if let Some(h) = &m.link {
        v.push(Open::Link(h.clone()));
    }
    if m.strong {
        v.push(Open::Strong);
    }
    if m.em {
        v.push(Open::Em);
    }
    if m.sub {
        v.push(Open::Sub);
    }
    if m.sup {
        v.push(Open::Sup);
    }
    v
}

/// Closes marks, keeping trailing whitespace outside the delimiters (CommonMark won't close
/// emphasis after a space).
fn close_to(out: &mut String, open: &mut Vec<Open>, keep: usize) {
    if open.len() <= keep {
        return;
    }
    let trimmed = out.trim_end_matches([' ', '\t']).len();
    let ws = out[trimmed..].to_owned();
    out.truncate(trimmed);
    while open.len() > keep {
        let m = open.pop().expect("non-empty");
        out.push_str(&closer(&m));
    }
    out.push_str(&ws);
}

fn inline_markdown(content: &[Inline]) -> String {
    let mut out = String::new();
    let mut open: Vec<Open> = Vec::new();
    for n in content {
        match n {
            Inline::Break => {
                close_to(&mut out, &mut open, 0);
                out.push_str("\\\n");
            }
            Inline::Text { text, marks } => {
                let want = wanted(marks);
                let keep = open.iter().zip(&want).take_while(|(a, b)| a == b).count();
                close_to(&mut out, &mut open, keep);
                let body = if marks.code { code_span(text) } else { escape(text) };
                // Leading whitespace goes before the new openers.
                let (lead, body) = if want.len() > open.len() {
                    let t = body.trim_start_matches([' ', '\t']);
                    (body[..body.len() - t.len()].to_owned(), t.to_owned())
                } else {
                    (String::new(), body)
                };
                out.push_str(&lead);
                for m in &want[open.len()..] {
                    out.push_str(&opener(m));
                }
                open = want;
                out.push_str(&body);
            }
        }
    }
    close_to(&mut out, &mut open, 0);
    out
}

/// The block's text without marks (for the outline and search).
pub fn plain_text(b: &Block) -> String {
    let inl = |c: &[Inline]| {
        c.iter()
            .map(|n| match n {
                Inline::Text { text, .. } => text.as_str(),
                Inline::Break => "\n",
            })
            .collect::<String>()
    };
    match b {
        Block::Heading { content, .. } | Block::Paragraph(content) => inl(content),
        Block::BulletList(items) | Block::OrderedList { items, .. } => {
            items.iter().map(|i| i.iter().map(plain_text).collect::<Vec<_>>().join("\n")).collect::<Vec<_>>().join("\n")
        }
        Block::Quote(bs) => bs.iter().map(plain_text).collect::<Vec<_>>().join("\n"),
        Block::Rule | Block::PageBreak => String::new(),
    }
}
