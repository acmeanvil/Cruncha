// SPDX-License-Identifier: GPL-3.0-or-later
//! Cruncha's Python worker, from the app's side (ADR-0005, design 02 §3.3).
//!
//! A [`Kernel`] is one worker process (`python/cruncha_worker`) running a document's Python
//! cells. It answers one request at a time over its stdin and stdout ([`protocol`]). A cell
//! that runs too long, or that the user stops, is interrupted with SIGINT (the worker reports
//! `interrupted` and keeps its state); a worker that doesn't respond to that is killed.
//!
//! Values cross as JSON in cruncha-eval's exchange form (`Plain`), kept opaque here as
//! [`serde_json::Value`] so this crate doesn't depend on the evaluator. Images come back as
//! binary blobs. [`env`] sets up the interpreter and packages.

pub mod env;
pub mod sandbox;
pub mod protocol;

use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The worker couldn't be started.
    Start(String),
    /// The worker died or broke the protocol; it must be restarted.
    Died(String),
    /// The worker answered with an error.
    Worker(String),
    /// A cell didn't stop after an interrupt and the worker was killed.
    Killed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Start(e) => write!(f, "couldn't start Python: {e}"),
            Error::Died(e) => write!(f, "the Python worker stopped: {e}"),
            Error::Worker(e) => write!(f, "{e}"),
            Error::Killed => write!(
                f,
                "the Python worker didn't respond to stop and was restarted"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// How to start a worker.
#[derive(Debug, Clone)]
pub struct Config {
    /// The interpreter (see [`env::setup`]).
    pub python: PathBuf,
    /// The directory holding the `cruncha_worker` and `cruncha` packages (the repo's `python/`
    /// in development, bundled resources in a release).
    pub worker_dir: PathBuf,
    /// Grace period between an interrupt and a kill.
    pub kill_after: Duration,
    /// Run the worker in this sandbox (macOS; see [`sandbox`]). `None`: unsandboxed.
    pub sandbox: Option<sandbox::Sandbox>,
}

impl Config {
    pub fn new(python: impl Into<PathBuf>, worker_dir: impl Into<PathBuf>) -> Config {
        Config {
            python: python.into(),
            worker_dir: worker_dir.into(),
            kill_after: Duration::from_secs(3),
            sandbox: None,
        }
    }
}

/// What the worker reports on start.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Hello {
    pub protocol: u32,
    pub python: String,
    pub executable: String,
    pub pid: u32,
}

/// An error in a cell (or in its source).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PyError {
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
    /// 1-based line in the cell's source.
    pub line: Option<u32>,
    #[serde(default)]
    pub traceback: String,
}

/// What a cell defines and reads (the worker's `ast` scan), or its syntax error.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Scan {
    #[serde(default)]
    pub defines: Vec<String>,
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub error: Option<PyError>,
}

/// Run one cell. Inputs from calc cells are sent as values; inputs defined by other Python
/// cells are already in the worker and are named in `python_inputs`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RunRequest {
    pub cell: String,
    pub source: String,
    pub defines: Vec<String>,
    pub inputs: BTreeMap<String, Json>,
    pub python_inputs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Done,
    Error,
    Interrupted,
}

/// One name a cell defined: a summary for display and, when it converts, its calc value.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Defined {
    pub name: String,
    pub summary: String,
    pub plain: Json,
}

/// Rich output of a cell.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Output {
    Image {
        mime: String,
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Table {
        columns: Vec<Json>,
        rows: Vec<Vec<Option<String>>>,
        total_rows: u64,
    },
    Text {
        text: String,
    },
    /// An interactive figure (ADR-0017): a JSON spec of an allowlisted MIME type, drawn by the
    /// app's sandboxed renderer. `ranges` holds each trace's data range; `static_image` is a
    /// PNG or SVG the cell supplied for reports.
    Figure {
        mime: String,
        spec: String,
        summary: String,
        ranges: Json,
        static_image: Option<(String, Vec<u8>)>,
    },
}

/// The figure formats the app can draw (ADR-0017). Anything else is dropped.
pub const FIGURE_MIMES: [&str; 2] = ["application/vnd.plotly.v1+json", "application/vnd.vegalite.v5+json"];

/// Largest figure spec accepted (the worker refuses bigger ones first).
pub const MAX_FIGURE_SPEC: usize = 8 << 20;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunResult {
    pub status: Status,
    pub defines: Vec<Defined>,
    pub stdout: String,
    pub stderr: String,
    pub outputs: Vec<Output>,
    pub error: Option<PyError>,
    pub ms: f64,
}

/// Interrupts a running cell from another task (the kernel itself is busy awaiting it).
#[derive(Debug, Clone, Copy)]
pub struct Interrupter {
    pid: u32,
}

impl Interrupter {
    /// Send SIGINT: the running cell raises `KeyboardInterrupt`.
    pub fn interrupt(&self) {
        #[cfg(unix)]
        // SAFETY: kill(2) with a pid we started; a stale pid only fails with ESRCH.
        unsafe {
            libc::kill(self.pid as libc::pid_t, libc::SIGINT);
        }
    }

