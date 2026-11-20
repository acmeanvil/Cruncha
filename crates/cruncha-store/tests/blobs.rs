// SPDX-License-Identifier: GPL-3.0-or-later
//! Out-of-line cell content (ADR-0013): large sources live once in `blobs/`, every version
//! still reads exactly as written, and only pruning deletes blobs, never one still needed.

use std::path::Path;

use cruncha_store::{Cell, CellKind, Change, DEFAULT_BLOB_THRESHOLD, Error, Retention, Store};

fn blob_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root.join("blobs")) else {
        return out;
    };
    for sub in rd.flatten() {
        for f in std::fs::read_dir(sub.path()).unwrap().flatten() {
            out.push(f.path());
        }
    }
    out.sort();
    out
}

fn big(tag: &str) -> String {
    format!("{tag}: {}", "The beam is checked for bending and shear. ".repeat(40))
}

fn cell(id: &str, key: &str, source: impl Into<String>) -> Cell {
    let mut c = Cell::new(CellKind::Prose, key, source);
    c.id = id.into();
    c
}

async fn put(store: &mut Store, cells: &[&Cell]) -> u64 {
    let changes: Vec<Change> = cells.iter().map(|c| Change::Upsert((*c).clone())).collect();
    store.commit(&changes, "user:test", "edit").await.unwrap()
}

fn sources(cells: &[Cell]) -> Vec<&str> {
    cells.iter().map(|c| c.source.as_str()).collect()
}

#[tokio::test]
async fn large_sources_go_out_of_line_and_read_back_at_every_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let mut store = Store::create(&path, "user:test").await.unwrap();
    assert!(big("a").len() >= DEFAULT_BLOB_THRESHOLD);

    let small = cell("S", "a0", "x = 1 m");
    let a1 = cell("A", "a1", big("one"));
    let v1 = put(&mut store, &[&small, &a1]).await;
    assert_eq!(blob_files(store.root()).len(), 1, "only the large source");

    let a2 = cell("A", "a1", big("two"));
    let v2 = put(&mut store, &[&a2]).await;
    assert_eq!(blob_files(store.root()).len(), 2);

    assert_eq!(
        sources(&store.cells_at(v1).await.unwrap()),
        [small.source.as_str(), &a1.source]
    );
    assert_eq!(
        sources(&store.cells_at(v2).await.unwrap()),
        [small.source.as_str(), &a2.source]
    );

    // A fresh process (no blob cache) reads the same.
    drop(store);
    let store = Store::open(&path).await.unwrap();
    assert_eq!(store.cells_at(v1).await.unwrap()[1].source, a1.source);
    assert_eq!(store.cells().await.unwrap()[1].source, a2.source);
}

#[tokio::test]
async fn text_is_stored_once_across_copies_moves_squashes_and_restores() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("t.cruncha"), "user:test")
        .await
        .unwrap();
    let a = cell("A", "a0", big("same"));
    let b = cell("B", "a1", big("same")); // identical text in another cell
    let v1 = put(&mut store, &[&a, &b]).await;
    assert_eq!(blob_files(store.root()).len(), 1);

    let moved = cell("A", "a9", big("same"));
    put(&mut store, &[&moved]).await;
    put(&mut store, &[&cell("A", "a9", big("edited"))]).await;
    let before = blob_files(store.root());
    assert_eq!(before.len(), 2);

    store.compact("system").await.unwrap();
    store.restore(v1, "user:test").await.unwrap();
    assert_eq!(blob_files(store.root()), before, "squash and restore copy no text");
    let now = store.cells().await.unwrap();
    assert_eq!(sources(&now), [a.source.as_str(), &b.source]);
    assert!(now[0].same_content(&a));
}

#[tokio::test]
async fn threshold_can_change_and_both_kinds_of_row_read_the_same() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("t.cruncha"), "user:test")
        .await
        .unwrap();
    store.set_blob_threshold(usize::MAX);
    let inline = cell("A", "a0", big("inline"));
    put(&mut store, &[&inline]).await;
    assert!(blob_files(store.root()).is_empty());

    store.set_blob_threshold(16);
    let out = cell("B", "a1", "twenty bytes of text");
    put(&mut store, &[&out]).await;
    assert_eq!(blob_files(store.root()).len(), 1);
    store.compact("system").await.unwrap();
    assert_eq!(
        sources(&store.cells().await.unwrap()),
        [inline.source.as_str(), &out.source]
    );
}

#[tokio::test]
async fn a_missing_blob_is_reported_not_papered_over() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let mut store = Store::create(&path, "user:test").await.unwrap();
    put(&mut store, &[&cell("A", "a0", big("x"))]).await;
    drop(store);
    for f in blob_files(&path) {
        std::fs::remove_file(f).unwrap();
    }
    let store = Store::open(&path).await.unwrap();
    assert!(matches!(store.cells().await, Err(Error::Corrupt(_))));
}

#[tokio::test]
async fn prune_deletes_only_blobs_nothing_available_needs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let mut store = Store::create(&path, "user:test").await.unwrap();
    store.set_squash_interval(5);
    let tagged = cell("T", "a0", big("issued"));
    let kept = cell("K", "a1", big("kept"));
    put(&mut store, &[&tagged, &kept]).await;
    store.tag("rev A", "", "user:test").await.unwrap();
    // `T` is rewritten every commit: its old texts are needed only by old versions.
    for i in 0..30 {
        put(&mut store, &[&cell("T", "a0", big(&format!("draft {i}")))]).await;
    }
    let before = blob_files(&path).len();
    assert_eq!(before, 32);
    let size_before = store.history_size_bytes().unwrap();

    let policy = Retention {
        saves: Some(10),
        floor: 10,
        ..Retention::keep_everything()
    };
    let r = store.prune(&policy, "system").await.unwrap().unwrap();
    assert!(r.blobs_deleted > 0, "{r:?}");
    assert_eq!(blob_files(&path).len(), before - r.blobs_deleted);
    assert!(store.history_size_bytes().unwrap() < size_before);

    // Every available version, the pinned ref and the current document still read fully.
    for v in store.earliest_version()..=store.version() {
        store.cells_at(v).await.unwrap();
    }
    let rev_a = store.cells_at_ref("rev A").await.unwrap();
    assert_eq!(sources(&rev_a), [tagged.source.as_str(), &kept.source]);
    drop(store);
    let store = Store::open(&path).await.unwrap();
    assert_eq!(store.cells_at_ref("rev A").await.unwrap()[0].source, tagged.source);
    assert_eq!(store.cells().await.unwrap()[1].source, kept.source);
}

#[tokio::test]
async fn blob_bytes_count_toward_history_size() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("t.cruncha"), "user:test")
        .await
        .unwrap();
    let doc_only = |s: &Store| {
        fn walk(p: &Path) -> u64 {
            std::fs::read_dir(p)
                .unwrap()
                .flatten()
                .map(|e| {
                    let m = e.metadata().unwrap();
                    if m.is_dir() { walk(&e.path()) } else { m.len() }
                })
                .sum()
        }
        walk(&s.root().join("doc"))
    };
    put(&mut store, &[&cell("A", "a0", big("x"))]).await;
    assert!(store.history_size_bytes().unwrap() > doc_only(&store));
}
