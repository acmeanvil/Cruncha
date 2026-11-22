// SPDX-License-Identifier: GPL-3.0-or-later
//! The on-disk container: a directory of Delta tables.
//!
//! ```text
//! name.cruncha/
//! ├── cruncha.json   container manifest (format version, id)
//! ├── .lock          single-writer lock (held while a Store is open for writing)
//! ├── doc/           Delta table: append-only log of cell revisions; each commit = one version
//! ├── refs/          Delta table: named versions (append-only)
//! ├── pins/          Delta table: copies of named versions whose history was pruned (ADR-0011;
//! │                  created by the first prune that needs it)
//! ├── blobs/         large cell sources, zstd-compressed, named by SHA-256 (ADR-0013)
//! ├── proposals/     Delta table: agents' proposed changes and their decisions (ADR-0016;
//! │                  created by the first proposal)
//! ├── results/       Delta table: stored runs of code cells, keyed by their inputs (ADR-0018;
//! │                  created by the first stored run)
//! └── archive/       Delta table: the save log, one row per save a prune or archive removed
//!                    (ADR-0023; created by the first one)
//! ```
//!
//! Every [`Store::commit`] is one atomic Delta transaction, so there is no "save": a crash
//! loses at most the changes not yet handed to `commit`.
//!
//! `doc/` layout (format 2, ADR-0010): a commit appends only the changed cells' rows, tagged
//! with `rev` (the Delta version being written) and a `deleted` tombstone flag. Reading any
//! version reduces the log to the highest-`rev` row per cell. Every [`DEFAULT_SQUASH_EVERY`]
//! commits the table is *squashed* (overwritten with just the current rows) to keep reads
//! fast; old files are never deleted, so every earlier version stays readable.
//!
//! Large sources are out of line (format 3, ADR-0013): a row stores `source_hash` and the
//! text lives once in `blobs/`, so squashes, restores and pins don't copy it again.
//!
//! History is kept forever unless the document has a retention policy (ADR-0011), and even then
//! only [`Store::prune`] deletes anything, only after new edits, and never by the calendar.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use deltalake::kernel::StructType;
use deltalake::kernel::engine::arrow_conversion::TryIntoKernel as _;
use deltalake::kernel::transaction::CommitProperties;
use deltalake::operations::vacuum::{Clock, VacuumMode};
use deltalake::protocol::SaveMode;
use deltalake::{DeltaTable, DeltaTableBuilder, checkpoints};

use crate::logindex::LogIndexStore;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

use crate::archive::{self, ArchivedSave};
use crate::blobs::{self, Blobs};
use crate::cell::{Cell, CellDiff, Change};
use crate::error::{Error, Result};
use crate::proposals::{self, Proposal, ProposalStatus, ProposedChange};
use crate::results::{self, PutReport, RunEntry, RunRecord};
use crate::retention::{self, LogEntry, Retention};
use crate::schema::{self, Ref, SOURCE_HASH_COLUMN};

pub const FORMAT: &str = "cruncha-container";
/// 1: `doc/` is one row per cell, updated by MERGE. 2: `doc/` is a revision log (ADR-0010).
/// 3: large sources live in `blobs/`, referenced by `source_hash` (ADR-0013).
/// 4: agents' proposals may live in `proposals/` (ADR-0016); an older Cruncha would ignore
/// them, so it must refuse the container.
/// 5: code-cell results may live in `results/` (ADR-0018), referencing blobs an older Cruncha's
/// prune would delete.
/// 6: prunes and archives record every save they remove in `archive/` (ADR-0023); an older
/// Cruncha would prune without that record, so it must refuse the container.
pub const FORMAT_VERSION: u32 = 6;
/// Squash the revision log after this many commits (spike S3b: 50 balances commit latency,
/// read latency and growth).
pub const DEFAULT_SQUASH_EVERY: u64 = 50;
const MANIFEST: &str = "cruncha.json";
const LOCK: &str = ".lock";

/// Commit-metadata keys written into Delta's `commitInfo`.
pub const META_ACTOR: &str = "cruncha.actor";
pub const META_MESSAGE: &str = "cruncha.message";
pub const META_KIND: &str = "cruncha.kind";
pub const META_RESTORED_FROM: &str = "cruncha.restoredFrom";
/// Cruncha's commit time, microseconds since the epoch. Activity-relative retention uses it.
pub const META_AT: &str = "cruncha.at";
/// On a prune's marker commit: the first version still available.
pub const META_PRUNED_BEFORE: &str = "cruncha.prunedBefore";
/// Bytes of new blobs the commit's rows reference (counted toward the size limit, ADR-0011).
pub const META_BLOB_BYTES: &str = "cruncha.blobBytes";
/// On the commit that accepted a proposal: its id (ADR-0016).
pub const META_PROPOSAL: &str = "cruncha.proposal";

