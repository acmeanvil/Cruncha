// SPDX-License-Identifier: GPL-3.0-or-later
//! The on-disk container: a directory of Delta tables.
//!
//! ```text
//! name.cruncha/
//! ├── cruncha.json   container manifest (format version, id)
//! ├── .lock          single-writer lock (held while a Store is open for writing)
//! ├── doc/           Delta table: one row per cell; each commit = one document version
//! └── refs/          Delta table: named versions (append-only)
//! ```
//!
//! Every [`Store::commit`] is one atomic Delta transaction, so there is no "save": a crash
//! loses at most the changes not yet handed to `commit`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use deltalake::datafusion::prelude::{SessionContext, col, lit};
use deltalake::kernel::StructType;
use deltalake::kernel::engine::arrow_conversion::TryIntoKernel as _;
use deltalake::kernel::transaction::CommitProperties;
use deltalake::protocol::SaveMode;
use deltalake::{DeltaTable, checkpoints};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

use crate::cell::{Cell, CellDiff, Change};
use crate::error::{Error, Result};
use crate::schema::{self, DOC_COLUMNS, OP_COLUMN, OP_DELETE, OP_UPSERT, Ref};

pub const FORMAT: &str = "cruncha-container";
pub const FORMAT_VERSION: u32 = 1;
const MANIFEST: &str = "cruncha.json";
const LOCK: &str = ".lock";

/// Commit-metadata keys written into Delta's `commitInfo`.
pub const META_ACTOR: &str = "cruncha.actor";
pub const META_MESSAGE: &str = "cruncha.message";
pub const META_KIND: &str = "cruncha.kind";
pub const META_RESTORED_FROM: &str = "cruncha.restoredFrom";

/// Table properties for every container table.
///
/// Delta's defaults expire log entries after 30 days, which silently destroys time travel.
/// Cruncha keeps full history unless the user explicitly compacts it.
const TABLE_PROPERTIES: [(&str, &str); 3] = [
    ("delta.enableExpiredLogCleanup", "false"),
    ("delta.logRetentionDuration", "interval 36500 days"),
    ("delta.deletedFileRetentionDuration", "interval 36500 days"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    pub id: String,
    pub created_at_us: i64,
}

/// What a commit was for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitKind {
    /// Created the table.
    Create,
    /// A user/AI/CLI edit.
    Edit,
    /// A rollback to an earlier version (recorded as a new version).
    Restore,
    /// Compaction or other maintenance: content unchanged.
    Maintenance,
}

/// One entry of the document history.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: u64,
    pub timestamp_ms: i64,
    pub kind: CommitKind,
    pub actor: String,
    pub message: String,
    pub restored_from: Option<u64>,
}

/// Result of [`Store::compact`].
#[derive(Debug, Clone, Default)]
pub struct CompactStats {
    pub files_removed: u64,
    pub files_added: u64,
    pub checkpointed_version: u64,
}

/// An open container.
pub struct Store {
    root: PathBuf,
    manifest: Manifest,
    doc: DeltaTable,
    refs: DeltaTable,
    ctx: SessionContext,
    /// Held for the life of the store; released on drop.
    _lock: File,
}

fn now_us() -> i64 {
    chrono::Utc::now().timestamp_micros()
}

fn dir_url(path: &Path) -> Result<Url> {
    Url::from_directory_path(path)
        .map_err(|_| Error::Corrupt(format!("cannot form a URL for {}", path.display())))
}

fn acquire_lock(root: &Path) -> Result<File> {
    let f = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join(LOCK))?;
    match f.try_lock() {
        Ok(()) => Ok(f),
        Err(fs::TryLockError::WouldBlock) => Err(Error::Locked(root.to_owned())),
        Err(fs::TryLockError::Error(e)) => Err(e.into()),
    }
}

