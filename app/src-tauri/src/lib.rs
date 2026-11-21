// SPDX-License-Identifier: GPL-3.0-or-later
//! Cruncha's desktop shell (ADR-0008): a Tauri backend that holds the open document's
//! `Store`, with a web UI in `app/src`: editing (commit on submit or idle), evaluation (calcs
//! here, Python cells in a worker, ADR-0005), history, snapshots and retention (ADR-0011).
//!
//! Events to the window: `results` (fresh results of the latest version, computed in the
//! background, `session::ResultsEvent`) and `python-setup` (environment setup progress).
//! Reports (spike S6): `export_report` typesets the latest version to PDF.
//! Agents (ADR-0009): `agent_server_start` serves the open document to agents on this
//! computer through `cruncha-mcp` (127.0.0.1, with a token), until `agent_server_stop`.

mod agents;
mod engine;
mod python;
mod report;
mod session;

use std::path::PathBuf;

use cruncha_store::{Cell, Retention};
use std::collections::HashMap;

use engine::CellResult;
use session::{
    At, ChangeView, CommitView, DocInfo, Prefs, PreviewView, PruneView, PythonStatus, Session,
    VersionView,
};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};
use tokio::sync::Mutex;

type Shared = std::sync::Arc<Mutex<Session>>;

/// The MCP server for agents, while the user allows it.
#[derive(Default)]
struct AgentServer(Mutex<Option<cruncha_mcp::http::Server>>);

#[tauri::command]
async fn agent_server_status(a: State<'_, AgentServer>) -> Result<Option<agents::AgentServerView>> {
    Ok(a.0.lock().await.as_ref().map(Into::into))
}

/// Allow agents on this computer to use the open document (read, results, checks, history).
#[tauri::command]
async fn agent_server_start(
    s: State<'_, Shared>,
    a: State<'_, AgentServer>,
) -> Result<agents::AgentServerView> {
    let mut slot = a.0.lock().await;
    if let Some(server) = slot.as_ref() {
        return Ok(server.into());
    }
    let server = cruncha_mcp::http::serve(agents::AppBackend(s.inner().clone()))
        .await
        .map_err(|e| format!("couldn't start the agent server: {e}"))?;
    let view = (&server).into();
    *slot = Some(server);
    Ok(view)
}

#[tauri::command]
async fn agent_server_stop(a: State<'_, AgentServer>) -> Result<()> {
    if let Some(server) = a.0.lock().await.take() {
        server.stop();
    }
    Ok(())
}
type Result<T> = session::Result<T>;

#[tauri::command]
async fn open_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.open(&path).await
}

#[tauri::command]
async fn create_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.create(&path).await
}

#[tauri::command]
async fn commit(
    changes: Vec<ChangeView>,
    message: String,
    s: State<'_, Shared>,
) -> Result<CommitView> {
    s.lock().await.commit(changes, &message).await
}

#[tauri::command]
async fn results(at: At, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    s.lock().await.results(&at).await
}

#[tauri::command]
async fn snapshot(name: String, note: String, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.snapshot(&name, &note).await
}

#[tauri::command]
async fn restore(at: At, s: State<'_, Shared>) -> Result<CommitView> {
    s.lock().await.restore(&at).await
}

#[tauri::command]
async fn preview_calc(source: String, s: State<'_, Shared>) -> Result<CellResult> {
    s.lock().await.preview_calc(&source)
}

#[tauri::command]
async fn create_sample_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.create_sample(&path).await
}

#[tauri::command]
async fn close_document(s: State<'_, Shared>) -> Result<Option<PruneView>> {
    Ok(s.lock().await.close().await)
}

#[tauri::command]
async fn document_info(s: State<'_, Shared>) -> Result<Option<DocInfo>> {
    let s = s.lock().await;
    if !s.is_open() {
        return Ok(None);
    }
    s.info().await.map(Some)
}

#[tauri::command]
async fn cells(version: Option<u64>, s: State<'_, Shared>) -> Result<Vec<Cell>> {
    s.lock().await.cells(version).await
}

#[tauri::command]
async fn cells_at_ref(name: String, s: State<'_, Shared>) -> Result<Vec<Cell>> {
    s.lock().await.cells_at_ref(&name).await
}

#[tauri::command]
async fn history(s: State<'_, Shared>) -> Result<Vec<VersionView>> {
    s.lock().await.history().await
}

#[tauri::command]
async fn get_prefs(s: State<'_, Shared>) -> Result<Prefs> {
    Ok(s.lock().await.prefs().clone())
}

#[tauri::command]
async fn forget_recent(path: String, s: State<'_, Shared>) -> Result<()> {
    s.lock().await.forget_recent(&path)
}

#[tauri::command]
async fn set_prefs(mut prefs: Prefs, s: State<'_, Shared>) -> Result<()> {
    let mut s = s.lock().await;
    // The recent and trusted lists are the backend's; the front end's copy may be stale.
    prefs.recent = s.prefs().recent.clone();
    prefs.trusted = s.prefs().trusted.clone();
    s.set_prefs(prefs)
}

#[tauri::command]
async fn set_document_retention(
    policy: Option<Retention>,
    s: State<'_, Shared>,
) -> Result<DocInfo> {
    s.lock().await.set_document_retention(policy).await
}

#[tauri::command]
async fn retention_preview(policy: Option<Retention>, s: State<'_, Shared>) -> Result<PreviewView> {
    s.lock().await.retention_preview(policy).await
}

#[tauri::command]
async fn python_status(s: State<'_, Shared>) -> Result<Option<PythonStatus>> {
    let s = s.lock().await;
    if !s.is_open() {
        return Ok(None);
    }
    s.python_status_full().await.map(Some)
}

