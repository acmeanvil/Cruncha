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
mod rename;
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
    let server = cruncha_mcp::http::serve(agents::AppBackend(s.inner().clone(), "ai:external-agent".into(), None))
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

/// How long a background tab's Python may sit idle before its workers stop (ADR-0024): 10
/// minutes, or `CRUNCHA_IDLE_WORKERS_SECS` seconds (for review).
fn idle_workers() -> std::time::Duration {
    let secs = std::env::var("CRUNCHA_IDLE_WORKERS_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(10 * 60);
    std::time::Duration::from_secs(secs)
}

/// The macOS menu: Tauri's default, except that ⌘W closes the tab (ADR-0024) and Close Window
/// moves to ⇧⌘W. The Edit menu stays, so copy and paste reach the page.
#[cfg(target_os = "macos")]
fn app_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
    let open = MenuItemBuilder::with_id("open", "Open…").accelerator("CmdOrCtrl+O").build(app)?;
    let close_tab = MenuItemBuilder::with_id("close_tab", "Close Tab").accelerator("CmdOrCtrl+W").build(app)?;
    let close_window = MenuItemBuilder::with_id("close_window", "Close Window")
        .accelerator("CmdOrCtrl+Shift+W")
        .build(app)?;
    let cruncha = SubmenuBuilder::new(app, "Cruncha")
        .about(None)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;
    let file = SubmenuBuilder::new(app, "File").item(&open).separator().item(&close_tab).build()?;
    let edit = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let view = SubmenuBuilder::new(app, "View").fullscreen().build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .separator()
        .item(&close_window)
        .build()?;
    MenuBuilder::new(app).items(&[&cruncha, &file, &edit, &view, &window]).build()
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
        let mut s = s.lock().await;
        s.select(None)?; // the active tab
        let info = s.info().await?;
        (info.id, info.version, s.prefs().agents.clone())
    };
    // The agent's settings: provider and model for the built-in agent, an API key from the
    // Keychain for one set to run on a connection.
    let agent = tauri::async_runtime::spawn_blocking(move || agent_panel::launch_def(agent, &settings))
        .await
        .map_err(|e| e.to_string())?;
    let server = cruncha_mcp::http::serve(agents::AppBackend(s.inner().clone(), format!("ai:{}", agent.id), Some(doc_id.clone())))
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
    let doc_id = s.lock().await.on(None)?.info().await?.id;
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(agent_panel::conversations(&data, &doc_id))
}

#[tauri::command]
async fn agent_conversation(
    id: String,
    app: AppHandle,
    s: State<'_, Shared>,
) -> Result<serde_json::Value> {
    let doc_id = s.lock().await.on(None)?.info().await?.id;
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
async fn open_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    // A new tab (or the document's tab, if it's open). An agent keeps its own document.
    s.lock().await.open(&path).await
}

/// Names that pasted cells define and the document already defines (ADR-0024).
#[tauri::command]
async fn paste_clashes(cells: Vec<session::SourceView>, doc: Option<String>, s: State<'_, Shared>) -> Result<Vec<session::ClashView>> {
    s.lock().await.on(doc.as_deref())?.paste_clashes(cells).await
}

/// The cells' sources with names renamed (old → new), each checked; refused if unsafe.
#[tauri::command]
async fn rename_cells(
    cells: Vec<session::SourceView>,
    names: std::collections::BTreeMap<String, String>,
    doc: Option<String>,
    s: State<'_, Shared>,
) -> Result<Vec<String>> {
    s.lock().await.on(doc.as_deref())?.rename_cells(cells, names).await
}

/// The open documents, in tab order (ADR-0024).
#[tauri::command]
async fn open_documents(s: State<'_, Shared>) -> Result<Vec<session::OpenDocView>> {
    Ok(s.lock().await.documents())
}

/// Switch to a tab.
#[tauri::command]
async fn activate_document(doc: String, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.activate(&doc).await
}

