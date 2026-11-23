// SPDX-License-Identifier: GPL-3.0-or-later
//! The app's state: the open document and the preferences. Tauri commands are thin wrappers
//! around these methods, so everything here can be tested without a window.
//!
//! The app process is the single writer of any document it has open (ADR-0003): it holds the
//! `Store` and its lock. The editor's edits arrive as batched change sets through
//! [`Session::commit`]: one commit per submitted calc or per idle pause in prose.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use cruncha_store::{
    ArchivePreview, ArchivedSave, Cell, CellKind, Change, CommitKind, Error, PruneReport, Ref, Retention,
    RetentionPreview, Store, VersionInfo,
};
use serde::{Deserialize, Serialize};

use crate::engine::{CellResult, Engine};
use crate::proposals::{self, ProposalView};
use crate::report::{self, DocFacts, ReportSettings, ReportStyle};
use crate::python::{Notify, Python};

pub type Result<T> = std::result::Result<T, String>;

/// One change from the editor.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
#[allow(
    clippy::large_enum_variant,
    reason = "short-lived values from one IPC call; boxing would only hurt ergonomics"
)]
pub enum ChangeView {
    Upsert { cell: Cell },
    Delete { id: String },
}

/// What a commit returns: the new document state and the results to show.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitView {
    pub info: DocInfo,
    pub results: HashMap<String, CellResult>,
}

/// Which state of the document: the latest, a version, or a named version.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct At {
    pub version: Option<u64>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
}

/// The actor recorded on this user's commits.
pub(crate) fn local_actor() -> String {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "local".into());
    format!("user:{user}")
}

fn err(e: Error) -> String {
    e.to_string()
}

/// App-wide preferences (the Preferences page).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prefs {
    /// Retention for documents that don't set their own (ADR-0011). Default: keep everything.
    #[serde(default)]
    pub default_retention: Retention,
    /// Recently opened documents, most recent first.
    #[serde(default)]
    pub recent: Vec<String>,
    /// Ids of documents trusted to run their Python cells (documents created here are).
    #[serde(default)]
    pub trusted: Vec<String>,
    /// Where each trusted document was last opened (id → path), for the Settings page.
    #[serde(default)]
    pub trusted_paths: std::collections::BTreeMap<String, String>,
    /// Type sizes in the editor, in CSS pixels.
    #[serde(default)]
    pub text_sizes: TextSizes,
    /// The office's report style (company, logo, accent color, paper).
    #[serde(default)]
    pub report_style: ReportStyle,
    /// Agents: the default one, per-agent models and connections, provider accounts.
    #[serde(default)]
    pub agents: crate::agent_panel::AgentSettings,
    /// The open tabs (paths, in order) and the active one, reopened at the next launch
    /// (ADR-0024). Quitting leaves them as they are; closing a tab removes it.
    #[serde(default)]
    pub open_tabs: Vec<String>,
    #[serde(default)]
    pub active_tab: Option<String>,
}

/// A document's own settings, kept in its meta cell (so they are versioned like any edit).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSettings {
    /// The report's title block and contents.
    pub report: ReportSettings,
    /// Type sizes for this document (`None`: the app's).
    pub text_sizes: Option<TextSizes>,
}

/// The Python environment, independent of any document.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnv {
    pub available: bool,
    pub interpreter: Option<String>,
    /// The environment's name (it identifies the package lockfile).
    pub label: String,
    /// Where environments live.
    pub base: String,
}

/// Type sizes in the editor, in CSS pixels. Headings scale with `text` (H1 2×, H2 1.5×, …).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TextSizes {
    /// Prose and headings.
    pub text: f64,
    /// Calc cells: the typeset math, its source and description.
    pub calc: f64,
    /// Python and other code cells, and their output.
    pub code: f64,
}

impl Default for TextSizes {
    fn default() -> Self {
        TextSizes {
            text: 15.0,
            calc: 15.0,
            code: 13.0,
        }
    }
}

/// Fresh results of the latest version, computed in the background (Python cells finished).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultsEvent {
    pub doc_id: String,
    pub version: u64,
    pub results: HashMap<String, CellResult>,
}

/// Python, as the open document sees it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonStatus {
    /// The environment is set up.
    pub available: bool,
    pub interpreter: Option<String>,
    /// The open document may run code.
    pub trusted: bool,
    /// The open document has Python cells.
    pub has_python: bool,
    /// Rust is set up (ADR-0006), and the document has Rust cells.
    pub rust_available: bool,
    pub has_rust: bool,
}

/// An open document, as the tab bar shows it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDocView {
    pub id: String,
    pub name: String,
    pub path: String,
    pub version: u64,
    pub active: bool,
}

/// How many recent documents to remember.
const MAX_RECENT: usize = 8;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefView {
    pub name: String,
    pub version: u64,
    pub message: String,
    pub actor: String,
    pub created_at_ms: i64,
    /// The version itself was pruned; the ref is read from its pin.
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocInfo {
    pub path: String,
    pub name: String,
    pub id: String,
    pub version: u64,
    pub earliest_version: u64,
    pub history_bytes: u64,
    /// Stored code-cell results (ADR-0018), counted apart from history.
    pub results_bytes: u64,
    /// The document's own policy; `None` means it uses the app default.
    pub retention: Option<Retention>,
    /// The policy that applies: the document's own or the app default.
    pub effective_retention: Retention,
    pub refs: Vec<RefView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionView {
    pub version: u64,
    pub timestamp_ms: i64,
    pub kind: CommitKind,
    pub actor: String,
    pub message: String,
    pub restored_from: Option<u64>,
    pub pruned_before: Option<u64>,
}

impl From<VersionInfo> for VersionView {
    fn from(v: VersionInfo) -> Self {
        VersionView {
            version: v.version,
            timestamp_ms: v.timestamp_ms,
            kind: v.kind,
            actor: v.actor,
            message: v.message,
            restored_from: v.restored_from,
            pruned_before: v.pruned_before,
        }
    }
}

/// What archiving the snapshot groups before `cutoff` would do (ADR-0023).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePreviewView {
    pub from_version: u64,
    pub cutoff: u64,
    pub saves: usize,
    pub bytes_freed: u64,
    /// Snapshots archived (kept and restorable), oldest first.
    pub snapshots: Vec<String>,
}

impl From<ArchivePreview> for ArchivePreviewView {
    fn from(p: ArchivePreview) -> Self {
        ArchivePreviewView {
            from_version: p.from_version,
            cutoff: p.cutoff,
            saves: p.saves,
            bytes_freed: p.bytes_freed,
            snapshots: p.snapshots.into_iter().map(|r| r.name).collect(),
        }
    }
}

/// One row of the save log (ADR-0023).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedSaveView {
    pub version: u64,
    pub at_ms: i64,
    pub actor: String,
    pub message: String,
    pub kind: String,
    pub snapshot: Option<String>,
    pub cells: Vec<String>,
}