/// Table properties for every container table.
///
/// Delta's defaults expire log entries after 30 days, which silently destroys time travel.
/// Cruncha keeps full history; only [`Store::prune`] removes any, with explicit parameters
/// (ADR-0011). Never derive these properties from a retention policy.
///
/// Column statistics are kept only for the first column (`id` / `name`): statistics on the
/// text columns cost about 1 KiB of log per commit and never help a query (spike S3b).
const TABLE_PROPERTIES: [(&str, &str); 4] = [
    ("delta.enableExpiredLogCleanup", "false"),
    ("delta.logRetentionDuration", "interval 36500 days"),
    ("delta.deletedFileRetentionDuration", "interval 36500 days"),
    ("delta.dataSkippingNumIndexedCols", "1"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    pub id: String,
    pub created_at_us: i64,
    /// The document's retention policy (ADR-0011). `None` means "use the app default".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention: Option<Retention>,
    /// The oldest version still available. Versions before it were pruned.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub earliest_version: u64,
}

fn is_zero(v: &u64) -> bool {
    *v == 0
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
    /// Set on a prune's marker commit: versions before this one were pruned.
    pub pruned_before: Option<u64>,
}

/// What [`Store::prune`] did.
#[derive(Debug, Clone)]
pub struct PruneReport {
    /// The new earliest available version.
    pub pruned_before: u64,
    /// Refs copied into `pins/` because their version was pruned.
    pub pinned: Vec<String>,
    pub data_files_deleted: usize,
    pub log_files_deleted: usize,
    /// Blobs no remaining version or pin references.
    pub blobs_deleted: usize,
    /// Decided proposals older than the cutoff (ADR-0016).
    pub proposals_deleted: usize,
}

/// What [`Store::archive_before`] would do, without deleting anything.
#[derive(Debug, Clone)]
pub struct ArchivePreview {
    /// Versions from this one up to `cutoff` would be archived.
    pub from_version: u64,
    pub cutoff: u64,
    /// Saves (edits, restores, accepted proposals) that would go to the save log.
    pub saves: usize,
    /// Estimated bytes freed (data files no longer needed, plus log entries).
    pub bytes_freed: u64,
    /// Snapshots archived (kept in `pins/`), oldest first.
    pub snapshots: Vec<Ref>,
}

/// What [`Store::prune`] would do under a policy, without deleting anything.
#[derive(Debug, Clone)]
pub struct RetentionPreview {
    pub earliest_version: u64,
    /// Versions before this one would be pruned. `None`: nothing would be.
    pub cutoff: Option<u64>,
    /// When the cutoff version was committed.
    pub cutoff_timestamp_ms: Option<i64>,
    /// Estimated bytes freed (data files no longer needed, plus log entries).
    pub bytes_freed: u64,
    /// Refs that would be copied into `pins/`.
    pub refs_to_pin: Vec<Ref>,
    /// No edits since the last prune, so `prune` won't act until the document is edited.
    pub waits_for_edit: bool,
}

/// Result of [`Store::compact`].
#[derive(Debug, Clone, Default)]
pub struct CompactStats {
    /// Live `doc` data files before and after compaction.
    pub files_before: u64,
    pub files_after: u64,
    pub checkpointed_version: u64,
}

/// An open container.
pub struct Store {
    root: PathBuf,
    manifest: Manifest,
    doc: DeltaTable,
    refs: DeltaTable,
    /// `pins/`, once a prune has created it.
    pins: Option<DeltaTable>,
    /// `proposals/`, once an agent has proposed something.
    proposals: Option<DeltaTable>,
    /// `results/`, once a run has been stored.
    results: Option<DeltaTable>,
    /// `archive/`, once a prune or archive has removed a save (ADR-0023).
    archive: Option<DeltaTable>,
    /// Blobs the live `results/` rows reference (counted apart from history).
    result_blobs: HashSet<String>,
    /// Added to the clock for Cruncha's own timestamps. Tests use it to simulate time passing.
    clock_offset_us: i64,
    /// Out-of-line sources.
    blobs: Blobs,
    /// Sources at least this many bytes go to `blobs/`.
    blob_threshold: usize,
    /// Edit commits since the last squash.
    since_squash: u64,
    squash_every: u64,
    /// The file system, with indexed `_delta_log` listings (ADR-0021).
    storage: Arc<LogIndexStore>,
    /// Held for the life of the store; released on drop.
    _lock: File,
}

fn now_us() -> i64 {
    chrono::Utc::now().timestamp_micros()
}

/// A fixed "now" for VACUUM, so its threshold is exactly the one computed.
#[derive(Debug)]
struct FixedClock(i64);

impl Clock for FixedClock {
    fn current_timestamp_millis(&self) -> i64 {
        self.0
    }
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
    at_us: i64,
    extra: &[(&str, Value)],
) -> CommitProperties {
    let mut meta = vec![
        (META_KIND.to_owned(), json!(kind)),
        (META_ACTOR.to_owned(), json!(actor)),
        (META_MESSAGE.to_owned(), json!(message)),
        (META_AT.to_owned(), json!(at_us)),
    ];
    for (k, v) in extra {
        meta.push(((*k).to_owned(), v.clone()));
    }
    CommitProperties::default().with_metadata(meta)
}

/// A container's storage: the local file system with indexed `_delta_log` listings.
fn new_storage() -> Arc<LogIndexStore> {
    Arc::new(LogIndexStore::new(Arc::new(deltalake::logstore::object_store::local::LocalFileSystem::new())))
}

/// The table at `path`, through `storage`.
async fn open_table(path: &Path, storage: &Arc<LogIndexStore>) -> Result<DeltaTable> {
    storage.register(path).map_err(|e| Error::Corrupt(e.to_string()))?;
    let url = dir_url(path)?;
    Ok(DeltaTableBuilder::from_url(url.clone())?.with_storage_backend(storage.clone(), url).load().await?)
}

async fn create_table(
    path: &Path,
    name: &str,
    schema: &deltalake::arrow::datatypes::Schema,
    actor: &str,
    storage: &Arc<LogIndexStore>,
) -> Result<DeltaTable> {
    fs::create_dir_all(path)?;
    storage.register(path).map_err(|e| Error::Corrupt(e.to_string()))?;
    let kernel: StructType = schema.try_into_kernel()?;
    let url = dir_url(path)?;
    let table = DeltaTableBuilder::from_url(url.clone())?.with_storage_backend(storage.clone(), url).build()?;
    let table = table
        .create()
        .with_table_name(name)
        .with_columns(kernel.fields().cloned())
        .with_configuration(TABLE_PROPERTIES.iter().map(|(k, v)| (*k, Some(*v))))
        .with_commit_properties(commit_props(
            CommitKind::Create,
            actor,
            "create",
            now_us(),
            &[],
        ))
        .await?;
    Ok(table)
}

async fn scan(table: &DeltaTable) -> Result<Vec<deltalake::arrow::array::RecordBatch>> {
    let (_, stream) = table.clone().scan_table().await?;
    Ok(stream.try_collect().await?)
}

fn live_files(table: &DeltaTable) -> Result<u64> {
    Ok(table.get_file_uris()?.count() as u64)
}

fn sort_cells(cells: &mut [Cell]) {
    cells.sort_by(|a, b| a.order_key.cmp(&b.order_key).then_with(|| a.id.cmp(&b.id)));
}

/// Every blob hash referenced by a data file under `dir` (recursively, skipping
/// `_delta_log`). Reads all files on disk, not just the live ones, so the result covers
/// every version still available.
fn referenced_blobs(dir: &Path, out: &mut HashSet<String>) -> Result<()> {
    use deltalake::parquet::arrow::ProjectionMask;
    use deltalake::parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    let Ok(rd) = fs::read_dir(dir) else {
        return Ok(());
    };
    for e in rd {
        let e = e?;
        let path = e.path();
        if e.file_type()?.is_dir() {
            if e.file_name() != "_delta_log" {
                referenced_blobs(&path, out)?;
            }
            continue;
        }
        if path.extension().is_none_or(|x| x != "parquet") {
            continue;
        }
        let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(&path)?)
            .map_err(|e| Error::Corrupt(format!("{}: {e}", path.display())))?;
        let schema = builder.parquet_schema();
        let Some(idx) = (0..schema.num_columns()).find(|&i| {
            let col = schema.column(i);
            let parts = col.path().parts();
            parts == [SOURCE_HASH_COLUMN.to_owned()] || parts == [results::BLOBS_COLUMN.to_owned()]
        }) else {
            continue; // written before format 3: no out-of-line sources
        };
        let mask = ProjectionMask::leaves(schema, [idx]);
        let reader = builder
            .with_projection(mask)
            .build()
            .map_err(|e| Error::Corrupt(format!("{}: {e}", path.display())))?;
        for batch in reader {
            let batch = batch?;
            if let Some(col) = batch.column_by_name(results::BLOBS_COLUMN) {
                let col = deltalake::arrow::compute::cast(col, &deltalake::arrow::datatypes::DataType::Utf8)?;
                let col = col
                    .as_any()
                    .downcast_ref::<deltalake::arrow::array::StringArray>()
                    .ok_or_else(|| Error::Corrupt("results blobs column mistyped".into()))?;
                for v in col.iter().flatten() {
                    out.extend(v.split_whitespace().map(str::to_owned));
                }
            } else {
                out.extend(schema::blob_hashes(&batch)?);
            }
        }
    }
    Ok(())
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

        let storage = new_storage();
        let doc = create_table(&root.join("doc"), "doc", &schema::DOC_LOG_SCHEMA, actor, &storage).await?;
        let refs = create_table(&root.join("refs"), "refs", &schema::REFS_SCHEMA, actor, &storage).await?;

        // The manifest is written last, so a half-created directory is never mistaken for a
        // valid container.
        let manifest = Manifest {
            format: FORMAT.into(),
            format_version: FORMAT_VERSION,
            id: ulid::Ulid::generate().to_string(),
            created_at_us: now_us(),
            retention: None,
            earliest_version: 0,
        };
        let tmp = root.join(format!("{MANIFEST}.tmp"));
        fs::write(&tmp, serde_json::to_vec_pretty(&manifest)?)?;
        fs::rename(&tmp, root.join(MANIFEST))?;

        Ok(Store {
            blobs: Blobs::new(&root),
            root,
            manifest,
            doc,
            refs,
            pins: None,
            proposals: None,
            results: None,
            archive: None,
            result_blobs: HashSet::new(),
            clock_offset_us: 0,
            blob_threshold: blobs::DEFAULT_BLOB_THRESHOLD,
            since_squash: 0,
            squash_every: DEFAULT_SQUASH_EVERY,
            storage,
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
        let storage = new_storage();
        let doc = open_table(&root.join("doc"), &storage).await?;
        let refs = open_table(&root.join("refs"), &storage).await?;
        let pins_dir = root.join("pins");
        let pins = if pins_dir.join("_delta_log").is_dir() {
            Some(open_table(&pins_dir, &storage).await?)
        } else {
            None
        };
        let proposals_dir = root.join("proposals");
        let proposals = if proposals_dir.join("_delta_log").is_dir() {
            Some(open_table(&proposals_dir, &storage).await?)
        } else {
            None
        };
        let results_dir = root.join("results");
        let results = if results_dir.join("_delta_log").is_dir() {
            Some(open_table(&results_dir, &storage).await?)
        } else {
            None
        };
        let archive_dir = root.join("archive");
        let archive = if archive_dir.join("_delta_log").is_dir() {
            Some(open_table(&archive_dir, &storage).await?)
        } else {
            None
        };
        // Every append adds one data file and a squash leaves at most one, so the live file
        // count recovers the squash counter across sessions.
        let since_squash = live_files(&doc)?.saturating_sub(1);
        let mut store = Store {
            blobs: Blobs::new(&root),
            blob_threshold: blobs::DEFAULT_BLOB_THRESHOLD,
            root,
            manifest,
            doc,
            refs,
            pins,
            proposals,
            results,
            archive,
            result_blobs: HashSet::new(),
            clock_offset_us: 0,
            since_squash,
            squash_every: DEFAULT_SQUASH_EVERY,
            storage,
            _lock: lock,
        };
        if store.manifest.format_version < FORMAT_VERSION {
            store.migrate().await?;
        }
        store.recover_accepted().await?;
        if let Some(t) = store.results.clone() {
            // `base_hash` (ADR-0019) came after the table; older ones gain it, still format 5.
            let field = deltalake::arrow::datatypes::Field::new(
                results::BASE_HASH_COLUMN,
                deltalake::arrow::datatypes::DataType::Utf8,
                true,
            );
            store.results = Some(store.add_missing_columns(t, vec![field], "add base_hash to results").await?);
        }
        store.result_blobs = store
            .result_rows()
            .await?
            .into_iter()
            .flat_map(|r| r.blobs)
            .collect();
        Ok(store)
    }

    /// Bring an older container up to [`FORMAT_VERSION`] by adding the nullable columns it
    /// lacks: `rev`/`deleted` (format 2) and `source_hash` (format 3). Older rows read as
    /// rev -1, not deleted, source inline, so no data is rewritten and all history stays
    /// readable. One maintenance commit per table at most; idempotent if interrupted before
    /// the manifest update.
    async fn migrate(&mut self) -> Result<()> {
        let from = self.manifest.format_version;
        let message = format!("migrate container to format {FORMAT_VERSION}");
        let mut wanted = schema::log_extra_fields();
        wanted.extend(schema::blob_fields());
        self.doc = self.add_missing_columns(self.doc.clone(), wanted, &message).await?;
        if let Some(pins) = self.pins.clone() {
            self.pins = Some(
                self.add_missing_columns(pins, schema::blob_fields(), &message)
                    .await?,
            );
        }
        debug_assert!(from < FORMAT_VERSION);
        self.manifest.format_version = FORMAT_VERSION;
        self.write_manifest()?;
        Ok(())
    }

    async fn add_missing_columns(
        &self,
        table: DeltaTable,
        wanted: Vec<deltalake::arrow::datatypes::Field>,
        message: &str,
    ) -> Result<DeltaTable> {
        let snapshot = table.snapshot()?;
        let have: HashSet<String> = snapshot
            .schema()
            .fields()
            .map(|f| f.name().to_owned())
            .collect();
        let missing: Vec<_> = wanted
            .into_iter()
            .filter(|f| !have.contains(f.name()))
            .collect();
        if missing.is_empty() {
            return Ok(table);
        }
        let extra = deltalake::arrow::datatypes::Schema::new(missing);
        let kernel: StructType = (&extra).try_into_kernel()?;
        Ok(table
            .add_columns()
            .with_fields(kernel.fields().cloned().collect::<Vec<_>>())
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                "system",
                message,
                self.now_us(),
                &[],
            ))
            .await?)
    }

    fn write_manifest(&self) -> Result<()> {
        let tmp = self.root.join(format!("{MANIFEST}.tmp"));
        fs::write(&tmp, serde_json::to_vec_pretty(&self.manifest)?)?;
        fs::rename(&tmp, self.root.join(MANIFEST))?;
        Ok(())
    }

    /// Cruncha's clock: the wall clock plus the test offset.
    fn now_us(&self) -> i64 {
        now_us() + self.clock_offset_us
    }

    /// Shift Cruncha's own timestamps (`cruncha.at`, cell `modified_at`) by `offset_us`.
    /// For tests that simulate time passing; Delta's timestamps are unaffected.
    #[doc(hidden)]
    pub fn set_clock_offset_us(&mut self, offset_us: i64) {
        self.clock_offset_us = offset_us;
    }

    /// Squash the revision log every `n` edit commits (minimum 1).
    pub fn set_squash_interval(&mut self, n: u64) {
        self.squash_every = n.max(1);
    }

    /// Store sources of at least `bytes` out of line, in `blobs/` (ADR-0013). `usize::MAX`
    /// keeps every source inline. Affects later commits only; both kinds of row read the same.
    pub fn set_blob_threshold(&mut self, bytes: usize) {
        self.blob_threshold = bytes;
    }

    /// The cells of `table` at its loaded version: the reduced revision log, with out-of-line
    /// sources resolved, in document order.
    async fn read_cells(&self, table: &DeltaTable) -> Result<Vec<Cell>> {
        let current = schema::reduce_log(&scan(table).await?)?;
        self.decode(&[current]).await
    }

    /// Decode `doc` or `pins` rows, loading their blobs.
    async fn decode(&self, batches: &[deltalake::arrow::array::RecordBatch]) -> Result<Vec<Cell>> {
        let mut hashes = Vec::new();
        for b in batches {
            hashes.extend(schema::blob_hashes(b)?);
        }
        let blobs = self.blobs.get_many(hashes.iter().map(String::as_str)).await?;
        let mut cells = Vec::with_capacity(batches.iter().map(|b| b.num_rows()).sum());
        for b in batches {
            schema::batch_to_cells_with(b, &blobs, &mut cells)?;
        }
        sort_cells(&mut cells);
        Ok(cells)
    }

    /// Write the large sources among `cells` to `blobs/`. Returns each cell's hash (`None`:
    /// inline) and the bytes newly written.
    async fn externalize<'a>(
        &self,
        cells: impl Iterator<Item = Option<&'a Cell>>,
    ) -> Result<(Vec<Option<String>>, u64)> {
        let large: Vec<Option<&str>> = cells
            .map(|c| c.map(|c| c.source.as_str()).filter(|s| s.len() >= self.blob_threshold))
            .collect();
        let sources: Vec<String> = large.iter().flatten().map(|s| (*s).to_owned()).collect();
        let mut stored = self.blobs.put_many(sources).await?.into_iter();
        let mut written = 0;
        let hashes = large
            .iter()
            .map(|s| {
                s.map(|_| {
                    let (h, n) = stored.next().expect("one result per source");
                    written += n;
                    h
                })
            })
            .collect();
        Ok((hashes, written))
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
        // Layout seam (ADR-0013): with the revision log (ADR-0010) a commit appends the
        // changed rows and `maintain` squashes now and then. A deletion-vector layout (S3
        // option C) would replace these two steps with a MERGE on `id`; the reader,
        // `reduce_log`, already handles a table holding one live row per id.
        if changes.is_empty() {
            return Ok(self.version());
        }
        let mut seen = HashSet::with_capacity(changes.len());
        for c in changes {
            if !seen.insert(c.id()) {
                return Err(Error::DuplicateId(c.id().to_owned()));
            }
        }

        // The single-writer lock means the next Delta version is known before committing.
        let rev = self.version() + 1;
        let at = self.now_us();
        // Blobs first: a crash between here and the commit leaves only unreferenced blobs.
        let (hashes, blob_bytes) = self
            .externalize(changes.iter().map(|c| match c {
                Change::Upsert(cell) => Some(cell),
                Change::Delete(_) => None,
            }))
            .await?;
        let batch = schema::changes_to_log_batch(changes, &hashes, actor, at, rev as i64)?;
        let mut meta: Vec<(&str, Value)> = extra.into_iter().collect();
        if blob_bytes > 0 {
            meta.push((META_BLOB_BYTES, json!(blob_bytes)));
        }
        let table = self
            .doc
            .clone()
            .write([batch])
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit_props(kind, actor, message, at, &meta))
            .await?;
        self.doc = table;
        if self.version() != rev {
            // Only possible if something other than this Store wrote to `doc/`: the rows just
            // written carry the wrong `rev`, so stop before compounding it.
            return Err(Error::Corrupt(format!(
                "expected to commit version {rev}, committed {}; another writer touched doc/",
                self.version()
            )));
        }
        self.maintain().await?;
        Ok(rev)
    }

    /// Per-commit upkeep of the layout: the revision log squashes every `squash_every` edits.
    async fn maintain(&mut self) -> Result<()> {
        self.since_squash += 1;
        if self.since_squash >= self.squash_every {
            self.squash("system", "squash", &[]).await?;
        }
        Ok(())
    }

    /// Overwrite `doc/` with just the current rows (one Delta version, content unchanged).
    /// The replaced files stay on disk, so earlier versions remain readable.
    async fn squash(&mut self, actor: &str, message: &str, extra: &[(&str, Value)]) -> Result<()> {
        let current = schema::reduce_log(&scan(&self.doc).await?)?;
        let table = self
            .doc
            .clone()
            .write([current])
            .with_save_mode(SaveMode::Overwrite)
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                message,
                self.now_us(),
                extra,
            ))
            .await?;
        self.doc = table;
        self.since_squash = 0;
        Ok(())
    }

    /// All cells of the current version, in document order.
    pub async fn cells(&self) -> Result<Vec<Cell>> {
        self.read_cells(&self.doc).await
    }

    /// All cells as of `version`, in document order (time travel).
    pub async fn cells_at(&self, version: u64) -> Result<Vec<Cell>> {
        if version > self.version() {
            return Err(Error::NoSuchVersion(version));
        }
        if version < self.earliest_version() {
            return Err(Error::Pruned {
                earliest: self.earliest_version(),
            });
        }
        if version == self.version() {
            return self.cells().await;
        }
        let mut t = self.doc.clone();
        t.load_version(version).await?;
        self.read_cells(&t).await
    }

    /// Document history, newest first, including maintenance commits, back to
    /// [`Store::earliest_version`].
    pub async fn history(&self) -> Result<Vec<VersionInfo>> {
        let get_str = |e: &LogEntry, k: &str| {
            e.info
                .get(k)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        Ok(self
            .log_entries()
            .await?
            .into_iter()
            .rev()
            .map(|e| VersionInfo {
                version: e.version,
                timestamp_ms: e.at_ms,
                kind: e.kind,
                actor: get_str(&e, META_ACTOR),
                message: get_str(&e, META_MESSAGE),
                restored_from: e.info.get(META_RESTORED_FROM).and_then(Value::as_u64),
                pruned_before: e.pruned_before(),
            })
            .collect())
    }

    /// Every available commit of `doc/`, oldest first. Expired-log cleanup is disabled on
    /// container tables, so every version from the earliest on is present.
    async fn log_entries(&self) -> Result<Vec<LogEntry>> {
        retention::read_log(
            &self.root.join("doc"),
            self.earliest_version(),
            self.version(),
        )
        .await
    }

    /// The oldest version still available (0 unless history has been pruned).
    pub fn earliest_version(&self) -> u64 {
        self.manifest.earliest_version
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
        let target = self.cells_at(version).await?;
        self.restore_to(target, version, &format!("restore to version {version}"), actor)
            .await
    }

    /// Roll the document back to the named version `name`, as [`Store::restore`] does. This
    /// works after pruning too: a named version's cells are kept in `pins/`.
    pub async fn restore_ref(&mut self, name: &str, actor: &str) -> Result<u64> {
        let r = self
            .refs()
            .await?
            .into_iter()
            .find(|r| r.name == name)
            .ok_or_else(|| Error::NoSuchRef(name.to_owned()))?;
        let target = self.cells_at_ref(name).await?;
        self.restore_to(target, r.doc_version, &format!("restore to “{name}”"), actor)
            .await
    }

    async fn restore_to(&mut self, target: Vec<Cell>, version: u64, message: &str, actor: &str) -> Result<u64> {
        let changes: Vec<Change> = diff_cells(self.cells().await?, target)
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
            message,
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
            created_at_us: self.now_us(),
        };
        let at = self.now_us();
        let table = self
            .refs
            .clone()
            .write([schema::ref_to_batch(&r)?])
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit_props(
                CommitKind::Edit,
                actor,
                &format!("tag {name}"),
                at,
                &[],
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

    /// Cells of a named version. Named versions survive pruning: once their version is gone
    /// they're read from `pins/`.
    pub async fn cells_at_ref(&self, name: &str) -> Result<Vec<Cell>> {
        let r = self
            .refs()
            .await?
            .into_iter()
            .find(|r| r.name == name)
            .ok_or_else(|| Error::NoSuchRef(name.to_owned()))?;
        if r.doc_version >= self.earliest_version() {
            return self.cells_at(r.doc_version).await;
        }
        let mut rows = Vec::new();
        let names = self.read_pins(Some(name), &mut rows).await?;
        if !names.contains(name) {
            return Err(Error::Corrupt(format!(
                "ref {name} (version {}) was pruned without a pin",
                r.doc_version
            )));
        }
        self.decode(&rows).await
    }

    async fn read_pins(
        &self,
        wanted: Option<&str>,
        rows: &mut Vec<deltalake::arrow::array::RecordBatch>,
    ) -> Result<std::collections::BTreeSet<String>> {
        match &self.pins {
            None => Ok(Default::default()),
            Some(t) => schema::read_pins(&scan(t).await?, wanted, rows),
        }
    }

    /// The document's own retention policy; `None` means "use the app default".
    pub fn retention(&self) -> Option<&Retention> {
        self.manifest.retention.as_ref()
    }

    /// Set (or, with `None`, clear) the document's retention policy. Nothing is deleted until
    /// the next [`Store::prune`], which runs only after new edits.
    pub fn set_retention(&mut self, policy: Option<Retention>) -> Result<()> {
        self.manifest.retention = policy;
        self.write_manifest()
    }

    /// Bytes of history on disk: `doc/` data and log, and blobs (pins and stored results
    /// excluded).
    pub fn history_size_bytes(&self) -> Result<u64> {
        let all = blobs::dir_size(&self.root.join("doc"))? + self.blobs.size_bytes()?;
        Ok(all.saturating_sub(self.blobs.size_of(&self.result_blobs)))
    }

    /// Bytes of stored results on disk: `results/` and the blobs its rows reference.
    pub fn results_size_bytes(&self) -> Result<u64> {
        Ok(blobs::dir_size(&self.root.join("results"))? + self.blobs.size_of(&self.result_blobs))
    }

    /// What [`Store::prune`] would do under `policy`, without deleting anything. The UI shows
    /// it before a stricter policy takes effect.
    pub async fn retention_preview(&self, policy: &Retention) -> Result<RetentionPreview> {
        let entries = self.log_entries().await?;
        let (_, edits) = prune_state(&entries);
        let cutoff = retention::cutoff(policy, &entries);
        let mut preview = RetentionPreview {
            earliest_version: self.earliest_version(),
            cutoff,
            cutoff_timestamp_ms: None,
            bytes_freed: 0,
            refs_to_pin: Vec::new(),
            waits_for_edit: !edits,
        };
        if let Some(c) = cutoff {
            preview.cutoff_timestamp_ms = entries.iter().find(|e| e.version == c).map(|e| e.at_ms);
            preview.bytes_freed = entries
                .iter()
                .take_while(|e| e.version < c)
                .map(|e| e.log_bytes + e.removed_bytes)
                .sum();
            let pinned = self.read_pins(None, &mut Vec::new()).await?;
            preview.refs_to_pin = self
                .refs()
                .await?
                .into_iter()
                .filter(|r| r.doc_version < c && !pinned.contains(&r.name))
                .collect();
        }
        Ok(preview)
    }

    /// Apply a retention policy (ADR-0011): make versions older than the policy allows
    /// unavailable and delete what only they needed. **Irreversible.**
    ///
    /// Acts only when there have been edits since the last prune, so a document nobody edits
    /// never loses history, however much time passes. Refs older than the cutoff are copied
    /// into `pins/` first. Delta's retention properties are never touched; the deletion steps
    /// get explicit parameters computed from the cutoff. A crash at any step leaves a readable
    /// container, and the next call finishes the job.
    ///
    /// Returns `None` when there was nothing to do.
    pub async fn prune(&mut self, policy: &Retention, actor: &str) -> Result<Option<PruneReport>> {
        let entries = self.log_entries().await?;
        let earliest = self.earliest_version();
        let (last_marker, edits) = prune_state(&entries);
        // An earlier prune moved `earliest` but crashed before writing its marker.
        let unfinished = earliest > last_marker.unwrap_or(0);
        let computed = if edits {
            retention::cutoff(policy, &entries)
        } else {
            None
        };
        let Some(cutoff) = computed.or(unfinished.then_some(earliest)) else {
            return Ok(None);
        };
        let date = entries
            .iter()
            .find(|e| e.version == cutoff)
            .and_then(|e| chrono::DateTime::from_timestamp_millis(e.at_ms))
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        let message = format!("history before version {cutoff} ({date}) pruned; policy: {policy}");
        self.prune_to(cutoff, &entries, actor, &message).await.map(Some)
    }

    /// Archive the oldest snapshot groups (ADR-0023): every version before `cutoff`, which
    /// must be a snapshot's version, becomes unavailable. Snapshots before it are kept in
    /// `pins/` and every removed save in the save log ([`Store::archived`]). **Irreversible.**
    ///
    /// Unlike [`Store::prune`] it is the user's explicit choice, so it needs no new edits and
    /// no retention floor applies. The newest snapshot group can't be archived: `cutoff` is at
    /// most the latest snapshot's version.
    pub async fn archive_before(&mut self, cutoff: u64, actor: &str) -> Result<PruneReport> {
        let (refs, entries) = self.check_archive_cutoff(cutoff).await?;
        let first = self.earliest_version();
        let upto = refs
            .iter()
            .filter(|r| r.doc_version < cutoff)
            .map(|r| format!("“{}”", r.name))
            .next_back();
        let message = match upto {
            Some(name) => format!("archived versions {first}–{} (up to {name})", cutoff - 1),
            None => format!("archived versions {first}–{} (draft)", cutoff - 1),
        };
        self.prune_to(cutoff, &entries, actor, &message).await
    }

    /// What [`Store::archive_before`] would do with `cutoff`, without deleting anything.
    pub async fn archive_preview(&self, cutoff: u64) -> Result<ArchivePreview> {
        let (refs, entries) = self.check_archive_cutoff(cutoff).await?;
        Ok(ArchivePreview {
            from_version: self.earliest_version(),
            cutoff,
            saves: entries.iter().filter(|e| e.version < cutoff && e.kind != CommitKind::Maintenance).count(),
            bytes_freed: entries
                .iter()
                .take_while(|e| e.version < cutoff)
                .map(|e| e.log_bytes + e.removed_bytes)
                .sum(),
            snapshots: refs.into_iter().filter(|r| r.doc_version < cutoff).collect(),
        })
    }

    /// The refs and log entries, after checking that `cutoff` starts a snapshot group after
    /// the earliest available version.
    async fn check_archive_cutoff(&self, cutoff: u64) -> Result<(Vec<Ref>, Vec<LogEntry>)> {
        let refs = self.refs().await?;
        let bad = |why: &str| Err(Error::BadArchive(format!("version {cutoff}: {why}")));
        if cutoff <= self.earliest_version() {
            return bad("nothing before it is still available");
        }
        if cutoff > self.version() || !refs.iter().any(|r| r.doc_version == cutoff) {
            return bad("an archive must end where a snapshot starts the next group");
        }
        Ok((refs, self.log_entries().await?))
    }

    /// The save log (ADR-0023): every save a prune or archive removed, oldest first; with
    /// `cell`, only the saves that wrote that cell.
    pub async fn archived(&self, cell: Option<&str>) -> Result<Vec<ArchivedSave>> {
        let rows = match &self.archive {
            None => Vec::new(),
            Some(t) => archive::batches_to_rows(&scan(t).await?)?,
        };
        Ok(match cell {
            None => rows,
            Some(id) => rows.into_iter().filter(|r| r.cells.iter().any(|c| c == id)).collect(),
        })
    }

    /// Make every version before `cutoff` unavailable (ADR-0011 §3, ADR-0023). Crash-safe:
    /// a later [`Store::prune`] finishes an interrupted run.
    async fn prune_to(&mut self, cutoff: u64, entries: &[LogEntry], actor: &str, message: &str) -> Result<PruneReport> {
        let entries = entries.to_vec();

        // 1. Named versions survive: copy any that are about to go into `pins/`.
        let pinned = self.pin_refs_before(cutoff, actor).await?;

        // 1b. The save log: a row for every save about to go (ADR-0023).
        self.log_saves_before(cutoff, &entries, actor).await?;

        // 2. A checkpoint at the cutoff, so every version from it on stays readable once the
        //    older log is gone.
        let doc_url = dir_url(&self.root.join("doc"))?;
        checkpoints::create_checkpoint_from_table_url_and_cleanup(
            doc_url,
            cutoff,
            Some(false),
            None,
        )
        .await?;
        // That wrote the log through its own storage: read the directories again.
        self.storage.forget();

        // 3. The point of no return: from here on, versions before the cutoff are reported as
        //    pruned even if the deletions below are interrupted.
        if self.manifest.earliest_version != cutoff {
            self.manifest.earliest_version = cutoff;
            self.write_manifest()?;
        }

        // 4. Delete data files that only pruned versions need. A file removed by a commit
        //    after the cutoff was removed after every version from the cutoff on was written,
        //    so a threshold at the oldest of those commit times spares it.
        let keep_from_ms = entries
            .iter()
            .filter(|e| e.version >= cutoff)
            .map(|e| e.delta_ts_ms)
            .min()
            .unwrap_or(0);
        let now_ms = chrono::Utc::now().timestamp_millis();
        let (table, metrics) = self
            .doc
            .clone()
            .vacuum()
            .with_mode(VacuumMode::Lite)
            .with_retention_period(chrono::Duration::milliseconds(
                (now_ms - keep_from_ms).max(0),
            ))
            .with_enforce_retention_duration(false)
            .with_clock(Arc::new(FixedClock(now_ms)))
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                "vacuum pruned history",
                self.now_us(),
                &[],
            ))
            .await?;
        self.doc = table;

        // 5. Expire log entries before the cutoff (aligned to the checkpoint from step 2).
        let log_files_deleted = checkpoints::cleanup_expired_logs_for(
            cutoff,
            self.doc.log_store().as_ref(),
            now_ms + 1000,
            None,
        )
        .await?;

        // 6. Delete blobs that no remaining data file (of any available version), pin or
        //    stored result references.
        let blobs_deleted = self.gc_blobs().await?;

        // 6b. Decided proposals older than the cutoff go too (ADR-0016); those accepted into a
        //     pinned version stay with the pin.
        let cutoff_us = entries
            .iter()
            .find(|e| e.version == cutoff)
            .map(|e| e.at_ms * 1000)
            .unwrap_or(i64::MIN);
        let keep_upto = self
            .refs()
            .await?
            .iter()
            .filter(|r| r.doc_version < cutoff)
            .map(|r| r.doc_version)
            .max();
        let proposals_deleted = self.prune_proposals(cutoff_us, keep_upto, actor).await?;

        // 7. The marker commit, so history shows where it now starts.
        self.squash(actor, message, &[(META_PRUNED_BEFORE, json!(cutoff))])
            .await?;
        checkpoints::create_checkpoint(&self.doc, None).await?;

        Ok(PruneReport {
            pruned_before: cutoff,
            pinned,
            data_files_deleted: metrics.files_deleted.len(),
            log_files_deleted,
            blobs_deleted,
            proposals_deleted,
        })
    }

    /// Append a save-log row for every save before `cutoff` not logged yet.
    async fn log_saves_before(&mut self, cutoff: u64, entries: &[LogEntry], actor: &str) -> Result<()> {
        let logged: HashSet<u64> = self.archived(None).await?.into_iter().map(|r| r.version).collect();
        let saves: Vec<&LogEntry> = entries
            .iter()
            .filter(|e| e.version < cutoff && e.kind != CommitKind::Maintenance && !logged.contains(&e.version))
            .collect();
        if saves.is_empty() {
            return Ok(());
        }
        let cells = self.cells_written(entries, saves[0].version, cutoff).await?;
        let refs = self.refs().await?;
        let rows: Vec<ArchivedSave> = saves
            .iter()
            .map(|e| {
                let text = |k: &str| e.info.get(k).and_then(Value::as_str).unwrap_or_default().to_owned();
                let kind = if e.info.contains_key(META_PROPOSAL) {
                    "proposal"
                } else {
                    match e.kind {
                        CommitKind::Create => "create",
                        CommitKind::Restore => "restore",
                        _ => "edit",
                    }
                };
                // The group: the latest snapshot at or before the save (the first one taken
                // there, as the History tab names it).
                let snapshot = refs
                    .iter()
                    .filter(|r| r.doc_version <= e.version)
                    .max_by(|a, b| a.doc_version.cmp(&b.doc_version).then(b.created_at_us.cmp(&a.created_at_us)))
                    .map(|r| r.name.clone());
                ArchivedSave {
                    version: e.version,
                    at_us: e.info.get(META_AT).and_then(Value::as_i64).unwrap_or(e.at_ms * 1000),
                    actor: text(META_ACTOR),
                    message: text(META_MESSAGE),
                    kind: kind.into(),
                    snapshot,
                    cells: cells.get(&e.version).cloned().unwrap_or_default(),
                    removed_before: cutoff,
                }
            })
            .collect();
        let table = match &self.archive {
            Some(t) => t.clone(),
            None => {
                let dir = self.root.join("archive");
                if dir.join("_delta_log").is_dir() {
                    open_table(&dir, &self.storage).await?
                } else {
                    create_table(&dir, "archive", &archive::ARCHIVE_SCHEMA, actor, &self.storage).await?
                }
            }
        };
        let table = table
            .write([archive::rows_to_batch(&rows)?])
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                &format!("log {} save(s) before version {cutoff}", rows.len()),
                self.now_us(),
                &[],
            ))
            .await?;
        self.archive = Some(table);
        Ok(())
    }

    /// The cells each version in `from..to` wrote: its rows (`rev` = the version). Between
    /// two maintenance commits the log is only appended to, so one scan at the end of each
    /// such window sees every row written in it (a squash is maintenance and starts a new
    /// window). About one table load per 50 versions.
    async fn cells_written(&self, entries: &[LogEntry], from: u64, to: u64) -> Result<HashMap<u64, Vec<String>>> {
        use deltalake::arrow::array::{Array, Int64Array, StringArray};
        let mut ends: Vec<u64> = entries
            .iter()
            .filter(|e| e.kind == CommitKind::Maintenance && e.version > from && e.version < to)
            .map(|e| e.version - 1)
            .collect();
        ends.push(to - 1);
        ends.dedup();
        let mut out: HashMap<u64, std::collections::BTreeSet<String>> = HashMap::new();
        let mut lo = from;
        for end in ends {
            if end >= lo {
                let mut t = self.doc.clone();
                t.load_version(end).await?;
                for b in scan(&t).await? {
                    // Scans may return other string types: conform to the log schema first.
                    let b = schema::conform_filling(&b, &schema::DOC_LOG_SCHEMA)?;
                    let ids = b.column_by_name("id").and_then(|c| c.as_any().downcast_ref::<StringArray>());
                    let revs = b
                        .column_by_name(schema::REV_COLUMN)
                        .and_then(|c| c.as_any().downcast_ref::<Int64Array>());
                    let (Some(ids), Some(revs)) = (ids, revs) else {
                        return Err(Error::Corrupt("doc table has no id or rev column".into()));
                    };
                    for i in 0..b.num_rows() {
                        if revs.is_valid(i) && revs.value(i) >= lo as i64 && revs.value(i) <= end as i64 {
                            out.entry(revs.value(i) as u64).or_default().insert(ids.value(i).to_owned());
                        }
                    }
                }
            }
            lo = end + 2;
        }
        Ok(out.into_iter().map(|(v, ids)| (v, ids.into_iter().collect())).collect())
    }

    /// Copy every ref older than `cutoff` that isn't pinned yet into `pins/`.
    async fn pin_refs_before(&mut self, cutoff: u64, actor: &str) -> Result<Vec<String>> {
        let have = self.read_pins(None, &mut Vec::new()).await?;
        let need: Vec<Ref> = self
            .refs()
            .await?
            .into_iter()
            .filter(|r| r.doc_version < cutoff && !have.contains(&r.name))
            .collect();
        if need.is_empty() {
            return Ok(Vec::new());
        }
        let mut batches = Vec::with_capacity(need.len());
        for r in &need {
            let cells = self.cells_at(r.doc_version).await?;
            let (hashes, _) = self.externalize(cells.iter().map(Some)).await?;
            batches.push(schema::cells_to_pin_batch(&r.name, &cells, &hashes)?);
        }
        let table = match &self.pins {
            Some(t) => t.clone(),
            None => {
                let dir = self.root.join("pins");
                if dir.join("_delta_log").is_dir() {
                    open_table(&dir, &self.storage).await?
                } else {
                    create_table(&dir, "pins", &schema::PINS_SCHEMA, actor, &self.storage).await?
                }
            }
        };
        let table = table
            .write(batches)
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                "pin refs before pruning",
                self.now_us(),
                &[],
            ))
            .await?;
        self.pins = Some(table);
        Ok(need.into_iter().map(|r| r.name).collect())
    }

    /// Squash the revision log now and write a checkpoint. Content and history are unchanged
    /// (old files are kept for time travel); reads of the latest version get faster.
    pub async fn compact(&mut self, actor: &str) -> Result<CompactStats> {
        let files_before = live_files(&self.doc)?;
        self.squash(actor, "squash", &[]).await?;
        checkpoints::create_checkpoint(&self.doc, None).await?;
        Ok(CompactStats {
            files_before,
            files_after: live_files(&self.doc)?,
            checkpointed_version: self.version(),
        })
    }
}

