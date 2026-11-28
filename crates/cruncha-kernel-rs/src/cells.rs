// SPDX-License-Identifier: GPL-3.0-or-later
//! Running Rust cells (ADR-0006): a cell's source becomes the body of the `run` export of a
//! small crate, compiled for `wasm32-wasip2` and run in wasmtime.
//!
//! - Each input the cell reads is a typed `let` at the top, its type chosen from the value
//!   (`Quantity`, `f64`, `bool`, `String`, `Vec<f64>`, `Vec<Quantity>`, `Vec<Vec<f64>>`), so
//!   the code is compiled per source *and* input types.
//! - Each name the cell defines goes back as `Plain` JSON with a summary; values with no
//!   `ToPlain` go back as opaque (calcs are told what they are).
//! - Builds are cached: a crate per (source, input types), named by its hash, and its compiled
//!   machine code next to it, so a cell that didn't change loads in under a millisecond.
//! - Compile errors and panics are reported at the cell's own line numbers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::Value;
use sha2::{Digest, Sha256};
use wasmtime::component::Component;

use crate::{Runtime, Toolchain, prelude_dir};

/// Bump when the generated code or the prelude changes, so cached builds aren't reused.
pub const CELL_FORMAT: &str = "rust-cell/2";

/// A cell to run.
#[derive(Debug, Clone, Default)]
pub struct Request {
    pub cell: String,
    pub source: String,
    /// What the scan found it defines (each goes back).
    pub defines: Vec<String>,
    /// Name → `Plain` value of everything it reads.
    pub inputs: serde_json::Map<String, Value>,
}

/// An error in a cell, at its own line when known.
#[derive(Debug, Clone, PartialEq)]
pub struct CellError {
    pub kind: String,
    pub message: String,
    pub line: Option<u32>,
    pub detail: String,
}

/// What a run produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub ok: bool,
    /// `(name, summary, plain)` per define.
    pub defines: Vec<(String, String, Value)>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<CellError>,
    pub interrupted: bool,
    pub compile_ms: f64,
    pub run_ms: f64,
}

/// The Rust type a `Plain` value arrives as, if it can.
pub fn rust_type(plain: &Value) -> Option<&'static str> {
    let has_unit = |v: &Value| {
        v.get("unit").is_some_and(|u| {
            !u.is_null() && (u["name"].is_string() || u["dims"].as_array().is_some_and(|d| d.iter().any(|x| x.as_f64() != Some(0.0))))
        })
    };
    match plain["type"].as_str()? {
        "number" => Some(if has_unit(plain) { "Quantity" } else { "f64" }),
        "bool" => Some("bool"),
        "text" => Some("String"),
        "array" => match plain["shape"].as_array().map(Vec::len) {
            Some(1) => Some(if has_unit(plain) { "Vec<Quantity>" } else { "Vec<f64>" }),
            Some(2) if !has_unit(plain) => Some("Vec<Vec<f64>>"),
            _ => None,
        },
        _ => None,
    }
}