fn commit_props(
    kind: CommitKind,
    actor: &str,
    message: &str,
    extra: Option<(&str, Value)>,
) -> CommitProperties {
    let mut meta = vec![
        (META_KIND.to_owned(), json!(kind)),
        (META_ACTOR.to_owned(), json!(actor)),
        (META_MESSAGE.to_owned(), json!(message)),
    ];
    if let Some((k, v)) = extra {
        meta.push((k.to_owned(), v));
    }
    CommitProperties::default().with_metadata(meta)
}

async fn create_table(
    path: &Path,
    name: &str,
    schema: &deltalake::arrow::datatypes::Schema,
    actor: &str,
) -> Result<DeltaTable> {
    fs::create_dir_all(path)?;
    let kernel: StructType = schema.try_into_kernel()?;
    let table = DeltaTable::try_from_url(dir_url(path)?).await?;
    let table = table
        .create()
        .with_table_name(name)
        .with_columns(kernel.fields().cloned())
        .with_configuration(TABLE_PROPERTIES.iter().map(|(k, v)| (*k, Some(*v))))
        .with_commit_properties(commit_props(CommitKind::Create, actor, "create", None))
        .await?;
    Ok(table)
}

async fn collect_cells(table: DeltaTable) -> Result<Vec<Cell>> {
    let (_, stream) = table.scan_table().await?;
    let batches: Vec<_> = stream.try_collect().await?;
    let mut cells = Vec::new();
    for b in &batches {
        schema::batch_to_cells(b, &mut cells)?;
    }
    cells.sort_by(|a, b| a.order_key.cmp(&b.order_key).then_with(|| a.id.cmp(&b.id)));
    Ok(cells)
}

impl Store {
    /// Create a new, empty container at `root` (which must not exist) and open it for writing.
    pub async fn create(root: impl AsRef<Path>, actor: &str) -> Result<Store> {
        let root = root.as_ref();
        if root.exists() {
            return Err(Error::AlreadyExists(root.to_owned()));
        }
        fs::create_dir_all(root)?;
        let root = root.canonicalize()?;
        let lock = acquire_lock(&root)?;

        let doc = create_table(&root.join("doc"), "doc", &schema::DOC_SCHEMA, actor).await?;
        let refs = create_table(&root.join("refs"), "refs", &schema::REFS_SCHEMA, actor).await?;

        // The manifest is written last, so a half-created directory is never mistaken for a
        // valid container.
        let manifest = Manifest {
            format: FORMAT.into(),
            format_version: FORMAT_VERSION,
            id: ulid::Ulid::generate().to_string(),
            created_at_us: now_us(),
        };
        let tmp = root.join(format!("{MANIFEST}.tmp"));
        fs::write(&tmp, serde_json::to_vec_pretty(&manifest)?)?;
        fs::rename(&tmp, root.join(MANIFEST))?;

        Ok(Store {
            root,
            manifest,
            doc,
            refs,
            ctx: SessionContext::new(),
            _lock: lock,
        })
    }