impl Store {
    // ---- Proposals (ADR-0016) ----

    /// The revision (`rev`) of every current cell: what a proposal's `base_rev` is checked
    /// against. Rows written before format 2 count as revision 0.
    pub async fn cell_revs(&self) -> Result<HashMap<String, u64>> {
        use deltalake::arrow::array::{Array, Int64Array, StringArray};
        let current = schema::reduce_log(&scan(&self.doc).await?)?;
        let ids = current
            .column_by_name("id")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| Error::Corrupt("doc table has no id column".into()))?;
        let revs = current
            .column_by_name(schema::REV_COLUMN)
            .and_then(|c| c.as_any().downcast_ref::<Int64Array>())
            .ok_or_else(|| Error::Corrupt("doc table has no rev column".into()))?;
        Ok((0..current.num_rows())
            .map(|i| {
                let rev = if revs.is_valid(i) { revs.value(i).max(0) as u64 } else { 0 };
                (ids.value(i).to_owned(), rev)
            })
            .collect())
    }

    async fn proposal_rows(&self) -> Result<Vec<proposals::Row>> {
        match &self.proposals {
            None => Ok(Vec::new()),
            Some(t) => proposals::batches_to_rows(&scan(t).await?),
        }
    }

    /// Every proposal the document has (open and decided), oldest first.
    pub async fn proposals(&self) -> Result<Vec<Proposal>> {
        proposals::fold(self.proposal_rows().await?)
    }

    /// One proposal.
    pub async fn proposal(&self, id: &str) -> Result<Proposal> {
        self.proposals()
            .await?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| Error::NoSuchProposal(id.to_owned()))
    }

    async fn append_proposal_rows(&mut self, rows: &[proposals::Row], message: &str) -> Result<()> {
        let actor = rows.first().map(|r| r.actor.clone()).unwrap_or_default();
        let table = match &self.proposals {
            Some(t) => t.clone(),
            None => {
                let dir = self.root.join("proposals");
                if dir.join("_delta_log").is_dir() {
                    open_table(&dir, &self.storage).await?
                } else {
                    create_table(&dir, "proposals", &proposals::PROPOSALS_SCHEMA, &actor, &self.storage).await?
                }
            }
        };
        let table = table
            .write([proposals::rows_to_batch(rows)?])
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit_props(
                CommitKind::Edit,
                &actor,
                message,
                self.now_us(),
                &[],
            ))
            .await?;
        self.proposals = Some(table);
        Ok(())
    }

    /// Record a proposal against the current version. The document itself is unchanged: the
    /// changes are applied only when a human accepts ([`Store::accept`]). Validation (that
    /// the cells exist, names and cycles) is the caller's.
    pub async fn propose(
        &mut self,
        changes: Vec<ProposedChange>,
        message: &str,
        actor: &str,
    ) -> Result<Proposal> {
        let id = ulid::Ulid::generate().to_string();
        let row = proposals::Row {
            proposal_id: id.clone(),
            seq: 0,
            status: ProposalStatus::Open,
            actor: actor.to_owned(),
            message: Some(message.to_owned()),
            base_version: Some(self.version()),
            changes: Some(serde_json::to_string(&changes)?),
            at_us: self.now_us(),
            accepted_version: None,
        };
        self.append_proposal_rows(&[row], &format!("propose {id}")).await?;
        self.proposal(&id).await
    }

    /// Decide an open proposal without applying it: `Rejected` (by a human) or `Withdrawn`
    /// (by its agent). `note` is the reason, if any.
    pub async fn decide(
        &mut self,
        id: &str,
        status: ProposalStatus,
        actor: &str,
        note: &str,
    ) -> Result<Proposal> {
        if !matches!(status, ProposalStatus::Rejected | ProposalStatus::Withdrawn) {
            return Err(Error::Corrupt(format!(
                "decide takes rejected or withdrawn, not {}",
                status.as_str()
            )));
        }
        let p = self.open_proposal(id).await?;
        let row = proposals::Row {
            proposal_id: id.to_owned(),
            seq: p.seq + 1,
            status,
            actor: actor.to_owned(),
            message: (!note.trim().is_empty()).then(|| note.trim().to_owned()),
            base_version: None,
            changes: None,
            at_us: self.now_us(),
            accepted_version: None,
        };
        self.append_proposal_rows(&[row], &format!("{} {id}", status.as_str()))
            .await?;
        self.proposal(id).await
    }

    async fn open_proposal(&self, id: &str) -> Result<Proposal> {
        let p = self.proposal(id).await?;
        if p.status != ProposalStatus::Open {
            return Err(Error::ProposalDecided {
                id: id.to_owned(),
                status: p.status.as_str(),
            });
        }
        Ok(p)
    }

    /// Accept an open proposal: commit `changes` (the proposal made concrete by the caller:
    /// order keys, names, provenance `ai:accepted`) as the human `actor`, carrying
    /// `cruncha.proposal`, then record the decision. Returns the new version.
    ///
    /// If the process dies between the two writes, [`Store::open`] finds the commit and
    /// records the missing decision.
    pub async fn accept(&mut self, id: &str, changes: &[Change], actor: &str, message: &str) -> Result<u64> {
        let version = self.commit_for_proposal(id, changes, actor, message).await?;
        let p = self.proposal(id).await?;
        self.record_accepted(&p, version, actor, self.now_us()).await?;
        Ok(version)
    }

    /// The first half of [`Store::accept`]: the commit alone. Public for crash tests.
    #[doc(hidden)]
    pub async fn commit_for_proposal(
        &mut self,
        id: &str,
        changes: &[Change],
        actor: &str,
        message: &str,
    ) -> Result<u64> {
        self.open_proposal(id).await?;
        self.commit_with(
            changes,
            actor,
            message,
            CommitKind::Edit,
            Some((META_PROPOSAL, json!(id))),
        )
        .await
    }

    async fn record_accepted(&mut self, p: &Proposal, version: u64, actor: &str, at_us: i64) -> Result<()> {
        let row = proposals::Row {
            proposal_id: p.id.clone(),
            seq: p.seq + 1,
            status: ProposalStatus::Accepted,
            actor: actor.to_owned(),
            message: None,
            base_version: None,
            changes: None,
            at_us,
            accepted_version: Some(version),
        };
        self.append_proposal_rows(&[row], &format!("accepted {} in version {version}", p.id))
            .await
    }

    /// Finish accepts interrupted between their commit and their decision row: any commit
    /// carrying `cruncha.proposal` of a proposal still open. Idempotent.
    async fn recover_accepted(&mut self) -> Result<()> {
        let open: Vec<Proposal> = self
            .proposals()
            .await?
            .into_iter()
            .filter(|p| p.status == ProposalStatus::Open)
            .collect();
        let Some(from) = open.iter().map(|p| p.base_version).min() else {
            return Ok(());
        };
        let from = from.max(self.earliest_version());
        let entries = retention::read_log(&self.root.join("doc"), from, self.version()).await?;
        for e in entries {
            let Some(id) = e.info.get(META_PROPOSAL).and_then(Value::as_str) else {
                continue;
            };
            if let Some(p) = open.iter().find(|p| p.id == id) {
                let actor = e.info.get(META_ACTOR).and_then(Value::as_str).unwrap_or("system").to_owned();
                let at = e.info.get(META_AT).and_then(Value::as_i64).unwrap_or(e.at_ms * 1000);
                self.record_accepted(p, e.version, &actor, at).await?;
            }
        }
        Ok(())
    }

    /// Prune step for `proposals/`: drop decided proposals decided before `cutoff_us`, except
    /// accepted ones that went into a pinned version (`accepted_version <= keep_upto`). Open
    /// proposals always stay. Returns how many were dropped.
    async fn prune_proposals(&mut self, cutoff_us: i64, keep_upto: Option<u64>, actor: &str) -> Result<usize> {
        let Some(table) = self.proposals.clone() else {
            return Ok(0);
        };
        let rows = self.proposal_rows().await?;
        let drop: HashSet<String> = proposals::fold(rows.clone())?
            .into_iter()
            .filter(|p| {
                p.status != ProposalStatus::Open
                    && p.decided_at_us.is_some_and(|t| t < cutoff_us)
                    && !(p.status == ProposalStatus::Accepted
                        && keep_upto.is_some_and(|k| p.accepted_version.is_some_and(|v| v <= k)))
            })
            .map(|p| p.id)
            .collect();
        let mut table = table;
        if !drop.is_empty() {
            let keep: Vec<proposals::Row> =
                rows.into_iter().filter(|r| !drop.contains(&r.proposal_id)).collect();
            table = table
                .write([proposals::rows_to_batch(&keep)?])
                .with_save_mode(SaveMode::Overwrite)
                .with_commit_properties(commit_props(
                    CommitKind::Maintenance,
                    actor,
                    "prune decided proposals",
                    self.now_us(),
                    &[],
                ))
                .await?;
        }
        // The table's rows are its history (each status change is a row), so its older
        // versions aren't needed: remove every replaced file and the log before the latest
        // version. Run every time, so an interrupted prune is finished.
        let latest = table.version().unwrap_or(0);
        checkpoints::create_checkpoint(&table, None).await?;
        let now_ms = chrono::Utc::now().timestamp_millis() + 1000;
        let (table, _) = table
            .vacuum()
            .with_mode(VacuumMode::Lite)
            .with_retention_period(chrono::Duration::zero())
            .with_enforce_retention_duration(false)
            .with_clock(Arc::new(FixedClock(now_ms)))
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                "vacuum pruned proposals",
                self.now_us(),
                &[],
            ))
            .await?;
        checkpoints::cleanup_expired_logs_for(latest, table.log_store().as_ref(), now_ms, None).await?;
        self.proposals = Some(table);
        Ok(drop.len())
    }

    /// Delete every blob that no data file left in `doc/`, `pins/` or `results/` references.
    /// Files on disk are a superset of what available versions read, so this never removes a
    /// blob they need. Returns the number deleted.
    async fn gc_blobs(&self) -> Result<usize> {
        let dirs = ["doc", "pins", "results"].map(|d| self.root.join(d));
        let keep = tokio::task::spawn_blocking(move || -> Result<HashSet<String>> {
            let mut keep = HashSet::new();
            for d in &dirs {
                referenced_blobs(d, &mut keep)?;
            }
            Ok(keep)
        })
        .await
        .map_err(|e| Error::Corrupt(format!("blob scan: {e}")))??;
        Ok(self.blobs.retain(&keep).await?.0)
    }

    async fn result_rows(&self) -> Result<Vec<results::Row>> {
        match &self.results {
            None => Ok(Vec::new()),
            Some(t) => results::batches_to_rows(&scan(t).await?),
        }
    }

    /// Write the blobs of `runs` and return their rows, unwritten. The blobs are on disk
    /// before any row references them, so a crash after this leaves only unreferenced blobs
    /// (removed by the next prune or [`Store::retain_results`]).
    async fn put_result_blobs(&self, runs: Vec<RunRecord>, report: &mut PutReport) -> Result<Vec<results::Row>> {
        let mut texts = Vec::new();
        let mut rows = Vec::with_capacity(runs.len());
        for r in runs {
            if results::run_bytes(&r) > results::MAX_RUN_BYTES {
                report.too_large.push(r.cell_id);
                continue;
            }
            let start = texts.len();
            let values = results::externalize(r.values, &mut texts)?;
            let outputs = results::externalize(r.outputs, &mut texts)?;
            let mut blobs: Vec<String> = texts[start..].iter().map(|t| blobs::hash(t)).collect();
            blobs.sort();
            blobs.dedup();
            rows.push(results::Row {
                entry: RunEntry {
                    cell_id: r.cell_id,
                    input_hash: r.input_hash,
                    base_hash: r.base_hash,
                    status: r.status,
                    env: r.env,
                    computed_at_us: r.computed_at_us,
                    computed_by: r.computed_by,
                    doc_version: r.doc_version,
                },
                values,
                outputs,
                stdout: results::cap(r.stdout, results::MAX_TEXT_BYTES),
                stderr: results::cap(r.stderr, results::MAX_TEXT_BYTES),
                error: r.error.map(|e| results::cap(e, results::MAX_TEXT_BYTES)),
                duration_ms: r.duration_ms,
                blobs,
            });
        }
        report.blob_bytes += self.blobs.put_many(texts).await?.iter().map(|(_, n)| n).sum::<u64>();
        Ok(rows)
    }

    /// Only the first half of [`Store::put_results`]: what a crash between writing the blobs
    /// and the row leaves. Returns the blob bytes written.
    #[doc(hidden)] // For crash tests.
    pub async fn put_result_blobs_only(&self, runs: Vec<RunRecord>) -> Result<u64> {
        let mut report = PutReport::default();
        self.put_result_blobs(runs, &mut report).await?;
        Ok(report.blob_bytes)
    }

    /// Store finished runs of code cells (ADR-0018): blobs first, then one append to
    /// `results/`. Never a document version: `doc/` and its history are untouched. A run
    /// larger than [`results::MAX_RUN_BYTES`] is skipped and reported.
    pub async fn put_results(&mut self, runs: Vec<RunRecord>, actor: &str) -> Result<PutReport> {
        let mut report = PutReport::default();
        let rows = self.put_result_blobs(runs, &mut report).await?;
        if rows.is_empty() {
            return Ok(report);
        }
        let table = match &self.results {
            Some(t) => t.clone(),
            None => {
                let dir = self.root.join("results");
                if dir.join("_delta_log").is_dir() {
                    open_table(&dir, &self.storage).await?
                } else {
                    create_table(&dir, "results", &results::RESULTS_SCHEMA, actor, &self.storage).await?
                }
            }
        };
        let table = table
            .write([results::rows_to_batch(&rows)?])
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                &format!("store {} result(s)", rows.len()),
                self.now_us(),
                &[],
            ))
            .await?;
        self.results = Some(table);
        report.written = rows.len();
        self.result_blobs.extend(rows.into_iter().flat_map(|r| r.blobs));
        Ok(report)
    }

    /// Every stored run, without values or outputs, oldest first.
    pub async fn result_entries(&self) -> Result<Vec<RunEntry>> {
        let mut out: Vec<RunEntry> = self.result_rows().await?.into_iter().map(|r| r.entry).collect();
        out.sort_by_key(|e| e.computed_at_us);
        Ok(out)
    }

    /// The stored runs for `keys` (`(cell_id, input_hash)`), newest per key. Keys with no
    /// stored run are absent.
    pub async fn results_for(&self, keys: &HashSet<(String, String)>) -> Result<HashMap<(String, String), RunRecord>> {
        self.lookup_results(keys, |e| Some(e.input_hash.as_str())).await
    }

    /// The stored runs for `keys` (`(cell_id, base_hash)`, ADR-0019), newest per key: runs
    /// with the same inputs from any Python environment.
    pub async fn results_by_base(&self, keys: &HashSet<(String, String)>) -> Result<HashMap<(String, String), RunRecord>> {
        self.lookup_results(keys, |e| e.base_hash.as_deref()).await
    }

    async fn lookup_results(
        &self,
        keys: &HashSet<(String, String)>,
        hash_of: impl Fn(&RunEntry) -> Option<&str>,
    ) -> Result<HashMap<(String, String), RunRecord>> {
        let mut newest: HashMap<(String, String), results::Row> = HashMap::new();
        for r in self.result_rows().await? {
            let Some(h) = hash_of(&r.entry) else { continue };
            let k = (r.entry.cell_id.clone(), h.to_owned());
            if !keys.contains(&k) {
                continue;
            }
            if newest.get(&k).is_none_or(|o| o.entry.computed_at_us <= r.entry.computed_at_us) {
                newest.insert(k, r);
            }
        }
        let loaded = self
            .blobs
            .get_many(newest.values().flat_map(|r| r.blobs.iter().map(String::as_str)))
            .await?;
        let get = |h: &str| {
            loaded
                .get(h)
                .cloned()
                .ok_or_else(|| Error::Corrupt(format!("result blob {h} not listed by its row")))
        };
        newest
            .into_iter()
            .map(|(k, r)| {
                let values = results::internalize(serde_json::from_str(&r.values)?, &get)?;
                let outputs = results::internalize(serde_json::from_str(&r.outputs)?, &get)?;
                Ok((
                    k,
                    RunRecord {
                        cell_id: r.entry.cell_id,
                        input_hash: r.entry.input_hash,
                        base_hash: r.entry.base_hash,
                        status: r.entry.status,
                        values,
                        outputs,
                        stdout: r.stdout,
                        stderr: r.stderr,
                        error: r.error,
                        duration_ms: r.duration_ms,
                        env: r.entry.env,
                        computed_at_us: r.entry.computed_at_us,
                        computed_by: r.entry.computed_by,
                        doc_version: r.entry.doc_version,
                    },
                ))
            })
            .collect()
    }

    /// Keep only the stored runs `keep` accepts (by their entry: cell id, hashes, …) and delete the
    /// rest, with their files, log and unreferenced blobs. The caller decides what the
    /// retained versions and snapshots can still use (ADR-0018 §5); a `keep` that accepts
    /// nothing clears the table. Results are a cache, so their own history isn't kept.
    /// Returns how many runs were dropped. Safe to repeat after a crash.
    pub async fn retain_results(&mut self, keep: impl Fn(&RunEntry) -> bool, actor: &str) -> Result<usize> {
        let Some(table) = self.results.clone() else {
            return Ok(0);
        };
        let rows = self.result_rows().await?;
        let total = rows.len();
        let kept: Vec<results::Row> = rows
            .into_iter()
            .filter(|r| keep(&r.entry))
            .collect();
        let dropped = total - kept.len();
        let mut table = table;
        if dropped > 0 {
            table = table
                .write([results::rows_to_batch(&kept)?])
                .with_save_mode(SaveMode::Overwrite)
                .with_commit_properties(commit_props(
                    CommitKind::Maintenance,
                    actor,
                    &format!("drop {dropped} stored result(s)"),
                    self.now_us(),
                    &[],
                ))
                .await?;
        }
        let latest = table.version().unwrap_or(0);
        checkpoints::create_checkpoint(&table, None).await?;
        let now_ms = chrono::Utc::now().timestamp_millis() + 1000;
        let (table, _) = table
            .vacuum()
            .with_mode(VacuumMode::Lite)
            .with_retention_period(chrono::Duration::zero())
            .with_enforce_retention_duration(false)
            .with_clock(Arc::new(FixedClock(now_ms)))
            .with_commit_properties(commit_props(
                CommitKind::Maintenance,
                actor,
                "vacuum dropped results",
                self.now_us(),
                &[],
            ))
            .await?;
        checkpoints::cleanup_expired_logs_for(latest, table.log_store().as_ref(), now_ms, None).await?;
        self.results = Some(table);
        self.result_blobs = kept.into_iter().flat_map(|r| r.blobs).collect();
        self.gc_blobs().await?;
        Ok(dropped)
    }

    /// Drop every stored result ("Clear stored results").
    pub async fn clear_results(&mut self, actor: &str) -> Result<usize> {
        self.retain_results(|_| false, actor).await
    }
}

/// The newest prune marker's cutoff, and whether there were edits after it.
fn prune_state(entries: &[LogEntry]) -> (Option<u64>, bool) {
    let mut edits = false;
    for e in entries.iter().rev() {
        if let Some(p) = e.pruned_before() {
            return (Some(p), edits);
        }
        edits |= e.is_edit();
    }
    (None, edits)
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
