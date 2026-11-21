// SPDX-License-Identifier: GPL-3.0-or-later

//! Spike S6: typeset a beam check (evaluated by cruncha-eval) as a calc package.
//!
//! cargo run -p cruncha-report --release --example report_demo -- <out dir> [runs]
//!
//! Writes `beam-check.pdf` and `page-N.png`, and prints timings and whether two renders give
//! the same bytes.

use cruncha_eval::{Vars, evaluate, prepare};
use cruncha_report::*;

fn run(text: &str) -> Vec<Run> {
    vec![Run { text: text.into(), ..Default::default() }]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let out = std::path::PathBuf::from(args.next().unwrap_or_else(|| ".".into()));
    let runs: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(5);
    std::fs::create_dir_all(&out).unwrap();

    let opts = cruncha_eval::Options { unit_aware: cruncha_eval::UnitAware::Always, ..Default::default() };
    let mut vars = Vars::new();
    let mut blocks = vec![
        Block::Heading { level: 1, text: run("Loads") },
        Block::Paragraph {
            text: vec![
                Run { text: "A simply supported floor beam carrying a ".into(), ..Default::default() },
                Run { text: "uniform".into(), italic: true, ..Default::default() },
                Run { text: " service load. Units are checked throughout; the ".into(), ..Default::default() },
                Run { text: "blue".into(), bold: true, ..Default::default() },
                Run { text: " line shows each value substituted.".into(), ..Default::default() },
            ],
        },
    ];
    let mut calc = |src: &str, desc: &str, blocks: &mut Vec<Block>| {
        let stmt = prepare(src, &opts).unwrap();
        let o = evaluate(&stmt, &vars, &opts);
        let check = src.trim_start().starts_with('@');
        let status = match (&o.error, o.check) {
            (Some(e), _) => Status::Error { message: e.clone() },
            (None, Some(true)) => Status::Pass,
            (None, Some(false)) => Status::Fail,
            _ => Status::Ok,
        };
        for (n, v) in o.values {
            vars.insert(n, v);
        }
        let math = tex::to_typst(&o.tex).unwrap_or_default();
        blocks.push(Block::Calc {
            id: format!("c{}", blocks.len()),
            math,
            description: desc.into(),
            status,
            text: o.result,
            check,
        });
    };
    calc("w_D = 0.45 kip/ft", "Dead load (slab and finishes)", &mut blocks);
    calc("w_L = 1.05 kip/ft", "Live load, office occupancy", &mut blocks);
    calc("w = 1.2 w_D + 1.6 w_L = ?? kip/ft", "Factored load, ASCE 7 §2.3.1 (2)", &mut blocks);
    calc("L = 24 ft", "Span", &mut blocks);
    blocks.push(Block::Heading { level: 1, text: run("Flexure") });
    blocks.push(Block::Heading { level: 2, text: run("Demand") });
    calc("M_u = w L^2 / 8 = ?? kip·ft", "Midspan moment", &mut blocks);
    calc("V_u = w L / 2 = ?? kips", "End shear", &mut blocks);
    blocks.push(Block::Heading { level: 2, text: run("Capacity") });
    calc("F_y = 50 ksi", "W-shape, ASTM A992", &mut blocks);
    calc("Z_x = 54.9 in³", "W14×34", &mut blocks);
    calc("φ = 0.9", "", &mut blocks);
    calc("M_n = F_y Z_x = ?? kip·ft", "AISC 360 Eq. F2-1", &mut blocks);
    calc("ratio = M_u / (φ M_n) = ?", "Demand / capacity", &mut blocks);
    calc("@test ratio ≤ 1", "", &mut blocks);
    calc("@test V_u < 25 'kips'", "", &mut blocks);
    calc("K = [1, 2; 3, 4] = ?", "A matrix, for the layout", &mut blocks);
    calc("x = 1 / 0 = ?", "An error, on purpose", &mut blocks);

    blocks.push(Block::Heading { level: 1, text: run("Moment along the span") });
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="480" height="220" viewBox="0 0 480 220">
<rect width="480" height="220" fill="white"/><line x1="40" y1="190" x2="460" y2="190" stroke="#333"/>
<line x1="40" y1="20" x2="40" y2="190" stroke="#333"/>
<path d="M40 190 Q250 -140 460 190" fill="none" stroke="#1f77b4" stroke-width="2.5"/>
<text x="250" y="212" font-size="12" text-anchor="middle" font-family="sans-serif">x (ft)</text></svg>"##;
    blocks.push(Block::Code {
        id: "py1".into(),
        lang: "python".into(),
        source: "import numpy as np\nimport matplotlib.pyplot as plt\n\nx = np.linspace(0, L, 51)\nmoment = (w * x * (L - x) / 2).to(\"kip*ft\")\nM_max = moment.max()\nplt.plot(x.magnitude, moment.magnitude)".into(),
        status: Status::Ok,
        outputs: vec![
            Output::Image { path: "/img/py1-0.svg".into() },
            Output::Table {
                columns: vec!["x (ft)".into(), "M (kip·ft)".into()],
                rows: vec![
                    vec!["0".into(), "0".into()],
                    vec!["6".into(), "200.2".into()],
                    vec!["12".into(), "266.9".into()],
                ],
                total_rows: 51,
            },
        ],
        defines: vec![("x".into(), "ndarray (51,)".into()), ("M_max".into(), "266.9 kip·ft".into())],
        provenance: None,
    });
    blocks.push(Block::Code {
        id: "py2".into(),
        lang: "python".into(),
        source: "deflection = solve_beam(L, w)".into(),
        status: Status::NotCalculated { reason: "edited since it last ran: press Run".into() },
        outputs: vec![],
        defines: vec![],
        provenance: None,
    });
    calc("Δ = 0.42 in", "Deflection (from the cell above — imagine)", &mut blocks);

    let report = Report {
        title: TitleBlock {
            title: "Floor beam B-12".into(),
            subject: "Flexure and shear, level 3".into(),
            project: "Riverside Office Building".into(),
            project_number: "2026-114".into(),
            client: "Riverside Holdings".into(),
            prepared_by: "A. Engineer".into(),
            checked_by: "C. Checker".into(),
            approved_by: String::new(),
        },
        revisions: vec![
            Revision { label: "A".into(), date: "2026-11-12".into(), description: "Issued for review".into(), by: "AE".into() },
            Revision { label: "B".into(), date: "2026-11-19".into(), description: "Live load per architect's occupancy change".into(), by: "AE".into() },
        ],
        style: HouseStyle { company: "Example Structural".into(), ..HouseStyle::default() },
        options: Options::default(),
        blocks,
        provenance: Provenance {
            document_name: "beam-b12.cruncha".into(),
            document_id: "01JD6Z8Q4XW3".into(),
            version: 37,
            committed_at: "2026-11-20T15:04:05Z".into(),
            content_hash: "9f1c2a7b44e0d3a58c1e6f0a2b9d7c3e5f8a1b2c3d4e5f60718293a4b5c6d7e8".into(),
            cruncha_version: "0.0.1".into(),
            python_env: Some("9a9710dddbc2".into()),
            snapshot: Some("B".into()),
        },
    };
    let files = vec![("/img/py1-0.svg".to_string(), svg.to_vec())];

    let t = std::time::Instant::now();
    let families = warm_up();
    println!("fonts: {families} families in {:.0} ms", t.elapsed().as_secs_f64() * 1e3);
    let first = render(&report, files.clone()).unwrap_or_else(|e| panic!("{e}"));
    for w in &first.warnings {
        println!("warning: {w}");
    }
    let mut compile = Vec::new();
    let mut pdf = Vec::new();
    let mut same = true;
    for _ in 0..runs {
        let r = render(&report, files.clone()).unwrap();
        compile.push(r.compile_ms);
        pdf.push(r.pdf_ms);
        same &= r.pdf == first.pdf;
    }
    compile.sort_by(f64::total_cmp);
    pdf.sort_by(f64::total_cmp);
    println!(
        "{} pages, {} KB; first render {:.0} ms + {:.0} ms PDF; then median compile {:.1} ms, PDF {:.1} ms; identical bytes: {same}",
        first.pages,
        first.pdf.len() / 1024,
        first.compile_ms,
        first.pdf_ms,
        compile.get(compile.len() / 2).copied().unwrap_or(0.0),
        pdf.get(pdf.len() / 2).copied().unwrap_or(0.0),
    );
    // An edit: one calc changes, as when the user re-exports after a change.
    let mut edited = report.clone();
    if let Some(Block::Calc { description, .. }) = edited.blocks.get_mut(3) {
        description.push_str(" (revised)");
    }
    edited.provenance.version += 1;
    let r = render(&edited, files.clone()).unwrap();
    println!("after an edit: compile {:.0} ms, PDF {:.0} ms", r.compile_ms, r.pdf_ms);
    std::fs::write(out.join("beam-check.pdf"), &first.pdf).unwrap();
    let pngs = render_png(CALC_PACKAGE, &report, files, 110.0).unwrap();
    for (i, png) in pngs.iter().enumerate() {
        std::fs::write(out.join(format!("page-{}.png", i + 1)), png).unwrap();
    }
    println!("wrote {}", out.display());
}
