// SPDX-License-Identifier: GPL-3.0-or-later
//! Set up the Python environment for Python cells where the app looks for it, e.g. on a new
//! development machine: `cargo run -p cruncha-kernel-py --example setup_env [base-dir]`.

use cruncha_kernel_py::env;
use std::path::PathBuf;

const LOCK: &str = include_str!("../../../python/requirements.lock");

#[tokio::main]
async fn main() {
    let base = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::default_base().expect("a home directory"));
    let t = std::time::Instant::now();
    match env::setup(&base, LOCK, |p| {
        eprintln!("{p:?} ({:.1} s)", t.elapsed().as_secs_f64())
    })
    .await
    {
        Ok(py) => println!("{}", py.display()),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
