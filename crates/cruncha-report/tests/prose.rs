// SPDX-License-Identifier: GPL-3.0-or-later
//! Prose in reports: lists (nested, numbered from any start), quotes, rules and links,
//! subscripts and superscripts typeset without warnings, deterministically.

use cruncha_report::{Block, Provenance, Report, Run, render, render_png};

fn run(text: &str) -> Run {
    Run { text: text.into(), ..Default::default() }
}

#[test]
fn lists_quotes_rules_and_links_typeset() {
    let para = |runs: Vec<Run>| Block::Paragraph { text: runs };
    let r = Report {
        blocks: vec![
            Block::Heading { level: 1, text: vec![run("Design basis")] },
            Block::List {
                ordered: false,
                start: 1,
                items: vec![
                    vec![para(vec![run("Dead load "), Run { text: "D".into(), bold: true, ..Default::default() }])],
                    vec![
                        para(vec![run("Live load")]),
                        Block::List { ordered: true, start: 3, items: vec![vec![para(vec![run("roof")])], vec![para(vec![run("floor")])]] },
                    ],
                ],
            },
            Block::Quote { blocks: vec![para(vec![run("Per "), Run { text: "AISC 360-22".into(), link: Some("https://www.aisc.org/".into()), ..Default::default() }])] },
            Block::Rule,
            para(vec![run("f"), Run { text: "c".into(), sub: true, ..Default::default() }, run("′ = 4 ksi, x"), Run { text: "2".into(), sup: true, ..Default::default() }]),
        ],
        provenance: Provenance {
            document_name: "prose.cruncha".into(),
            document_id: "01TEST".into(),
            version: 1,
            committed_at: "2026-11-22T10:00:00Z".into(),
            content_hash: "0".repeat(64),
            cruncha_version: "0.0.1".into(),
            python_env: None,
            snapshot: None,
        },
        ..Default::default()
    };
    let a = render(&r, vec![]).expect("renders");
    let b = render(&r, vec![]).expect("renders");
    assert_eq!(a.pdf, b.pdf, "same input, same PDF");
    assert!(a.warnings.is_empty(), "{:?}", a.warnings);
    if let Ok(dir) = std::env::var("CRUNCHA_REPORT_OUT") {
        let pages = render_png(cruncha_report::CALC_PACKAGE, &r, vec![], 110.0).unwrap();
        std::fs::write(format!("{dir}/prose-page.png"), &pages[pages.len() - 2]).unwrap();
    }
}

#[test]
fn a_page_break_starts_a_new_page_and_two_in_a_row_start_one() {
    let para = |t: &str| Block::Paragraph { text: vec![run(t)] };
    let pages = |blocks: Vec<Block>| {
        let provenance = Provenance {
            document_name: "pages.cruncha".into(),
            document_id: "01TEST".into(),
            version: 1,
            committed_at: "2026-11-26T10:00:00Z".into(),
            content_hash: "0".repeat(64),
            cruncha_version: "0.0.1".into(),
            ..Default::default()
        };
        let r = Report { blocks, provenance, ..Default::default() };
        render_png(cruncha_report::CALC_PACKAGE, &r, vec![], 20.0).expect("renders").len()
    };
    let flowing = pages(vec![para("one"), para("two")]);
    assert_eq!(pages(vec![para("one"), Block::PageBreak, para("two")]), flowing + 1);
    assert_eq!(pages(vec![para("one"), Block::PageBreak, Block::PageBreak, para("two")]), flowing + 1, "weak: no blank page");
    assert_eq!(pages(vec![Block::PageBreak, para("one"), para("two")]), flowing, "none at the top of a page");
}
