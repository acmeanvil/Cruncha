// SPDX-License-Identifier: GPL-3.0-or-later
//! What a Rust cell defines and reads (ADR-0006), like the Python worker's `ast` scan.
//!
//! A cell is a function body. Its **defines** are the names its top-level `let`s bind, and its
//! top-level `const`/`static` items. Its **reads** are the free names it uses: identifiers in
//! expressions that no enclosing `let`, parameter, pattern, `use` or item binds. Names inside
//! the usual formatting macros (`println!`, `format!`, `assert!`, `vec!`, …) count too,
//! including `{name}` captures in format strings. Functions defined in the cell are local.

use std::collections::{BTreeSet, HashSet};

use syn::visit::{self, Visit};
use syn::{Expr, Pat, Stmt};

/// The result of scanning a cell.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Names {
    pub defines: Vec<String>,
    pub reads: Vec<String>,
    /// A syntax error, with its 1-based line in the cell.
    pub syntax_error: Option<(String, Option<u32>)>,
}

/// Names that are in scope in every cell (Rust's prelude, the cell prelude's functions).
const AMBIENT: &[&str] = &["Some", "None", "Ok", "Err", "drop", "Box", "Vec", "String", "unit", "self", "Self"];

/// Macros whose arguments are comma-separated expressions (a format string first for some).
pub(crate) const EXPR_MACROS: &[&str] = &[
    "println", "print", "eprintln", "eprint", "format", "panic", "assert", "assert_eq", "assert_ne",
    "debug_assert", "debug_assert_eq", "vec", "write", "writeln", "dbg", "unreachable", "todo",
];

/// The wrapper the cell is parsed in (one line, so error lines shift by one).
const PREFIX: &str = "fn __cell() {\n";

pub fn scan(source: &str) -> Names {
    let file: syn::File = match syn::parse_str(&format!("{PREFIX}{source}\n}}")) {
        Ok(f) => f,
        Err(e) => {
            let line = e.span().start().line;
            return Names {
                syntax_error: Some((e.to_string(), (line > 1).then(|| line as u32 - 1))),
                ..Default::default()
            };
        }
    };
    let Some(syn::Item::Fn(f)) = file.items.into_iter().next() else {
        return Names::default();
    };
    let mut defines: Vec<String> = Vec::new();
    let mut local_items: HashSet<String> = HashSet::new();
    for stmt in &f.block.stmts {
        match stmt {
            Stmt::Local(l) => {
                for n in pat_names(&l.pat) {
                    if !defines.contains(&n) {
                        defines.push(n);
                    }
                }
            }
            Stmt::Item(item) => item_names(item, &mut defines, &mut local_items),
            _ => {}
        }
    }
    let mut v = Reads { scopes: vec![HashSet::new()], items: local_items, reads: BTreeSet::new() };
    v.block_stmts(&f.block.stmts);
    Names { defines, reads: v.reads.into_iter().collect(), syntax_error: None }
}

/// Items: consts and statics are values (defines); fns, uses and types are local names.
fn item_names(item: &syn::Item, defines: &mut Vec<String>, local: &mut HashSet<String>) {
    match item {
        syn::Item::Const(c) => defines.push(c.ident.to_string()),
        syn::Item::Static(s) => defines.push(s.ident.to_string()),
        syn::Item::Fn(f) => {
            local.insert(f.sig.ident.to_string());
        }
        syn::Item::Use(u) => use_names(&u.tree, local),
        syn::Item::Struct(s) => {
            local.insert(s.ident.to_string());
        }
        syn::Item::Enum(e) => {
            local.insert(e.ident.to_string());
            for v in &e.variants {
                local.insert(v.ident.to_string());
            }
        }
        _ => {}
    }
}

