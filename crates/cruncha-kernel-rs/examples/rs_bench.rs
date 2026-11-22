// SPDX-License-Identifier: GPL-3.0-or-later
//! Spike S5: how long a Rust cell takes to compile, load and call.
//!
//!     cargo run -p cruncha-kernel-rs --release --example rs_bench -- <work dir>
//!
//! Uses Cruncha's toolchain (`Toolchain::default_base()`). Steps:
//! 1. cold build: an empty target directory (the prelude, wit-bindgen and serde_json compile);
//! 2. warm builds after editing the cell, release and debug profiles;
//! 3. compiling the component to machine code, caching it, loading it back, instantiating;
//! 4. calls: no data, a 100k-element array, the JSON `run` with two inputs.

use std::time::Instant;

use cruncha_kernel_rs::{Runtime, Toolchain, write_cell};

const CELL: &str = r#"
struct C;

impl Guest for C {
    fn run(inputs: String) -> Result<String, String> {
        let v: Value = serde_json::from_str(&inputs).map_err(|e| e.to_string())?;
        let w = number(&v, "w")?;
        let l = number(&v, "L")?;
        Ok(json!({ "M": float(w * l * l / 8.0 * SCALE) }).to_string())
    }
    fn scalar(x: f64) -> f64 {
        x * 2.0 + SCALE
    }
    fn sum(xs: Vec<f64>) -> f64 {
        xs.iter().sum()
    }
}

const SCALE: f64 = 1.0;

cruncha_prelude::bindings::export_cell!(C with_types_in cruncha_prelude::bindings);
"#;

fn ms(d: std::time::Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn main() -> wasmtime::Result<()> {
    let work = std::path::PathBuf::from(std::env::args().nth(1).expect("usage: rs_bench <work dir>"));
    let tc = Toolchain::at(&Toolchain::default_base().unwrap()).expect("Cruncha's Rust toolchain isn't set up");
    let cell = work.join("cell_bench");
    let target = work.join("target");
    write_cell(&cell, "cell_bench", CELL)?;

    // A first build fetches the crates from the registry (not timed as compile).
    let (_, fetch) = tc.build(&cell, &target, true)?;
    println!("first build (incl. registry download): {:.1} s", fetch.as_secs_f64());
    let _ = std::fs::remove_dir_all(&target);
    let (wasm, cold) = tc.build(&cell, &target, true)?;
    println!("cold build, release (deps compile):    {:.1} s", cold.as_secs_f64());
    let (_, cold_dbg) = tc.build(&cell, &target, false)?;
    println!("cold build, debug:                     {:.1} s", cold_dbg.as_secs_f64());

    for (i, release) in [(1, true), (2, true), (3, false), (4, false)] {
        write_cell(&cell, "cell_bench", &CELL.replace("SCALE: f64 = 1.0", &format!("SCALE: f64 = 1.{i}")))?;
        let (_, warm) = tc.build(&cell, &target, release)?;
        println!("warm build after an edit, {}:   {:.0} ms", if release { "release" } else { "debug  " }, ms(warm));
    }
    write_cell(&cell, "cell_bench", CELL)?;
    let (wasm2, _) = tc.build(&cell, &target, true)?;
    assert_eq!(wasm, wasm2);
    let size = std::fs::metadata(&wasm)?.len();
    println!("component size (release):              {:.0} KB", size as f64 / 1024.0);

    let rt = Runtime::new()?;
    let t = Instant::now();
    let component = rt.compile(&wasm)?;
    println!("Cranelift compile of the component:    {:.0} ms", ms(t.elapsed()));
    let t = Instant::now();
    let bytes = rt.precompile(&wasm)?;
    println!("precompile (to cache):                 {:.0} ms, {:.0} KB", ms(t.elapsed()), bytes.len() as f64 / 1024.0);
    let t = Instant::now();
    let cached = unsafe { rt.load_precompiled(&bytes)? };
    println!("load precompiled:                      {:.2} ms", ms(t.elapsed()));
    drop(component);
    let t = Instant::now();
    let mut c = rt.instantiate(&cached)?;
    println!("instantiate:                           {:.2} ms", ms(t.elapsed()));
    let t = Instant::now();
    let n = 20;
    for _ in 0..n {
        rt.instantiate(&cached)?;
    }
    println!("instantiate (avg of {n}):               {:.3} ms", ms(t.elapsed()) / n as f64);

    assert_eq!(c.scalar(3.0)?, 7.0);
    let n = 100_000;
    let t = Instant::now();
    let mut acc = 0.0;
    for i in 0..n {
        acc += c.scalar(i as f64)?;
    }
    println!("call, no data (avg of {n}):         {:.2} µs  (checksum {acc})", ms(t.elapsed()) * 1e3 / n as f64);
    let xs: Vec<f64> = (0..100_000).map(|i| i as f64).collect();
    let n = 200;
    let t = Instant::now();
    for _ in 0..n {
        assert_eq!(c.sum(&xs)?, 4_999_950_000.0);
    }
    println!("call with 100k f64 (avg of {n}):        {:.3} ms", ms(t.elapsed()) / n as f64);
    let inputs = r#"{"w":{"type":"number","value":{"type":"float","value":1.5},"unit":null},
                     "L":{"type":"number","value":{"type":"exact","num":"20","den":"1"},"unit":null}}"#;
    assert_eq!(c.run(inputs)?, Ok(r#"{"M":{"type":"number","unit":null,"value":{"type":"float","value":75.0}}}"#.into()));
    let n = 20_000;
    let t = Instant::now();
    for _ in 0..n {
        c.run(inputs)?.unwrap();
    }
    println!("run, JSON with two inputs (avg {n}):   {:.2} µs", ms(t.elapsed()) * 1e3 / n as f64);
    Ok(())
}
