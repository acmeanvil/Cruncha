// SPDX-License-Identifier: GPL-3.0-or-later
//! Cruncha's desktop shell (ADR-0008): a Tauri backend that holds the open document's
//! `Store`, with a web UI in `app/src`: editing (commit on submit or idle), evaluation (calcs
//! here, Python cells in a worker, ADR-0005), history, snapshots and retention (ADR-0011).
//!
//! Events to the window: `results` (fresh results of the latest version, computed in the
//! background, `session::ResultsEvent`) and `python-setup` (environment setup progress).
//! Reports (spike S6): `export_report` typesets the latest version, a past version or a
//! snapshot to PDF.
//! Agents (ADR-0009): `agent_server_start` serves the open document to agents on this
//! computer through `cruncha-mcp` (127.0.0.1, with a token), until `agent_server_stop`.
//! Their proposals (ADR-0016) are listed by `proposals` and decided by `accept_proposal` /
//! `reject_proposal`; the `proposals` event says a document's proposals changed.

mod agent_panel;
mod agents;
mod connection;
mod engine;
mod figures;
mod login_path;
mod proposals;
mod python;
mod report;
mod secrets;
mod session;
mod stored;

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
    app: AppHandle,
    s: State<'_, Shared>,
    a: State<'_, AgentServer>,
) -> Result<agents::AgentServerView> {
    let mut slot = a.0.lock().await;
    if let Some(server) = slot.as_ref() {
        return Ok(server.into());
    }
    let server = cruncha_mcp::http::serve(agents::AppBackend(s.inner().clone(), "ai:external-agent".into()))
        .await
        .map_err(|e| format!("couldn't start the agent server: {e}"))?;
    // The connection file, so clients on this computer find the server without copy and paste.
    if let Ok(data) = app.path().app_data_dir() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as i64);
        let c = connection::Connection::new(&server.url, &server.token, report::iso_utc(now));
        if let Err(e) = connection::write(&connection::path(&data), &c) {
            eprintln!("cruncha: couldn't write the agent connection file: {e}");
        }
    }
    let view = (&server).into();
    *slot = Some(server);
    Ok(view)
}

/// The in-app agent panel for the open document.
type PanelState = std::sync::Arc<Mutex<agent_panel::Panel>>;

/// Cruncha's skills: `CRUNCHA_SKILLS_DIR`, the bundled resources, or the repository's `skills/`.
fn skills_dir(app: &AppHandle) -> PathBuf {
    if let Some(d) = std::env::var_os("CRUNCHA_SKILLS_DIR") {
        return d.into();
    }
    if let Ok(res) = app.path().resource_dir() {
        let d = res.join("skills");
        if d.join("cruncha-core").is_dir() {
            return d;
        }
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../skills"))
}

#[tauri::command]
fn agent_catalog() -> Vec<agent_panel::AgentDef> {
    agent_panel::catalog()
}

#[tauri::command]
async fn agent_panel_view(p: State<'_, PanelState>) -> Result<agent_panel::PanelView> {
    Ok(p.lock().await.view())
}

/// Start an agent on the open document, in the panel.
#[tauri::command]
async fn agent_start(
    agent_id: String,
    app: AppHandle,
    s: State<'_, Shared>,
    p: State<'_, PanelState>,
) -> Result<agent_panel::PanelView> {
    let agent = agent_panel::catalog()
        .into_iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| format!("unknown agent {agent_id}"))?;
    let (doc_id, version, settings) = {
        let s = s.lock().await;
        let info = s.info().await?;
        (info.id, info.version, s.prefs().agents.clone())
    };
    // The agent's settings: provider and model for the built-in agent, an API key from the
    // Keychain for one set to run on a connection.
    let agent = tauri::async_runtime::spawn_blocking(move || agent_panel::launch_def(agent, &settings))
        .await
        .map_err(|e| e.to_string())?;
    let server = cruncha_mcp::http::serve(agents::AppBackend(s.inner().clone(), format!("ai:{}", agent.id)))
        .await
        .map_err(|e| format!("couldn't start the agent server: {e}"))?;
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let panel = p.inner().clone();
    let (generation, events) = panel
        .lock()
        .await
        .start(&agent, &doc_id, version, &data, &skills_dir(&app), server)?;
    let handle = app.clone();
    tauri::async_runtime::spawn(agent_panel::pump(panel.clone(), generation, events, move |view| {
        let _ = handle.emit("agent-panel", view);
    }));
    Ok(panel.lock().await.view())
}