fn use_names(tree: &syn::UseTree, out: &mut HashSet<String>) {
    match tree {
        syn::UseTree::Path(p) => use_names(&p.tree, out),
        syn::UseTree::Name(n) => {
            out.insert(n.ident.to_string());
        }
        syn::UseTree::Rename(r) => {
            out.insert(r.rename.to_string());
        }
        syn::UseTree::Group(g) => g.items.iter().for_each(|t| use_names(t, out)),
        syn::UseTree::Glob(_) => {}
    }
}

/// The names a pattern binds.
fn pat_names(p: &Pat) -> Vec<String> {
    struct Binds(Vec<String>);
    impl<'a> Visit<'a> for Binds {
        fn visit_pat_ident(&mut self, i: &'a syn::PatIdent) {
            self.0.push(i.ident.to_string());
            visit::visit_pat_ident(self, i);
        }
    }
    let mut b = Binds(Vec::new());
    b.visit_pat(p);
    b.0
}

struct Reads {
    scopes: Vec<HashSet<String>>,
    items: HashSet<String>,
    reads: BTreeSet<String>,
}

impl Reads {
    fn bound(&self, name: &str) -> bool {
        self.scopes.iter().any(|s| s.contains(name)) || self.items.contains(name) || AMBIENT.contains(&name)
    }

    fn bind(&mut self, p: &Pat) {
        let names = pat_names(p);
        let scope = self.scopes.last_mut().expect("a scope");
        scope.extend(names);
    }

    fn use_name(&mut self, name: &str) {
        if !self.bound(name) {
            self.reads.insert(name.to_owned());
        }
    }

    /// A block's statements in order: a `let` binds after its initializer is read.
    fn block_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::Local(l) => {
                    if let Some(init) = &l.init {
                        self.visit_expr(&init.expr);
                        if let Some((_, diverge)) = &init.diverge {
                            self.visit_expr(diverge);
                        }
                    }
                    self.bind(&l.pat);
                }
                // A nested fn can't capture the cell's locals; what it uses isn't a read.
                Stmt::Item(syn::Item::Fn(_)) => {}
                Stmt::Item(item) => {
                    let mut ignored = Vec::new();
                    let mut names = HashSet::new();
                    item_names(item, &mut ignored, &mut names);
                    self.items.extend(names);
                    self.items.extend(ignored);
                    if let syn::Item::Const(c) = item {
                        self.visit_expr(&c.expr);
                    }
                    if let syn::Item::Static(s) = item {
                        self.visit_expr(&s.expr);
                    }
                }
                other => self.visit_stmt(other),
            }
        }
    }

    fn scoped(&mut self, f: impl FnOnce(&mut Self)) {
        self.scopes.push(HashSet::new());
        f(self);
        self.scopes.pop();
    }

    /// Arguments of a formatting-style macro, and `{name}` captures in its format string.
    fn macro_args(&mut self, m: &syn::Macro) {
        let name = m.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
        if !EXPR_MACROS.contains(&name.as_str()) {
            return;
        }
        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        let Ok(args) = m.parse_body_with(parser) else { return };
        for a in &args {
            if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = a {
                for cap in captures(&s.value()) {
                    self.use_name(&cap);
                }
            }
            self.visit_expr(a);
        }
    }
}

/// `{name}` and `{name:…}` in a format string (not `{{`, positions or `{}`).
fn captures(fmt: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = fmt.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '{' {
            continue;
        }
        if chars.peek().map(|p| p.1) == Some('{') {
            chars.next();
            continue;
        }
        let rest = &fmt[i + 1..];
        let end = rest.find(['}', ':']).unwrap_or(rest.len());
        let name = &rest[..end];
        if !name.is_empty() && name.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            out.push(name.to_owned());
        }
    }
    out
}

impl<'a> Visit<'a> for Reads {
    fn visit_block(&mut self, b: &'a syn::Block) {
        self.scoped(|s| s.block_stmts(&b.stmts));
    }

    fn visit_expr_closure(&mut self, c: &'a syn::ExprClosure) {
        self.scoped(|s| {
            for p in &c.inputs {
                s.bind(p);
            }
            s.visit_expr(&c.body);
        });
    }

