// SPDX-License-Identifier: GPL-3.0-or-later
use cruncha_prose::*;

fn t(text: &str) -> Inline {
    Inline::Text { text: text.into(), marks: Marks::default() }
}
fn m(text: &str, f: impl FnOnce(&mut Marks)) -> Inline {
    let mut marks = Marks::default();
    f(&mut marks);
    Inline::Text { text: text.into(), marks }
}
fn prose(md: &str) -> Vec<Block> {
    from_markdown(md)
        .into_iter()
        .map(|p| match p {
            Piece::Prose(b) => b,
            other => panic!("not prose: {other:?}"),
        })
        .collect()
}

#[test]
fn canonical_json_round_trips() {
    // The editor writes these exactly (app/src/lib/editor/cells.test.ts checks the same file).
    let sources: Vec<String> = serde_json::from_str(include_str!("fixtures/canonical.json")).unwrap();
    for s in &sources {
        let b = parse_json(s);
        assert_eq!(&to_json(&b), s);
        // And Markdown carries the same tree.
        let md = to_markdown(&b);
        let back = prose(&md);
        if b == Block::Paragraph(vec![]) {
            assert!(back.is_empty());
        } else {
            assert_eq!(back, vec![b.clone()], "{md}");
        }
    }
}

#[test]
fn markdown_blocks_and_marks() {
    let got = prose("# Beam\n\nThe **moment** is *w L²/8*,\nsee [AISC](https://www.aisc.org) and `M_u`.\n\n- a\n- b\n  1. c\n\n> quoted\n\n---\n");
    assert_eq!(got[0], Block::Heading { level: 1, content: vec![t("Beam")] });
    assert_eq!(
        got[1],
        Block::Paragraph(vec![
            t("The "),
            m("moment", |m| m.strong = true),
            t(" is "),
            m("w L²/8", |m| m.em = true),
            t(", see "),
            m("AISC", |m| m.link = Some("https://www.aisc.org".into())),
            t(" and "),
            m("M_u", |m| m.code = true),
            t("."),
        ])
    );
    assert_eq!(
        got[2],
        Block::BulletList(vec![
            vec![Block::Paragraph(vec![t("a")])],
            vec![Block::Paragraph(vec![t("b")]), Block::OrderedList { start: 1, items: vec![vec![Block::Paragraph(vec![t("c")])]] }],
        ])
    );
    assert_eq!(got[3], Block::Quote(vec![Block::Paragraph(vec![t("quoted")])]));
    assert_eq!(got[4], Block::Rule);
}

#[test]
fn fences_become_cells_at_the_top_level_only() {
    let got = from_markdown("Text\n\n```calc\nL = 20 ft\n\nw = 2 kip/ft\n```\n\n```py\nx = 1\n```\n\n- item\n\n  ```python\n  y = 2\n  ```\n\n```json\n{}\n```\n");
    let calc = |s: &str| Piece::Calc { source: s.into(), attrs: Default::default() };
    assert_eq!(got[1], calc("L = 20 ft"));
    assert_eq!(got[2], calc("w = 2 kip/ft"));
    assert_eq!(got[3], Piece::Code { lang: "python".into(), source: "x = 1".into(), attrs: Default::default() });
    let code = |s: &str| m(s, |m| m.code = true);
    assert_eq!(got[4], Piece::Prose(Block::BulletList(vec![vec![Block::Paragraph(vec![t("item")]), Block::Paragraph(vec![code("y = 2")])]])));
    assert_eq!(got[5], Piece::Prose(Block::Paragraph(vec![code("{}")])));
}

#[test]
fn what_the_editor_cannot_hold_degrades_to_text() {
    let got = prose("![a beam](beam.png) H<sub>2</sub>O x<sup>2</sup> <span>hi</span>\n\n- ## Inner\n\n[bad](javascript:alert(1))");
    assert_eq!(
        got[0],
        Block::Paragraph(vec![
            t("a beam H"),
            m("2", |m| m.sub = true),
            t("O x"),
            m("2", |m| m.sup = true),
            t(" <span>hi</span>"),
        ])
    );
    assert_eq!(got[1], Block::BulletList(vec![vec![Block::Paragraph(vec![m("Inner", |m| m.strong = true)])]]));
    assert_eq!(got[2], Block::Paragraph(vec![t("bad")]));
}

#[test]
fn markdown_out_escapes_what_would_change_meaning() {
    let cases = [
        Block::Paragraph(vec![t("# not a heading")]),
        Block::Paragraph(vec![t("1. not a list"), Inline::Break, t("- nor this"), Inline::Break, t("---")]),
        Block::Paragraph(vec![t("a*b*c_d_ [x](y) <b> & `q` \\")]),
        Block::Paragraph(vec![m("bold ", |m| m.strong = true), m("both", |m| {
            m.strong = true;
            m.em = true;
        }), t(" after")]),
        Block::Paragraph(vec![m("a`b", |m| m.code = true)]),
        Block::Heading { level: 3, content: vec![t("Title"), Inline::Break, t("more")] },
    ];
    for b in &cases {
        let md = to_markdown(b);
        let back = prose(&md);
        let expect = match b {
            Block::Heading { level, .. } => Block::Heading { level: *level, content: vec![t("Title more")] },
            b => b.clone(),
        };
        assert_eq!(back, vec![expect], "{md}");
    }
}

#[test]
fn unknown_or_broken_sources_keep_their_text() {
    assert_eq!(parse_json("just text"), Block::Paragraph(vec![t("just text")]));
    assert_eq!(
        parse_json(r#"{"type":"table","content":[{"type":"text","text":"cell"}]}"#),
        Block::Paragraph(vec![t("cell")])
    );
    assert_eq!(plain_text(&prose("- a\n- b")[0]), "a\nb");
}

#[test]
fn fence_attributes_go_with_the_cell() {
    let got = from_markdown("```calc {description=\"Radial depth of the section\"}\nh = 4 in_\n```\n\n```python {description=\"Moments\" hidden}\nx = 1\n```\n");
    let Piece::Calc { source, attrs } = &got[0] else { panic!("{got:?}") };
    assert_eq!((source.as_str(), attrs.get("description").map(String::as_str)), ("h = 4 in_", Some("Radial depth of the section")));
    let Piece::Code { attrs, .. } = &got[1] else { panic!() };
    assert_eq!(attrs.get("description").map(String::as_str), Some("Moments"));
    assert_eq!(attrs.get("hidden").map(String::as_str), Some(""));
    assert_eq!(fence_info("calc{description=\"a \\\"quoted\\\" word\"}").1["description"], "a \"quoted\" word");
}

#[test]
fn a_group_describes_each_calc_on_the_line_before_it() {
    let got = from_markdown("```calc\n> Internal thread shear area\nA_n = 2 in²_\nB = 3\n> External thread shear area\nA_s = 3 in²_\n```\n");
    let calcs: Vec<(&str, Option<&str>)> = got
        .iter()
        .map(|p| match p {
            Piece::Calc { source, attrs } => (source.as_str(), attrs.get("description").map(String::as_str)),
            _ => panic!("{got:?}"),
        })
        .collect();
    assert_eq!(
        calcs,
        [("A_n = 2 in²_", Some("Internal thread shear area")), ("B = 3", None), ("A_s = 3 in²_", Some("External thread shear area"))]
    );
}
