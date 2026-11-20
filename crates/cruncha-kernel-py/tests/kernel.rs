// SPDX-License-Identifier: GPL-3.0-or-later
//! The worker process end to end. Needs the Python environment: set `CRUNCHA_TEST_PYTHON`, or
//! set it up once (the tests then find it under the app's data directory). Without one, the
//! tests say so and pass.

use cruncha_kernel_py::{Config, Error, Kernel, Output, RunRequest, Status, env};
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const LOCK: &str = include_str!("../../../python/requirements.lock");

fn worker_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python")
}

fn python() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("CRUNCHA_TEST_PYTHON") {
        return Some(p.into());
    }
    let base = PathBuf::from(std::env::var_os("HOME")?)
        .join("Library/Application Support/io.github.acmeanvil.cruncha/python");
    env::ready(&base, LOCK)
}

async fn kernel() -> Option<Kernel> {
    let Some(py) = python() else {
        eprintln!("no Python environment; skipping (see tests/kernel.rs)");
        return None;
    };
    let mut c = Config::new(py, worker_dir());
    c.kill_after = Duration::from_millis(1500);
    Some(Kernel::start(c).await.expect("worker starts"))
}

fn num(v: f64, unit: Option<serde_json::Value>) -> serde_json::Value {
    json!({"type": "number", "value": {"type": "float", "value": v}, "unit": unit})
}

#[tokio::test]
async fn hello_scan_and_run() {
    let Some(mut k) = kernel().await else { return };
    assert_eq!(k.hello().protocol, 1);
    assert!(k.hello().python.starts_with("3.13"), "{:?}", k.hello());

    let s = k
        .scan("import numpy as np\nx = np.linspace(0, L, 50)\nmoment = w * x * (L - x) / 2")
        .await
        .unwrap();
    assert_eq!(
        (s.defines, s.reads),
        (
            vec!["x".to_string(), "moment".into()],
            vec!["L".to_string(), "w".into()]
        )
    );
    let bad = k.scan("x = (").await.unwrap();
    assert_eq!(bad.error.unwrap().kind, "SyntaxError");

    let ft =
        json!({"name": "ft", "factor": 0.3048, "gauge": 0.0, "dims": [1, 0, 0, 0, 0, 0, 0, 0]});
    let klf = json!({"name": "kip/ft", "factor": 14593.902937206364, "gauge": 0.0, "dims": [0, 1, -2, 0, 0, 0, 0, 0]});
    let mut req = RunRequest {
        cell: "PY".into(),
        source: "import numpy as np\nx = np.linspace(0, L, 5)\nmoment = (w * x * (L - x) / 2).to('kip*ft')\nM_max = moment.max()\nprint('ok')".into(),
        defines: vec!["x".into(), "moment".into(), "M_max".into()],
        ..Default::default()
    };
    req.inputs.insert("L".into(), num(26.0, Some(ft)));
    req.inputs.insert("w".into(), num(1.5, Some(klf)));
    let r = k.run(&req, None).await.unwrap();
    assert_eq!(r.status, Status::Done, "{:?}", r.error);
    assert_eq!(r.stdout, "ok\n");
    let m = r.defines.iter().find(|d| d.name == "M_max").unwrap();
    assert!(
        (m.plain["value"]["value"].as_f64().unwrap() - 126.75).abs() < 1e-9,
        "{m:?}"
    );
    assert_eq!(m.plain["unit"]["name"], "ft·kip");
    let moment = r.defines.iter().find(|d| d.name == "moment").unwrap();
    assert_eq!(moment.plain["type"], "array");

    // A later cell reads from the worker.
    let r = k
        .run(
            &RunRequest {
                cell: "P2".into(),
                source: "half = M_max / 2\nhalf".into(),
                defines: vec!["half".into()],
                python_inputs: vec!["M_max".into()],
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(r.status, Status::Done, "{:?}", r.error);
    assert!(
        matches!(&r.outputs[0], Output::Text { text } if text.contains("63.375")),
        "{:?}",
        r.outputs
    );
    k.shutdown().await;
}

#[tokio::test]
async fn errors_figures_and_interrupts() {
    let Some(mut k) = kernel().await else { return };
    let r = k
        .run(
            &RunRequest {
                cell: "E".into(),
                source: "a = 1\nb = a / 0".into(),
                defines: vec!["a".into(), "b".into()],
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(r.status, Status::Error);
    assert_eq!(r.error.as_ref().unwrap().line, Some(2));

    let r = k
        .run(
            &RunRequest {
                cell: "F".into(),
                source: "import matplotlib.pyplot as plt\nplt.plot([1, 3, 2])".into(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert!(
        matches!(&r.outputs[0], Output::Image { data, .. } if data.starts_with(b"\x89PNG")),
        "{:?}",
        r.status
    );

    // A timeout interrupts the cell; the worker survives and keeps going.
    let t = Instant::now();
    let r = k
        .run(
            &RunRequest {
                cell: "L".into(),
                source: "while True:\n    pass".into(),
                ..Default::default()
            },
            Some(Duration::from_millis(300)),
        )
        .await
        .unwrap();
    assert_eq!(r.status, Status::Interrupted);
    assert!(t.elapsed() < Duration::from_secs(2));
    let r = k
        .run(
            &RunRequest {
                cell: "OK".into(),
                source: "z = 1".into(),
                defines: vec!["z".into()],
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(r.status, Status::Done);

    // C code that ignores SIGINT (here: SIGINT blocked) gets the worker killed.
    let r = k
        .run(
            &RunRequest { cell: "H".into(), source: "import signal, time\nsignal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})\ntime.sleep(30)".into(), ..Default::default() },
            Some(Duration::from_millis(200)),
        )
        .await;
    assert_eq!(r.unwrap_err(), Error::Killed);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!k.is_alive());
}
