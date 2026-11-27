// SPDX-License-Identifier: GPL-3.0-or-later
//! Spike S4: what talking to the Python worker costs.
//!
//! `cargo run -p cruncha-kernel-py --release --example py_bench -- [rows]`
//!
//! Measures worker start-up, first imports, the dependency scan, a trivial cell's round trip,
//! raw blob throughput over the pipes, and a frame of `rows` (default 1,000,000) rows sent as
//! an Arrow IPC stream with column units in field metadata, turned into Polars, extended with
//! a computed column and sent back.

use cruncha_kernel_py::{Config, Kernel, RunRequest, env};
use deltalake::arrow::array::{Float64Array, Int64Array, RecordBatch};
use deltalake::arrow::datatypes::{DataType, Field, Schema};
use deltalake::arrow::ipc::reader::StreamReader;
use deltalake::arrow::ipc::writer::StreamWriter;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

const LOCK: &str = include_str!("../../../python/requirements.lock");

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

#[tokio::main]
async fn main() {
    let rows: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    let base = env::default_base().expect("a home directory");
    let python = env::ready(&base, LOCK).expect("run the setup_env example first");
    let worker = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python");

    let mut starts = vec![];
    for _ in 0..5 {
        let t = Instant::now();
        let k = Kernel::start(Config::new(&python, &worker)).await.unwrap();
        starts.push(ms(t));
        k.shutdown().await;
    }
    println!(
        "worker start (hello), median of 5: {:.0} ms",
        median(starts)
    );

    let mut k = Kernel::start(Config::new(&python, &worker)).await.unwrap();
    for (what, src) in [
        ("import numpy", "import numpy"),
        ("import polars", "import polars"),
        ("import pyarrow", "import pyarrow, pyarrow.ipc"),
        ("import pint + registry", "import cruncha\ncruncha.ureg"),
        ("import scipy.optimize", "import scipy.optimize"),
        ("import matplotlib.pyplot", "import matplotlib.pyplot"),
    ] {
        let t = Instant::now();
        let r = k
            .run(
                &RunRequest {
                    cell: "I".into(),
                    source: src.into(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        println!("first {what}: {:.0} ms (in cell: {:.0} ms)", ms(t), r.ms);
    }

    let cell = "import numpy as np\nx = np.linspace(0, L, 50)\nmoment = w * x * (L - x) / 2\nM_max = moment.max()";
    let mut scans = vec![];
    for _ in 0..50 {
        let t = Instant::now();
        k.scan(cell).await.unwrap();
        scans.push(ms(t));
    }
    println!("scan of a 4-line cell, median: {:.2} ms", median(scans));

    let mut trips = vec![];
    for _ in 0..50 {
        let t = Instant::now();
        k.run(
            &RunRequest {
                cell: "T".into(),
                source: "y = 1".into(),
                defines: vec!["y".into()],
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
        trips.push(ms(t));
    }
    println!(
        "run of `y = 1` (round trip), median: {:.2} ms",
        median(trips)
    );

    for mb in [1usize, 16, 128] {
        let blob = vec![7u8; mb << 20];
        let t = Instant::now();
        let (_, back) = k.request(json!({"op": "echo"}), vec![blob]).await.unwrap();
        let took = ms(t);
        assert_eq!(back[0].len(), mb << 20);
        println!(
            "echo {mb} MB blob: {took:.0} ms ({:.0} MB/s each way)",
            2.0 * mb as f64 / (took / 1000.0)
        );
    }

    // A frame with units in field metadata.
    let unit = |u: &str| HashMap::from([("cruncha.unit".to_string(), u.to_string())]);
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("load", DataType::Float64, false).with_metadata(unit("kip/ft")),
        Field::new("span", DataType::Float64, false).with_metadata(unit("ft")),
    ]));
    let t = Instant::now();
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from_iter_values(0..rows as i64)),
            Arc::new(Float64Array::from_iter_values(
                (0..rows).map(|i| 1.0 + (i % 100) as f64 / 100.0),
            )),
            Arc::new(Float64Array::from_iter_values(
                (0..rows).map(|i| 10.0 + (i % 30) as f64),
            )),
        ],
    )
    .unwrap();
    let mut ipc = Vec::new();
    {
        let mut w = StreamWriter::try_new(&mut ipc, &schema).unwrap();
        w.write(&batch).unwrap();
        w.finish().unwrap();
    }
    let encode = ms(t);
    let size = ipc.len();
    let mut trips = vec![];
    let mut answer = None;
    for _ in 0..3 {
        let t = Instant::now();
        answer = Some(
            k.request(json!({"op": "arrow_roundtrip"}), vec![ipc.clone()])
                .await
                .unwrap(),
        );
        trips.push(ms(t));
    }
    let (h, back) = answer.unwrap();
    let trip = median(trips);
    let t = Instant::now();
    let reader = StreamReader::try_new(back[0].as_slice(), None).unwrap();
    let schema_back = reader.schema();
    let batches: Vec<RecordBatch> = reader.map(|b| b.unwrap()).collect();
    let decode = ms(t);
    let n: usize = batches.iter().map(|b| b.num_rows()).sum();
    let moment = schema_back.field_with_name("moment").unwrap();
    assert_eq!(n, rows);
    assert_eq!(
        moment.metadata().get("cruncha.unit").map(String::as_str),
        Some("kip·ft")
    );
    assert_eq!(h["units_in"]["load"], "kip/ft");
    println!(
        "frame {rows} rows × 3 cols ({:.1} MB IPC), 4 cols back ({:.1} MB): Rust encode {encode:.0} ms, round trip (median of 3) {trip:.0} ms, Rust decode {decode:.0} ms",
        size as f64 / 1e6,
        back[0].len() as f64 / 1e6
    );
    println!("  in Python: {}", h["ms"]);
    println!(
        "  units survived: load={} span={} moment={}",
        h["units_in"]["load"],
        h["units_in"]["span"],
        moment.metadata()["cruncha.unit"]
    );
    k.shutdown().await;
}
