// SPDX-License-Identifier: GPL-3.0-or-later
//! A Rust cell end to end (ADR-0006): built with Cruncha's toolchain, run sandboxed. Needs the
//! toolchain (`~/Library/Application Support/io.github.acmeanvil.cruncha/rust`); without it
//! the test says so and passes.

use std::time::{Duration, Instant};

use cruncha_kernel_rs::{Runtime, Toolchain, write_cell};

const PROBE: &str = r#"
struct C;

impl Guest for C {
    fn run(inputs: String) -> Result<String, String> {
        match inputs.as_str() {
            "file" => std::fs::read_to_string("/etc/hosts").map_err(|e| format!("file: {e}")),
            "env" => std::env::var("HOME").map_err(|e| format!("env: {e}")),
            "net" => std::net::TcpStream::connect("127.0.0.1:22").map(|_| "connected".into()).map_err(|e| format!("net: {e}")),
            "loop" => { let mut i: u64 = 0; loop { i = std::hint::black_box(i.wrapping_add(1)); } }
            _ => Ok("ok".into()),
        }
    }
    fn scalar(x: f64) -> f64 { x }
    fn sum(xs: Vec<f64>) -> f64 { xs.iter().sum() }
}

cruncha_prelude::bindings::export_cell!(C with_types_in cruncha_prelude::bindings);
"#;

#[test]
fn a_cell_has_no_files_environment_or_network_and_can_be_stopped() {
    let Some(tc) = Toolchain::default_base().and_then(|b| Toolchain::at(&b)) else {
        eprintln!("no Rust toolchain for cells; skipping");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let target = std::env::temp_dir().join("cruncha-rust-cells-test-target");
    write_cell(&dir.path().join("probe"), "probe", PROBE).unwrap();
    let (wasm, _) = tc.build(&dir.path().join("probe"), &target, true).unwrap();
    let rt = Runtime::new().unwrap();
    let component = rt.compile(&wasm).unwrap();
    let mut c = rt.instantiate(&component).unwrap();
    assert_eq!(c.run("hello").unwrap(), Ok("ok".into()));
    let file = c.run("file").unwrap();
    assert!(file.as_ref().is_err_and(|e| e.starts_with("file:")), "{file:?}");
    let env = c.run("env").unwrap();
    assert!(env.as_ref().is_err_and(|e| e.starts_with("env:")), "{env:?}");
    let net = c.run("net").unwrap();
    assert!(net.as_ref().is_err_and(|e| e.starts_with("net:")), "{net:?}");

    let t = Instant::now();
    let stuck = c.run_with_timeout("loop", Duration::from_millis(200));
    assert!(stuck.is_err(), "trapped");
    assert!(t.elapsed() < Duration::from_secs(2), "{:?}", t.elapsed());
    // A fresh instance works after a trap.
    let mut c = rt.instantiate(&component).unwrap();
    assert_eq!(c.run_with_timeout("hello", Duration::from_secs(5)).unwrap(), Ok("ok".into()));
}