    fn visit_expr_for_loop(&mut self, f: &'a syn::ExprForLoop) {
        self.visit_expr(&f.expr);
        self.scoped(|s| {
            s.bind(&f.pat);
            s.visit_block(&f.body);
        });
    }

    fn visit_arm(&mut self, a: &'a syn::Arm) {
        self.scoped(|s| {
            s.bind(&a.pat);
            if let Some((_, g)) = &a.guard {
                s.visit_expr(g);
            }
            s.visit_expr(&a.body);
        });
    }

    fn visit_expr_let(&mut self, l: &'a syn::ExprLet) {
        // `if let` / `while let`: the binding is visible in the following block.
        self.visit_expr(&l.expr);
        self.bind(&l.pat);
    }

    fn visit_expr_if(&mut self, i: &'a syn::ExprIf) {
        self.scoped(|s| {
            s.visit_expr(&i.cond);
            s.visit_block(&i.then_branch);
        });
        if let Some((_, e)) = &i.else_branch {
            self.visit_expr(e);
        }
    }

    fn visit_expr_while(&mut self, w: &'a syn::ExprWhile) {
        self.scoped(|s| {
            s.visit_expr(&w.cond);
            s.visit_block(&w.body);
        });
    }

    fn visit_expr_path(&mut self, p: &'a syn::ExprPath) {
        if p.qself.is_none() && p.path.segments.len() == 1 && p.path.leading_colon.is_none() {
            let name = p.path.segments[0].ident.to_string();
            self.use_name(&name);
        }
        visit::visit_expr_path(self, p);
    }

    fn visit_macro(&mut self, m: &'a syn::Macro) {
        self.macro_args(m);
    }

    fn visit_item_fn(&mut self, _: &'a syn::ItemFn) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(src: &str) -> (Vec<String>, Vec<String>) {
        let n = scan(src);
        assert!(n.syntax_error.is_none(), "{:?}", n.syntax_error);
        (n.defines, n.reads)
    }

    #[test]
    fn lets_define_and_free_names_read() {
        let (d, r) = names("let M = w * L * L / 8.0;\nlet x: Vec<f64> = (0..=50).map(|i| L.si() * i as f64 / 50.0).collect();");
        assert_eq!(d, ["M", "x"]);
        assert_eq!(r, ["L", "w"]);
    }

    #[test]
    fn scopes_patterns_loops_and_shadowing() {
        let src = "let mut total = 0.0;\nfor (i, v) in loads.iter().enumerate() { let k = v * 2.0; total += k + i as f64 + offset; }\n\
                   let best = match mode { Some(m) => m, None => fallback };\n\
                   if let Some(z) = maybe { let _ = z; }\nlet y = { let t = 3.0; t * scale };";
        let (d, r) = names(src);
        assert_eq!(d, ["total", "best", "y"]);
        assert_eq!(r, ["fallback", "loads", "maybe", "mode", "offset", "scale"]);
    }

    #[test]
    fn functions_uses_consts_and_macros() {
        let src = "use std::f64::consts::PI;\nconst N: usize = 51;\nfn area(d: f64) -> f64 { PI * d * d / 4.0 + hidden }\n\
                   let A = area(D);\nprintln!(\"{A:.3} with {}\", n_bolts);\nlet s = format!(\"{label}-{{x}}\");";
        let (d, r) = names(src);
        assert_eq!(d, ["N", "A", "s"]);
        assert_eq!(r, ["D", "label", "n_bolts"], "fn bodies, uses, consts and {{x}} aren't reads");
    }

    #[test]
    fn a_let_reading_its_own_name_reads_it() {
        let (d, r) = names("let w = w * 1.2;");
        assert_eq!((d, r), (vec!["w".to_string()], vec!["w".to_string()]));
    }

    #[test]
    fn syntax_errors_have_cell_lines() {
        let n = scan("let a = 1;\nlet b = ;\n");
        let (msg, line) = n.syntax_error.unwrap();
        assert_eq!(line, Some(2), "{msg}");
    }
}
