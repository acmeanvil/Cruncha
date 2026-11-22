// SPDX-License-Identifier: GPL-3.0-or-later
//! Rust cells end to end (ADR-0006): quantities in and out with their units, printed output,
//! opaque values, and compile errors and panics at the cell's own lines. Needs Cruncha's Rust
//! toolchain; without it the test says so and passes.

use std::time::Duration;

use cruncha_kernel_rs::cells::{Cells, Request};
use cruncha_kernel_rs::Toolchain;
use serde_json::{Value, json};

fn qty(v: f64, name: &str, factor: f64, dims: [f64; 8]) -> Value {
    json!({"type": "number", "value": {"type": "float", "value": v},
           "unit": {"name": name, "factor": factor, "gauge": 0.0, "dims": dims}})
}

fn request(source: &str, defines: &[&str]) -> Request {
    let mut inputs = serde_json::Map::new();
    inputs.insert("L".into(), qty(24.0, "ft", 0.3048, [1., 0., 0., 0., 0., 0., 0., 0.]));
    inputs.insert("w".into(), qty(1.5, "kip/ft", 14593.902937206364, [0., 1., -2., 0., 0., 0., 0., 0.]));
    Request {
        cell: "R".into(),
        source: source.into(),
        defines: defines.iter().map(|s| s.to_string()).collect(),
        inputs: inputs.into_iter().filter(|(k, _)| source.contains(k.as_str())).collect(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rust_cells_compute_with_units() {
    let Some(tc) = Toolchain::default_base().and_then(|b| Toolchain::at(&b)) else {
        eprintln!("no Rust toolchain for cells; skipping");
        return;
    };
    let work = std::env::temp_dir().join("cruncha-rust-cells-e2e");
    let cells = Cells::new(tc, work).unwrap();

    let r = cells
        .run(
            &request(
                "let M = (w * L * L / 8.0).to(\"kip*ft\");\nlet n = 51usize;\nlet xs: Vec<f64> = (0..n).map(|i| i as f64).collect();\n\
                 let ok = M.value() > 100.0;\nprintln!(\"M = {M}\");\nlet table = std::collections::HashMap::<u8, u8>::new();",
                &["M", "n", "xs", "ok", "table"],
            ),
            None,
        )
        .await;
    assert!(r.ok, "{:?}", r.error);
    let get = |n: &str| r.defines.iter().find(|d| d.0 == n).cloned().unwrap();
    let (_, summary, m) = get("M");
    assert_eq!(summary, "108 kip·ft");
    assert_eq!(m["unit"]["name"], "kip·ft");
    assert!((m["value"]["value"].as_f64().unwrap() - 108.0).abs() < 1e-9);
    assert_eq!(get("n").2["value"]["value"], 51.0);
    assert_eq!(get("xs").2["shape"], json!([51]));
    assert_eq!(get("ok").2["value"], true);
    assert_eq!(get("table").2["type"], "unsupported", "no ToPlain: opaque");
    assert_eq!(r.stdout.trim(), "M = 108 kip·ft");

    // Unchanged source and input types: no compile.
    let again = cells.run(&request("let M = (w * L * L / 8.0).to(\"kip*ft\");\nlet n = 51usize;\nlet xs: Vec<f64> = (0..n).map(|i| i as f64).collect();\n\
                 let ok = M.value() > 100.0;\nprintln!(\"M = {M}\");\nlet table = std::collections::HashMap::<u8, u8>::new();", &["M", "n", "xs", "ok", "table"]), None).await;
    assert!(again.ok && again.compile_ms < 50.0, "{}", again.compile_ms);

    // A compile error at the cell's line.
    let r = cells.run(&request("let a = 1.0;\nlet b = a + nope;", &["a", "b"]), None).await;
    let e = r.error.unwrap();
    assert_eq!((e.kind.as_str(), e.line), ("CompileError", Some(2)), "{e:?}");
    assert!(e.message.contains("nope"), "{}", e.message);

    // A panic (adding a length to a load) at the cell's line.
    let r = cells.run(&request("let a = 1.0;\nlet bad = L + w;", &["a", "bad"]), None).await;
    let e = r.error.unwrap();
    assert_eq!((e.kind.as_str(), e.line), ("panic", Some(2)), "{e:?}");
    assert!(e.message.contains("different dimensions"), "{}", e.message);

    // A runaway cell is stopped.
    let r = cells.run(&request("let mut i: u64 = 0;\nloop { i = std::hint::black_box(i + 1); }", &["i"]), Some(Duration::from_millis(300))).await;
    assert!(r.interrupted, "{:?}", r.error);
}
