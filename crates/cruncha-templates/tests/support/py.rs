// SPDX-License-Identifier: GPL-3.0-or-later

//! A [`CodeRunner`] on the real Python worker (ADR-0005), for the library test and the
//! `check` example. It finds the environment as the kernel's tests do: `CRUNCHA_TEST_PYTHON`,
//! or the one the app set up. Without one, `PyRunner::start` returns `None`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use cruncha_eval::Plain;
use cruncha_kernel_py::{Config, Kernel, Output, RunRequest, Status, env};
use cruncha_templates::{CodeOutcome, CodeRunner};

const LOCK: &str = include_str!("../../../../python/requirements.lock");

pub struct PyRunner {
    rt: tokio::runtime::Runtime,
    kernel: Kernel,
    cells: usize,
}

fn python() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("CRUNCHA_TEST_PYTHON") {
        return Some(p.into());
    }
    let base = PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support/io.github.acmeanvil.cruncha/python");
    env::ready(&base, LOCK)
}

impl PyRunner {
    pub fn start() -> Option<PyRunner> {
        let py = python()?;
        let worker = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python");
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().ok()?;
        let kernel = rt.block_on(Kernel::start(Config::new(py, worker))).ok()?;
        Some(PyRunner { rt, kernel, cells: 0 })
    }
}

impl CodeRunner for PyRunner {
    fn reset(&mut self) -> Result<(), String> {
        self.rt.block_on(self.kernel.reset()).map_err(|e| e.to_string())
    }

    fn scan(&mut self, lang: &str, source: &str) -> Result<(Vec<String>, Vec<String>), String> {
        if lang != "python" {
            return Err(format!("{lang} cells aren't run by the template tests yet"));
        }
        let s = self.rt.block_on(self.kernel.scan(source)).map_err(|e| e.to_string())?;
        match s.error {
            Some(e) => Err(format!("{}: {}", e.kind, e.message)),
            None => Ok((s.defines, s.reads)),
        }
    }

    fn run(&mut self, _lang: &str, source: &str, defines: &[String], inputs: BTreeMap<String, Plain>, code_inputs: Vec<String>)
    -> Result<CodeOutcome, String> {
        self.cells += 1;
        let req = RunRequest {
            cell: format!("template-cell-{}", self.cells),
            source: source.to_owned(),
            defines: defines.to_vec(),
            inputs: inputs.into_iter().map(|(k, v)| (k, serde_json::to_value(v).unwrap_or_default())).collect(),
            python_inputs: code_inputs,
        };
        let r = self.rt.block_on(self.kernel.run(&req, Some(Duration::from_secs(60)))).map_err(|e| e.to_string())?;
        if r.status != Status::Done {
            let e = r.error.map(|e| format!("{}: {} (line {:?})", e.kind, e.message, e.line)).unwrap_or_default();
            return Err(format!("{:?}: {e}", r.status));
        }
        let mut defined = BTreeMap::new();
        for d in r.defines {
            if let Ok(p) = serde_json::from_value::<Plain>(d.plain) {
                defined.insert(d.name, p);
            }
        }
        let figures = r.outputs.iter().filter(|o| matches!(o, Output::Figure { .. })).count();
        Ok(CodeOutcome { defined, figures })
    }
}
