// SPDX-License-Identifier: GPL-3.0-or-later
//! The worker process end to end. Needs the Python environment: set `CRUNCHA_TEST_PYTHON`, or
//! set it up once (the tests then find it under the app's data directory). Without one, the
//! tests say so and pass.

use cruncha_kernel_py::{Config, Error, InterruptMode, Kernel, Output, RunRequest, Status, env};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const LOCK: &str = include_str!("../../../python/requirements.lock");

fn worker_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python")
}

fn python() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("CRUNCHA_TEST_PYTHON") {
        return Some(p.into());
    }
    env::ready(&env::default_base()?, LOCK)
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

    // C code that ignores SIGINT (here: SIGINT blocked) gets the worker killed. (Windows
    // interrupts by message; message_interrupts_… covers its kill.)
    if !cfg!(unix) {
        return;
    }
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

/// Windows' interrupt (an `interrupt` message read by a thread in the worker), checked here on
/// every platform.
#[tokio::test]
async fn message_interrupts_stop_a_cell_and_the_worker_keeps_going() {
    let Some(py) = python() else { return };
    let mut c = Config::new(py, worker_dir());
    c.kill_after = Duration::from_millis(1500);
    c.interrupt = InterruptMode::Message;
    let mut k = Kernel::start(c).await.expect("worker starts");
    let run = |cell: &str, src: &str| RunRequest { cell: cell.into(), source: src.into(), defines: vec![], ..Default::default() };
    for (cell, src) in [("L", "while True:\n    pass"), ("S", "import time\ntime.sleep(30)")] {
        let t = Instant::now();
        let r = k.run(&run(cell, src), Some(Duration::from_millis(300))).await.unwrap();
        assert_eq!(r.status, Status::Interrupted, "{cell}");
        assert!(t.elapsed() < Duration::from_secs(2), "{cell}: {:?}", t.elapsed());
    }
    let mut ok = run("OK", "z = 1");
    ok.defines = vec!["z".into()];
    assert_eq!(k.run(&ok, None).await.unwrap().status, Status::Done);
    // An interrupt sent while nothing runs is dropped, not raised in the next cell.
    k.interrupter().interrupt();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(k.run(&ok, None).await.unwrap().status, Status::Done);
    // A cell that swallows the interrupt gets the worker killed.
    let r = k
        .run(&run("H", "while True:\n    try:\n        while True:\n            pass\n    except KeyboardInterrupt:\n        pass"), Some(Duration::from_millis(200)))
        .await;
    assert_eq!(r.unwrap_err(), Error::Killed);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!k.is_alive());
}

