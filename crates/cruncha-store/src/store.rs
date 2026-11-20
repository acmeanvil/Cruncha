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
//! └── blobs/         large cell sources, zstd-compressed, named by SHA-256 (ADR-0013)
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

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use deltalake::kernel::StructType;
use deltalake::kernel::engine::arrow_conversion::TryIntoKernel as _;
use deltalake::kernel::transaction::CommitProperties;
use deltalake::operations::vacuum::{Clock, VacuumMode};
use deltalake::protocol::SaveMode;
use deltalake::{DeltaTable, checkpoints};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

use crate::blobs::{self, Blobs};
use crate::cell::{Cell, CellDiff, Change};
use crate::error::{Error, Result};
use crate::retention::{self, LogEntry, Retention};
use crate::schema::{self, Ref, SOURCE_HASH_COLUMN};

pub const FORMAT: &str = "cruncha-container";
/// 1: `doc/` is one row per cell, updated by MERGE. 2: `doc/` is a revision log (ADR-0010).
/// 3: large sources live in `blobs/`, referenced by `source_hash` (ADR-0013).
pub const FORMAT_VERSION: u32 = 3;
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
    /// Added to the clock for Cruncha's own timestamps. Tests use it to simulate time passing.
    clock_offset_us: i64,
    /// Out-of-line sources.
    blobs: Blobs,
    /// Sources at least this many bytes go to `blobs/`.
    blob_threshold: usize,
    /// Edit commits since the last squash.
    since_squash: u64,
    squash_every: u64,
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
            schema.column(i).path().parts() == [SOURCE_HASH_COLUMN.to_owned()]
        }) else {
            continue; // written before format 3: no out-of-line sources
        };
        let mask = ProjectionMask::leaves(schema, [idx]);
        let reader = builder
            .with_projection(mask)
            .build()
            .map_err(|e| Error::Corrupt(format!("{}: {e}", path.display())))?;
        for batch in reader {
            out.extend(schema::blob_hashes(&batch?)?);
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

        let doc = create_table(&root.join("doc"), "doc", &schema::DOC_LOG_SCHEMA, actor).await?;
        let refs = create_table(&root.join("refs"), "refs", &schema::REFS_SCHEMA, actor).await?;

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
            clock_offset_us: 0,
            blob_threshold: blobs::DEFAULT_BLOB_THRESHOLD,
            since_squash: 0,
            squash_every: DEFAULT_SQUASH_EVERY,
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
        let pins_dir = root.join("pins");
        let pins = if pins_dir.join("_delta_log").is_dir() {
            Some(deltalake::open_table(dir_url(&pins_dir)?).await?)
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
            clock_offset_us: 0,
            since_squash,
            squash_every: DEFAULT_SQUASH_EVERY,
            _lock: lock,
        };
        if store.manifest.format_version < FORMAT_VERSION {
            store.migrate().await?;
        }
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

    /// Bytes of history on disk: `doc/` data and log, and blobs (pins excluded).
    pub fn history_size_bytes(&self) -> Result<u64> {
        Ok(blobs::dir_size(&self.root.join("doc"))? + self.blobs.size_bytes()?)
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

        // 1. Named versions survive: copy any that are about to go into `pins/`.
        let pinned = self.pin_refs_before(cutoff, actor).await?;

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

        // 6. Delete blobs that no remaining data file (of any available version) or pin
        //    references. Files on disk are a superset of what versions from the cutoff on
        //    read, so this never removes a blob they need.
        let mut keep = HashSet::new();
        let (doc_dir, pins_dir) = (self.root.join("doc"), self.root.join("pins"));
        let keep = tokio::task::spawn_blocking(move || -> Result<HashSet<String>> {
            referenced_blobs(&doc_dir, &mut keep)?;
            referenced_blobs(&pins_dir, &mut keep)?;
            Ok(keep)
        })
        .await
        .map_err(|e| Error::Corrupt(format!("blob scan: {e}")))??;
        let (blobs_deleted, _) = self.blobs.retain(&keep).await?;

        // 7. The marker commit, so history shows where it now starts.
        let date = entries
            .iter()
            .find(|e| e.version == cutoff)
            .and_then(|e| chrono::DateTime::from_timestamp_millis(e.at_ms))
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        let message = format!("history before version {cutoff} ({date}) pruned; policy: {policy}");
        self.squash(actor, &message, &[(META_PRUNED_BEFORE, json!(cutoff))])
            .await?;
        checkpoints::create_checkpoint(&self.doc, None).await?;

        Ok(Some(PruneReport {
            pruned_before: cutoff,
            pinned,
            data_files_deleted: metrics.files_deleted.len(),
            log_files_deleted,
            blobs_deleted,
        }))
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
                    deltalake::open_table(dir_url(&dir)?).await?
                } else {
                    create_table(&dir, "pins", &schema::PINS_SCHEMA, actor).await?
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