    /// Kill the worker outright (it is restarted on next use).
    pub fn kill(&self) {
        #[cfg(unix)]
        // SAFETY: as above.
        unsafe {
            libc::kill(self.pid as libc::pid_t, libc::SIGKILL);
        }
    }
}

pub struct Kernel {
    config: Config,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    hello: Hello,
}

impl Kernel {
    /// Start a worker and wait for its greeting.
    pub async fn start(config: Config) -> Result<Kernel> {
        let boot = "import sys; sys.path.insert(0, sys.argv[1]); from cruncha_worker.main import main; main()";
        let mut cmd = match (&config.sandbox, sandbox::program()) {
            (Some(sb), Some(exec)) => {
                std::fs::create_dir_all(&sb.scratch)
                    .map_err(|e| Error::Start(format!("{}: {e}", sb.scratch.display())))?;
                let mut c = Command::new(exec);
                c.arg("-p").arg(sb.profile()).arg(&config.python);
                // Everything the worker writes goes to its scratch directory.
                c.env("HOME", &sb.scratch).env("TMPDIR", &sb.scratch).env("MPLCONFIGDIR", sb.scratch.join("matplotlib"));
                c.current_dir(&sb.scratch);
                c
            }
            _ => Command::new(&config.python),
        };
        // -I: isolated (no PYTHON* variables, user site or current directory on the path);
        // -B: no bytecode written next to the worker's sources.
        cmd.args(["-I", "-B", "-c", boot])
            .arg(&config.worker_dir)
            .env("MPLBACKEND", "Agg")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = cmd
            .spawn()
            .map_err(|e| Error::Start(format!("{}: {e}", config.python.display())))?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        let mut k = Kernel {
            config,
            child,
            stdin,
            stdout,
            next_id: 0,
            hello: Hello {
                protocol: 0,
                python: String::new(),
                executable: String::new(),
                pid: 0,
            },
        };
        let h = tokio::time::timeout(
            Duration::from_secs(30),
            k.request(json!({"op": "hello"}), vec![]),
        )
        .await
        .map_err(|_| Error::Start("the worker didn't answer within 30 s".into()))?
        .map_err(|e| Error::Start(e.to_string()))?
        .0;
        k.hello = serde_json::from_value(h).map_err(|e| Error::Start(e.to_string()))?;
        Ok(k)
    }

    pub fn hello(&self) -> &Hello {
        &self.hello
    }

    pub fn interrupter(&self) -> Interrupter {
        Interrupter {
            pid: self.child.id().unwrap_or(self.hello.pid),
        }
    }

    /// Send a request and wait for its answer.
    pub async fn request(
        &mut self,
        mut header: Json,
        blobs: Vec<Vec<u8>>,
    ) -> Result<(Json, Vec<Vec<u8>>)> {
        self.next_id += 1;
        let id = self.next_id;
        header["id"] = id.into();
        protocol::write_message(&mut self.stdin, &header, &blobs)
            .await
            .map_err(|e| Error::Died(e.to_string()))?;
        loop {
            let (h, blobs) = protocol::read_message(&mut self.stdout)
                .await
                .map_err(|e| Error::Died(e.to_string()))?;
            // Answers to an earlier, abandoned request (after a timeout) are skipped.
            if h.get("id").and_then(Json::as_u64) != Some(id) {
                continue;
            }
            if h.get("ok") != Some(&Json::Bool(true)) {
                return Err(Error::Worker(
                    h.get("error")
                        .and_then(Json::as_str)
                        .unwrap_or("unknown error")
                        .to_owned(),
                ));
            }
            return Ok((h, blobs));
        }
    }

    pub async fn scan(&mut self, source: &str) -> Result<Scan> {
        let (h, _) = self
            .request(json!({"op": "scan", "source": source}), vec![])
            .await?;
        serde_json::from_value(h).map_err(|e| Error::Died(e.to_string()))
    }

    /// `source` with these names renamed (old → new), by Python's tokenizer: strings,
    /// comments, attributes and keyword arguments are left alone (ADR-0024).
    pub async fn rename(&mut self, source: &str, names: &std::collections::BTreeMap<String, String>) -> Result<String> {
        let (h, _) = self
            .request(json!({"op": "rename", "source": source, "names": names}), vec![])
            .await?;
        h["source"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::Died("rename gave no source".into()))
    }

    /// Run a cell. With a `timeout`, the cell is interrupted when it expires (and the worker
    /// killed if it doesn't stop); the result then has status `interrupted`.
    pub async fn run(&mut self, req: &RunRequest, timeout: Option<Duration>) -> Result<RunResult> {
        let mut header = serde_json::to_value(req).expect("serializable");
        header["op"] = "run".into();
        let interrupter = self.interrupter();
        let kill_after = self.config.kill_after;
        let fut = self.request(header, vec![]);
        tokio::pin!(fut);
        let (h, blobs) = match timeout {
            None => fut.await?,
            Some(t) => match tokio::time::timeout(t, &mut fut).await {
                Ok(r) => r?,
                Err(_) => {
                    interrupter.interrupt();
                    match tokio::time::timeout(kill_after, &mut fut).await {
                        Ok(r) => r?,
                        Err(_) => {
                            interrupter.kill();
                            return Err(Error::Killed);
                        }
                    }
                }
            },
        };
        parse_run(h, blobs)
    }