/// The crate's `lib.rs` for a cell, and the line where the cell's own source starts.
pub fn generate(req: &Request) -> Result<(String, usize), String> {
    let mut head = String::from(
        "#![allow(non_snake_case, unused_imports, unused_mut, unused_variables, dead_code, unused_parens, clippy::all)]\n\
         use cruncha_prelude::*;\n\
         use cruncha_prelude::__private::{Wrap, ViaPlain as _, ViaOpaque as _, inputs as __inputs, Outputs as __Outputs};\n\
         struct __Cell;\n\
         impl Guest for __Cell {\n\
         fn run(__json: String) -> Result<String, String> {\n\
         let __in = __inputs(&__json)?;\n",
    );
    let mut names: Vec<&String> = req.inputs.keys().collect();
    names.sort();
    for name in names {
        let ty = rust_type(&req.inputs[name]).ok_or_else(|| format!("{name} can't be passed to a Rust cell"))?;
        head.push_str(&format!("let {name}: {ty} = __in.get({name:?})?;\n"));
    }
    let start = head.lines().count() + 1;
    let mut code = head;
    code.push_str(&req.source);
    code.push_str("\n;\nlet mut __out = __Outputs::new();\n");
    for d in &req.defines {
        code.push_str(&format!("__out.put({d:?}, (&&Wrap(&{d})).out());\n"));
    }
    code.push_str(
        "Ok(__out.finish())\n}\n\
         fn scalar(x: f64) -> f64 { x }\n\
         fn sum(xs: Vec<f64>) -> f64 { xs.iter().sum() }\n\
         }\n\
         cruncha_prelude::bindings::export_cell!(__Cell with_types_in cruncha_prelude::bindings);\n",
    );
    Ok((code, start))
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Builds, caches and runs cells. One build at a time; runs are independent.
pub struct Cells {
    toolchain: Toolchain,
    runtime: Runtime,
    /// `<base>/cells/<hash>/` per build, `<base>/target` shared.
    work: PathBuf,
    loaded: Mutex<HashMap<String, Component>>,
    building: tokio::sync::Mutex<()>,
}

impl Cells {
    /// Cells built with `toolchain`, kept under `work`.
    pub fn new(toolchain: Toolchain, work: PathBuf) -> wasmtime::Result<Cells> {
        Ok(Cells { toolchain, runtime: Runtime::new()?, work, loaded: Mutex::default(), building: tokio::sync::Mutex::new(()) })
    }

    /// Stops the running cell (it traps at its next check).
    pub fn interrupter(&self) -> wasmtime::Engine {
        self.runtime.engine.clone()
    }

    /// A label for what Rust results depend on: the toolchain and the cell format (part of a
    /// stored result's input hash).
    pub fn env_label(&self) -> String {
        let rustc = std::fs::read_dir(self.toolchain.rustup_home.join("toolchains"))
            .ok()
            .and_then(|mut d| d.next())
            .and_then(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .unwrap_or_default();
        format!("{rustc}+{CELL_FORMAT}")
    }

    /// Compile (or load) a cell's code. `Err` is a compile error at the cell's lines.
    async fn component(&self, code: &str, start: usize) -> Result<(Component, f64), CellError> {
        let hash = hex(&Sha256::new().chain_update(CELL_FORMAT).chain_update(code).finalize())[..16].to_owned();
        if let Some(c) = self.loaded.lock().unwrap().get(&hash) {
            return Ok((c.clone(), 0.0));
        }
        let _guard = self.building.lock().await;
        let dir = self.work.join("cells").join(&hash);
        let cached = dir.join("cell.cwasm");
        let t = Instant::now();
        let component = if cached.is_file() {
            let bytes = std::fs::read(&cached).map_err(|e| internal(e.to_string()))?;
            // Safety: written by `precompile` of this same wasmtime build (checked by wasmtime).
            unsafe { self.runtime.load_precompiled(&bytes) }.map_err(|e| internal(e.to_string()))?
        } else {
            let name = format!("cell_{hash}");
            let prelude = prelude_dir(&self.work).map_err(|e| internal(format!("the Rust prelude: {e}")))?;
            write_crate(&dir, &name, code, &prelude).map_err(|e| internal(e.to_string()))?;
            let wasm = build(&self.toolchain, &dir, &self.work.join("target"), &name, start).await?;
            let bytes = self.runtime.precompile(&wasm).map_err(|e| internal(e.to_string()))?;
            let _ = std::fs::write(&cached, &bytes);
            unsafe { self.runtime.load_precompiled(&bytes) }.map_err(|e| internal(e.to_string()))?
        };
        self.loaded.lock().unwrap().insert(hash, component.clone());
        Ok((component, t.elapsed().as_secs_f64() * 1e3))
    }

    /// Compile and run a cell, stopping it after `timeout`.
    pub async fn run(&self, req: &Request, timeout: Option<Duration>) -> Outcome {
        let mut out = Outcome {
            ok: false,
            defines: vec![],
            stdout: String::new(),
            stderr: String::new(),
            error: None,
            interrupted: false,
            compile_ms: 0.0,
            run_ms: 0.0,
        };
        let (code, start) = match generate(req) {
            Ok(g) => g,
            Err(e) => {
                out.error = Some(CellError { kind: "TypeError".into(), message: e, line: None, detail: String::new() });
                return out;
            }
        };
        let (component, compile_ms) = match self.component(&code, start).await {
            Ok(c) => c,
            Err(e) => {
                out.error = Some(e);
                return out;
            }
        };
        out.compile_ms = compile_ms;
        let inputs = Value::Object(req.inputs.clone()).to_string();
        let runtime = &self.runtime;
        let t = Instant::now();
        let result = tokio::task::block_in_place(|| {
            let mut cell = runtime.instantiate(&component)?;
            let r = cell.run_with_timeout(&inputs, timeout.unwrap_or(Duration::from_secs(3600)));
            Ok::<_, wasmtime::Error>((r, cell.stdout(), cell.stderr()))
        });
        out.run_ms = t.elapsed().as_secs_f64() * 1e3;
        match result {
            Err(e) => out.error = Some(internal(e.to_string())),
            Ok((r, stdout, stderr)) => {
                out.stdout = stdout;
                out.stderr = stderr;
                match r {
                    Ok(Ok(json)) => match serde_json::from_str::<Vec<Value>>(&json) {
                        Ok(list) => {
                            out.defines = list
                                .into_iter()
                                .map(|v| {
                                    (
                                        v["name"].as_str().unwrap_or_default().to_owned(),
                                        v["summary"].as_str().unwrap_or_default().to_owned(),
                                        v["plain"].clone(),
                                    )
                                })
                                .collect();
                            out.ok = true;
                        }
                        Err(e) => out.error = Some(internal(format!("unreadable outputs: {e}"))),
                    },
                    Ok(Err(message)) => {
                        out.error = Some(CellError { kind: "Error".into(), message, line: None, detail: String::new() })
                    }
                    Err(trap) => {
                        let (message, line) = panic_of(&out.stderr, start);
                        let interrupted = trap.downcast_ref::<wasmtime::Trap>() == Some(&wasmtime::Trap::Interrupt);
                        out.interrupted = interrupted && message.is_none();
                        out.error = Some(match message {
                            Some(m) => CellError { kind: "panic".into(), message: m, line, detail: String::new() },
                            None if out.interrupted => CellError { kind: "interrupted".into(), message: "stopped".into(), line: None, detail: String::new() },
                            None => CellError { kind: "trap".into(), message: format!("{trap:#}"), line: None, detail: String::new() },
                        });
                    }
                }
            }
        }
        out
    }
}

fn internal(message: String) -> CellError {
    CellError { kind: "internal".into(), message, line: None, detail: String::new() }
}

fn write_crate(dir: &Path, name: &str, code: &str, prelude: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n\
             [dependencies]\ncruncha-prelude = {{ path = {:?} }}\n\n[profile.release]\nopt-level = 2\ndebug = false\n\n[workspace]\n",
            prelude.display().to_string()
        ),
    )?;
    std::fs::write(dir.join("src/lib.rs"), code)
}