#[tauri::command]
async fn create_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.create(&path).await
}

#[tauri::command]
async fn commit(
    changes: Vec<ChangeView>,
    message: String,
    doc: Option<String>, s: State<'_, Shared>,
) -> Result<CommitView> {
    s.lock().await.on(doc.as_deref())?.commit(changes, &message).await
}

#[tauri::command]
async fn results(at: At, doc: Option<String>, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    s.lock().await.on(doc.as_deref())?.results(&at).await
}

#[tauri::command]
async fn snapshot(name: String, note: String, doc: Option<String>, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.on(doc.as_deref())?.snapshot(&name, &note).await
}

#[tauri::command]
async fn restore(at: At, doc: Option<String>, s: State<'_, Shared>) -> Result<CommitView> {
    s.lock().await.on(doc.as_deref())?.restore(&at).await
}

#[tauri::command]
async fn preview_calc(source: String, doc: Option<String>, s: State<'_, Shared>) -> Result<CellResult> {
    s.lock().await.on(doc.as_deref())?.preview_calc(&source)
}

#[tauri::command]
async fn create_sample_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.create_sample(&path).await
}

#[tauri::command]
async fn close_document(
    doc: Option<String>, s: State<'_, Shared>,
    p: State<'_, PanelState>,
) -> Result<Option<PruneView>> {
    let mut s = s.lock().await;
    s.select(doc.as_deref())?;
    // The agent working on this document stops with it.
    let closing = s.info().await.ok().map(|i| i.id);
    let mut panel = p.lock().await;
    if closing.is_some() && panel.doc_id() == closing {
        panel.stop();
    }
    drop(panel);
    Ok(s.close().await)
}

#[tauri::command]
async fn document_info(doc: Option<String>, s: State<'_, Shared>) -> Result<Option<DocInfo>> {
    let mut s = s.lock().await;
    s.select(doc.as_deref())?;
    if !s.is_open() {
        return Ok(None);
    }
    s.info().await.map(Some)
}

#[tauri::command]
async fn cells(version: Option<u64>, doc: Option<String>, s: State<'_, Shared>) -> Result<Vec<Cell>> {
    s.lock().await.on(doc.as_deref())?.cells(version).await
}

#[tauri::command]
async fn cells_at_ref(name: String, doc: Option<String>, s: State<'_, Shared>) -> Result<Vec<Cell>> {
    s.lock().await.on(doc.as_deref())?.cells_at_ref(&name).await
}

