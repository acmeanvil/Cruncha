// SPDX-License-Identifier: GPL-3.0-or-later
//! Out-of-line cell content (ADR-0013, option D of spike S3).
//!
//! A cell whose `source` is at least the store's blob threshold is written once to
//! `blobs/ab/cdef….zst` (zstd-compressed, named by the SHA-256 of the source) and its `doc` row
//! carries only `source_hash`. Squashes, restores, moves and pins then rewrite 64 hex
//! characters instead of the text, and identical text is stored once.
//!
//! Blobs are immutable. One is written (temp file, then rename) *before* the Delta commit that
//! references it, so a crash leaves at most an unreferenced blob. Only [`crate::Store::prune`]
//! deletes blobs, and only those no remaining `doc/` or `pins/` data file references.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use futures::{StreamExt, stream};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// Sources at least this long (in bytes) are stored out of line by default.
pub const DEFAULT_BLOB_THRESHOLD: usize = 1024;
/// Decoded blobs kept in memory. Blobs are immutable, so the cache is never stale.
const CACHE_BYTES: usize = 64 << 20;
const ZSTD_LEVEL: i32 = 3;
/// Blob files read or written at once: a cold document has one per large cell.
const IO_CONCURRENCY: usize = 32;

/// The hex SHA-256 of `source`: its blob's name.
pub fn hash(source: &str) -> String {
    let digest = Sha256::digest(source.as_bytes());
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn valid_hash(h: &str) -> bool {
    h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

#[derive(Default)]
struct Cache {
    map: HashMap<String, Arc<str>>,
    bytes: usize,
}

/// The container's `blobs/` directory.
pub(crate) struct Blobs {
    dir: PathBuf,
    cache: Mutex<Cache>,
}

impl Blobs {
    pub fn new(container_root: &Path) -> Self {
        Blobs {
            dir: container_root.join("blobs"),
            cache: Mutex::new(Cache::default()),
        }
    }

    fn path(&self, h: &str) -> PathBuf {
        self.dir.join(&h[..2]).join(format!("{}.zst", &h[2..]))
    }

    fn cached(&self, h: &str) -> Option<Arc<str>> {
        self.cache.lock().expect("blob cache").map.get(h).cloned()
    }

    fn remember(&self, h: &str, source: Arc<str>) {
        let mut c = self.cache.lock().expect("blob cache");
        if c.bytes + source.len() > CACHE_BYTES {
            *c = Cache::default();
        }
        c.bytes += source.len();
        c.map.insert(h.to_owned(), source);
    }

    /// Store `source` (if it isn't already) and return its hash and the bytes newly written.
    pub async fn put(&self, source: &str) -> Result<(String, u64)> {
        let h = hash(source);
        if self.cached(&h).is_some() {
            return Ok((h, 0));
        }
        let path = self.path(&h);
        if tokio::fs::try_exists(&path).await? {
            self.remember(&h, source.into());
            return Ok((h, 0));
        }
        let compressed = zstd::bulk::compress(source.as_bytes(), ZSTD_LEVEL)?;
        let parent = path.parent().expect("blob path has a parent");
        tokio::fs::create_dir_all(parent).await?;
        let tmp = parent.join(format!(".{}.{}.tmp", &h[2..], ulid::Ulid::generate()));
        tokio::fs::write(&tmp, &compressed).await?;
        tokio::fs::rename(&tmp, &path).await?;
        self.remember(&h, source.into());
        Ok((h, compressed.len() as u64))
    }

    /// Store each of `sources` (see [`Blobs::put`]), several at a time. Returns one result
    /// per source, in order; a text repeated in `sources` is written (and counted) once.
    pub async fn put_many(&self, sources: Vec<String>) -> Result<Vec<(String, u64)>> {
        let mut first: HashMap<&str, usize> = HashMap::new();
        let mut unique: Vec<String> = Vec::new();
        for src in &sources {
            if !first.contains_key(src.as_str()) {
                first.insert(src.as_str(), unique.len());
                unique.push(src.clone());
            }
        }
        // Owned items: futures borrowing `&str` here make callers' futures non-`Send`.
        let mut stored = Vec::with_capacity(unique.len());
        let mut s = stream::iter(unique)
            .map(|src| async move { self.put(&src).await })
            .buffered(IO_CONCURRENCY);
        while let Some(r) = s.next().await {
            stored.push(r?);
        }
        let mut seen = HashSet::new();
        Ok(sources
            .iter()
            .map(|src| {
                let (h, n) = &stored[first[src.as_str()]];
                (h.clone(), if seen.insert(h.clone()) { *n } else { 0 })
            })
            .collect())
    }

    /// Load the blobs named by `hashes`. A missing or damaged blob is [`Error::Corrupt`].
    pub async fn get_many<'a>(
        &self,
        hashes: impl IntoIterator<Item = &'a str>,
    ) -> Result<HashMap<String, Arc<str>>> {
        let mut out = HashMap::new();
        let mut missing: HashSet<String> = HashSet::new();
        for h in hashes {
            match self.cached(h) {
                Some(s) => {
                    out.insert(h.to_owned(), s);
                }
                None => {
                    missing.insert(h.to_owned());
                }
            }
        }
        let mut loaded = stream::iter(missing)
            .map(|h| async move {
                let s = self.load(&h).await;
                (h, s)
            })
            .buffer_unordered(IO_CONCURRENCY);
        while let Some((h, s)) = loaded.next().await {
            let s: Arc<str> = s?.into();
            self.remember(&h, s.clone());
            out.insert(h, s);
        }
        Ok(out)
    }

    async fn load(&self, h: &str) -> Result<String> {
        if !valid_hash(h) {
            return Err(Error::Corrupt(format!("bad blob reference `{h}`")));
        }
        let bytes = tokio::fs::read(self.path(h))
            .await
            .map_err(|e| Error::Corrupt(format!("blob {h}: {e}")))?;
        let raw = zstd::stream::decode_all(bytes.as_slice())
            .map_err(|e| Error::Corrupt(format!("blob {h}: {e}")))?;
        let source =
            String::from_utf8(raw).map_err(|e| Error::Corrupt(format!("blob {h}: {e}")))?;
        if hash(&source) != h {
            return Err(Error::Corrupt(format!("blob {h} does not match its hash")));
        }
        Ok(source)
    }

    /// Delete every blob not in `keep`. Returns the number of blobs and bytes deleted.
    pub async fn retain(&self, keep: &HashSet<String>) -> Result<(usize, u64)> {
        let (mut files, mut bytes) = (0, 0);
        let Ok(mut fanout) = tokio::fs::read_dir(&self.dir).await else {
            return Ok((0, 0));
        };
        while let Some(sub) = fanout.next_entry().await? {
            let prefix = sub.file_name().to_string_lossy().into_owned();
            if !sub.file_type().await?.is_dir() || prefix.len() != 2 {
                continue;
            }
            let mut entries = tokio::fs::read_dir(sub.path()).await?;
            while let Some(e) = entries.next_entry().await? {
                let name = e.file_name().to_string_lossy().into_owned();
                // Leftover temp files from an interrupted write are never referenced.
                let h = name
                    .strip_suffix(".zst")
                    .map(|rest| format!("{prefix}{rest}"));
                if h.as_ref().is_some_and(|h| keep.contains(h)) {
                    continue;
                }
                bytes += e.metadata().await?.len();
                tokio::fs::remove_file(e.path()).await?;
                files += 1;
            }
        }
        self.cache
            .lock()
            .expect("blob cache")
            .map
            .retain(|h, _| keep.contains(h));
        Ok((files, bytes))
    }

    /// Bytes of blobs on disk.
    pub fn size_bytes(&self) -> Result<u64> {
        dir_size(&self.dir)
    }
}

/// Total size of the files under `p` (0 if it doesn't exist).
pub(crate) fn dir_size(p: &Path) -> Result<u64> {
    let Ok(rd) = std::fs::read_dir(p) else {
        return Ok(0);
    };
    let mut total = 0;
    for e in rd {
        let e = e?;
        let m = e.metadata()?;
        total += if m.is_dir() { dir_size(&e.path())? } else { m.len() };
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trip_dedup_and_retain() {
        let dir = tempfile::tempdir().unwrap();
        let blobs = Blobs::new(dir.path());
        let text = "paragraph ".repeat(500);
        let (h, n) = blobs.put(&text).await.unwrap();
        assert_eq!(h, hash(&text));
        assert!(n > 0 && n < text.len() as u64, "compressed: {n}");
        assert_eq!(blobs.put(&text).await.unwrap().1, 0, "stored once");

        let fresh = Blobs::new(dir.path()); // no cache: reads the file
        assert_eq!(&*fresh.get_many([h.as_str()]).await.unwrap()[&h], text);

        let (other, _) = blobs.put("other text").await.unwrap();
        let keep = HashSet::from([h.clone()]);
        assert_eq!(blobs.retain(&keep).await.unwrap().0, 1);
        assert!(fresh.get_many([other.as_str()]).await.is_err());
        assert!(fresh.get_many([h.as_str()]).await.is_ok());
    }

    #[tokio::test]
    async fn damaged_blob_is_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let blobs = Blobs::new(dir.path());
        let (h, _) = blobs.put("hello world").await.unwrap();
        std::fs::write(
            blobs.path(&h),
            zstd::bulk::compress(b"tampered", 3).unwrap(),
        )
        .unwrap();
        let err = Blobs::new(dir.path()).get_many([h.as_str()]).await.unwrap_err();
        assert!(matches!(err, Error::Corrupt(_)), "{err}");
        assert!(blobs.get_many(["../etc"]).await.is_err());
    }
}