    /// Open an existing container for writing. Fails with [`Error::Locked`] if another
    /// process (or another `Store` in this process) has it open.
    pub async fn open(root: impl AsRef<Path>) -> Result<Store> {
        let root = root.as_ref().canonicalize()?;
        let manifest_path = root.join(MANIFEST);
        if !manifest_path.is_file() {
            return Err(Error::NotAContainer(root));
        }
        let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        if manifest.format != FORMAT {
            return Err(Error::NotAContainer(root));
        }
        if manifest.format_version > FORMAT_VERSION {
            return Err(Error::UnsupportedFormat(manifest.format_version));
        }
        let lock = acquire_lock(&root)?;
        let doc = deltalake::open_table(dir_url(&root.join("doc"))?).await?;
        let refs = deltalake::open_table(dir_url(&root.join("refs"))?).await?;
        Ok(Store {
            root,
            manifest,
            doc,
            refs,
            ctx: SessionContext::new(),
            _lock: lock,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The current document version.
    pub fn version(&self) -> u64 {
        self.doc.version().unwrap_or(0)
    }

    /// Atomically apply `changes` as one new document version and return that version.
    ///
    /// An empty change set is a no-op and returns the current version.
    pub async fn commit(&mut self, changes: &[Change], actor: &str, message: &str) -> Result<u64> {
        self.commit_with(changes, actor, message, CommitKind::Edit, None)
            .await
    }

    async fn commit_with(
        &mut self,
        changes: &[Change],
        actor: &str,
        message: &str,
        kind: CommitKind,
        extra: Option<(&str, Value)>,
    ) -> Result<u64> {
        if changes.is_empty() {
            return Ok(self.version());
        }
        let mut seen = HashSet::with_capacity(changes.len());
        for c in changes {
            if !seen.insert(c.id()) {
                return Err(Error::DuplicateId(c.id().to_owned()));
            }
        }

        let batch = schema::changes_to_batch(changes, actor, now_us())?;
        let source = self.ctx.read_batch(batch)?;
        let s = |c: &str| col(format!("s.{c}"));

        let (table, _metrics) = self
            .doc
            .clone()
            .merge(source, col("t.id").eq(col("s.id")))
            .with_source_alias("s")
            .with_target_alias("t")
            .with_commit_properties(commit_props(kind, actor, message, extra))
            .when_matched_delete(|d| d.predicate(s(OP_COLUMN).eq(lit(OP_DELETE))))?
            .when_matched_update(|mut u| {
                u = u.predicate(s(OP_COLUMN).eq(lit(OP_UPSERT)));
                for c in &DOC_COLUMNS[1..] {
                    u = u.update(*c, s(c));
                }
                u
            })?
            .when_not_matched_insert(|mut i| {
                i = i.predicate(s(OP_COLUMN).eq(lit(OP_UPSERT)));
                for c in DOC_COLUMNS {
                    i = i.set(c, s(c));
                }
                i
            })?
            .await?;
        self.doc = table;
        Ok(self.version())
    }

    /// All cells of the current version, in document order.
    pub async fn cells(&self) -> Result<Vec<Cell>> {
        collect_cells(self.doc.clone()).await
    }

    /// All cells as of `version`, in document order (time travel).
    pub async fn cells_at(&self, version: u64) -> Result<Vec<Cell>> {
        if version > self.version() {
            return Err(Error::NoSuchVersion(version));
        }
        if version == self.version() {
            return self.cells().await;
        }
        let mut t = self.doc.clone();
        t.load_version(version).await?;
        collect_cells(t).await
    }

    /// Document history, newest first, including maintenance commits.
    pub async fn history(&self) -> Result<Vec<VersionInfo>> {
        let infos: Vec<_> = self.doc.history(None).try_collect().await?;
        let latest = self.version();
        // `history` yields one commitInfo per version, newest first. Expired-log cleanup is
        // disabled on container tables, so every version is present.
        if infos.len() as u64 != latest + 1 {
            return Err(Error::Corrupt(format!(
                "history has {} entries but latest version is {latest}",
                infos.len()
            )));
        }
        let get_str = |info: &HashMap<String, Value>, k: &str| {
            info.get(k)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        Ok(infos
            .into_iter()
            .enumerate()
            .map(|(i, ci)| {
                let kind = ci
                    .info
                    .get(META_KIND)
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or(match ci.operation.as_deref() {
                        Some("CREATE TABLE") => CommitKind::Create,
                        _ => CommitKind::Maintenance,
                    });
                VersionInfo {
                    version: latest - i as u64,
                    timestamp_ms: ci.timestamp.unwrap_or_default(),
                    kind,
                    actor: get_str(&ci.info, META_ACTOR),
                    message: get_str(&ci.info, META_MESSAGE),
                    restored_from: ci.info.get(META_RESTORED_FROM).and_then(Value::as_u64),
                }
            })
            .collect())
    }

    /// Per-cell differences going from version `from` to version `to`.
    pub async fn diff(&self, from: u64, to: u64) -> Result<Vec<CellDiff>> {
        let before = self.cells_at(from).await?;
        let after = self.cells_at(to).await?;
        Ok(diff_cells(before, after))
    }

    /// Roll the document back to `version` by committing a *new* version with that content.
    /// History is never rewritten, so a restore can itself be undone.
    pub async fn restore(&mut self, version: u64, actor: &str) -> Result<u64> {
        let changes: Vec<Change> = self
            .diff(self.version(), version)
            .await?
            .into_iter()
            .map(|d| match d {
                CellDiff::Added(c) | CellDiff::Modified { after: c, .. } => Change::Upsert(c),
                CellDiff::Removed(c) => Change::Delete(c.id),
            })
            .collect();
        if changes.is_empty() {
            return Ok(self.version());
        }
        self.commit_with(
            &changes,
            actor,
            &format!("restore to version {version}"),
            CommitKind::Restore,
            Some((META_RESTORED_FROM, json!(version))),
        )
        .await
    }

    /// Name the current version (e.g. "Issued for review, rev A").
    pub async fn tag(&mut self, name: &str, message: &str, actor: &str) -> Result<Ref> {
        if self.refs().await?.iter().any(|r| r.name == name) {
            return Err(Error::RefExists(name.to_owned()));
        }
        let r = Ref {
            name: name.to_owned(),
            doc_version: self.version(),
            message: message.to_owned(),
            actor: actor.to_owned(),
            created_at_us: now_us(),
        };
        let table = self
            .refs
            .clone()
            .write([schema::ref_to_batch(&r)?])
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit_props(
                CommitKind::Edit,
                actor,
                &format!("tag {name}"),
                None,
            ))
            .await?;
        self.refs = table;
        Ok(r)
    }

    /// All named versions, oldest first.
    pub async fn refs(&self) -> Result<Vec<Ref>> {
        let (_, stream) = self.refs.clone().scan_table().await?;
        let batches: Vec<_> = stream.try_collect().await?;
        let mut out = Vec::new();
        for b in &batches {
            schema::batch_to_refs(b, &mut out)?;
        }
        out.sort_by_key(|r| (r.created_at_us, r.doc_version));
        Ok(out)
    }

    /// Cells of a named version.
    pub async fn cells_at_ref(&self, name: &str) -> Result<Vec<Cell>> {
        let r = self
            .refs()
            .await?
            .into_iter()
            .find(|r| r.name == name)
            .ok_or_else(|| Error::NoSuchRef(name.to_owned()))?;
        self.cells_at(r.doc_version).await
    }

    /// Compact the `doc` table's small files and write a checkpoint. Content and history are
    /// unchanged (old files are kept for time travel); reads of the latest version get faster.
    pub async fn compact(&mut self, actor: &str) -> Result<CompactStats> {
        let (table, metrics) = self
            .doc
            .clone()
            .optimize()
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                "compact",
                None,
            ))
            .await?;
        self.doc = table;
        checkpoints::create_checkpoint(&self.doc, None).await?;
        Ok(CompactStats {
            files_removed: metrics.num_files_removed,
            files_added: metrics.num_files_added,
            checkpointed_version: self.version(),
        })
    }
}

/// Per-cell differences between two cell lists (matched by id).
pub fn diff_cells(before: Vec<Cell>, after: Vec<Cell>) -> Vec<CellDiff> {
    let mut before: BTreeMap<String, Cell> =
        before.into_iter().map(|c| (c.id.clone(), c)).collect();
    let mut out = Vec::new();
    for a in after {
        match before.remove(&a.id) {
            None => out.push(CellDiff::Added(a)),
            Some(b) if !b.same_content(&a) => out.push(CellDiff::Modified {
                before: b,
                after: a,
            }),
            Some(_) => {}
        }
    }
    out.extend(before.into_values().map(CellDiff::Removed));
    out
}