#[tauri::command]
async fn history(doc: Option<String>, s: State<'_, Shared>) -> Result<Vec<VersionView>> {
    s.lock().await.on(doc.as_deref())?.history().await
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
    prefs.open_tabs = s.prefs().open_tabs.clone();
    prefs.active_tab = s.prefs().active_tab.clone();
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
async fn document_settings(doc: Option<String>, s: State<'_, Shared>) -> Result<session::DocumentSettings> {
    s.lock().await.on(doc.as_deref())?.document_settings().await
}

#[tauri::command]
async fn set_document_settings(settings: session::DocumentSettings, doc: Option<String>, s: State<'_, Shared>) -> Result<CommitView> {
    s.lock().await.on(doc.as_deref())?.set_document_settings(settings).await
}

#[tauri::command]
async fn set_document_retention(
    policy: Option<Retention>,
    doc: Option<String>, s: State<'_, Shared>,
) -> Result<DocInfo> {
    s.lock().await.on(doc.as_deref())?.set_document_retention(policy).await
}

#[tauri::command]
async fn archive_preview(cutoff: u64, doc: Option<String>, s: State<'_, Shared>) -> Result<session::ArchivePreviewView> {
    s.lock().await.on(doc.as_deref())?.archive_preview(cutoff).await
}

/// Archive the oldest snapshot groups, up to the one starting at `cutoff` (ADR-0023).
#[tauri::command]
async fn archive_history(cutoff: u64, doc: Option<String>, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.on(doc.as_deref())?.archive(cutoff).await
}

/// The save log: removed saves, oldest first; with `cell`, those that wrote it.
#[tauri::command]
async fn archived_saves(cell: Option<String>, doc: Option<String>, s: State<'_, Shared>) -> Result<Vec<session::ArchivedSaveView>> {
    s.lock().await.on(doc.as_deref())?.archived(cell.as_deref()).await
}

#[tauri::command]
async fn retention_preview(policy: Option<Retention>, doc: Option<String>, s: State<'_, Shared>) -> Result<PreviewView> {
    s.lock().await.on(doc.as_deref())?.retention_preview(policy).await
}

#[tauri::command]
async fn python_status(doc: Option<String>, s: State<'_, Shared>) -> Result<Option<PythonStatus>> {
    let mut s = s.lock().await;
    s.select(doc.as_deref())?;
    if !s.is_open() {
        return Ok(None);
    }
    s.python_status_full().await.map(Some)
}

/// Set up the Python environment (minutes on first use; progress as `python-setup` events).
/// The session isn't locked meanwhile, so editing goes on.
#[tauri::command]
async fn setup_python(app: AppHandle, doc: Option<String>, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    let base = s.lock().await.on(doc.as_deref())?.python_base();
    let progress = |p: cruncha_kernel_py::env::Progress| {
        let _ = app.emit("python-setup", p);
    };
    cruncha_kernel_py::env::setup(&base, python::LOCK, progress).await?;
    let mut s = s.lock().await;
    s.select(doc.as_deref())?;
    if !s.is_open() {
        return Ok(HashMap::new());
    }
    s.python_ready().await
}

/// Set up Rust for Rust cells (ADR-0006): Cruncha's own toolchain, then a first build of the
/// prelude (a minute or two; progress as `rust-setup` events). The session isn't locked meanwhile.
/// The calc language's function reference (`cruncha_eval::REFERENCE`).
/// The units a calc can use, for the Units palette (from `cruncha_units::catalog`).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UnitCatalog {
    quantities: Vec<QuantityView>,
    prefixes: Vec<PrefixView>,
    units: Vec<UnitView>,
}

#[derive(serde::Serialize)]
struct QuantityView {
    name: &'static str,
    si: &'static str,
}