    /// Forget names (their cells were deleted or now define something else).
    pub async fn drop_names(&mut self, names: &[String]) -> Result<()> {
        self.request(json!({"op": "drop", "names": names}), vec![])
            .await
            .map(|_| ())
    }

    /// Forget every value.
    pub async fn reset(&mut self) -> Result<()> {
        self.request(json!({"op": "reset"}), vec![])
            .await
            .map(|_| ())
    }

    /// Ask the worker to exit, killing it if it doesn't within the grace period.
    pub async fn shutdown(mut self) {
        let grace = self.config.kill_after;
        let _ = tokio::time::timeout(grace, self.request(json!({"op": "shutdown"}), vec![])).await;
        if tokio::time::timeout(grace, self.child.wait())
            .await
            .is_err()
        {
            let _ = self.child.start_kill();
        }
    }

    /// Is the process still running?
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

fn parse_run(h: Json, mut blobs: Vec<Vec<u8>>) -> Result<RunResult> {
    let bad = |e: &str| Error::Died(format!("malformed run result: {e}"));
    let status: Status =
        serde_json::from_value(h["status"].clone()).map_err(|e| bad(&e.to_string()))?;
    let defines: Vec<Defined> =
        serde_json::from_value(h.get("defines").cloned().unwrap_or(json!([])))
            .map_err(|e| bad(&e.to_string()))?;
    let error: Option<PyError> =
        serde_json::from_value(h.get("error").cloned().unwrap_or(Json::Null))
            .map_err(|e| bad(&e.to_string()))?;
    let mut outputs = Vec::new();
    for o in h
        .get("outputs")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
    {
        match o["kind"].as_str() {
            Some("image") => {
                let i = o["blob"]
                    .as_u64()
                    .ok_or_else(|| bad("image without blob"))? as usize;
                let data = blobs
                    .get_mut(i)
                    .map(std::mem::take)
                    .ok_or_else(|| bad("missing blob"))?;
                outputs.push(Output::Image {
                    mime: o["mime"].as_str().unwrap_or("image/png").to_owned(),
                    data,
                    width: o["width"].as_u64().unwrap_or(0) as u32,
                    height: o["height"].as_u64().unwrap_or(0) as u32,
                });
            }
            Some("table") => outputs.push(Output::Table {
                columns: o["columns"].as_array().cloned().unwrap_or_default(),
                rows: serde_json::from_value(o["rows"].clone()).map_err(|e| bad(&e.to_string()))?,
                total_rows: o["total_rows"].as_u64().unwrap_or(0),
            }),
            Some("text") => outputs.push(Output::Text {
                text: o["text"].as_str().unwrap_or("").to_owned(),
            }),
            Some("figure") => {
                let mime = o["mime"].as_str().unwrap_or_default();
                if !FIGURE_MIMES.contains(&mime) {
                    continue; // never shown: not an allowlisted figure type
                }
                let mut blob = |key: &str| -> Result<Option<Vec<u8>>> {
                    match o.get(key).and_then(Json::as_u64) {
                        None => Ok(None),
                        Some(i) => blobs.get_mut(i as usize).map(std::mem::take).map(Some).ok_or_else(|| bad("missing blob")),
                    }
                };
                let spec = blob("blob")?.ok_or_else(|| bad("figure without blob"))?;
                let static_image = blob("static")?.and_then(|data| {
                    let m = o["static_mime"].as_str().unwrap_or("image/png");
                    matches!(m, "image/png" | "image/svg+xml").then(|| (m.to_owned(), data))
                });
                if spec.len() > MAX_FIGURE_SPEC {
                    outputs.push(Output::Text { text: "(figure not shown: its spec is over the 8 MB limit)".into() });
                    continue;
                }
                let Ok(spec) = String::from_utf8(spec) else { return Err(bad("figure spec isn't UTF-8")) };
                if serde_json::from_str::<serde::de::IgnoredAny>(&spec).is_err() {
                    return Err(bad("figure spec isn't JSON"));
                }
                outputs.push(Output::Figure {
                    mime: mime.to_owned(),
                    spec,
                    summary: o["summary"].as_str().unwrap_or("").to_owned(),
                    ranges: o.get("ranges").cloned().unwrap_or(json!([])),
                    static_image,
                });
            }
            _ => {}
        }
    }
    let s = |k: &str| h.get(k).and_then(Json::as_str).unwrap_or("").to_owned();
    Ok(RunResult {
        status,
        defines,
        stdout: s("stdout"),
        stderr: s("stderr"),
        outputs,
        error,
        ms: h["ms"].as_f64().unwrap_or(0.0),
    })
}