/// Set up the Python environment (minutes on first use; progress as `python-setup` events).
/// The session isn't locked meanwhile, so editing goes on.
#[tauri::command]
async fn setup_python(app: AppHandle, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    let base = s.lock().await.python_base();
    let progress = |p: cruncha_kernel_py::env::Progress| {
        let _ = app.emit("python-setup", p);
    };
    cruncha_kernel_py::env::setup(&base, python::LOCK, progress).await?;
    let mut s = s.lock().await;
    if !s.is_open() {
        return Ok(HashMap::new());
    }
    s.python_ready().await
}

#[tauri::command]
async fn trust_document(s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    s.lock().await.trust_document().await
}

#[tauri::command]
async fn run_python(
    ids: Option<Vec<String>>,
    s: State<'_, Shared>,
) -> Result<HashMap<String, CellResult>> {
    s.lock().await.run_python(ids).await
}

#[tauri::command]
async fn stop_python(s: State<'_, Shared>) -> Result<()> {
    s.lock().await.stop_python()
}

#[tauri::command]
async fn restart_python(s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    s.lock().await.restart_python().await
}

/// The `cruncha_worker` package: `CRUNCHA_WORKER_DIR`, the app's bundled resources, or the
/// repository's `python/` (development builds).
fn worker_dir(app: &tauri::App) -> PathBuf {
    if let Some(d) = std::env::var_os("CRUNCHA_WORKER_DIR") {
        return d.into();
    }
    if let Ok(res) = app.path().resource_dir() {
        let d = res.join("python");
        if d.join("cruncha_worker").is_dir() {
            return d;
        }
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../python"))
}

#[tauri::command]
async fn report_settings(s: State<'_, Shared>) -> Result<report::ReportSettings> {
    s.lock().await.report_settings().await
}

#[tauri::command]
async fn set_report_settings(
    settings: report::ReportSettings,
    s: State<'_, Shared>,
) -> Result<CommitView> {
    s.lock().await.set_report_settings(settings).await
}

/// What an export produced.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportView {
    path: String,
    pages: usize,
    bytes: usize,
    ms: f64,
    /// Checks that fail, errors, and cells not calculated (the cover's summary).
    problems: usize,
    /// Python cells were still running when the wait ran out.
    timed_out: bool,
}

/// Typeset the latest version to a PDF at `path`. Waits (up to two minutes) for Python cells
/// that are due to run, so the report shows their results; whatever still isn't computed is
/// printed as NOT CALCULATED.
#[tauri::command]
async fn export_report(path: PathBuf, s: State<'_, Shared>) -> Result<ExportView> {
    let start = std::time::Instant::now();
    let mut timed_out = false;
    loop {
        // The session lock is taken per check, so the window stays usable while Python runs.
        if !s.lock().await.python_busy().await? {
            break;
        }
        if start.elapsed() > std::time::Duration::from_secs(120) {
            timed_out = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    let (report, files) = s.lock().await.report().await?;
    let problems = report
        .blocks
        .iter()
        .filter(|b| match b {
            cruncha_report::Block::Calc { status, .. } | cruncha_report::Block::Code { status, .. } => {
                !matches!(status, cruncha_report::Status::Ok | cruncha_report::Status::Pass)
            }
            _ => false,
        })
        .count();
    let rendered = tokio::task::spawn_blocking(move || cruncha_report::render(&report, files))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    std::fs::write(&path, &rendered.pdf).map_err(|e| format!("couldn't write {}: {e}", path.display()))?;
    Ok(ExportView {
        path: path.display().to_string(),
        pages: rendered.pages,
        bytes: rendered.pdf.len(),
        ms: start.elapsed().as_secs_f64() * 1e3,
        problems,
        timed_out,
    })
}

/// Open a file with the system's default application (the exported PDF).
#[tauri::command]
fn open_path(path: PathBuf) -> Result<()> {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("couldn't open {}: {e}", path.display()))
}

#[tauri::command]
async fn prune_now(s: State<'_, Shared>) -> Result<Option<PruneView>> {
    s.lock().await.prune_now().await
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Scan fonts for reports in the background, so the first export is quick.
            std::thread::spawn(cruncha_report::warm_up);
            let prefs = app.path().app_config_dir()?.join("preferences.json");
            let python = app.path().app_data_dir()?.join("python");
            let mut session = Session::new(Some(prefs)).with_python(python, worker_dir(app));
            let handle = app.handle().clone();
            session.on_results(move |ev| {
                let _ = handle.emit("results", ev);
            });
            app.manage::<Shared>(std::sync::Arc::new(Mutex::new(session)));
            app.manage(AgentServer::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_document,
            create_document,
            create_sample_document,
            commit,
            results,
            preview_calc,
            snapshot,
            restore,
            close_document,
            document_info,
            cells,
            cells_at_ref,
            history,
            get_prefs,
            set_prefs,
            forget_recent,
            set_document_retention,
            retention_preview,
            prune_now,
            python_status,
            setup_python,
            trust_document,
            run_python,
            stop_python,
            restart_python,
            report_settings,
            set_report_settings,
            export_report,
            open_path,
            agent_server_status,
            agent_server_start,
            agent_server_stop,
        ])
        .build(tauri::generate_context!())
        .expect("failed to start Cruncha")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                // Release the document (and apply its retention policy if this session edited
                // it) before the process ends.
                let s = app.state::<Shared>();
                let a = app.state::<AgentServer>();
                tauri::async_runtime::block_on(async {
                    if let Some(server) = a.0.lock().await.take() {
                        server.stop();
                    }
                    s.lock().await.close().await;
                });
            }
        });
}