impl From<ArchivedSave> for ArchivedSaveView {
    fn from(s: ArchivedSave) -> Self {
        ArchivedSaveView {
            version: s.version,
            at_ms: s.at_us / 1000,
            actor: s.actor,
            message: s.message,
            kind: s.kind,
            snapshot: s.snapshot,
            cells: s.cells,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewView {
    pub earliest_version: u64,
    pub cutoff: Option<u64>,
    pub cutoff_timestamp_ms: Option<i64>,
    pub bytes_freed: u64,
    pub refs_to_pin: Vec<String>,
    pub waits_for_edit: bool,
}

impl From<RetentionPreview> for PreviewView {
    fn from(p: RetentionPreview) -> Self {
        PreviewView {
            earliest_version: p.earliest_version,
            cutoff: p.cutoff,
            cutoff_timestamp_ms: p.cutoff_timestamp_ms,
            bytes_freed: p.bytes_freed,
            refs_to_pin: p.refs_to_pin.into_iter().map(|r| r.name).collect(),
            waits_for_edit: p.waits_for_edit,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PruneView {
    pub pruned_before: u64,
    pub pinned: Vec<String>,
    pub data_files_deleted: usize,
    pub log_files_deleted: usize,
}

impl From<PruneReport> for PruneView {
    fn from(r: PruneReport) -> Self {
        PruneView {
            pruned_before: r.pruned_before,
            pinned: r.pinned,
            data_files_deleted: r.data_files_deleted,
            log_files_deleted: r.log_files_deleted,
        }
    }
}

struct OpenDoc {
    /// The container's id (its manifest's).
    id: String,
    store: Store,
    /// Evaluation state of the latest version (memoized results, current values). Shared with
    /// the Python run loop.
    engine: Arc<Mutex<Engine>>,
    /// The document's Python workers.
    python: Arc<Python>,
    /// This session committed edits, so the document is due a prune when it closes.
    edited: bool,
    /// Pruned already this session (at most once per session, ADR-0011).
    pruned: bool,
}

pub struct Session {
    /// The open documents, in tab order (ADR-0024). The app is the single writer of each.
    docs: Vec<OpenDoc>,
    /// The document the next call acts on: chosen per call with [`Session::select`].
    current: Option<String>,
    /// The tab the user is looking at: the default for calls that name no document (the
    /// external agent server's tools, for one).
    active: Option<String>,
    prefs: Prefs,
    prefs_path: Option<PathBuf>,
    /// Where Python environments live, and the worker package.
    python_base: PathBuf,
    /// Cruncha's Rust toolchain (ADR-0006) and the cells built with it, once set up.
    rust_base: PathBuf,
    rust_cells: Option<Arc<cruncha_kernel_rs::cells::Cells>>,
    /// Static renderings of figures for reports (ADR-0017).
    figures: crate::figures::Cache,
    worker_dir: PathBuf,
    /// Receives results computed in the background.
    on_results: Option<Arc<dyn Fn(ResultsEvent) + Send + Sync>>,
    /// Told (with the document id) when proposals change, also when an agent made them.
    on_proposals: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl Session {
    /// A session whose preferences live at `prefs_path` (none: in memory only).
    pub fn new(prefs_path: Option<PathBuf>) -> Session {
        let prefs = prefs_path
            .as_deref()
            .and_then(|p| fs::read(p).ok())
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Session {
            docs: Vec::new(),
            current: None,
            active: None,
            prefs,
            prefs_path,
            python_base: std::env::temp_dir().join("cruncha-no-python"),
            rust_base: std::env::temp_dir().join("cruncha-no-rust"),
            rust_cells: None,
            worker_dir: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../python")),
            figures: crate::figures::Cache::new(std::env::temp_dir().join("cruncha-figures")),
            on_results: None,
            on_proposals: None,
        }
    }

    /// Keep figures' static renderings (ADR-0017 §4) in `dir`.
    pub fn with_figure_cache(mut self, dir: PathBuf) -> Session {
        self.figures = crate::figures::Cache::new(dir);
        self
    }

    /// The figures a report of `at` needs drawn by the app before export (none cached).
    pub async fn figures_to_render(&mut self, at: &At) -> Result<Vec<crate::figures::FigureJob>> {
        let results = self.results(at).await?;
        Ok(crate::figures::missing(&results, &self.figures))
    }

    /// Keep the app's static rendering of a figure.
    pub fn put_figure(&self, key: &str, svg: &str) -> Result<()> {
        self.figures.put(key, svg)
    }

    /// Use the Python environments under `base` and the worker package in `worker_dir`.
    /// Use Cruncha's Rust toolchain under `base` for Rust cells.
    pub fn with_rust(mut self, base: PathBuf) -> Session {
        self.rust_base = base;
        self
    }

    pub fn rust_base(&self) -> PathBuf {
        self.rust_base.clone()
    }

    /// The Rust cell runner, if Rust is set up (made once, shared by documents).
    fn rust_cells(&mut self) -> Option<Arc<cruncha_kernel_rs::cells::Cells>> {
        if self.rust_cells.is_none()
            && let Some(tc) = cruncha_kernel_rs::env::ready(&self.rust_base)
        {
            match cruncha_kernel_rs::cells::Cells::new(tc, cruncha_kernel_rs::env::work_dir(&self.rust_base)) {
                Ok(c) => self.rust_cells = Some(Arc::new(c)),
                Err(e) => eprintln!("cruncha: Rust cells unavailable: {e}"),
            }
        }
        self.rust_cells.clone()
    }

    pub fn with_python(mut self, base: PathBuf, worker_dir: PathBuf) -> Session {
        self.python_base = base;
        self.worker_dir = worker_dir;
        self
    }

    /// Receive results that are computed in the background (after Python cells run).
    pub fn on_proposals(&mut self, f: impl Fn(String) + Send + Sync + 'static) {
        self.on_proposals = Some(Arc::new(f));
    }

    fn proposals_changed(&self) {
        if let (Some(f), Ok(doc)) = (&self.on_proposals, self.doc()) {
            f(doc.id.clone());
        }
    }

    pub fn on_results(&mut self, f: impl Fn(ResultsEvent) + Send + Sync + 'static) {
        self.on_results = Some(Arc::new(f));
    }

    pub fn python_base(&self) -> PathBuf {
        self.python_base.clone()
    }

    fn doc(&self) -> Result<&OpenDoc> {
        let id = self.current.as_deref().ok_or("no document is open")?;
        self.docs
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| "no document is open".into())
    }

    fn doc_mut(&mut self) -> Result<&mut OpenDoc> {
        let id = self.current.clone().ok_or("no document is open")?;
        self.docs
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or_else(|| "no document is open".into())
    }

    /// Choose the document the following calls act on: `Some(id)` an open document, `None` the
    /// active tab. Every command that acts on a document calls this first (ADR-0024).
    pub fn select(&mut self, doc: Option<&str>) -> Result<()> {
        match doc {
            Some(id) => {
                if !self.docs.iter().any(|d| d.id == id) {
                    return Err(format!("document {id} isn't open"));
                }
                self.current = Some(id.to_owned());
            }
            None => self.current = self.active.clone(),
        }
        Ok(())
    }

    /// [`Session::select`], for chaining: `session.on(doc)?.info()`.
    pub fn on(&mut self, doc: Option<&str>) -> Result<&mut Session> {
        self.select(doc)?;
        Ok(self)
    }

    /// Make `id` the active tab.
    pub async fn activate(&mut self, id: &str) -> Result<DocInfo> {
        self.select(Some(id))?;
        self.active = Some(id.to_owned());
        self.remember_tabs();
        self.info().await
    }

    /// The open documents, in tab order.
    pub fn documents(&self) -> Vec<OpenDocView> {
        self.docs
            .iter()
            .map(|d| OpenDocView {
                id: d.id.clone(),
                name: d
                    .store
                    .root()
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                path: d.store.root().display().to_string(),
                version: d.store.version(),
                active: self.active.as_deref() == Some(d.id.as_str()),
            })
            .collect()
    }

    fn effective_retention(&self, store: &Store) -> Retention {
        store
            .retention()
            .cloned()
            .unwrap_or_else(|| self.prefs.default_retention.clone())
    }

    /// Open the document at `path` in a new tab, or switch to its tab if it's open already.
    pub async fn open(&mut self, path: &Path) -> Result<DocInfo> {
        let root = path.canonicalize().map_err(|e| format!("{}: {e}", path.display()))?;
        if let Some(id) = self.docs.iter().find(|d| d.store.root() == root).map(|d| d.id.clone()) {
            self.remember_recent(&root.display().to_string());
            return self.activate(&id).await;
        }
        let store = Store::open(path).await.map_err(err)?;
        self.opened(store, false, false).await
    }

    /// Hold `store` as the open document and remember it in the recent list. `created`: made
    /// here (a new or sample document), so it is trusted to run its Python cells.
    async fn opened(&mut self, store: Store, edited: bool, created: bool) -> Result<DocInfo> {
        let path = store.root().display().to_string();
        let path_shown = path.clone();
        let id = store.manifest().id.clone();
        let python = Python::new(&self.python_base, &self.worker_dir);
        let rust = self.rust_cells();
        let mut engine = Engine::new();
        engine.python_available = python.interpreter().is_some();
        engine.rust_available = rust.is_some();
        engine.rust_env = rust.as_ref().map(|r| r.env_label()).unwrap_or_default();
        python.set_rust(rust);
        engine.env = crate::python::env_label(&self.python_base);
        engine.computed_by = crate::stored::this_computer(&local_actor());
        if self.docs.iter().any(|d| d.id == id) {
            return Err(format!(
                "a document with the same identity is already open ({}); is this a copy of it?",
                path_shown
            ));
        }
        self.docs.push(OpenDoc {
            id: id.clone(),
            store,
            engine: Arc::new(Mutex::new(engine)),
            python,
            edited,
            pruned: false,
        });
        self.current = Some(id.clone());
        self.active = Some(id.clone());
        self.remember_recent(&path);
        self.remember_tabs();
        let mut prefs = self.prefs.clone();
        if created && !prefs.trusted.contains(&id) {
            prefs.trusted.push(id.clone());
        }
        if prefs.trusted.contains(&id) {
            prefs.trusted_paths.insert(id.clone(), path_shown.clone());
        }
        if let Err(e) = self.set_prefs(prefs) {
            eprintln!("cruncha: could not save preferences: {e}");
        }
        let trusted = self.prefs.trusted.contains(&id);
        let cells = self.cells(None).await?;
        self.scan_python(&cells).await?;
        {
            let mut e = self.doc()?.engine.lock().unwrap();
            e.trusted = trusted;
            // What the document holds when it's opened may run (if it's trusted).
            e.approve(&cells);
        }
        self.info().await
    }

    /// Record the open tabs and the active one in the preferences (reopened at launch).
    fn remember_tabs(&mut self) {
        let mut prefs = self.prefs.clone();
        prefs.open_tabs = self.docs.iter().map(|d| d.store.root().display().to_string()).collect();
        prefs.active_tab = self
            .active
            .as_deref()
            .and_then(|id| self.docs.iter().find(|d| d.id == id))
            .map(|d| d.store.root().display().to_string());
        if (prefs.open_tabs != self.prefs.open_tabs || prefs.active_tab != self.prefs.active_tab)
            && let Err(e) = self.set_prefs(prefs)
        {
            eprintln!("cruncha: could not save preferences: {e}");
        }
    }

    /// Put `path` first in the recent list.
    fn remember_recent(&mut self, path: &str) {
        let mut prefs = self.prefs.clone();
        prefs.recent.retain(|p| p != path);
        prefs.recent.insert(0, path.to_owned());
        prefs.recent.truncate(MAX_RECENT);
        if let Err(e) = self.set_prefs(prefs) {
            eprintln!("cruncha: could not save preferences: {e}");
        }
    }

    /// Have the worker scan Python sources the engine hasn't seen.
    async fn scan_python(&self, cells: &[Cell]) -> Result<()> {
        let doc = self.doc()?;
        let todo: Vec<String> = doc
            .engine
            .lock()
            .unwrap()
            .unscanned(cells)
            .into_iter()
            .map(str::to_owned)
            .collect();
        let refs: Vec<&str> = todo.iter().map(String::as_str).collect();
        let scanned = doc.python.scan(&refs).await;
        let mut e = doc.engine.lock().unwrap();
        for (source, names) in scanned {
            e.set_py_names(&source, names);
        }
        Ok(())
    }

    /// Evaluate the latest version and start running the Python cells that are due.
    async fn latest_results(&mut self) -> Result<HashMap<String, CellResult>> {
        let cells = self.cells(None).await?;
        self.scan_python(&cells).await?;
        self.flush_results().await?;
        let doc = self.doc()?;
        doc.engine.lock().unwrap().doc_version = doc.store.version();
        let run = crate::stored::evaluate(&doc.store, &doc.engine, &cells).await.map_err(err)?;
        if !run.jobs.is_empty() {
            let doc_id = doc.store.manifest().id.clone();
            let version = doc.store.version();
            let sink = self.on_results.clone();
            let notify: Notify = Arc::new(move |results| {
                if let Some(f) = &sink {
                    f(ResultsEvent {
                        doc_id: doc_id.clone(),
                        version,
                        results,
                    });
                }
            });
            doc.python
                .start(doc.engine.clone(), Arc::new(cells), notify);
        }
        Ok(run.results)
    }

    /// Python for the open document.
    pub fn python_status(&self) -> Result<PythonStatus> {
        let doc = self.doc()?;
        let interpreter = doc.python.interpreter();
        let trusted = doc.engine.lock().unwrap().trusted;
        Ok(PythonStatus {
            available: interpreter.is_some(),
            interpreter: interpreter.map(|p| p.display().to_string()),
            trusted,
            has_python: false,
            rust_available: doc.python.rust().is_some(),
            has_rust: false,
        })
    }

    /// Python for the open document, including whether it has Python cells.
    pub async fn python_status_full(&self) -> Result<PythonStatus> {
        let mut st = self.python_status()?;
        let cells = self.cells(None).await?;
        st.has_python = cells.iter().any(|c| c.kind == CellKind::Python);
        st.has_rust = cells.iter().any(|c| c.kind == CellKind::Rust);
        Ok(st)
    }

    /// The environment was just set up: Python cells can run now.
    pub async fn python_ready(&mut self) -> Result<HashMap<String, CellResult>> {
        let available = self.doc()?.python.interpreter().is_some();
        self.doc()?.engine.lock().unwrap().python_available = available;
        self.latest_results().await
    }

    /// Rust was just set up: Rust cells can run now.
    pub async fn rust_ready(&mut self) -> Result<HashMap<String, CellResult>> {
        let rust = self.rust_cells();
        {
            let doc = self.doc()?;
            let mut e = doc.engine.lock().unwrap();
            e.rust_available = rust.is_some();
            e.rust_env = rust.as_ref().map(|r| r.env_label()).unwrap_or_default();
            doc.python.set_rust(rust);
        }
        self.latest_results().await
    }

    /// Trust the open document to run its Python cells (remembered on this computer).
    pub async fn trust_document(&mut self) -> Result<HashMap<String, CellResult>> {
        let id = self.doc()?.store.manifest().id.clone();
        let mut prefs = self.prefs.clone();
        if !prefs.trusted.contains(&id) {
            prefs.trusted.push(id.clone());
            prefs.trusted_paths.insert(id, self.doc()?.store.root().display().to_string());
            self.set_prefs(prefs)?;
        }
        let cells = self.cells(None).await?;
        {
            let mut e = self.doc()?.engine.lock().unwrap();
            e.trusted = true;
            e.approve(&cells);
        }
        self.latest_results().await
    }

    /// Run Python cells: these ids, or all of them. They run even if nothing changed.
    pub async fn run_python(
        &mut self,
        ids: Option<Vec<String>>,
    ) -> Result<HashMap<String, CellResult>> {
        let cells = self.cells(None).await?;
        let chosen: Vec<&Cell> = cells
            .iter()
            .filter(|c| {
                matches!(c.kind, CellKind::Python | CellKind::Rust) && ids.as_ref().is_none_or(|ids| ids.contains(&c.id))
            })
            .collect();
        {
            let mut e = self.doc()?.engine.lock().unwrap();
            e.approve(chosen.iter().copied());
            e.rerun(&chosen.iter().map(|c| c.id.clone()).collect::<Vec<_>>());
        }
        self.latest_results().await
    }

    /// Stop the running Python cell, and anything queued.
    pub fn stop_python(&self) -> Result<()> {
        self.doc()?.python.stop();
        Ok(())
    }

    /// Restart the Python worker, then rerun what's approved.
    pub async fn restart_python(&mut self) -> Result<HashMap<String, CellResult>> {
        let doc = self.doc()?;
        doc.python.restart(&doc.engine).await;
        self.latest_results().await
    }

    /// Forget a recent document (it was moved or deleted).
    /// Stop trusting a document to run its Python cells (it asks again next time it opens).
    pub fn forget_trusted(&mut self, id: &str) -> Result<()> {
        let mut prefs = self.prefs.clone();
        prefs.trusted.retain(|t| t != id);
        prefs.trusted_paths.remove(id);
        self.set_prefs(prefs)?;
        for doc in self.docs.iter().filter(|d| d.id == id) {
            doc.engine.lock().unwrap().trusted = false;
        }
        Ok(())
    }

    pub fn forget_recent(&mut self, path: &str) -> Result<()> {
        let mut prefs = self.prefs.clone();
        prefs.recent.retain(|p| p != path);
        self.set_prefs(prefs)
    }

    /// Create a new, empty document at `path` and open it.
    pub async fn create(&mut self, path: &Path) -> Result<DocInfo> {
        let store = Store::create(path, &local_actor()).await.map_err(err)?;
        self.opened(store, false, true).await
    }

    /// Create a small example worksheet with some history.
    pub async fn create_sample(&mut self, path: &Path) -> Result<DocInfo> {
        let store = sample_document(path).await.map_err(err)?;
        self.opened(store, true, true).await
    }

    /// Close every open document (the app is quitting).
    pub async fn close_all(&mut self) {
        while let Some(id) = self.docs.first().map(|d| d.id.clone()) {
            self.current = Some(id);
            self.close_tab(false).await;
        }
    }

    /// Close the selected document (its tab). If this session edited it, apply its retention
    /// policy first; `Store::prune` itself refuses when there were no edits since the last
    /// prune. The next tab (or the previous) becomes active if this one was.
    pub async fn close(&mut self) -> Option<PruneView> {
        self.close_tab(true).await
    }

    /// Close the selected document; `forget`: also drop it from the tabs reopened at launch.
    async fn close_tab(&mut self, forget: bool) -> Option<PruneView> {
        let id = self.current.clone()?;
        let at = self.docs.iter().position(|d| d.id == id)?;
        let mut doc = self.docs.remove(at);
        if self.active.as_deref() == Some(id.as_str()) {
            self.active = self.docs.get(at).or_else(|| self.docs.last()).map(|d| d.id.clone());
        }
        self.current = self.active.clone();
        if forget {
            self.remember_tabs();
        }
        doc.python.shutdown().await;
        if let Err(e) = crate::stored::flush(&mut doc.store, &doc.engine, &local_actor()).await {
            eprintln!("cruncha: storing results on close failed: {e}");
        }
        if !doc.edited || doc.pruned {
            return None;
        }
        let policy = self.effective_retention(&doc.store);
        match doc.store.prune(&policy, "system").await {
            Ok(r) => {
                if r.is_some()
                    && let Err(e) = crate::stored::retain_after_prune(&mut doc.store, &doc.engine, "system").await
                {
                    eprintln!("cruncha: trimming stored results failed: {e}");
                }
                r.map(Into::into)
            }
            Err(e) => {
                eprintln!("cruncha: pruning on close failed: {e}");
                None
            }
        }
    }

    pub async fn info(&self) -> Result<DocInfo> {
        let doc = self.doc()?;
        let s = &doc.store;
        let earliest = s.earliest_version();
        let refs = s
            .refs()
            .await
            .map_err(err)?
            .into_iter()
            .map(|r: Ref| RefView {
                pinned: r.doc_version < earliest,
                name: r.name,
                version: r.doc_version,
                message: r.message,
                actor: r.actor,
                created_at_ms: r.created_at_us / 1000,
            })
            .collect();
        Ok(DocInfo {
            path: s.root().display().to_string(),
            name: s
                .root()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            id: s.manifest().id.clone(),
            version: s.version(),
            earliest_version: earliest,
            history_bytes: s.history_size_bytes().map_err(err)?,
            results_bytes: s.results_size_bytes().map_err(err)?,
            retention: s.retention().cloned(),
            effective_retention: self.effective_retention(s),
            refs,
        })
    }

    /// Is a document selected (and open)?
    pub fn is_open(&self) -> bool {
        self.doc().is_ok()
    }

    /// Cells of `version` (default: the latest).
    pub async fn cells(&self, version: Option<u64>) -> Result<Vec<Cell>> {
        let s = &self.doc()?.store;
        match version {
            None => s.cells().await,
            Some(v) => s.cells_at(v).await,
        }
        .map_err(err)
    }

    pub async fn cells_at_ref(&self, name: &str) -> Result<Vec<Cell>> {
        self.doc()?.store.cells_at_ref(name).await.map_err(err)
    }

    async fn cells_at(&self, at: &At) -> Result<Vec<Cell>> {
        match &at.ref_name {
            Some(name) => self.cells_at_ref(name).await,
            None => self.cells(at.version).await,
        }
    }

    /// Commit the editor's changes as one new version. Returns the new state and its results.
    /// A calc cell's `defines` and `reads` are recomputed from its source, so the stored names
    /// are always the evaluator's.
    pub async fn commit(&mut self, changes: Vec<ChangeView>, message: &str) -> Result<CommitView> {
        let changes: Vec<Change> = changes
            .into_iter()
            .map(|c| match c {
                ChangeView::Upsert { cell } => Change::Upsert(cell),
                ChangeView::Delete { id } => Change::Delete(id),
            })
            .collect();
        let changes = self.with_names(changes).await?;
        let doc = self.doc_mut()?;
        if !changes.is_empty() {
            let message = if message.trim().is_empty() {
                "edit"
            } else {
                message
            };
            doc.store
                .commit(&changes, &local_actor(), message)
                .await
                .map_err(err)?;
            doc.edited = true;
        }
        let results = self.results(&At::default()).await?;
        Ok(CommitView {
            info: self.info().await?,
            results,
        })
    }

    /// Set the stored names of the cells in `changes`: a calc's from its prepared statement,
    /// a Python cell's from the worker's scan (when Python is set up).
    async fn with_names(&mut self, changes: Vec<Change>) -> Result<Vec<Change>> {
        let python: Vec<Cell> = changes
            .iter()
            .filter_map(|c| match c {
                Change::Upsert(cell) if cell.kind == CellKind::Python => Some(cell.clone()),
                _ => None,
            })
            .collect();
        self.scan_python(&python).await?;
        let mut engine = self.doc()?.engine.lock().unwrap();
        Ok(changes
            .into_iter()
            .map(|c| match c {
                Change::Upsert(mut cell) => {
                    match cell.kind {
                        CellKind::Calc => (cell.defines, cell.reads) = engine.names(&cell.source),
                        CellKind::Python => {
                            if let Some(n) = engine.py_names(&cell.source) {
                                (cell.defines, cell.reads) = (n.defines.clone(), n.reads.clone());
                            }
                        }
                        CellKind::Rust => {
                            let n = engine.rust_names(&cell.source);
                            (cell.defines, cell.reads) = (n.defines.clone(), n.reads.clone());
                        }
                        _ => {}
                    }
                    Change::Upsert(cell)
                }
                d => d,
            })
            .collect())
    }

    // ---- Proposals (ADR-0016) ----

    /// Every proposal of the open document, newest first. Open ones carry their "if accepted"
    /// results, evaluated on a scratch copy of the latest version.
    pub async fn proposals(&mut self) -> Result<Vec<ProposalView>> {
        let list = self.doc()?.store.proposals().await.map_err(err)?;
        let cells = self.cells(None).await?;
        let revs = self.doc()?.store.cell_revs().await.map_err(err)?;
        let before = self.latest_results().await?;
        let mut out = Vec::with_capacity(list.len());
        for p in list.iter().rev() {
            let after = if p.status == cruncha_store::ProposalStatus::Open {
                Some(self.what_if(&cells, p).await?)
            } else {
                None
            };
            out.push(proposals::view(p, &cells, &revs, &before, after.as_ref()));
        }
        Ok(out)
    }

    /// The results of the latest version with `p` applied (nothing is committed).
    async fn what_if(
        &mut self,
        cells: &[Cell],
        p: &cruncha_store::Proposal,
    ) -> Result<HashMap<String, CellResult>> {
        let provenance = cruncha_store::Provenance::AiProposed { session: proposals::session_of(&p.proposed_by) };
        let (applied, _) = proposals::apply(cells, p, &provenance);
        self.scan_python(&applied).await?;
        Ok(self.doc()?.engine.lock().unwrap().what_if(&applied))
    }

    /// Record an agent's proposal against the latest version. Refused if a cell it touches
    /// changed after `base_version`, or if it would leave a duplicate name or a cycle.
    pub async fn propose(
        &mut self,
        actor: &str,
        base_version: Option<u64>,
        message: &str,
        edits: Vec<cruncha_mcp::Edit>,
    ) -> Result<ProposalView> {
        if message.trim().is_empty() {
            return Err("a proposal needs a message saying what it does and why".into());
        }
        let cells = self.cells(None).await?;
        let revs = self.doc()?.store.cell_revs().await.map_err(err)?;
        let changes = proposals::to_changes(edits, &cells, &revs, base_version)?;
        // Names and cycles are checked on the combined document.
        let draft = cruncha_store::Proposal {
            id: String::new(),
            status: cruncha_store::ProposalStatus::Open,
            proposed_by: actor.to_owned(),
            message: message.to_owned(),
            base_version: self.doc()?.store.version(),
            changes: changes.clone(),
            created_at_us: 0,
            decided_by: None,
            decided_at_us: None,
            decision_note: None,
            accepted_version: None,
            seq: 0,
        };
        let provenance = cruncha_store::Provenance::AiProposed { session: proposals::session_of(actor) };
        let (applied, _) = proposals::apply(&cells, &draft, &provenance);
        self.scan_python(&applied).await?;
        {
            let mut e = self.doc()?.engine.lock().unwrap();
            let now = e.graph(&cells);
            let then = e.graph(&applied);
            let mut problems = Vec::new();
            for c in &applied {
                for issue in then.issues(&c.id) {
                    let structural = matches!(
                        issue,
                        cruncha_graph::Issue::DuplicateDefinition { .. } | cruncha_graph::Issue::Cycle { .. }
                    );
                    if structural && !now.issues(&c.id).contains(issue) {
                        problems.push(format!("{}: {issue}", c.id));
                    }
                }
            }
            if !problems.is_empty() {
                return Err(format!("the proposal would break the document: {}", problems.join("; ")));
            }
        }
        let p = self
            .doc_mut()?
            .store
            .propose(changes, message.trim(), actor)
            .await
            .map_err(err)?;
        self.proposals_changed();
        self.proposals()
            .await?
            .into_iter()
            .find(|v| v.id == p.id)
            .ok_or_else(|| "the proposal vanished".into())
    }

    /// Accept an open proposal: one commit by this user, with the proposal's cells marked
    /// `ai:accepted`. A stale proposal (a cell changed since) needs `force`, which replaces
    /// the current source. Changed Python cells still wait for Run (ADR-0005).
    pub async fn accept_proposal(&mut self, id: &str, force: bool) -> Result<CommitView> {
        let actor = local_actor();
        let p = self.doc()?.store.proposal(id).await.map_err(err)?;
        if p.status != cruncha_store::ProposalStatus::Open {
            return Err(format!("this proposal is already {}", p.status.as_str()));
        }
        let cells = self.cells(None).await?;
        let revs = self.doc()?.store.cell_revs().await.map_err(err)?;
        let stale = proposals::stale_ids(&p, &cells, &revs);
        if !stale.is_empty() && !force {
            return Err(format!(
                "cells changed since the agent proposed this: {}; review and accept anyway, or reject it",
                stale.join(", ")
            ));
        }
        let provenance = cruncha_store::Provenance::AiAccepted {
            session: proposals::session_of(&p.proposed_by),
            by: actor.clone(),
        };
        let (_, changes) = proposals::apply(&cells, &p, &provenance);
        let changes = self.with_names(changes).await?;
        let message = format!("accept proposal: {}", p.message.lines().next().unwrap_or_default());
        let doc = self.doc_mut()?;
        doc.store.accept(id, &changes, &actor, &message).await.map_err(err)?;
        doc.edited = true;
        self.proposals_changed();
        let results = self.results(&At::default()).await?;
        Ok(CommitView { info: self.info().await?, results })
    }

    /// Reject an open proposal, with an optional note the agent can read.
    pub async fn reject_proposal(&mut self, id: &str, note: &str) -> Result<ProposalView> {
        self.decide(id, cruncha_store::ProposalStatus::Rejected, &local_actor(), note).await
    }

    /// Withdraw one of `actor`'s own open proposals.
    pub async fn withdraw_proposal(&mut self, actor: &str, id: &str) -> Result<ProposalView> {
        let p = self.doc()?.store.proposal(id).await.map_err(err)?;
        if p.proposed_by != actor {
            return Err(format!("proposal {id} isn't yours to withdraw"));
        }
        self.decide(id, cruncha_store::ProposalStatus::Withdrawn, actor, "").await
    }

    async fn decide(
        &mut self,
        id: &str,
        status: cruncha_store::ProposalStatus,
        actor: &str,
        note: &str,
    ) -> Result<ProposalView> {
        self.doc_mut()?.store.decide(id, status, actor, note).await.map_err(err)?;
        self.proposals_changed();
        self.proposals()
            .await?
            .into_iter()
            .find(|v| v.id == id)
            .ok_or_else(|| "the proposal vanished".into())
    }

    /// Take a snapshot: name the latest version as a known-good point (a store tag; it
    /// survives history trimming). Names are unique within a document.
    pub async fn snapshot(&mut self, name: &str, note: &str) -> Result<DocInfo> {
        let name = name.trim();
        if name.is_empty() {
            return Err("a snapshot needs a name".into());
        }
        let doc = self.doc_mut()?;
        doc.store
            .tag(name, note.trim(), &local_actor())
            .await
            .map_err(|e| match e {
                Error::RefExists(n) => format!("there is already a snapshot named “{n}”"),
                e => err(e),
            })?;
        self.info().await
    }

    /// Make an earlier version (or snapshot) the latest again, as a new version: history is
    /// never rewritten, so a restore can be undone the same way.
    pub async fn restore(&mut self, at: &At) -> Result<CommitView> {
        let actor = local_actor();
        let doc = self.doc_mut()?;
        let before = doc.store.version();
        let after = match (&at.ref_name, at.version) {
            (Some(name), _) => doc.store.restore_ref(name, &actor).await,
            (None, Some(v)) => doc.store.restore(v, &actor).await,
            (None, None) => Ok(before),
        }
        .map_err(err)?;
        if after != before {
            doc.edited = true;
        }
        // Restoring was the user's choice, so the restored Python sources may run.
        let cells = self.cells(None).await?;
        self.doc()?.engine.lock().unwrap().approve(&cells);
        let results = self.results(&At::default()).await?;
        Ok(CommitView {
            info: self.info().await?,
            results,
        })
    }

    /// Results for every calc cell at `at`. The latest version uses the document's engine
    /// (memoized); history is evaluated afresh.
    pub async fn results(&mut self, at: &At) -> Result<HashMap<String, CellResult>> {
        let cells = self.cells_at(at).await?;
        let current = self.doc()?.store.version();
        let latest = at.ref_name.is_none() && at.version.is_none_or(|v| v == current);
        if latest {
            self.latest_results().await
        } else {
            // History is evaluated afresh; Python cells show their stored results (ADR-0018).
            let doc = self.doc()?;
            let past = Mutex::new(doc.engine.lock().unwrap().for_past());
            Ok(crate::stored::evaluate(&doc.store, &past, &cells).await.map_err(err)?.results)
        }
    }

    /// Write the Python runs finished since the last call to the document's stored results.
    /// Never a document version. Returns how many were stored.
    pub async fn flush_results(&mut self) -> Result<usize> {
        let Ok(doc) = self.doc_mut() else { return Ok(0) };
        crate::stored::flush(&mut doc.store, &doc.engine, &local_actor()).await.map_err(err)
    }

    /// Drop every stored result of the open document ("Clear stored results"). This session's
    /// outcomes stay on screen; they are stored again when cells next run.
    pub async fn clear_results(&mut self) -> Result<DocInfo> {
        let doc = self.doc_mut()?;
        doc.store.clear_results(&local_actor()).await.map_err(err)?;
        self.info().await
    }

    /// Evaluate a calc source while it is being typed, against the latest values (nothing is
    /// committed).
    pub fn preview_calc(&mut self, source: &str) -> Result<CellResult> {
        Ok(self.doc()?.engine.lock().unwrap().preview(source))
    }

    /// Cells that changed between two versions of the open document.
    pub async fn diff(&self, from: u64, to: u64) -> Result<Vec<cruncha_store::CellDiff>> {
        self.doc()?.store.diff(from, to).await.map_err(err)
    }

    pub async fn history(&self) -> Result<Vec<VersionView>> {
        let h = self.doc()?.store.history().await.map_err(err)?;
        Ok(h.into_iter().map(Into::into).collect())
    }

    /// The document's report settings (title block and options), from its meta cell.
    pub async fn report_settings(&self) -> Result<ReportSettings> {
        Ok(report::settings_of(&self.cells(None).await?))
    }

    /// Save the report settings into the meta cell (created if the document has none): one
    /// commit, so the title block has history like everything else.
    pub async fn set_report_settings(&mut self, settings: ReportSettings) -> Result<CommitView> {
        let mut d = self.document_settings().await?;
        d.report = settings;
        self.set_document_settings(d).await
    }

    /// The document's own settings (from its meta cell).
    pub async fn document_settings(&self) -> Result<DocumentSettings> {
        let cells = self.cells(None).await?;
        let text_sizes = cells
            .iter()
            .find(|c| c.kind == CellKind::Meta)
            .and_then(|c| c.attrs.get("textSizes"))
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        Ok(DocumentSettings { report: report::settings_of(&cells), text_sizes })
    }

    /// Save the document's settings into its meta cell (created if it has none), as one
    /// commit. Nothing is committed if they didn't change.
    pub async fn set_document_settings(&mut self, settings: DocumentSettings) -> Result<CommitView> {
        if settings == self.document_settings().await? {
            let results = self.results(&At::default()).await?;
            return Ok(CommitView { info: self.info().await?, results });
        }
        let cells = self.cells(None).await?;
        let mut cell = match cells.into_iter().find(|c| c.kind == CellKind::Meta) {
            Some(c) => c,
            None => Cell::new(CellKind::Meta, "0", ""),
        };
        if !cell.attrs.is_object() {
            cell.attrs = serde_json::json!({});
        }
        cell.attrs["report"] = serde_json::to_value(&settings.report).map_err(|e| e.to_string())?;
        match &settings.text_sizes {
            Some(t) => cell.attrs["textSizes"] = serde_json::to_value(t).map_err(|e| e.to_string())?,
            None => {
                cell.attrs.as_object_mut().map(|o| o.remove("textSizes"));
            }
        }
        self.commit(vec![ChangeView::Upsert { cell }], "edit document settings")
            .await
    }

    /// The Python environment (whether it is set up), with no document needed.
    pub fn python_env(&self) -> PythonEnv {
        let interpreter = cruncha_kernel_py::env::ready(&self.python_base, crate::python::LOCK);
        PythonEnv {
            available: interpreter.is_some(),
            interpreter: interpreter.map(|p| p.display().to_string()),
            label: crate::python::env_label(&self.python_base),
            base: self.python_base.display().to_string(),
        }
    }

    /// Python cells of the latest version that are due to run or running. Evaluates only:
    /// nothing is started.
    pub async fn python_busy(&self) -> Result<bool> {
        let cells = self.cells(None).await?;
        let doc = self.doc()?;
        let jobs = doc.engine.lock().unwrap().evaluate(&cells).jobs;
        Ok(!jobs.is_empty() && doc.python.interpreter().is_some())
    }

    /// The latest version as a report, with the figures and logo it refers to.
    /// The version or snapshot `at` as a report (ADR-0015, ADR-0018). A past version prints
    /// its own cells, title block and options, the snapshots taken up to it, and its Python
    /// cells' stored results; nothing runs for it. Same version, same results → same PDF.
    pub async fn report_at(&mut self, at: &At) -> Result<(cruncha_report::Report, Vec<(String, Vec<u8>)>)> {
        let cells = self.cells_at(at).await?;
        let results = self.results(at).await?;
        let info = self.info().await?;
        let version = match (&at.ref_name, at.version) {
            (Some(name), _) => info
                .refs
                .iter()
                .find(|r| &r.name == name)
                .map(|r| r.version)
                .ok_or_else(|| format!("no snapshot named {name}"))?,
            (None, Some(v)) => v,
            (None, None) => info.version,
        };
        // When that version was committed; a pinned snapshot's history may be pruned, so
        // fall back to when it was taken.
        let committed_ms = self
            .history()
            .await?
            .iter()
            .find(|v| v.version == version)
            .map(|v| v.timestamp_ms)
            .or_else(|| info.refs.iter().find(|r| r.version == version).map(|r| r.created_at_ms))
            .unwrap_or_default();
        let has_python = cells.iter().any(|c| c.kind == CellKind::Python);
        // The environments the Python output came from: stored runs say theirs.
        let mut envs: Vec<String> = results
            .values()
            .filter_map(|r| r.python.as_ref()?.stored.as_ref().map(|f| f.env.clone()))
            .filter(|e| !e.is_empty())
            .collect();
        envs.sort();
        envs.dedup();
        let python_env = has_python.then(|| {
            let here = crate::python::env_label(&self.python_base);
            let live = version == info.version
                && results.values().any(|r| r.python.as_ref().is_some_and(|p| p.stored.is_none() && p.ms.is_some()));
            if live && !envs.contains(&here) {
                envs.push(here.clone());
            }
            if envs.is_empty() { here } else { envs.join(", ") }
        });
        let facts = DocFacts {
            name: info.name,
            id: info.id,
            version,
            committed_ms,
            refs: info.refs.into_iter().filter(|r| r.version <= version).collect(),
            python_env,
        };
        let fonts = crate::figures::report_fonts();
        let figure_svg = |mime: &str, spec: &str| self.figures.get(&crate::figures::static_key(mime, spec, &fonts));
        Ok(report::build(&cells, &results, &facts, &self.prefs.report_style, &figure_svg))
    }

    pub fn prefs(&self) -> &Prefs {
        &self.prefs
    }

    pub fn set_prefs(&mut self, prefs: Prefs) -> Result<()> {
        if let Some(path) = &self.prefs_path {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let tmp = path.with_extension("json.tmp");
            let bytes = serde_json::to_vec_pretty(&prefs).map_err(|e| e.to_string())?;
            fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
            fs::rename(&tmp, path).map_err(|e| e.to_string())?;
        }
        self.prefs = prefs;
        Ok(())
    }

    /// Set the document's own policy, or `None` to use the app default.
    pub async fn set_document_retention(&mut self, policy: Option<Retention>) -> Result<DocInfo> {
        self.doc_mut()?.store.set_retention(policy).map_err(err)?;
        self.info().await
    }

    /// What pruning the open document under `policy` (default: its effective policy) would do.
    pub async fn retention_preview(&self, policy: Option<Retention>) -> Result<PreviewView> {
        let s = &self.doc()?.store;
        let policy = policy.unwrap_or_else(|| self.effective_retention(s));
        Ok(s.retention_preview(&policy).await.map_err(err)?.into())
    }

    /// What archiving every version before `cutoff` (a snapshot's version) would do.
    pub async fn archive_preview(&self, cutoff: u64) -> Result<ArchivePreviewView> {
        Ok(self.doc()?.store.archive_preview(cutoff).await.map_err(err)?.into())
    }

    /// Archive the oldest snapshot groups, up to the one starting at `cutoff` (ADR-0023; the
    /// user confirmed the preview). Stored results are trimmed as after any prune.
    pub async fn archive(&mut self, cutoff: u64) -> Result<DocInfo> {
        let doc = self.doc_mut()?;
        doc.store.archive_before(cutoff, &local_actor()).await.map_err(err)?;
        doc.pruned = true;
        crate::stored::retain_after_prune(&mut doc.store, &doc.engine, "system")
            .await
            .map_err(err)?;
        self.info().await
    }

    /// The save log: every removed save, oldest first; with `cell`, only those that wrote it.
    pub async fn archived(&self, cell: Option<&str>) -> Result<Vec<ArchivedSaveView>> {
        Ok(self.doc()?.store.archived(cell).await.map_err(err)?.into_iter().map(Into::into).collect())
    }

    /// Apply the open document's effective policy now (the user confirmed the preview).
    pub async fn prune_now(&mut self) -> Result<Option<PruneView>> {
        let policy = self.effective_retention(&self.doc()?.store);
        let doc = self.doc_mut()?;
        let r = doc.store.prune(&policy, "system").await.map_err(err)?;
        if r.is_some() {
            doc.pruned = true;
            crate::stored::retain_after_prune(&mut doc.store, &doc.engine, "system")
                .await
                .map_err(err)?;
        }
        Ok(r.map(Into::into))
    }
}

fn prose(order: &str, text: &str) -> Cell {
    let doc = serde_json::json!({
        "type": "paragraph",
        "content": [{ "type": "text", "text": text }],
    });
    Cell::new(CellKind::Prose, order, doc.to_string())
}

fn heading(order: &str, level: u8, text: &str) -> Cell {
    let doc = serde_json::json!({
        "type": "heading",
        "attrs": { "level": level },
        "content": [{ "type": "text", "text": text }],
    });
    Cell::new(CellKind::Prose, order, doc.to_string())
}

fn calc(id: &str, order: &str, src: &str, defines: &[&str], reads: &[&str]) -> Cell {
    let mut c = Cell::new(CellKind::Calc, order, src);
    c.id = id.into();
    c.defines = defines.iter().map(|s| s.to_string()).collect();
    c.reads = reads.iter().map(|s| s.to_string()).collect();
    c.attrs = serde_json::json!({ "display": "?" });
    c
}

/// The sample's Python cell: the moment along the span, plotted.
const SAMPLE_PYTHON: &str = "import numpy as np
import matplotlib.pyplot as plt

x = np.linspace(0, L, 51)
moment = (w * x * (L - x) / 2).to(\"kip*ft\")
M_max = moment.max()

plt.figure(figsize=(6, 2.4))
plt.plot(x.magnitude, moment.magnitude)
plt.xlabel(\"x (ft)\")
plt.ylabel(\"M (kip·ft)\")
plt.grid(alpha=0.3)";

/// `c` with a description (shown beside the calc and in the variable browser).
fn described(mut c: Cell, description: &str) -> Cell {
    c.attrs["description"] = description.into();
    c
}

/// A simply supported beam check with a few revisions and a tag.
pub async fn sample_document(path: &Path) -> std::result::Result<Store, Error> {
    let me = "user:sample";
    let mut s = Store::create(path, me).await?;
    let w = |v: &str| {
        let c = calc("W", "b1", &format!("w = {v} kip/ft"), &["w"], &[]);
        described(c, "Uniform load, dead + live")
    };
    let l = |v: &str| described(calc("L", "b2", &format!("L = {v} ft"), &["L"], &[]), "Span");
    let m = described(
        calc("M", "b3", "M = w L^2 / 8 = ?? kip·ft", &["M"], &["w", "L"]),
        "Maximum bending moment, at midspan",
    );
    s.commit(
        &[
            Change::Upsert(heading("a0", 1, "Beam check")),
            Change::Upsert(prose(
                "a1",
                "Simply supported beam under a uniform load: maximum bending moment.",
            )),
            Change::Upsert(heading("a2", 2, "Loads and span")),
            Change::Upsert(w("1.2")),
            Change::Upsert(l("24")),
            Change::Upsert(heading("b25", 2, "Bending moment")),
            Change::Upsert(m),
        ],
        me,
        "start beam check",
    )
    .await?;
    s.commit(&[Change::Upsert(w("1.5"))], me, "update dead load")
        .await?;
    let mut py = Cell::new(CellKind::Python, "c0", SAMPLE_PYTHON);
    py.id = "PY".into();
    py.reads = vec!["L".into(), "w".into()];
    py.defines = vec!["x".into(), "moment".into(), "M_max".into()];
    s.commit(
        &[
            Change::Upsert(prose("b9", "Moment diagram along the span:")),
            Change::Upsert(py),
        ],
        me,
        "add moment diagram",
    )
    .await?;
    s.tag("Rev A", "issued for review", me).await?;
    s.commit(&[Change::Upsert(l("26"))], me, "span revised to 26 ft")
        .await?;
    s.commit(
        &[
            Change::Upsert(heading("c5", 2, "Check")),
            Change::Upsert(described(
                calc("MA", "c7", "M_allow = 150 kip·ft", &["M_allow"], &[]),
                "Allowable moment",
            )),
            Change::Upsert(calc(
                "CHK",
                "d0",
                "@test M < M_allow",
                &[],
                &["M", "M_allow"],
            )),
            Change::Upsert(prose(
                "d1",
                "The largest moment along the plotted span, against the formula:",
            )),
            Change::Upsert(described(
                calc(
                    "RATIO",
                    "d2",
                    "ratio = M_max / M = ?",
                    &["ratio"],
                    &["M_max", "M"],
                ),
                "Plotted maximum over w L²/8",
            )),
        ],
        me,
        "add allowable-moment check",
    )
    .await?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sample_document_opens_with_history_and_a_tag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        let info = session.create_sample(&path).await.unwrap();
        assert_eq!(info.version, 5);
        assert_eq!(info.refs.len(), 1);
        assert_eq!(info.refs[0].name, "Rev A");
        assert!(info.retention.is_none());
        assert!(info.effective_retention.is_keep_everything());

        let latest = session.cells(None).await.unwrap();
        assert_eq!(latest.len(), 14);
        // The sample evaluates: M = 1.5 kip/ft × (26 ft)² / 8, and its check passes.
        let r = session.results(&At::default()).await.unwrap();
        assert!(r["M"].text.ends_with("126.75 kip·ft"), "{:?}", r["M"]);
        assert_eq!(r["CHK"].error, None, "{:?}", r["CHK"]);
        let at_v1 = At {
            version: Some(1),
            ref_name: None,
        };
        assert!(
            session.results(&at_v1).await.unwrap()["M"]
                .text
                .ends_with("86.4 kip·ft")
        );
        let rev_a = session.cells_at_ref("Rev A").await.unwrap();
        assert!(rev_a.iter().any(|c| c.source == "L = 24 ft"));
        let v1 = session.cells(Some(1)).await.unwrap();
        assert!(v1.iter().any(|c| c.source == "w = 1.2 kip/ft"));
        let h = session.history().await.unwrap();
        assert_eq!(h.len(), 6);
        assert_eq!(h[0].message, "add allowable-moment check");

        // Keep everything: closing doesn't prune; reopening works (the lock was released).
        assert!(session.close().await.is_none());
        let info = session.open(&path).await.unwrap();
        assert_eq!(info.version, 5);
        let m = latest.iter().find(|c| c.id == "M").unwrap();
        assert_eq!(m.attrs["description"], "Maximum bending moment, at midspan");
    }

    /// The real worker, when the Python environment is set up on this machine.
    fn python_base() -> Option<PathBuf> {
        let base = PathBuf::from(std::env::var_os("HOME")?)
            .join("Library/Application Support/io.github.acmeanvil.cruncha/python");
        cruncha_kernel_py::env::ready(&base, crate::python::LOCK).map(|_| base)
    }

    async fn wait_for(
        rx: &std::sync::mpsc::Receiver<ResultsEvent>,
        done: impl Fn(&ResultsEvent) -> bool,
    ) -> ResultsEvent {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        loop {
            while let Ok(ev) = rx.try_recv() {
                if done(&ev) {
                    return ev;
                }
            }
            assert!(std::time::Instant::now() < deadline, "no results in time");
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn rust_cells_read_calcs_and_calcs_read_them() {
        let Some(base) = cruncha_kernel_rs::Toolchain::default_base().filter(|b| cruncha_kernel_rs::env::ready(b).is_some()) else {
            eprintln!("no Rust toolchain; skipping");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(None).with_rust(base);
        let (tx, rx) = std::sync::mpsc::channel();
        session.on_results(move |ev| {
            let _ = tx.send(ev);
        });
        session.create(&dir.path().join("rust.cruncha")).await.unwrap();
        let mut r = Cell::new(CellKind::Rust, "a2", "let M = (w * L * L / 8.0).to(\"kip*ft\");\nprintln!(\"M = {M}\");");
        r.id = "R".into();
        session
            .commit(
                vec![
                    ChangeView::Upsert { cell: calc("L", "a0", "L = 24 ft", &[], &[]) },
                    ChangeView::Upsert { cell: calc("W", "a1", "w = 1.5 kip/ft", &[], &[]) },
                    ChangeView::Upsert { cell: r },
                    ChangeView::Upsert { cell: calc("C", "a3", "ratio = M / 100 'kip·ft' = ?", &[], &[]) },
                ],
                "rust",
            )
            .await
            .unwrap();
        let cells = session.cells(None).await.unwrap();
        let rc = cells.iter().find(|c| c.id == "R").unwrap();
        assert_eq!((rc.defines.clone(), rc.reads.clone()), (vec!["M".to_string()], vec!["L".to_string(), "w".to_string()]), "names from the scan");
        session.run_python(Some(vec!["R".into()])).await.unwrap();
        let ev = wait_for(&rx, |ev| {
            ev.results.get("R").and_then(|r| r.python.as_ref()).is_some_and(|p| matches!(p.state, crate::engine::PyState::Done | crate::engine::PyState::Error))
        })
        .await;
        let py = ev.results["R"].python.as_ref().unwrap();
        assert_eq!(py.state, crate::engine::PyState::Done, "{:?}", py.error);
        assert_eq!(py.stdout.trim(), "M = 108 kip·ft");
        assert!(ev.results["C"].text.contains("1.08"), "{:?}", ev.results["C"]);
        // Kept with the document like a Python run (ADR-0018).
        assert_eq!(session.flush_results().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn plotly_figures_print_once_the_app_has_drawn_them() {
        use cruncha_report::{Block, Output};
        let Some(base) = python_base() else {
            eprintln!("no Python environment; skipping");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let worker = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../python"));
        let mut session = Session::new(None)
            .with_python(base, worker)
            .with_figure_cache(dir.path().join("figures"));
        let (tx, rx) = std::sync::mpsc::channel();
        session.on_results(move |ev| {
            let _ = tx.send(ev);
        });
        session.create(&dir.path().join("fig.cruncha")).await.unwrap();
        let mut c = Cell::new(CellKind::Python, "a0", "import plotly.graph_objects as go\ngo.Figure(go.Bar(x=[1, 2], y=[3, 4], name='load'))");
        c.id = "FIG".into();
        session.commit(vec![ChangeView::Upsert { cell: c }], "a figure").await.unwrap();
        session.run_python(Some(vec!["FIG".into()])).await.unwrap();
        let ev = wait_for(&rx, |ev| {
            ev.results.get("FIG").and_then(|r| r.python.as_ref()).is_some_and(|p| p.state == crate::engine::PyState::Done)
        })
        .await;
        let py = ev.results["FIG"].python.as_ref().unwrap();
        assert!(matches!(&py.outputs[0], crate::engine::OutputView::Figure { summary, .. } if summary.contains("load (bar, 2 points)")));

        let figure_of = |r: &cruncha_report::Report| {
            r.blocks.iter().find_map(|b| match b {
                Block::Code { outputs, .. } => outputs.iter().find_map(|o| match o {
                    Output::Figure { path, reason, .. } => Some((path.clone(), reason.clone())),
                    _ => None,
                }),
                _ => None,
            })
        };
        // Not drawn yet: the report says so rather than leaving it out.
        let (report, _) = session.report_at(&At::default()).await.unwrap();
        let (path, reason) = figure_of(&report).expect("a figure block");
        assert!(path.is_none() && reason.is_some());
        let jobs = session.figures_to_render(&At::default()).await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].mime, "application/vnd.plotly.v1+json");
        assert!(session.put_figure(&jobs[0].key, "<html/>").is_err());
        session.put_figure(&jobs[0].key, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"/>").unwrap();
        assert!(session.figures_to_render(&At::default()).await.unwrap().is_empty(), "drawn once");
        let (report, files) = session.report_at(&At::default()).await.unwrap();
        let (path, _) = figure_of(&report).unwrap();
        let path = path.expect("the drawn SVG");
        assert!(path.ends_with(".svg") && files.iter().any(|(p, _)| *p == path));
        cruncha_report::render(&report, files).expect("typesets");
        // The agent's view: the summary in the mirror, the ranges in explain.
        let r = crate::agents::result_view_for_tests(&ev.results["FIG"]);
        assert!(r.outputs[0].starts_with("figure (plotly): Figure; 1 trace: load (bar, 2 points)"), "{:?}", r.outputs);
        assert_eq!(r.figure_data, vec!["load (bar, 2 points): x 1 … 2; y 3 … 4".to_string()]);
    }

    #[tokio::test]
    async fn stored_python_results_survive_reopening_and_show_in_history() {
        use cruncha_report::{Block, Status};
        let Some(base) = python_base() else {
            eprintln!("no Python environment; skipping");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let worker = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../python"));
        let mut session = Session::new(None).with_python(base.clone(), worker.clone());
        let (tx, rx) = std::sync::mpsc::channel();
        session.on_results(move |ev| {
            let _ = tx.send(ev);
        });
        let info = session.create_sample(&path).await.unwrap();
        session.results(&At::default()).await.unwrap();
        let state = |r: &HashMap<String, CellResult>| r["PY"].python.as_ref().map(|p| p.state);
        let ev = wait_for(&rx, |ev| state(&ev.results) == Some(crate::engine::PyState::Done)).await;
        assert!(ev.results["PY"].python.as_ref().unwrap().stored.is_none(), "ran here");
        assert_eq!(session.flush_results().await.unwrap(), 1);
        assert!(session.info().await.unwrap().results_bytes > 0);
        let version = info.version;
        session.close().await;

        // Reopened: the cell's inputs are unchanged, so its stored run is used, nothing queued.
        let mut session = Session::new(None).with_python(base, worker);
        session.open(&path).await.unwrap();
        let r = session.results(&At::default()).await.unwrap();
        let py = r["PY"].python.as_ref().unwrap();
        assert_eq!(py.state, crate::engine::PyState::Done, "{py:?}");
        let from = py.stored.as_ref().expect("from stored results");
        assert!(!from.elsewhere);
        assert!(matches!(py.outputs.first(), Some(crate::engine::OutputView::Image { .. })));
        assert!(r["RATIO"].error.is_none(), "calcs read the stored values: {:?}", r["RATIO"]);
        // Once it's a past version, it shows them too, without running anything.
        session
            .commit(vec![ChangeView::Upsert { cell: prose("zz", "a note") }], "note")
            .await
            .unwrap();
        let past = session.results(&At { version: Some(version), ref_name: None }).await.unwrap();
        assert_eq!(past["PY"].python.as_ref().unwrap().state, crate::engine::PyState::Done);
        // …and its report prints them, from the store, with nothing run.
        let (report, files) = session.report_at(&At { version: Some(version), ref_name: None }).await.unwrap();
        assert_eq!(report.provenance.version, version);
        let code = report.blocks.iter().find_map(|b| match b {
            Block::Code { status, outputs, provenance, .. } => Some((status, outputs, provenance)),
            _ => None,
        });
        let (status, outputs, provenance) = code.expect("the Python cell is printed");
        assert_eq!(*status, Status::Ok);
        assert!(!outputs.is_empty() && !files.is_empty(), "its figure is printed");
        assert!(provenance.is_none(), "computed on this computer");

        // Cleared: the past version has nothing to show.
        let before = session.info().await.unwrap().results_bytes;
        let info = session.clear_results().await.unwrap();
        assert!(info.results_bytes < before / 2, "{} of {before} left", info.results_bytes);
        let past = session.results(&At { version: Some(version), ref_name: None }).await.unwrap();
        assert_eq!(past["PY"].python.as_ref().unwrap().state, crate::engine::PyState::Past);
        session.close().await;
    }

    #[tokio::test]
    async fn python_cells_run_in_the_background_and_feed_calcs() {
        let Some(base) = python_base() else {
            eprintln!("no Python environment; skipping");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let worker = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../python"));
        let mut session = Session::new(None).with_python(base, worker);
        let (tx, rx) = std::sync::mpsc::channel();
        session.on_results(move |ev| {
            let _ = tx.send(ev);
        });
        session
            .create_sample(&dir.path().join("beam.cruncha"))
            .await
            .unwrap();
        assert!(session.python_status_full().await.unwrap().has_python);

        // At first the Python cell is queued and the ratio waits for it.
        let r = session.results(&At::default()).await.unwrap();
        assert!(
            r["RATIO"].pending || r["RATIO"].error.is_none(),
            "{:?}",
            r["RATIO"]
        );
        let state = |ev: &ResultsEvent| ev.results["PY"].python.as_ref().map(|p| p.state);
        let ev = wait_for(&rx, |ev| {
            matches!(
                state(ev),
                Some(crate::engine::PyState::Done | crate::engine::PyState::Error)
            )
        })
        .await;
        let py = ev.results["PY"].python.as_ref().unwrap();
        assert_eq!(py.state, crate::engine::PyState::Done, "{py:?}");
        assert!(
            matches!(py.outputs.first(), Some(crate::engine::OutputView::Image { src, .. }) if src.starts_with("data:image/png;base64,"))
        );
        assert_eq!(ev.results["PY"].defines, ["x", "moment", "M_max"]);
        assert!(
            ev.results["RATIO"].text.ends_with("ratio = 1")
                || ev.results["RATIO"].text.ends_with(" 1"),
            "{:?}",
            ev.results["RATIO"]
        );
        let m_max = ev.results["PY"]
            .values
            .iter()
            .find(|v| v.name == "M_max")
            .unwrap();
        assert!(m_max.tex.contains("126.75"), "{m_max:?}");

        // Editing the span re-runs the (approved) Python cell on its own.
        let mut l = session
            .cells(None)
            .await
            .unwrap()
            .into_iter()
            .find(|c| c.id == "L")
            .unwrap();
        l.source = "L = 20 ft".into();
        session
            .commit(vec![ChangeView::Upsert { cell: l }], "span")
            .await
            .unwrap();
        let ev = wait_for(&rx, |ev| {
            ev.results["PY"]
                .values
                .iter()
                .any(|v| v.name == "M_max" && v.tex.contains("75") && !v.tex.contains("126.75"))
        })
        .await;
        assert!(
            ev.results["RATIO"].text.ends_with("1"),
            "{:?}",
            ev.results["RATIO"]
        );

        // An edited Python cell waits for Run; a syntax error is reported without running.
        let mut py = session
            .cells(None)
            .await
            .unwrap()
            .into_iter()
            .find(|c| c.id == "PY")
            .unwrap();
        py.source = "x = (".into();
        let out = session
            .commit(vec![ChangeView::Upsert { cell: py.clone() }], "oops")
            .await
            .unwrap();
        assert!(
            out.results["PY"]
                .error
                .as_deref()
                .unwrap()
                .contains("SyntaxError"),
            "{:?}",
            out.results["PY"]
        );
        py.source = "import time\ntime.sleep(30)\nx = 1".into();
        let out = session
            .commit(vec![ChangeView::Upsert { cell: py }], "slow")
            .await
            .unwrap();
        assert_eq!(
            out.results["PY"].python.as_ref().unwrap().state,
            crate::engine::PyState::Edited
        );
        // Run, then Stop.
        session.run_python(Some(vec!["PY".into()])).await.unwrap();
        wait_for(&rx, |ev| state(ev) == Some(crate::engine::PyState::Running)).await;
        session.stop_python().unwrap();
        wait_for(&rx, |ev| {
            state(ev) == Some(crate::engine::PyState::Interrupted)
        })
        .await;
        session.close().await;
    }

    #[tokio::test]
    async fn agent_proposals_preview_accept_reject_and_go_stale() {
        use cruncha_mcp::Edit;
        use cruncha_store::{ProposalStatus, Provenance};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        let v0 = session.info().await.unwrap().version;
        let m_before = session.results(&At::default()).await.unwrap()["M"].text.clone();

        // A proposal changes nothing, but shows what would change.
        let p = session
            .propose(
                "ai:claude",
                Some(v0),
                "Use a 20 ft span",
                vec![
                    Edit::Modify { id: "L".into(), source: Some("L = 20 ft".into()), description: None },
                    Edit::Insert {
                        after: Some("M".into()),
                        kind: "calc".into(),
                        source: "M_half = M / 2 = ?? kip·ft".into(),
                        description: Some("Half the moment".into()),
                    },
                ],
            )
            .await
            .unwrap();
        assert_eq!(session.info().await.unwrap().version, v0, "proposing commits nothing");
        assert_eq!(p.status, ProposalStatus::Open);
        assert!(!p.stale);
        let m = p.effects.iter().find(|e| e.id == "M").expect("M changes");
        assert_eq!(m.before.as_ref().unwrap().text, m_before);
        assert_ne!(m.after.as_ref().unwrap().text, m_before);
        let new_id = p.changes[1].id.clone();
        let half = p.effects.iter().find(|e| e.id == new_id).expect("the new cell's result");
        assert!(half.before.is_none());
        assert!(half.after.as_ref().unwrap().error.is_none(), "{:?}", half.after);
        // The document's own results are untouched by the preview.
        assert_eq!(session.results(&At::default()).await.unwrap()["M"].text, m_before);

        // Accepting is one commit by the user, carrying the agent as provenance.
        let view = session.accept_proposal(&p.id, false).await.unwrap();
        assert_eq!(view.info.version, v0 + 1);
        assert_ne!(view.results["M"].text, m_before);
        let cells = session.cells(None).await.unwrap();
        let l = cells.iter().find(|c| c.id == "L").unwrap();
        assert_eq!(l.source, "L = 20 ft");
        assert!(matches!(&l.provenance, Provenance::AiAccepted { session, .. } if session == "claude"));
        let new = cells.iter().find(|c| c.id == new_id).unwrap();
        assert_eq!(new.defines, vec!["M_half".to_string()], "names computed on accept");
        assert_eq!(new.attrs["description"], "Half the moment");
        let m_pos = cells.iter().position(|c| c.id == "M").unwrap();
        assert_eq!(cells[m_pos + 1].id, new_id, "inserted right after M");
        let h = session.history().await.unwrap();
        assert!(h[0].message.starts_with("accept proposal: Use a 20 ft span"), "{:?}", h[0]);

        // Duplicate names are refused at propose time.
        let dup = session
            .propose("ai:claude", None, "dup", vec![Edit::Insert {
                after: None,
                kind: "calc".into(),
                source: "L = 30 ft".into(),
                description: None,
            }])
            .await
            .unwrap_err();
        assert!(dup.contains("would break"), "{dup}");
        // A cell changed after the version the agent read is refused.
        let old = session
            .propose("ai:claude", Some(v0), "old", vec![Edit::Delete { id: "L".into() }])
            .await
            .unwrap_err();
        assert!(old.contains("call read_doc again"), "{old}");

        // A proposal goes stale when its cell is edited, and needs force.
        let v1 = session.info().await.unwrap().version;
        let p2 = session
            .propose("ai:claude", Some(v1), "span 22", vec![Edit::Modify {
                id: "L".into(),
                source: Some("L = 22 ft".into()),
                description: None,
            }])
            .await
            .unwrap();
        let mut l = session.cells(None).await.unwrap().into_iter().find(|c| c.id == "L").unwrap();
        l.source = "L = 21 ft".into();
        session.commit(vec![ChangeView::Upsert { cell: l }], "edit").await.unwrap();
        let list = session.proposals().await.unwrap();
        assert!(list.iter().find(|x| x.id == p2.id).unwrap().stale);
        assert!(session.accept_proposal(&p2.id, false).await.unwrap_err().contains("changed since"));
        // Rejecting records the note; withdrawing someone else's is refused.
        assert!(session.withdraw_proposal("ai:other", &p2.id).await.is_err());
        let r = session.reject_proposal(&p2.id, "keep 21 ft").await.unwrap();
        assert_eq!(r.status, ProposalStatus::Rejected);
        assert_eq!(r.decision_note.as_deref(), Some("keep 21 ft"));
        assert!(r.effects.is_empty());
    }

    #[tokio::test]
    async fn document_settings_live_in_the_meta_cell() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(None);
        session.create_sample(&dir.path().join("d.cruncha")).await.unwrap();
        let mut d = session.document_settings().await.unwrap();
        assert_eq!(d.text_sizes, None);
        let v0 = session.info().await.unwrap().version;
        // Unchanged settings commit nothing.
        session.set_document_settings(d.clone()).await.unwrap();
        assert_eq!(session.info().await.unwrap().version, v0);
        d.text_sizes = Some(TextSizes { text: 17.0, calc: 16.0, code: 12.0 });
        d.report.title.project = "Bridge".into();
        session.set_document_settings(d.clone()).await.unwrap();
        assert_eq!(session.info().await.unwrap().version, v0 + 1, "one commit for both");
        assert_eq!(session.document_settings().await.unwrap(), d);
        d.text_sizes = None;
        session.set_document_settings(d.clone()).await.unwrap();
        assert_eq!(session.document_settings().await.unwrap().text_sizes, None);
        assert_eq!(session.report_settings().await.unwrap().title.project, "Bridge");
    }

    #[tokio::test]
    async fn the_sample_document_as_a_report() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        let mut settings = session.report_settings().await.unwrap();
        assert_eq!(settings, ReportSettings::default());
        settings.title.project = "Riverside".into();
        settings.title.checked_by = "C. Checker".into();
        let before = session.info().await.unwrap().version;
        session.set_report_settings(settings.clone()).await.unwrap();
        assert_eq!(session.info().await.unwrap().version, before + 1);
        assert_eq!(session.report_settings().await.unwrap(), settings);

        let (report, files) = session.report_at(&At::default()).await.unwrap();
        assert_eq!(report.title.project, "Riverside");
        assert!(!report.revisions.is_empty(), "the sample's snapshots are revisions");
        use cruncha_report::{Block, Status};
        let calcs: Vec<_> = report
            .blocks
            .iter()
            .filter_map(|b| match b {
                Block::Calc { math, status, check, .. } => Some((math, status, check)),
                _ => None,
            })
            .collect();
        for b in &report.blocks {
            if let Block::Calc { id, math, text, status, .. } = b {
                assert!(!math.is_empty(), "calc {id} has no Typst math: {text} ({status:?})");
            }
        }
        assert!(calcs.iter().any(|(_, s, c)| **c && **s == Status::Pass), "the check passes");
        // Without Python set up, the Python cell (and the calc that reads it) aren't calculated.
        assert!(report.blocks.iter().any(|b| matches!(b,
            Block::Code { status: Status::NotCalculated { .. }, .. })));
        assert!(!report.blocks.iter().any(|b| matches!(b, Block::Other { .. })), "the meta cell isn't printed");
        assert_eq!(report.provenance.content_hash.len(), 64);

        let pdf = cruncha_report::render(&report, files).unwrap();
        assert!(pdf.pages >= 2);
        assert!(pdf.pdf.starts_with(b"%PDF"));
        // A stored result from another computer prints with where it came from.
        let mut stored = report.clone();
        for b in &mut stored.blocks {
            if let Block::Code { provenance, .. } = b {
                *provenance = Some("Computed on office-mac, 2026-11-01; not run on this computer (stored result)".into());
            }
        }
        let a = cruncha_report::render(&stored, Vec::new()).unwrap();
        assert!(a.pdf.len() > pdf.pdf.len() / 2);

        // A past version prints its own title block, and only the snapshots up to it.
        session.snapshot("Rev Q", "").await.unwrap();
        let (old, _) = session.report_at(&At { version: Some(before), ref_name: None }).await.unwrap();
        assert_eq!(old.provenance.version, before);
        assert_eq!(old.title.project, "", "the title block as it was");
        assert!(!old.revisions.iter().any(|r| r.label == "Rev Q"), "Rev Q came later");
        let (rev, _) = session.report_at(&At { version: None, ref_name: Some("Rev Q".into()) }).await.unwrap();
        assert_eq!(rev.title.project, "Riverside");
        assert_eq!(rev.provenance.snapshot.as_deref(), Some("Rev Q"));
        let again = cruncha_report::render(&old, Vec::new()).unwrap();
        assert_eq!(again.pdf, cruncha_report::render(&old, Vec::new()).unwrap().pdf, "same version, same PDF");
    }

    #[tokio::test]
    async fn snapshots_and_restoring_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        let info = session
            .snapshot("  Checked  ", "hand check agrees")
            .await
            .unwrap();
        let r = info.refs.iter().find(|r| r.name == "Checked").unwrap();
        assert_eq!(r.version, 5);
        assert_eq!(r.message, "hand check agrees");
        let dup = session.snapshot("Checked", "").await.unwrap_err();
        assert!(dup.contains("already a snapshot"), "{dup}");
        assert!(session.snapshot(" ", "").await.is_err());

        // Back to Rev A (span 24 ft): a new version, evaluated, and M is as it was then.
        let out = session
            .restore(&At {
                version: None,
                ref_name: Some("Rev A".into()),
            })
            .await
            .unwrap();
        assert_eq!(out.info.version, 6);
        assert!(
            out.results["M"].text.ends_with("108 kip·ft"),
            "{:?}",
            out.results["M"]
        );
        assert!(
            !out.results.contains_key("CHK"),
            "the check came after Rev A"
        );
        let h = session.history().await.unwrap();
        assert_eq!(h[0].message, "restore to “Rev A”");
        // And forward again to the snapshot.
        let out = session
            .restore(&At {
                version: None,
                ref_name: Some("Checked".into()),
            })
            .await
            .unwrap();
        assert!(out.results["M"].text.ends_with("126.75 kip·ft"));
        assert_eq!(out.results["M"].values[0].name, "M");
    }

    #[tokio::test]
    async fn editor_changes_commit_as_one_version_each() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new.cruncha");
        let mut session = Session::new(None);
        let info = session.create(&path).await.unwrap();
        assert_eq!(info.version, 0);
        assert!(session.cells(None).await.unwrap().is_empty());

        let mut c = calc("C1", "a1", "x = 2 m", &[], &[]);
        let p = prose("a0", "Intro");
        let out = session
            .commit(
                vec![
                    ChangeView::Upsert { cell: p.clone() },
                    ChangeView::Upsert { cell: c.clone() },
                ],
                "add intro and x",
            )
            .await
            .unwrap();
        assert_eq!(out.info.version, 1);
        assert_eq!(out.results["C1"].error, None);
        assert!(out.results["C1"].text.contains("2"), "{:?}", out.results);
        // The stored names come from the evaluator, not from the editor.
        let stored = session.cells(None).await.unwrap();
        let c1 = stored.iter().find(|c| c.id == "C1").unwrap();
        assert_eq!(c1.defines, ["x"]);

        c.source = "x = 3 m".into();
        let out = session
            .commit(vec![ChangeView::Upsert { cell: c }], "")
            .await
            .unwrap();
        assert_eq!(out.info.version, 2);
        assert!(out.results["C1"].text.contains("3"), "{:?}", out.results);
        let h = session.history().await.unwrap();
        assert_eq!(h[0].message, "edit");
        assert!(h[0].actor.starts_with("user:"));

        let out = session
            .commit(vec![ChangeView::Delete { id: p.id }], "delete intro")
            .await
            .unwrap();
        assert_eq!(out.info.version, 3);
        assert_eq!(session.cells(None).await.unwrap().len(), 1);
        // An empty change set commits nothing.
        assert_eq!(
            session.commit(vec![], "noop").await.unwrap().info.version,
            3
        );
        // Results of an earlier version.
        let at = At {
            version: Some(1),
            ref_name: None,
        };
        assert_eq!(session.results(&at).await.unwrap()["C1"].error, None);

        // The edited session prunes on close only if a policy says so: keep everything here.
        assert!(session.close().await.is_none());
    }

    #[test]
    fn change_json_from_the_front_end_parses() {
        let json = serde_json::json!([
            { "op": "delete", "id": "A" },
            { "op": "upsert", "cell": {
                "id": "B", "kind": "calc", "order_key": "a0", "parent_id": null,
                "source": "x = 1", "attrs": {}, "defines": [], "reads": [],
                "provenance": { "state": "human" }, "modified_by": "", "modified_at_us": 0
            }}
        ]);
        let v: Vec<ChangeView> = serde_json::from_value(json).unwrap();
        assert!(matches!(&v[0], ChangeView::Delete { id } if id == "A"));
        assert!(matches!(&v[1], ChangeView::Upsert { cell } if cell.kind == CellKind::Calc));
    }

    #[tokio::test]
    async fn several_documents_stay_separate() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Some(dir.path().join("prefs.json")));
        let a = dir.path().join("a.cruncha");
        let b = dir.path().join("b.cruncha");
        let ia = session.create_sample(&a).await.unwrap().id;
        let ib = session.create(&b).await.unwrap().id;
        // The newest is active; both are open, in tab order.
        let docs = session.documents();
        assert_eq!(docs.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), [ia.as_str(), ib.as_str()]);
        assert!(docs[1].active && !docs[0].active);

        // A commit to B (selected by id) leaves A alone, and each has its own results.
        let va = session.on(Some(&ia)).unwrap().info().await.unwrap().version;
        let c = calc("X", "a0", "x = 2 m", &["x"], &[]);
        session.on(Some(&ib)).unwrap().commit(vec![ChangeView::Upsert { cell: c }], "x").await.unwrap();
        assert_eq!(session.on(Some(&ia)).unwrap().info().await.unwrap().version, va);
        let rb = session.on(Some(&ib)).unwrap().results(&At::default()).await.unwrap();
        assert!(rb["X"].text.contains("2 m"), "{:?}", rb["X"]);
        let ra = session.on(Some(&ia)).unwrap().results(&At::default()).await.unwrap();
        assert!(!ra.contains_key("X"));

        // No document named: the active tab.
        session.select(None).unwrap();
        assert_eq!(session.info().await.unwrap().id, ib);
        session.activate(&ia).await.unwrap();
        session.select(None).unwrap();
        assert_eq!(session.info().await.unwrap().id, ia);

        // Opening an open document switches to its tab; a copy of one (same identity) is refused.
        session.activate(&ib).await.unwrap();
        assert_eq!(session.open(&a).await.unwrap().id, ia);
        assert_eq!(session.documents().len(), 2);
        let copy = dir.path().join("copy.cruncha");
        copy_dir(&a, &copy);
        let _ = std::fs::remove_file(copy.join(".lock"));
        assert!(session.open(&copy).await.unwrap_err().contains("same identity"));

        // Closing the active tab activates its neighbour; an unknown id is an error.
        session.select(Some(&ia)).unwrap();
        session.close().await;
        assert_eq!(session.documents().iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), [ib.as_str()]);
        session.select(None).unwrap();
        assert_eq!(session.info().await.unwrap().id, ib);
        assert!(session.select(Some(&ia)).is_err());
        session.close_all().await;
        assert!(session.documents().is_empty() && !session.is_open());
    }

    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for e in std::fs::read_dir(from).unwrap() {
            let e = e.unwrap();
            let target = to.join(e.file_name());
            if e.file_type().unwrap().is_dir() {
                copy_dir(&e.path(), &target);
            } else {
                std::fs::copy(e.path(), target).unwrap();
            }
        }
    }

    #[tokio::test]
    async fn opened_documents_are_remembered_most_recent_first() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Some(dir.path().join("prefs.json")));
        let a = dir.path().join("a.cruncha");
        let b = dir.path().join("b.cruncha");
        session.create(&a).await.unwrap();
        session.create(&b).await.unwrap();
        session.open(&a).await.unwrap();
        let recent = &session.prefs().recent;
        assert_eq!(recent.len(), 2);
        assert!(recent[0].ends_with("a.cruncha") && recent[1].ends_with("b.cruncha"));
        let b_path = recent[1].clone();
        session.forget_recent(&b_path).unwrap();
        let session = Session::new(Some(dir.path().join("prefs.json")));
        assert_eq!(session.prefs().recent.len(), 1);
    }

    #[tokio::test]
    async fn prefs_persist_and_apply_as_the_default() {
        let dir = tempfile::tempdir().unwrap();
        let prefs_path = dir.path().join("config/preferences.json");
        let mut session = Session::new(Some(prefs_path.clone()));
        assert_eq!(session.prefs(), &Prefs::default());
        let policy = Retention {
            saves: Some(500),
            ..Retention::keep_everything()
        };
        session
            .set_prefs(Prefs {
                default_retention: policy.clone(),
                ..Prefs::default()
            })
            .unwrap();
        let session = Session::new(Some(prefs_path));
        assert_eq!(session.prefs().default_retention, policy);
    }

    #[tokio::test]
    async fn document_policy_overrides_default_and_close_prunes_edited_docs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        let tight = Retention {
            saves: Some(1),
            floor: 3, // clamped to the minimum floor of 10
            ..Retention::keep_everything()
        };
        let info = session
            .set_document_retention(Some(tight.clone()))
            .await
            .unwrap();
        assert_eq!(info.effective_retention, tight);
        // Only six versions: the floor of ten keeps them all.
        let p = session.retention_preview(None).await.unwrap();
        assert_eq!(p.cutoff, None);
        assert!(session.close().await.is_none());

        // Grow the history, then closing an edited session applies the policy.
        let mut store = Store::open(&path).await.unwrap();
        for i in 0..20 {
            let c = calc("L", "b2", &format!("L = {} ft", 26 + i), &["L"], &[]);
            store
                .commit(&[Change::Upsert(c)], "user:t", "edit")
                .await
                .unwrap();
        }
        drop(store);
        session.open(&path).await.unwrap();
        let p = session.retention_preview(None).await.unwrap();
        assert!(p.cutoff.is_some());
        assert_eq!(p.refs_to_pin, ["Rev A"]);
        let r = session.prune_now().await.unwrap().unwrap();
        assert_eq!(r.pinned, ["Rev A"]);
        let info = session.info().await.unwrap();
        assert!(info.refs[0].pinned);
        assert!(session.cells_at_ref("Rev A").await.is_ok());
    }

    #[tokio::test]
    async fn archive_the_oldest_groups_from_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        session.close().await;
        let mut store = Store::open(&path).await.unwrap();
        for i in 0..6 {
            let c = calc("L", "b2", &format!("L = {} ft", 26 + i), &["L"], &[]);
            store.commit(&[Change::Upsert(c)], "user:t", "edit").await.unwrap();
        }
        let rev_b = store.tag("Rev B", "issued", "user:t").await.unwrap().doc_version;
        let c = calc("L", "b2", "L = 40 ft", &["L"], &[]);
        store.commit(&[Change::Upsert(c)], "user:t", "edit").await.unwrap();
        drop(store);
        session.open(&path).await.unwrap();

        // The newest group (Rev B on) can't go; everything before it can.
        let p = session.archive_preview(rev_b).await.unwrap();
        assert_eq!(p.snapshots, ["Rev A"]);
        assert!(p.saves >= 6 && p.bytes_freed > 0);
        assert!(session.archive_preview(rev_b + 1).await.is_err());
        let info = session.archive(rev_b).await.unwrap();
        assert_eq!(info.earliest_version, rev_b);
        assert!(info.refs.iter().any(|r| r.name == "Rev A" && r.pinned));
        assert!(session.cells_at_ref("Rev A").await.is_ok());
        let log = session.archived(None).await.unwrap();
        assert_eq!(log.len(), p.saves);
        assert!(log.iter().any(|s| s.actor == "user:t" && s.cells == ["L"]));
        assert!(!session.archived(Some("L")).await.unwrap().is_empty());
    }
}