#[tauri::command]
async fn agent_prompt(text: String, p: State<'_, PanelState>) -> Result<agent_panel::PanelView> {
    let mut p = p.lock().await;
    p.prompt(text)?;
    Ok(p.view())
}

#[tauri::command]
async fn agent_cancel(p: State<'_, PanelState>) -> Result<agent_panel::PanelView> {
    let mut p = p.lock().await;
    p.cancel();
    Ok(p.view())
}

#[tauri::command]
async fn agent_stop(p: State<'_, PanelState>) -> Result<agent_panel::PanelView> {
    let mut p = p.lock().await;
    p.stop();
    Ok(p.view())
}

#[tauri::command]
async fn agent_set_option(id: String, value: String, p: State<'_, PanelState>) -> Result<()> {
    p.lock().await.set_option(id, value);
    Ok(())
}

/// Answer a permission prompt with an option id, or `None` to cancel it.
#[tauri::command]
async fn agent_permission(
    id: u64,
    option: Option<String>,
    p: State<'_, PanelState>,
) -> Result<agent_panel::PanelView> {
    let mut p = p.lock().await;
    p.answer(id, option);
    Ok(p.view())
}

#[tauri::command]
async fn agent_conversations(
    app: AppHandle,
    s: State<'_, Shared>,
) -> Result<Vec<agent_panel::ConversationSummary>> {
    let doc_id = s.lock().await.info().await?.id;
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(agent_panel::conversations(&data, &doc_id))
}

#[tauri::command]
async fn agent_conversation(
    id: String,
    app: AppHandle,
    s: State<'_, Shared>,
) -> Result<serde_json::Value> {
    let doc_id = s.lock().await.info().await?.id;
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    agent_panel::conversation(&data, &doc_id, &id)
}

#[tauri::command]
async fn agent_server_stop(app: AppHandle, a: State<'_, AgentServer>) -> Result<()> {
    if let Some(server) = a.0.lock().await.take() {
        stop_server(&app, &server);
    }
    Ok(())
}

/// Stop the agent server and remove its connection file.
fn stop_server(app: &AppHandle, server: &cruncha_mcp::http::Server) {
    server.stop();
    if let Ok(data) = app.path().app_data_dir() {
        connection::remove(&connection::path(&data), &server.token);
    }
}
type Result<T> = session::Result<T>;

#[tauri::command]
async fn open_document(path: PathBuf, s: State<'_, Shared>, p: State<'_, PanelState>) -> Result<DocInfo> {
    // An agent works on one document: a different one stops it.
    p.lock().await.stop();
    s.lock().await.open(&path).await
}