/// `cargo build` with JSON diagnostics; errors mapped to the cell's lines.
async fn build(tc: &Toolchain, dir: &Path, target: &Path, name: &str, start: usize) -> Result<PathBuf, CellError> {
    let out = crate::no_window(&mut tokio::process::Command::new(tc.cargo()))
        .args(["build", "--release", "--quiet", "--message-format=json", "--target", "wasm32-wasip2", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .env("RUSTUP_HOME", &tc.rustup_home)
        .env("CARGO_HOME", &tc.cargo_home)
        .env("CARGO_TARGET_DIR", target)
        .env_remove("RUSTC")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .output()
        .await
        .map_err(|e| internal(format!("couldn't start cargo: {e}")))?;
    if out.status.success() {
        return Ok(target.join("wasm32-wasip2/release").join(format!("{name}.wasm")));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut errors = Vec::new();
    for line in stdout.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        if v["reason"] != "compiler-message" || v["message"]["level"] != "error" {
            continue;
        }
        let m = &v["message"];
        let span = m["spans"].as_array().and_then(|s| s.iter().find(|s| s["is_primary"] == true));
        let line = span
            .filter(|s| s["file_name"].as_str().is_some_and(|f| f.ends_with("src/lib.rs")))
            .and_then(|s| s["line_start"].as_u64())
            .and_then(|l| (l as usize).checked_sub(start - 1))
            .filter(|l| *l >= 1)
            .map(|l| l as u32);
        let label = span.and_then(|s| s["label"].as_str()).filter(|l| !l.is_empty());
        let text = match label {
            Some(l) => format!("{}: {l}", m["message"].as_str().unwrap_or("error")),
            None => m["message"].as_str().unwrap_or("error").to_owned(),
        };
        errors.push((text, line));
    }
    let Some((first, line)) = errors.first().cloned() else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(internal(format!("the build failed: {}", stderr.lines().last().unwrap_or_default())));
    };
    let detail = errors
        .iter()
        .map(|(m, l)| match l {
            Some(l) => format!("  line {l}: {m}"),
            None => format!("  {m}"),
        })
        .collect::<Vec<_>>()
        .join("\n");
    Err(CellError { kind: "CompileError".into(), message: first, line, detail })
}

/// A panic's message and cell line, from what the cell wrote to stderr.
fn panic_of(stderr: &str, start: usize) -> (Option<String>, Option<u32>) {
    let mut lines = stderr.lines();
    while let Some(l) = lines.next() {
        if let Some(rest) = l.split("panicked at ").nth(1) {
            let line = rest
                .split(':')
                .nth(1)
                .and_then(|n| n.parse::<usize>().ok())
                .filter(|_| rest.starts_with("src/lib.rs"))
                .and_then(|n| n.checked_sub(start - 1))
                .filter(|n| *n >= 1)
                .map(|n| n as u32);
            let message = lines.next().unwrap_or("panicked").trim().to_owned();
            return (Some(message), line);
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn input_types_follow_the_values() {
        let num = |unit: Value| json!({"type": "number", "value": {"type": "float", "value": 1.0}, "unit": unit});
        assert_eq!(rust_type(&num(Value::Null)), Some("f64"));
        assert_eq!(rust_type(&num(json!({"name": "ft", "factor": 0.3048, "gauge": 0.0, "dims": [1,0,0,0,0,0,0,0]}))), Some("Quantity"));
        assert_eq!(rust_type(&json!({"type": "array", "shape": [3], "values": [1,2,3], "unit": null})), Some("Vec<f64>"));
        assert_eq!(rust_type(&json!({"type": "frame", "columns": []})), None);
    }

    #[test]
    fn generated_code_puts_the_cell_at_a_known_line() {
        let mut inputs = serde_json::Map::new();
        inputs.insert("w".into(), json!({"type": "number", "value": {"type": "float", "value": 1.0}, "unit": null}));
        let (code, start) = generate(&Request { cell: "R".into(), source: "let y = w * 2.0;".into(), defines: vec!["y".into()], inputs }).unwrap();
        assert_eq!(code.lines().nth(start - 1), Some("let y = w * 2.0;"));
        assert!(code.contains("let w: f64 = __in.get(\"w\")?;"));
        assert!(code.contains("__out.put(\"y\", (&&Wrap(&y)).out());"));
    }

    #[test]
    fn panics_are_found_at_cell_lines() {
        let err = "thread '<unnamed>' panicked at src/lib.rs:12:5:\n+: length and mass have different dimensions\nnote: …";
        assert_eq!(panic_of(err, 10), (Some("+: length and mass have different dimensions".into()), Some(3)));
    }
}
