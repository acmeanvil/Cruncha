// SPDX-License-Identifier: GPL-3.0-or-later
//! An index of each table's `_delta_log` (spike S3c, ADR-0021).
//!
//! delta-rs lists `_delta_log` to find the latest version and the nearest checkpoint, on every
//! commit and every time travel. The local file system lists by reading the whole directory,
//! so with one file per version that cost grows without bound (66 ms at 10,000 versions).
//!
//! A container has a single writer (the `.lock` file), and every write to its tables goes
//! through this store. So the listing is read from disk once per `Store` and then kept up to
//! date by the writes themselves: puts (including multipart uploads such as checkpoints),
//! copies, renames and deletes. Listings of a `_delta_log` are answered from the index; every
//! other call goes straight to the file system. The files on disk are unchanged.
//!
//! Anything that writes a log without going through this store (delta-rs functions that build
//! their own storage from a URL) must be followed by [`LogIndexStore::forget`].

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::ops::Range;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bytes::Bytes;
use deltalake::logstore::object_store::path::Path;
use deltalake::logstore::object_store::{
    CopyOptions, GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore, ObjectStoreExt as _,
    PutMultipartOptions, PutOptions, PutPayload, PutResult, RenameOptions, Result, UploadPart,
};
use futures::stream::{self, BoxStream, StreamExt, TryStreamExt};

const LOG_DIR: &str = "_delta_log";

/// Per log directory: each file, with its metadata once something listed it (`None`: only its
/// name is known; stat-ing 10,000 files up front would cost what the index saves).
type Index = Arc<Mutex<HashMap<Path, BTreeMap<Path, Option<ObjectMeta>>>>>;

/// The file system with indexed `_delta_log` listings.
#[derive(Debug, Clone)]
pub struct LogIndexStore {
    inner: Arc<dyn ObjectStore>,
    index: Index,
    /// Log directories the store registered, with their place on disk.
    dirs: Arc<Mutex<HashMap<Path, std::path::PathBuf>>>,
}

impl fmt::Display for LogIndexStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LogIndexStore({})", self.inner)
    }
}

/// The `_delta_log` directory `p` is directly inside, if any.
fn log_dir_of(p: &Path) -> Option<Path> {
    let parts: Vec<_> = p.parts().collect();
    let n = parts.len();
    (n >= 2 && parts[n - 2].as_ref() == LOG_DIR).then(|| Path::from_iter(parts[..n - 1].iter().cloned()))
}

/// Whether `p` names a `_delta_log` directory itself.
fn is_log_dir(p: &Path) -> bool {
    p.filename() == Some(LOG_DIR)
}

impl LogIndexStore {
    pub fn new(inner: Arc<dyn ObjectStore>) -> LogIndexStore {
        LogIndexStore { inner, index: Arc::default(), dirs: Arc::default() }
    }

    /// Index the table at `table_dir` (its `_delta_log`). Unregistered logs are listed from
    /// the file system each time.
    pub fn register(&self, table_dir: &std::path::Path) -> Result<()> {
        let fs = table_dir.join(LOG_DIR);
        std::fs::create_dir_all(&fs).map_err(|e| deltalake::logstore::object_store::Error::Generic { store: "LogIndexStore", source: Box::new(e) })?;
        let path = Path::from_filesystem_path(&fs)?;
        self.dirs.lock().unwrap().insert(path, fs);
        Ok(())
    }

    /// Drop every index; the next listing reads the directories again.
    pub fn forget(&self) {
        self.index.lock().unwrap().clear();
    }

