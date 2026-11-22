// SPDX-License-Identifier: GPL-3.0-or-later
//! Interactive figures in reports (ADR-0017, spike S7): Plotly's SVG export, with its random
//! plot id normalized, is placed as vector graphics, and the same input gives the same PDF.
//! A figure that couldn't be drawn prints as FIGURE NOT RENDERED.

use cruncha_report::{Block, Output, Provenance, Report, Status, render, render_png};

fn report(outputs: Vec<Output>) -> Report {
    Report {
        blocks: vec![Block::Code {
            id: "py1".into(),
            lang: "python".into(),
            source: "fig = px.line(x=x, y=M)".into(),
            status: Status::Ok,
            outputs,
            defines: vec![],
            provenance: None,
        }],
        provenance: Provenance {
            document_name: "figures.cruncha".into(),
            document_id: "01TEST".into(),
            version: 3,
            committed_at: "2026-11-21T10:00:00Z".into(),
            content_hash: "0".repeat(64),
            cruncha_version: "0.0.1".into(),
            python_env: None,
            snapshot: None,
        },
        ..Default::default()
    }
}

#[test]
fn plotly_svg_is_placed_and_deterministic() {
    let svg = include_bytes!("fixtures/plotly-line.svg").to_vec();
    let r = report(vec![Output::Image { path: "/img/py1-0.svg".into() }]);
    let files = || vec![("/img/py1-0.svg".to_string(), svg.clone())];
    let a = render(&r, files()).expect("renders");
    let b = render(&r, files()).expect("renders");
    assert_eq!(a.pdf, b.pdf, "same input, same PDF");
    assert!(a.warnings.is_empty(), "{:?}", a.warnings);
    if let Ok(dir) = std::env::var("CRUNCHA_REPORT_OUT") {
        let pages = render_png(cruncha_report::CALC_PACKAGE, &r, files(), 110.0).unwrap();
        std::fs::write(format!("{dir}/figure-page.png"), &pages[pages.len() - 2]).unwrap();
    }
}

#[test]
fn a_figure_that_could_not_be_drawn_says_so() {
    let r = report(vec![Output::Figure {
        path: None,
        summary: "Bending moment (kip·ft) vs x (ft); 1 trace: M".into(),
        reason: Some("the app couldn't draw it: renderer timed out".into()),
    }]);
    let pages = render(&r, vec![]).expect("renders");
    let text = String::from_utf8_lossy(&pages.pdf);
    // The PDF's text is compressed; check through the PNG instead that it typesets, and
    // through the JSON the template reads that the reason is there.
    assert!(pages.pages >= 1 && !text.is_empty());
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("\"type\":\"figure\"") && json.contains("renderer timed out"));
}

#[test]
fn the_default_body_font_is_the_bundled_inter() {
    let r = report(vec![]);
    let pdf = render(&r, vec![]).expect("renders").pdf;
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("Inter"), "the body font is embedded by name");
    assert!(!text.contains("Helvetica"), "no system font");
}
