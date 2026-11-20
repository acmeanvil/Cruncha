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

use cruncha_store::{
    Cell, CellKind, Change, CommitKind, Error, PruneReport, Ref, Retention, RetentionPreview,
    Store, VersionInfo,
};
use serde::{Deserialize, Serialize};

use crate::engine::{CellResult, Engine};

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
fn local_actor() -> String {
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
    store: Store,
    /// Evaluation state of the latest version (memoized results, current values).
    engine: Engine,
    /// This session committed edits, so the document is due a prune when it closes.
    edited: bool,
    /// Pruned already this session (at most once per session, ADR-0011).
    pruned: bool,
}

pub struct Session {
    doc: Option<OpenDoc>,
    prefs: Prefs,
    prefs_path: Option<PathBuf>,
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
            doc: None,
            prefs,
            prefs_path,
        }
    }

    fn doc(&self) -> Result<&OpenDoc> {
        self.doc.as_ref().ok_or_else(|| "no document is open".into())
    }

    fn doc_mut(&mut self) -> Result<&mut OpenDoc> {
        self.doc.as_mut().ok_or_else(|| "no document is open".into())
    }

    fn effective_retention(&self, store: &Store) -> Retention {
        store
            .retention()
            .cloned()
            .unwrap_or_else(|| self.prefs.default_retention.clone())
    }

    pub async fn open(&mut self, path: &Path) -> Result<DocInfo> {
        self.close().await;
        let store = Store::open(path).await.map_err(err)?;
        self.opened(store, false).await
    }

    /// Hold `store` as the open document and remember it in the recent list.
    async fn opened(&mut self, store: Store, edited: bool) -> Result<DocInfo> {
        let path = store.root().display().to_string();
        self.doc = Some(OpenDoc {
            store,
            engine: Engine::new(),
            edited,
            pruned: false,
        });
        let mut prefs = self.prefs.clone();
        prefs.recent.retain(|p| p != &path);
        prefs.recent.insert(0, path);
        prefs.recent.truncate(MAX_RECENT);
        if let Err(e) = self.set_prefs(prefs) {
            eprintln!("cruncha: could not save the recent-documents list: {e}");
        }
        self.info().await
    }

    /// Forget a recent document (it was moved or deleted).
    pub fn forget_recent(&mut self, path: &str) -> Result<()> {
        let mut prefs = self.prefs.clone();
        prefs.recent.retain(|p| p != path);
        self.set_prefs(prefs)
    }

    /// Create a new, empty document at `path` and open it.
    pub async fn create(&mut self, path: &Path) -> Result<DocInfo> {
        self.close().await;
        let store = Store::create(path, &local_actor()).await.map_err(err)?;
        self.opened(store, false).await
    }

    /// Create a small example worksheet with some history.
    pub async fn create_sample(&mut self, path: &Path) -> Result<DocInfo> {
        self.close().await;
        let store = sample_document(path).await.map_err(err)?;
        self.opened(store, true).await
    }

    /// Close the open document. If this session edited it, apply its retention policy first;
    /// `Store::prune` itself refuses when there were no edits since the last prune.
    pub async fn close(&mut self) -> Option<PruneView> {
        let mut doc = self.doc.take()?;
        if !doc.edited || doc.pruned {
            return None;
        }
        let policy = self.effective_retention(&doc.store);
        match doc.store.prune(&policy, "system").await {
            Ok(r) => r.map(Into::into),
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
            retention: s.retention().cloned(),
            effective_retention: self.effective_retention(s),
            refs,
        })
    }

    pub fn is_open(&self) -> bool {
        self.doc.is_some()
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
        let doc = self.doc_mut()?;
        let changes: Vec<Change> = changes
            .into_iter()
            .map(|c| match c {
                ChangeView::Upsert { mut cell } => {
                    if cell.kind == CellKind::Calc {
                        (cell.defines, cell.reads) = doc.engine.names(&cell.source);
                    }
                    Change::Upsert(cell)
                }
                ChangeView::Delete { id } => Change::Delete(id),
            })
            .collect();
        if !changes.is_empty() {
            let message = if message.trim().is_empty() { "edit" } else { message };
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
            Ok(self.doc_mut()?.engine.run(&cells))
        } else {
            Ok(Engine::new().run(&cells))
        }
    }

    /// Evaluate a calc source while it is being typed, against the latest values (nothing is
    /// committed).
    pub fn preview_calc(&mut self, source: &str) -> Result<CellResult> {
        Ok(self.doc_mut()?.engine.preview(source))
    }

    pub async fn history(&self) -> Result<Vec<VersionView>> {
        let h = self.doc()?.store.history().await.map_err(err)?;
        Ok(h.into_iter().map(Into::into).collect())
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
        self.doc_mut()?
            .store
            .set_retention(policy)
            .map_err(err)?;
        self.info().await
    }

    /// What pruning the open document under `policy` (default: its effective policy) would do.
    pub async fn retention_preview(&self, policy: Option<Retention>) -> Result<PreviewView> {
        let s = &self.doc()?.store;
        let policy = policy.unwrap_or_else(|| self.effective_retention(s));
        Ok(s.retention_preview(&policy).await.map_err(err)?.into())
    }

    /// Apply the open document's effective policy now (the user confirmed the preview).
    pub async fn prune_now(&mut self) -> Result<Option<PruneView>> {
        let policy = self.effective_retention(&self.doc()?.store);
        let doc = self.doc_mut()?;
        let r = doc.store.prune(&policy, "system").await.map_err(err)?;
        if r.is_some() {
            doc.pruned = true;
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
    let m = described(calc(
        "M",
        "b3",
        "M = w L^2 / 8 = ?? kip·ft",
        &["M"],
        &["w", "L"],
    ), "Maximum bending moment, at midspan");
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
    let mut py = Cell::new(
        CellKind::Python,
        "c0",
        "import numpy as np\nx = np.linspace(0, L, 50)\nmoment = w * x * (L - x) / 2",
    );
    py.reads = vec!["w".into(), "L".into()];
    py.defines = vec!["x".into(), "moment".into()];
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
        assert_eq!(latest.len(), 12);
        // The sample evaluates: M = 1.5 kip/ft × (26 ft)² / 8, and its check passes.
        let r = session.results(&At::default()).await.unwrap();
        assert!(r["M"].text.ends_with("126.75 kip·ft"), "{:?}", r["M"]);
        assert_eq!(r["CHK"].error, None, "{:?}", r["CHK"]);
        let at_v1 = At { version: Some(1), ref_name: None };
        assert!(session.results(&at_v1).await.unwrap()["M"].text.ends_with("86.4 kip·ft"));
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

    #[tokio::test]
    async fn snapshots_and_restoring_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("beam.cruncha");
        let mut session = Session::new(None);
        session.create_sample(&path).await.unwrap();
        let info = session.snapshot("  Checked  ", "hand check agrees").await.unwrap();
        let r = info.refs.iter().find(|r| r.name == "Checked").unwrap();
        assert_eq!(r.version, 5);
        assert_eq!(r.message, "hand check agrees");
        let dup = session.snapshot("Checked", "").await.unwrap_err();
        assert!(dup.contains("already a snapshot"), "{dup}");
        assert!(session.snapshot(" ", "").await.is_err());

        // Back to Rev A (span 24 ft): a new version, evaluated, and M is as it was then.
        let out = session
            .restore(&At { version: None, ref_name: Some("Rev A".into()) })
            .await
            .unwrap();
        assert_eq!(out.info.version, 6);
        assert!(out.results["M"].text.ends_with("108 kip·ft"), "{:?}", out.results["M"]);
        assert!(!out.results.contains_key("CHK"), "the check came after Rev A");
        let h = session.history().await.unwrap();
        assert_eq!(h[0].message, "restore to “Rev A”");
        // And forward again to the snapshot.
        let out = session
            .restore(&At { version: None, ref_name: Some("Checked".into()) })
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
        assert_eq!(session.commit(vec![], "noop").await.unwrap().info.version, 3);
        // Results of an earlier version.
        let at = At { version: Some(1), ref_name: None };
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
            store.commit(&[Change::Upsert(c)], "user:t", "edit").await.unwrap();
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
}