    /// The entries of `dir` after `offset` (all of them without one), in order. The names are
    /// read from disk once; each entry is stat-ed the first time it is listed.
    async fn entries(&self, dir: &Path, offset: Option<&Path>) -> Result<Vec<ObjectMeta>> {
        let fs = self.dirs.lock().unwrap().get(dir).cloned();
        let Some(fs) = fs else {
            let listed: Vec<ObjectMeta> = match offset {
                Some(o) => self.inner.list_with_offset(Some(dir), o).try_collect().await?,
                None => self.inner.list(Some(dir)).try_collect().await?,
            };
            return Ok(listed);
        };
        if !self.index.lock().unwrap().contains_key(dir) {
            let mut names = BTreeMap::new();
            let rd = std::fs::read_dir(&fs).map_err(|e| deltalake::logstore::object_store::Error::Generic { store: "LogIndexStore", source: Box::new(e) })?;
            for e in rd.flatten() {
                if e.file_type().is_ok_and(|t| t.is_file())
                    && let Some(name) = e.file_name().to_str()
                {
                    names.insert(dir.clone().join(name), None);
                }
            }
            self.index.lock().unwrap().entry(dir.clone()).or_insert(names);
        }
        let wanted: Vec<(Path, Option<ObjectMeta>)> = {
            let index = self.index.lock().unwrap();
            let Some(m) = index.get(dir) else { return Ok(vec![]) };
            m.iter().filter(|(p, _)| offset.is_none_or(|o| *p > o)).map(|(p, v)| (p.clone(), v.clone())).collect()
        };
        let mut out = Vec::with_capacity(wanted.len());
        for (p, meta) in wanted {
            let meta = match meta {
                Some(m) => m,
                None => match self.inner.head(&p).await {
                    Ok(m) => {
                        if let Some(map) = self.index.lock().unwrap().get_mut(dir) {
                            map.insert(p.clone(), Some(m.clone()));
                        }
                        m
                    }
                    // Gone since the names were read (a cleanup elsewhere): leave it out.
                    Err(deltalake::logstore::object_store::Error::NotFound { .. }) => {
                        if let Some(map) = self.index.lock().unwrap().get_mut(dir) {
                            map.remove(&p);
                        }
                        continue;
                    }
                    Err(e) => return Err(e),
                },
            };
            out.push(meta);
        }
        Ok(out)
    }

    /// After a write at `location`: record it, if its log is indexed.
    async fn added(&self, location: &Path) {
        let Some(dir) = log_dir_of(location) else { return };
        if !self.index.lock().unwrap().contains_key(&dir) {
            return;
        }
        match self.inner.head(location).await {
            Ok(meta) => {
                if let Some(m) = self.index.lock().unwrap().get_mut(&dir) {
                    m.insert(location.clone(), Some(meta));
                }
            }
            // Can't tell what's there: read the directory again next time.
            Err(_) => {
                self.index.lock().unwrap().remove(&dir);
            }
        }
    }

    fn removed(index: &Index, location: &Path) {
        if let Some(dir) = log_dir_of(location)
            && let Some(m) = index.lock().unwrap().get_mut(&dir)
        {
            m.remove(location);
        }
    }

    fn listing(entries: Vec<ObjectMeta>) -> BoxStream<'static, Result<ObjectMeta>> {
        stream::iter(entries.into_iter().map(Ok)).boxed()
    }
}

/// A multipart upload that records its object in the index once it is complete.
#[derive(Debug)]
struct Recorded {
    inner: Box<dyn MultipartUpload>,
    store: LogIndexStore,
    location: Path,
}

#[async_trait]
impl MultipartUpload for Recorded {
    fn put_part(&mut self, data: PutPayload) -> UploadPart {
        self.inner.put_part(data)
    }

    async fn complete(&mut self) -> Result<PutResult> {
        let r = self.inner.complete().await?;
        self.store.added(&self.location).await;
        Ok(r)
    }

    async fn abort(&mut self) -> Result<()> {
        self.inner.abort().await
    }
}

#[async_trait]
impl ObjectStore for LogIndexStore {
    async fn put_opts(&self, location: &Path, payload: PutPayload, opts: PutOptions) -> Result<PutResult> {
        let r = self.inner.put_opts(location, payload, opts).await?;
        self.added(location).await;
        Ok(r)
    }

    async fn put_multipart_opts(&self, location: &Path, opts: PutMultipartOptions) -> Result<Box<dyn MultipartUpload>> {
        let inner = self.inner.put_multipart_opts(location, opts).await?;
        if log_dir_of(location).is_none() {
            return Ok(inner);
        }
        Ok(Box::new(Recorded { inner, store: self.clone(), location: location.clone() }))
    }

    async fn get_opts(&self, location: &Path, options: GetOptions) -> Result<GetResult> {
        self.inner.get_opts(location, options).await
    }

    async fn get_ranges(&self, location: &Path, ranges: &[Range<u64>]) -> Result<Vec<Bytes>> {
        self.inner.get_ranges(location, ranges).await
    }