#[tokio::test]
async fn plotly_figures_arrive_as_json_specs() {
    let Some(mut k) = kernel().await else { return };
    let r = k
        .run(
            &RunRequest {
                cell: "P".into(),
                source: "import plotly.graph_objects as go\nfig = go.Figure(go.Bar(x=[1, 2], y=[3, 4], name='load'))\nfig.show()".into(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(r.status, Status::Done, "{:?}", r.error);
    let Some(Output::Figure { mime, spec, summary, ranges, static_image }) = r.outputs.first() else {
        panic!("{:?}", r.outputs)
    };
    assert_eq!(mime, "application/vnd.plotly.v1+json");
    let v: serde_json::Value = serde_json::from_str(spec).unwrap();
    assert_eq!(v["data"][0]["uid"], "P-0-0");
    assert_eq!(summary, "Figure; 1 trace: load (bar, 2 points)");
    assert_eq!(ranges[0]["y"], json!([3.0, 4.0]));
    assert!(static_image.is_none());
}

#[tokio::test]
async fn the_sandboxed_worker_computes_but_reaches_nothing() {
    use cruncha_kernel_py::sandbox::{self, Launcher, Sandbox};
    let Some(py) = python() else { return };
    // macOS: sandbox-exec; Windows: an AppContainer and Job Object (ADR-0030), started by
    // this crate's `cruncha-sandbox` launcher.
    if sandbox::program().is_none() && !cfg!(windows) {
        eprintln!("no sandbox on this system; skipping");
        return;
    }
    let scratch = tempfile::tempdir().unwrap();
    // The app's layout (`…/python/env-*/…`), or a venv elsewhere (CI's `.venv`; on Windows
    // the launcher adds the venv's base interpreter itself).
    let base = py
        .ancestors()
        .find(|p| p.is_dir() && p.file_name().is_some_and(|n| n == "python"))
        .unwrap_or_else(|| py.parent().and_then(Path::parent).expect("the interpreter is in an environment"))
        .to_path_buf();
    let mut c = Config::new(py, worker_dir());
    let launcher = Launcher { program: env!("CARGO_BIN_EXE_cruncha-sandbox").into(), args: vec![] };
    c.sandbox = Some(Sandbox { read: vec![base, worker_dir()], scratch: scratch.path().join("w"), launcher: Some(launcher) });
    let mut k = Kernel::start(c).await.expect("the sandboxed worker starts");
    let run = |src: &str| RunRequest { cell: "S".into(), source: src.into(), defines: vec!["r".into()], ..Default::default() };
    let value = |r: &cruncha_kernel_py::RunResult| r.defines.first().map(|d| d.summary.clone()).unwrap_or_default();

    // Numeric work, plots and figures still work.
    let r = k
        .run(&run("import numpy as np, scipy.linalg as la, polars as pl\nimport matplotlib.pyplot as plt\nimport plotly.graph_objects as go\nfrom cruncha import Q_\nplt.plot([1, 2]); go.Figure(go.Bar(y=[1])).show()\nr = float(la.det(np.eye(3) * 2)) + pl.DataFrame({'a': [1]}).height + Q_(1, 'kip').to('lbf').magnitude"), None)
        .await
        .unwrap();
    assert_eq!(r.status, Status::Done, "{:?} {}", r.error, r.stderr);
    assert_eq!(value(&r), "1009.0");
    assert_eq!(r.outputs.len(), 2, "a PNG and a figure");

    // The user's files, the network and other programs are out of reach.
    let home = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap();
    let probe = std::path::Path::new(&home).join("cruncha-sandbox-probe.txt");
    // A file of the user's beside the scratch directory.
    let secret = scratch.path().join("secret.txt");
    std::fs::write(&secret, "secret").unwrap();
    // System files are readable to every Windows app by design, so there: the user's AppData.
    let system = match std::env::var("APPDATA") {
        Ok(d) if cfg!(windows) => format!("import os\nr = str(len(os.listdir({d:?})))"),
        _ => "r = open('/etc/hosts').read()".to_string(),
    };
    for (what, src) in [
        ("home", format!("import os\nr = str(len(os.listdir({home:?})))")),
        ("file", system),
        ("secret", format!("r = open({:?}).read()", secret.display().to_string())),
        ("write", format!("open({:?}, 'w').write('x'); r = 'wrote'", probe.display().to_string())),
        ("net", "import socket\ns = socket.create_connection(('1.1.1.1', 443), timeout=3)\nr = 'connected'".to_string()),
        ("loopback", "import socket\ns = socket.socket(); s.bind(('127.0.0.1', 0)); s.listen()\nc = socket.create_connection(s.getsockname(), timeout=3)\nr = 'connected'".to_string()),
        ("exec", "import subprocess, sys\nr = subprocess.run([sys.executable, '-c', 'print(1)'], capture_output=True).stdout.decode()".to_string()),
    ] {
        let r = k.run(&run(&src), None).await.unwrap();
        assert_eq!(r.status, Status::Error, "{what}: {:?}", r.defines);
    }
    assert!(!probe.exists());
    // Its scratch directory is writable.
    let r = k.run(&run("import tempfile\nwith tempfile.NamedTemporaryFile() as f:\n    f.write(b'ok')\nr = 'ok'"), None).await.unwrap();
    assert_eq!(r.status, Status::Done, "{:?}", r.error);
}