#[derive(serde::Serialize)]
struct PrefixView {
    symbol: &'static str,
    word: &'static str,
    factor: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UnitView {
    name: &'static str,
    aliases: Vec<&'static str>,
    quantity: &'static str,
    si_factor: f64,
    gauge: f64,
    prefixes: Option<&'static str>,
    currency: Option<&'static str>,
}

#[tauri::command]
fn unit_catalog() -> UnitCatalog {
    use cruncha_units::catalog;
    let units = catalog::catalog();
    UnitCatalog {
        quantities: catalog::QUANTITIES
            .iter()
            .filter(|q| units.iter().any(|u| u.quantity == q.name))
            .map(|q| QuantityView { name: q.name, si: q.si })
            .collect(),
        prefixes: catalog::prefixes()
            .iter()
            .map(|&(symbol, word, factor)| PrefixView { symbol, word, factor })
            .collect(),
        units: units
            .into_iter()
            .map(|u| UnitView {
                name: u.name,
                aliases: u.aliases,
                quantity: u.quantity,
                si_factor: u.si_factor,
                gauge: u.gauge,
                prefixes: u.prefixes,
                currency: u.currency,
            })
            .collect(),
    }
}

#[tauri::command]
fn function_reference() -> Result<serde_json::Value> {
    serde_json::from_str(cruncha_eval::REFERENCE).map_err(|e| e.to_string())
}

#[tauri::command]
async fn setup_rust(app: AppHandle, doc: Option<String>, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    let base = s.lock().await.on(doc.as_deref())?.rust_base();
    let progress = |step: cruncha_kernel_rs::env::Step| {
        let _ = app.emit("rust-setup", step);
    };
    cruncha_kernel_rs::env::setup(&base, progress).await?;
    let mut s = s.lock().await;
    s.select(doc.as_deref())?;
    if !s.is_open() {
        return Ok(HashMap::new());
    }
    s.rust_ready().await
}

#[tauri::command]
async fn trust_document(doc: Option<String>, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    s.lock().await.on(doc.as_deref())?.trust_document().await
}

#[tauri::command]
async fn run_python(
    ids: Option<Vec<String>>,
    doc: Option<String>, s: State<'_, Shared>,
) -> Result<HashMap<String, CellResult>> {
    s.lock().await.on(doc.as_deref())?.run_python(ids).await
}

#[tauri::command]
async fn stop_python(doc: Option<String>, s: State<'_, Shared>) -> Result<()> {
    s.lock().await.on(doc.as_deref())?.stop_python()
}

#[tauri::command]
async fn restart_python(doc: Option<String>, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    s.lock().await.on(doc.as_deref())?.restart_python().await
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
async fn report_settings(doc: Option<String>, s: State<'_, Shared>) -> Result<report::ReportSettings> {
    s.lock().await.on(doc.as_deref())?.report_settings().await
}

#[tauri::command]
async fn set_report_settings(
    settings: report::ReportSettings,
    doc: Option<String>, s: State<'_, Shared>,
) -> Result<CommitView> {
    s.lock().await.on(doc.as_deref())?.set_report_settings(settings).await
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
async fn export_report(path: PathBuf, at: Option<At>, doc: Option<String>, s: State<'_, Shared>) -> Result<ExportView> {
    let start = std::time::Instant::now();
    let mut timed_out = false;
    let at = at.unwrap_or_default();
    let latest = at.ref_name.is_none() && at.version.is_none();
    if latest {
        loop {
            // The session lock is taken per check, so the window stays usable while Python runs.
            if !s.lock().await.on(doc.as_deref())?.python_busy().await? {
                break;
            }
            if start.elapsed() > std::time::Duration::from_secs(120) {
                timed_out = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }
    let (report, files) = s.lock().await.on(doc.as_deref())?.report_at(&at).await?;
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
async fn figures_to_render(at: Option<At>, doc: Option<String>, s: State<'_, Shared>) -> Result<Vec<figures::FigureJob>> {
    s.lock().await.on(doc.as_deref())?.figures_to_render(&at.unwrap_or_default()).await
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

/// Open a link from a prose cell in the system's browser (or mail app): http, https and
/// mailto only, so a document can't make the app open a file or another scheme.
#[tauri::command]
fn open_url(url: String) -> Result<()> {
    let url = cruncha_prose::safe_href(&url).ok_or_else(|| format!("not a web or mail link: {url}"))?;
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(&url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("couldn't open {url}: {e}"))
}

/// A cell made from pasted Markdown.
#[derive(serde::Serialize)]
struct PastedCell {
    kind: &'static str,
    source: String,
    /// The cell's attributes from the fence (`{description="…"}`).
    attrs: serde_json::Value,
}

/// A fence's attributes as a cell's: only `description` for now (ADR-0014 §1).
fn fence_attrs(mut attrs: std::collections::BTreeMap<String, String>) -> serde_json::Value {
    match attrs.remove("description") {
        Some(d) if !d.trim().is_empty() => serde_json::json!({ "description": d.trim() }),
        _ => serde_json::json!({}),
    }
}

/// Pasted Markdown as cells: prose blocks (as the editor stores them), and fenced `calc`
/// lines and `python`/`rust`/`tex` blocks as their own cells.
#[tauri::command]
fn markdown_cells(markdown: String) -> Vec<PastedCell> {
    cruncha_prose::from_markdown(&markdown)
        .into_iter()
        .map(|p| match p {
            cruncha_prose::Piece::Prose(b) => PastedCell { kind: "prose", source: cruncha_prose::to_json(&b), attrs: serde_json::json!({}) },
            cruncha_prose::Piece::Calc { source, attrs } => PastedCell { kind: "calc", source, attrs: fence_attrs(attrs) },
            cruncha_prose::Piece::Code { lang, source, attrs } => PastedCell {
                attrs: fence_attrs(attrs),
                kind: match lang.as_str() {
                    "rust" => "rust",
                    "tex" => "tex",
                    _ => "python",
                },
                source,
            },
        })
        .collect()
}

/// The open document's proposals (ADR-0016), newest first, with "if accepted" results.
#[tauri::command]
async fn proposals(doc: Option<String>, s: State<'_, Shared>) -> Result<Vec<proposals::ProposalView>> {
    s.lock().await.on(doc.as_deref())?.proposals().await
}

/// Accept a proposal: one commit by the user. `force` accepts a stale one anyway.
#[tauri::command]
async fn accept_proposal(id: String, force: bool, doc: Option<String>, s: State<'_, Shared>) -> Result<CommitView> {
    s.lock().await.on(doc.as_deref())?.accept_proposal(&id, force).await
}

#[tauri::command]
async fn reject_proposal(id: String, note: String, doc: Option<String>, s: State<'_, Shared>) -> Result<proposals::ProposalView> {
    s.lock().await.on(doc.as_deref())?.reject_proposal(&id, &note).await
}

#[tauri::command]
async fn prune_now(doc: Option<String>, s: State<'_, Shared>) -> Result<Option<PruneView>> {
    s.lock().await.on(doc.as_deref())?.prune_now().await
}

/// Drop the open document's stored code-cell results (ADR-0018).
#[tauri::command]
async fn clear_results(doc: Option<String>, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.on(doc.as_deref())?.clear_results().await
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
                let doc_id = ev.doc_id.clone();
                let _ = handle.emit("results", ev);
                // Keep finished runs with their document (ADR-0018), off the run loop. With
                // several tabs open, that's the run's document, not whichever is selected.
                if let Some(shared) = handle.try_state::<Shared>() {
                    let shared = shared.inner().clone();
                    tauri::async_runtime::spawn(async move {
                        let mut s = shared.lock().await;
                        if s.select(Some(&doc_id)).is_ok()
                            && let Err(e) = s.flush_results().await
                        {
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
            let shared: Shared = std::sync::Arc::new(Mutex::new(session));
            // Background tabs' Python stops after 10 idle minutes (ADR-0024), checked each minute
            // (or as often as the limit, when it's shorter).
            let sleeper = shared.clone();
            let idle = idle_workers();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(idle.min(std::time::Duration::from_secs(60))).await;
                    let stopped = sleeper.lock().await.sleep_idle_workers(idle).await;
                    for id in stopped {
                        eprintln!("cruncha: stopped the idle Python workers of document {id}");
                    }
                }
            });
            app.manage::<Shared>(shared);
            #[cfg(target_os = "macos")]
            app.set_menu(app_menu(app.handle())?)?;
            app.on_menu_event(|app, event| match event.id().as_ref() {
                "close_tab" => {
                    let _ = app.emit("menu", "close-tab");
                }
                "open" => {
                    let _ = app.emit("menu", "open");
                }
                "close_window" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.close();
                    }
                }
                _ => {}
            });
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
            open_documents,
            paste_clashes,
            rename_cells,
            activate_document,
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
            archive_preview,
            archive_history,
            archived_saves,
            clear_results,
            proposals,
            accept_proposal,
            reject_proposal,
            python_status,
            setup_python,
            trust_document,
            setup_rust,
            function_reference,
            unit_catalog,
            run_python,
            stop_python,
            restart_python,
            report_settings,
            set_report_settings,
            export_report,
            figures_to_render,
            put_figure,
            open_path,
            open_url,
            markdown_cells,
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
                    s.lock().await.close_all().await;
                });
            }
        });
}