    fn delete_stream(&self, locations: BoxStream<'static, Result<Path>>) -> BoxStream<'static, Result<Path>> {
        let index = self.index.clone();
        self.inner
            .delete_stream(locations)
            .inspect_ok(move |p| LogIndexStore::removed(&index, p))
            .boxed()
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        match prefix {
            Some(dir) if is_log_dir(dir) => {
                let (store, dir) = (self.clone(), dir.clone());
                stream::once(async move { store.entries(&dir, None).await })
                    .map_ok(|v| stream::iter(v.into_iter().map(Ok)))
                    .try_flatten()
                    .boxed()
            }
            _ => self.inner.list(prefix),
        }
    }

    fn list_with_offset(&self, prefix: Option<&Path>, offset: &Path) -> BoxStream<'static, Result<ObjectMeta>> {
        match prefix {
            Some(dir) if is_log_dir(dir) => {
                let (store, dir, offset) = (self.clone(), dir.clone(), offset.clone());
                stream::once(async move {
                    let after = store.entries(&dir, Some(&offset)).await?;
                    Ok::<_, deltalake::logstore::object_store::Error>(LogIndexStore::listing(after))
                })
                .try_flatten()
                .boxed()
            }
            _ => self.inner.list_with_offset(prefix, offset),
        }
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
        match prefix {
            Some(dir) if is_log_dir(dir) => {
                Ok(ListResult { common_prefixes: vec![], objects: self.entries(dir, None).await? })
            }
            _ => self.inner.list_with_delimiter(prefix).await,
        }
    }

    async fn copy_opts(&self, from: &Path, to: &Path, options: CopyOptions) -> Result<()> {
        self.inner.copy_opts(from, to, options).await?;
        self.added(to).await;
        Ok(())
    }

    async fn rename_opts(&self, from: &Path, to: &Path, options: RenameOptions) -> Result<()> {
        self.inner.rename_opts(from, to, options).await?;
        LogIndexStore::removed(&self.index, from);
        self.added(to).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deltalake::logstore::object_store::local::LocalFileSystem;

    #[tokio::test]
    async fn listings_follow_writes_without_rereading() {
        let dir = tempfile::tempdir().unwrap();
        let root = Path::from_filesystem_path(dir.path()).unwrap();
        let log = root.clone().join(LOG_DIR);
        std::fs::create_dir_all(dir.path().join(LOG_DIR)).unwrap();
        std::fs::write(dir.path().join("_delta_log/00000000000000000000.json"), "{}").unwrap();
        let store = LogIndexStore::new(Arc::new(LocalFileSystem::new()));
        store.register(dir.path()).unwrap();
        let names = |v: Vec<ObjectMeta>| v.into_iter().map(|m| m.location.filename().unwrap().to_owned()).collect::<Vec<_>>();
        let list = |s: &LogIndexStore| s.list(Some(&log)).try_collect::<Vec<_>>();
        assert_eq!(names(list(&store).await.unwrap()), ["00000000000000000000.json"]);

        // A file written behind its back isn't seen: the index is what this store wrote.
        std::fs::write(dir.path().join("_delta_log/stray.json"), "{}").unwrap();
        assert_eq!(list(&store).await.unwrap().len(), 1);
        store.put(&log.clone().join("00000000000000000001.json"), PutPayload::from("{}")).await.unwrap();
        let mut up = store.put_multipart(&log.clone().join("00000000000000000001.checkpoint.parquet")).await.unwrap();
        up.put_part(PutPayload::from("x")).await.unwrap();
        up.complete().await.unwrap();
        assert_eq!(
            names(list(&store).await.unwrap()),
            ["00000000000000000000.json", "00000000000000000001.checkpoint.parquet", "00000000000000000001.json"]
        );
        let after: Vec<_> = store.list_with_offset(Some(&log), &log.clone().join("00000000000000000000.json")).try_collect().await.unwrap();
        assert_eq!(after.len(), 2);
        store.delete(&log.clone().join("00000000000000000000.json")).await.unwrap();
        assert_eq!(list(&store).await.unwrap().len(), 2);
        // forget() reads the directory again (now with the stray file).
        store.forget();
        assert_eq!(list(&store).await.unwrap().len(), 3);
        // Other directories go straight to the file system.
        std::fs::write(dir.path().join("data.parquet"), "x").unwrap();
        assert!(store.list(Some(&root)).try_collect::<Vec<_>>().await.unwrap().len() >= 4);
    }
}