#[tauri::command]
async fn create_document(path: PathBuf, s: State<'_, Shared>, p: State<'_, PanelState>) -> Result<DocInfo> {
    // An agent works on one document: a different one stops it.
    p.lock().await.stop();
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
async fn create_sample_document(path: PathBuf, s: State<'_, Shared>, p: State<'_, PanelState>) -> Result<DocInfo> {
    // An agent works on one document: a different one stops it.
    p.lock().await.stop();
    s.lock().await.create_sample(&path).await
}

#[tauri::command]
async fn close_document(
    s: State<'_, Shared>,
    p: State<'_, PanelState>,
) -> Result<Option<PruneView>> {
    p.lock().await.stop();
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
    prefs.trusted_paths = s.prefs().trusted_paths.clone();
    // Connections are changed only by their own commands (their keys live in the Keychain).
    prefs.agents.connections = s.prefs().agents.connections.clone();
    s.set_prefs(prefs)
}

#[tauri::command]
async fn forget_trusted(id: String, s: State<'_, Shared>) -> Result<Prefs> {
    let mut s = s.lock().await;
    s.forget_trusted(&id)?;
    Ok(s.prefs().clone())
}

#[tauri::command]
async fn python_env(s: State<'_, Shared>) -> Result<session::PythonEnv> {
    Ok(s.lock().await.python_env())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderView {
    id: &'static str,
    label: &'static str,
    /// The API key's environment variable (empty: no key needed).
    key_var: &'static str,
}

#[tauri::command]
fn providers() -> Vec<ProviderView> {
    agent_panel::PROVIDERS
        .iter()
        .map(|&(id, label, key_var)| ProviderView { id, label, key_var })
        .collect()
}

/// Where each connection's key comes from. Reads the Keychain (which may ask the user once).
#[tauri::command]
async fn connection_keys(s: State<'_, Shared>) -> Result<HashMap<String, secrets::KeySource>> {
    let conns = s.lock().await.prefs().agents.connections.clone();
    tauri::async_runtime::spawn_blocking(move || {
        conns
            .iter()
            .map(|c| (c.id.clone(), secrets::source(agent_panel::provider_key_var(&c.provider), &c.key_account())))
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

/// Add a provider account; its key (if given) goes to the Keychain, never to a file.
#[tauri::command]
async fn connection_add(
    provider: String,
    name: String,
    base_url: String,
    key: String,
    s: State<'_, Shared>,
) -> Result<Prefs> {
    if !agent_panel::PROVIDERS.iter().any(|p| p.0 == provider) {
        return Err(format!("unknown provider {provider}"));
    }
    let mut s = s.lock().await;
    let mut prefs = s.prefs().clone();
    let taken = |id: &str| prefs.agents.connections.iter().any(|c| c.id == id);
    let mut id = provider.clone();
    let mut n = 2;
    while taken(&id) {
        id = format!("{provider}-{n}");
        n += 1;
    }
    let label = agent_panel::PROVIDERS.iter().find(|p| p.0 == provider).map(|p| p.1).unwrap_or_default();
    let conn = agent_panel::Connection {
        id,
        provider,
        name: if name.trim().is_empty() { label.to_owned() } else { name.trim().to_owned() },
        base_url: base_url.trim().to_owned(),
    };
    if !key.trim().is_empty() {
        let (account, key) = (conn.key_account(), key.clone());
        tauri::async_runtime::spawn_blocking(move || secrets::set(&account, &key))
            .await
            .map_err(|e| e.to_string())??;
    }
    prefs.agents.connections.push(conn);
    s.set_prefs(prefs)?;
    Ok(s.prefs().clone())
}

/// Replace a connection's key in the Keychain.
#[tauri::command]
async fn connection_set_key(id: String, key: String, s: State<'_, Shared>) -> Result<()> {
    let conn = s
        .lock()
        .await
        .prefs()
        .agents
        .connections
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| format!("no connection {id}"))?;
    tauri::async_runtime::spawn_blocking(move || secrets::set(&conn.key_account(), &key))
        .await
        .map_err(|e| e.to_string())?
}

/// Remove a connection and its key; agents that used it go back to their own login.
#[tauri::command]
async fn connection_remove(id: String, s: State<'_, Shared>) -> Result<Prefs> {
    let mut s = s.lock().await;
    let mut prefs = s.prefs().clone();
    let Some(conn) = prefs.agents.connections.iter().find(|c| c.id == id).cloned() else {
        return Ok(prefs);
    };
    tauri::async_runtime::spawn_blocking(move || secrets::delete(&conn.key_account()))
        .await
        .map_err(|e| e.to_string())??;
    prefs.agents.connections.retain(|c| c.id != id);
    for p in prefs.agents.agents.values_mut() {
        if p.connection.as_deref() == Some(id.as_str()) {
            p.connection = None;
        }
    }
    s.set_prefs(prefs)?;
    Ok(s.prefs().clone())
}

/// Start an agent briefly to list its models, and remember them for Settings.
#[tauri::command]
async fn agent_models(agent_id: String, app: AppHandle, s: State<'_, Shared>) -> Result<Prefs> {
    let def = agent_panel::catalog()
        .into_iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| format!("unknown agent {agent_id}"))?;
    let settings = s.lock().await.prefs().agents.clone();
    let def = tauri::async_runtime::spawn_blocking(move || agent_panel::launch_def(def, &settings))
        .await
        .map_err(|e| e.to_string())?;
    let cwd = app.path().app_data_dir().map_err(|e| e.to_string())?.join("agents").join("probe");
    let models = agent_panel::probe_models(&def, &cwd, std::time::Duration::from_secs(90)).await?;
    let mut s = s.lock().await;
    let mut prefs = s.prefs().clone();
    prefs.agents.agents.entry(agent_id).or_default().known_models = models;
    s.set_prefs(prefs)?;
    Ok(s.prefs().clone())
}

#[tauri::command]
async fn document_settings(s: State<'_, Shared>) -> Result<session::DocumentSettings> {
    s.lock().await.document_settings().await
}

#[tauri::command]
async fn set_document_settings(settings: session::DocumentSettings, s: State<'_, Shared>) -> Result<CommitView> {
    s.lock().await.set_document_settings(settings).await
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

/// Set up Rust for Rust cells (ADR-0006): Cruncha's own toolchain, then a first build of the
/// prelude (a minute or two; progress as `rust-setup` events). The session isn't locked meanwhile.
/// The calc language's function reference (`cruncha_eval::REFERENCE`).
#[tauri::command]
fn function_reference() -> Result<serde_json::Value> {
    serde_json::from_str(cruncha_eval::REFERENCE).map_err(|e| e.to_string())
}

#[tauri::command]
async fn setup_rust(app: AppHandle, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    let base = s.lock().await.rust_base();
    let progress = |step: cruncha_kernel_rs::env::Step| {
        let _ = app.emit("rust-setup", step);
    };
    cruncha_kernel_rs::env::setup(&base, progress).await?;
    let mut s = s.lock().await;
    if !s.is_open() {
        return Ok(HashMap::new());
    }
    s.rust_ready().await
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
    /// Figures printed as FIGURE NOT RENDERED (the app hadn't drawn them; ADR-0017 §4).
    figures_missing: usize,
}

/// Typeset a version to a PDF at `path`: the latest (`at` absent), a past version or a
/// snapshot. For the latest, waits (up to two minutes) for Python cells that are due to run,
/// so the report shows their results; a past version uses its stored results and runs
/// nothing. Whatever isn't computed is printed as NOT CALCULATED.
#[tauri::command]
async fn export_report(path: PathBuf, at: Option<At>, s: State<'_, Shared>) -> Result<ExportView> {
    let start = std::time::Instant::now();
    let mut timed_out = false;
    let at = at.unwrap_or_default();
    let latest = at.ref_name.is_none() && at.version.is_none();
    if latest {
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
    }
    let (report, files) = s.lock().await.report_at(&at).await?;
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
    let figures_missing = report
        .blocks
        .iter()
        .flat_map(|b| match b {
            cruncha_report::Block::Code { outputs, .. } => outputs.as_slice(),
            _ => &[],
        })
        .filter(|o| matches!(o, cruncha_report::Output::Figure { path: None, .. }))
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
        figures_missing,
    })
}

/// The figures a report of `at` needs drawn first (ADR-0017 §4): the front end draws each in
/// a figure frame and hands back its SVG with `put_figure`.
#[tauri::command]
async fn figures_to_render(at: Option<At>, s: State<'_, Shared>) -> Result<Vec<figures::FigureJob>> {
    s.lock().await.figures_to_render(&at.unwrap_or_default()).await
}

#[tauri::command]
async fn put_figure(key: String, svg: String, s: State<'_, Shared>) -> Result<()> {
    s.lock().await.put_figure(&key, &svg)
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

/// The open document's proposals (ADR-0016), newest first, with "if accepted" results.
#[tauri::command]
async fn proposals(s: State<'_, Shared>) -> Result<Vec<proposals::ProposalView>> {
    s.lock().await.proposals().await
}

/// Accept a proposal: one commit by the user. `force` accepts a stale one anyway.
#[tauri::command]
async fn accept_proposal(id: String, force: bool, s: State<'_, Shared>) -> Result<CommitView> {
    s.lock().await.accept_proposal(&id, force).await
}

#[tauri::command]
async fn reject_proposal(id: String, note: String, s: State<'_, Shared>) -> Result<proposals::ProposalView> {
    s.lock().await.reject_proposal(&id, &note).await
}

#[tauri::command]
async fn prune_now(s: State<'_, Shared>) -> Result<Option<PruneView>> {
    s.lock().await.prune_now().await
}

/// Drop the open document's stored code-cell results (ADR-0018).
#[tauri::command]
async fn clear_results(s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.clear_results().await
}

pub fn run() {
    // Opened from Finder: take the login shell's PATH, so agents and tools are found.
    login_path::adopt_login_path();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Scan fonts for reports in the background, so the first export is quick.
            std::thread::spawn(cruncha_report::warm_up);
            let prefs = app.path().app_config_dir()?.join("preferences.json");
            let python = app.path().app_data_dir()?.join("python");
            let mut session = Session::new(Some(prefs))
                .with_python(python, worker_dir(app))
                .with_rust(app.path().app_data_dir()?.join("rust"))
                .with_figure_cache(app.path().app_cache_dir()?.join("figures"));
            let handle = app.handle().clone();
            session.on_results(move |ev| {
                let _ = handle.emit("results", ev);
                // Keep finished runs with the document (ADR-0018), off the run loop.
                if let Some(shared) = handle.try_state::<Shared>() {
                    let shared = shared.inner().clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = shared.lock().await.flush_results().await {
                            eprintln!("cruncha: storing results failed: {e}");
                        }
                    });
                }
            });
            let handle = app.handle().clone();
            session.on_proposals(move |doc_id| {
                let _ = handle.emit("proposals", doc_id);
            });
            // For review (`scripts/run-app.sh --open <doc>`): start with that document open.
            if let Some(path) = std::env::var_os("CRUNCHA_OPEN")
                && let Err(e) = tauri::async_runtime::block_on(session.open(std::path::Path::new(&path)))
            {
                eprintln!("cruncha: couldn't open {}: {e}", PathBuf::from(&path).display());
            }
            app.manage::<Shared>(std::sync::Arc::new(Mutex::new(session)));
            app.manage(AgentServer::default());
            app.manage::<PanelState>(Default::default());
            Ok(())
        })
        // Figure frames (ADR-0020): their own origin and CSP, sandboxed by the page.
        .register_uri_scheme_protocol("cruncha-figure", |_ctx, req| {
            let (status, mime, body) = figures::frame_file(req.uri().path());
            tauri::http::Response::builder()
                .status(status)
                .header("Content-Type", mime)
                .header("Content-Security-Policy", figures::FRAME_CSP)
                .header("X-Content-Type-Options", "nosniff")
                .body(body.to_vec())
                .unwrap()
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
            forget_trusted,
            python_env,
            providers,
            connection_keys,
            connection_add,
            connection_set_key,
            connection_remove,
            document_settings,
            set_document_settings,
            agent_models,
            set_document_retention,
            retention_preview,
            prune_now,
            clear_results,
            proposals,
            accept_proposal,
            reject_proposal,
            python_status,
            setup_python,
            trust_document,
            setup_rust,
            function_reference,
            run_python,
            stop_python,
            restart_python,
            report_settings,
            set_report_settings,
            export_report,
            figures_to_render,
            put_figure,
            open_path,
            agent_server_status,
            agent_server_start,
            agent_server_stop,
            agent_catalog,
            agent_panel_view,
            agent_start,
            agent_prompt,
            agent_cancel,
            agent_stop,
            agent_set_option,
            agent_permission,
            agent_conversations,
            agent_conversation,
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
                        stop_server(app, &server);
                    }
                    s.lock().await.close().await;
                });
            }
        });
}
