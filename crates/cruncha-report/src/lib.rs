// SPDX-License-Identifier: GPL-3.0-or-later

//! Cruncha's report engine (spike S6): an evaluated document to a PDF calc package.
//!
//! The app turns the latest version and its results into a [`Report`] (title block,
//! revisions from snapshots, house style, one [`Block`] per cell). [`render`] hands it to an
//! embedded Typst, with a template that lays out the cover, contents, check summary, the
//! calcs with their blue echo, Python figures and tables, and a provenance appendix.
//!
//! Calc math arrives as the engine's TeX and is converted with [`tex::to_typst`]. Rendering
//! is deterministic: the same report gives the same PDF bytes.

pub mod model;
pub mod tex;
mod world;

use std::time::Instant;

pub use model::*;
use typst::diag::SourceDiagnostic;
use typst::foundations::{Datetime, Smart};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, Timestamp};

/// The built-in calc-package template.
pub const CALC_PACKAGE: &str = include_str!("../templates/calc-package.typ");

/// Load the fonts (the system's are scanned once per process, which takes a few seconds on
/// a machine with many fonts). Call it early, off the UI thread, so the first report is quick.
pub fn warm_up() -> usize {
    world::fonts().book().families().count()
}

/// A rendered report.
pub struct Rendered {
    pub pdf: Vec<u8>,
    pub pages: usize,
    /// Time spent in Typst (compile, layout) and in writing the PDF, in milliseconds.
    pub compile_ms: f64,
    pub pdf_ms: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct Error(pub Vec<String>);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the report could not be typeset: {}", self.0.join("; "))
    }
}

impl std::error::Error for Error {}

fn messages(diags: &[SourceDiagnostic]) -> Vec<String> {
    diags
        .iter()
        .map(|d| {
            let mut m = d.message.to_string();
            for h in &d.hints {
                m.push_str(" (hint: ");
                m.push_str(&h.v);
                m.push(')');
            }
            m
        })
        .collect()
}

/// `2026-11-20T…` → (2026, 11, 20).
fn ymd(iso: &str) -> Option<(i32, u8, u8)> {
    let y = iso.get(0..4)?.parse().ok()?;
    let m = iso.get(5..7)?.parse().ok()?;
    let d = iso.get(8..10)?.parse().ok()?;
    Some((y, m, d))
}

/// Typeset `report` with `template` (normally [`CALC_PACKAGE`]). `files` are images and the
/// logo, at the paths the report refers to.
pub fn render_with(template: &str, report: &Report, files: Vec<(String, Vec<u8>)>) -> Result<Rendered, Error> {
    let json = serde_json::to_vec(report).map_err(|e| Error(vec![e.to_string()]))?;
    let mut all = files;
    all.push(("/report.json".into(), json));
    let date = ymd(&report.provenance.committed_at);
    let world = world::ReportWorld::new(template, all, date);

    let t = Instant::now();
    let result = typst::compile::<PagedDocument>(&world);
    let compile_ms = t.elapsed().as_secs_f64() * 1e3;
    let warnings = messages(&result.warnings);
    let doc = result.output.map_err(|e| Error(messages(&e)))?;

    let t = Instant::now();
    let options = PdfOptions {
        ident: Smart::Custom(format!("{}@{}", report.provenance.document_id, report.provenance.version)),
        timestamp: date
            .and_then(|(y, m, d)| Datetime::from_ymd(y, m, d))
            .map(Timestamp::new_utc),
        ..PdfOptions::default()
    };
    let pdf = typst_pdf::pdf(&doc, &options).map_err(|e| Error(messages(&e)))?;
    Ok(Rendered {
        pages: doc.pages().len(),
        pdf,
        compile_ms,
        pdf_ms: t.elapsed().as_secs_f64() * 1e3,
        warnings,
    })
}

/// Typeset `report` and draw each page as a PNG at `ppi` pixels per inch (previews, and
/// looking at the output in tests).
pub fn render_png(template: &str, report: &Report, files: Vec<(String, Vec<u8>)>, ppi: f32) -> Result<Vec<Vec<u8>>, Error> {
    let json = serde_json::to_vec(report).map_err(|e| Error(vec![e.to_string()]))?;
    let mut all = files;
    all.push(("/report.json".into(), json));
    let world = world::ReportWorld::new(template, all, ymd(&report.provenance.committed_at));
    let doc = typst::compile::<PagedDocument>(&world).output.map_err(|e| Error(messages(&e)))?;
    let opts = typst_render::RenderOptions { pixel_per_pt: (f64::from(ppi) / 72.0).into(), ..Default::default() };
    doc.pages()
        .iter()
        .map(|page| {
            typst_render::render(page, &opts)
                .encode_png()
                .map_err(|e| Error(vec![e.to_string()]))
        })
        .collect()
}

/// Typeset `report` with the built-in calc-package template.
pub fn render(report: &Report, files: Vec<(String, Vec<u8>)>) -> Result<Rendered, Error> {
    render_with(CALC_PACKAGE, report, files)
}

/// Check that Typst math code compiles on its own (used to test the TeX converter).
pub fn check_math(math: &str) -> Result<(), Error> {
    let src = format!(
        "#set page(width: auto, height: auto, margin: 2pt)\n#eval({}, mode: \"math\")\n",
        typst_string(math)
    );
    let world = world::ReportWorld::new(&src, vec![], Some((2026, 1, 1)));
    typst::compile::<PagedDocument>(&world)
        .output
        .map(|_| ())
        .map_err(|e| Error(messages(&e)))
}

/// Many math snippets at once: one document, one compile.
pub fn check_math_batch(maths: &[String]) -> Result<(), Error> {
    let mut src = String::from("#set page(width: 20cm, height: auto, margin: 2pt)\n");
    for m in maths {
        src.push_str(&format!("#eval({}, mode: \"math\")\n\n", typst_string(m)));
    }
    let world = world::ReportWorld::new(&src, vec![], Some((2026, 1, 1)));
    typst::compile::<PagedDocument>(&world)
        .output
        .map(|_| ())
        .map_err(|e| Error(messages(&e)))
}

fn typst_string(s: &str) -> String {
    let mut q = String::from("\"");
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
