// SPDX-License-Identifier: GPL-3.0-or-later
//! History retention (ADR-0011): limits, the floor, named versions, the marker commit, and
//! the hard requirement that time passing never prunes anything.

use cruncha_store::{Cell, CellKind, Change, CommitKind, Error, Retention, Store};

const DAY_US: i64 = 86_400_000_000;
const YEAR_US: i64 = 365 * DAY_US;

async fn new_store(dir: &tempfile::TempDir) -> Store {
    Store::create(dir.path().join("t.cruncha"), "user:test")
        .await
        .unwrap()
}

/// Commit `n` edits: each rewrites cell X and adds a new cell.
async fn edits(store: &mut Store, n: u64) {
    for _ in 0..n {
        let v = store.version() + 1;
        let mut x = Cell::new(CellKind::Calc, "a0", format!("x = {v}"));
        x.id = "X".into();
        let y = Cell::new(CellKind::Calc, format!("b{v:06}"), format!("y{v} = x"));
        store
            .commit(&[Change::Upsert(x), Change::Upsert(y)], "user:test", "edit")
            .await
            .unwrap();
    }
}

fn limits(saves: Option<u64>, size: Option<u64>, days: Option<u32>) -> Retention {
    Retention {
        saves,
        size_bytes: size,
        activity_days: days,
        floor: 10,
    }
}

async fn all_versions(store: &Store) -> Vec<(u64, Vec<Cell>)> {
    let mut out = Vec::new();
    for v in store.earliest_version()..=store.version() {
        out.push((v, store.cells_at(v).await.unwrap()));
    }
    out
}

fn same(a: &[Cell], b: &[Cell]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.id == y.id && x.same_content(y))
}

#[tokio::test]
async fn keep_everything_never_prunes() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    edits(&mut store, 30).await;
    let r = store
        .prune(&Retention::keep_everything(), "system")
        .await
        .unwrap();
    assert!(r.is_none());
    assert_eq!(store.earliest_version(), 0);
    assert_eq!(store.history().await.unwrap().len() as u64, store.version() + 1);
}

#[tokio::test]
async fn saves_limit_prunes_keeps_the_rest_identical_and_marks_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    store.set_squash_interval(8); // squashes leave removed files for VACUUM to delete
    edits(&mut store, 40).await;
    let before = all_versions(&store).await;
    let size_before = store.history_size_bytes().unwrap();
    let latest = store.version();

    let policy = limits(Some(15), None, None);
    let preview = store.retention_preview(&policy).await.unwrap();
    assert_eq!(preview.cutoff, Some(latest + 1 - 15));
    assert!(!preview.waits_for_edit);
    assert!(preview.bytes_freed > 0);

    let r = store.prune(&policy, "system").await.unwrap().unwrap();
    assert_eq!(r.pruned_before, latest + 1 - 15);
    assert!(r.data_files_deleted > 0, "{r:?}");
    assert!(r.log_files_deleted > 0, "{r:?}");
    assert_eq!(store.earliest_version(), r.pruned_before);
    assert!(store.history_size_bytes().unwrap() < size_before);

    // Pruned versions say so; the rest are exactly as before.
    let err = store.cells_at(r.pruned_before - 1).await.unwrap_err();
    assert!(matches!(err, Error::Pruned { earliest } if earliest == r.pruned_before));
    for (v, cells) in before.iter().filter(|(v, _)| *v >= r.pruned_before) {
        assert!(same(cells, &store.cells_at(*v).await.unwrap()), "v{v} changed");
    }
    assert!(same(&before.last().unwrap().1, &store.cells().await.unwrap()));

    // History starts at the cutoff and ends with the marker.
    let h = store.history().await.unwrap();
    assert_eq!(h.len() as u64, store.version() - store.earliest_version() + 1);
    assert_eq!(h.last().unwrap().version, r.pruned_before);
    let marker = &h[0];
    assert_eq!(marker.kind, CommitKind::Maintenance);
    assert_eq!(marker.pruned_before, Some(r.pruned_before));
    assert!(marker.message.contains("pruned"), "{}", marker.message);

    // It all survives a reopen, and the document keeps working.
    drop(store);
    let mut store = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    assert_eq!(store.earliest_version(), r.pruned_before);
    edits(&mut store, 3).await;
    assert_eq!(
        store.history().await.unwrap().len() as u64,
        store.version() - store.earliest_version() + 1
    );
    assert_eq!(store.cells().await.unwrap().len(), 41 + 3);
}

#[tokio::test]
async fn size_limit_prunes_to_budget() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    edits(&mut store, 40).await;
    let latest = store.version();
    // Big enough for some versions, too small for all.
    let budget = store.history_size_bytes().unwrap() / 3;
    let r = store
        .prune(&limits(None, Some(budget), None), "system")
        .await
        .unwrap()
        .unwrap();
    assert!(r.pruned_before > 0 && r.pruned_before <= latest + 1 - 10, "{r:?}");
}

#[tokio::test]
async fn activity_window_counts_back_from_the_last_edit() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    edits(&mut store, 20).await; // day 0
    let recent_start = store.version() + 1;
    store.set_clock_offset_us(60 * DAY_US);
    edits(&mut store, 20).await; // day 60: the last edits

    // 30 days before the last edit reaches day 30: only the day-60 batch survives.
    let r = store
        .prune(&limits(None, None, Some(30)), "system")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(r.pruned_before, recent_start);
}

#[tokio::test]
async fn strictest_limit_wins_and_floor_holds() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    edits(&mut store, 60).await;
    let latest = store.version();
    let p = limits(Some(40), None, Some(3650));
    assert_eq!(
        store.retention_preview(&p).await.unwrap().cutoff,
        Some(latest + 1 - 40)
    );
    let p = Retention {
        saves: Some(1),
        floor: 25,
        ..Retention::keep_everything()
    };
    let r = store.prune(&p, "system").await.unwrap().unwrap();
    assert_eq!(r.pruned_before, latest + 1 - 25);
    // The 25 kept, then the maintenance commits of the prune itself (vacuum, marker).
    let h = store.history().await.unwrap();
    assert_eq!(h.len() as u64, store.version() - r.pruned_before + 1);
    assert!(h[..h.len() - 25].iter().all(|v| v.kind == CommitKind::Maintenance));
}

/// The hard requirement: a finished study opened years later has exactly its history.
#[tokio::test]
async fn dormant_document_never_prunes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    {
        let mut store = Store::create(&path, "user:test").await.unwrap();
        // Every edit made five years ago.
        store.set_clock_offset_us(-5 * YEAR_US);
        edits(&mut store, 30).await;
    }
    let mut store = Store::open(&path).await.unwrap();
    let before = store.history().await.unwrap();

    // An activity window counts from the last edit, so it keeps all of it.
    let p = limits(None, None, Some(30));
    assert!(store.prune(&p, "system").await.unwrap().is_none());

    // A saves limit prunes once...
    let p = limits(Some(12), None, Some(30));
    let r = store.prune(&p, "system").await.unwrap().unwrap();
    let after = store.history().await.unwrap();
    // ...and never again without new edits, however long the document sits.
    drop(store);
    let mut store = Store::open(&path).await.unwrap();
    store.set_clock_offset_us(5 * YEAR_US);
    assert!(store.prune(&p, "system").await.unwrap().is_none());
    assert!(store.retention_preview(&p).await.unwrap().waits_for_edit);
    let again = store.history().await.unwrap();
    assert_eq!(again.len(), after.len());
    assert_eq!(store.earliest_version(), r.pruned_before);
    assert!(before.len() > after.len());
}

#[tokio::test]
async fn prune_waits_for_new_edits() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    edits(&mut store, 30).await;
    let p = limits(Some(12), None, None);
    let first = store.prune(&p, "system").await.unwrap().unwrap();
    assert!(store.prune(&p, "system").await.unwrap().is_none());
    // Tagging and compaction aren't edits.
    store.tag("rev A", "", "user:test").await.unwrap();
    store.compact("system").await.unwrap();
    assert!(store.prune(&p, "system").await.unwrap().is_none());
    edits(&mut store, 5).await;
    let second = store.prune(&p, "system").await.unwrap().unwrap();
    assert!(second.pruned_before > first.pruned_before);
}

#[tokio::test]
async fn named_versions_survive_pruning() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let mut store = Store::create(&path, "user:test").await.unwrap();
    store.tag("empty", "before anything", "user:test").await.unwrap();
    edits(&mut store, 5).await;
    store.tag("rev A", "issued for review", "user:test").await.unwrap();
    let rev_a = store.cells().await.unwrap();
    edits(&mut store, 30).await;
    store.tag("rev B", "", "user:test").await.unwrap();

    let p = limits(Some(10), None, None);
    let preview = store.retention_preview(&p).await.unwrap();
    let names: Vec<_> = preview.refs_to_pin.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["empty", "rev A"]);

    let r = store.prune(&p, "system").await.unwrap().unwrap();
    assert_eq!(r.pinned, ["empty", "rev A"]);
    assert!(store.cells_at(6).await.is_err());
    assert!(same(&store.cells_at_ref("rev A").await.unwrap(), &rev_a));
    assert!(store.cells_at_ref("empty").await.unwrap().is_empty());
    assert_eq!(store.cells_at_ref("rev B").await.unwrap().len(), 36);

    // Pins persist, aren't duplicated by later prunes, and survive a reopen.
    edits(&mut store, 5).await;
    let r2 = store.prune(&p, "system").await.unwrap().unwrap();
    assert!(r2.pinned.is_empty());
    drop(store);
    let mut store = Store::open(&path).await.unwrap();
    assert!(same(&store.cells_at_ref("rev A").await.unwrap(), &rev_a));
    assert_eq!(store.refs().await.unwrap().len(), 3);

    // A named version can be restored after its version was pruned (from its kept copy).
    let v = store.restore_ref("rev A", "user:test").await.unwrap();
    assert!(same(&store.cells().await.unwrap(), &rev_a));
    let h = store.history().await.unwrap();
    assert_eq!(h[0].version, v);
    assert_eq!(h[0].kind, CommitKind::Restore);
    assert_eq!(h[0].message, "restore to “rev A”");
    assert!(matches!(store.restore_ref("nope", "user:test").await, Err(Error::NoSuchRef(_))));
}

#[tokio::test]
async fn restore_past_the_cutoff_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    edits(&mut store, 30).await;
    let r = store
        .prune(&limits(Some(10), None, None), "system")
        .await
        .unwrap()
        .unwrap();
    let err = store.restore(1, "user:test").await.unwrap_err();
    assert!(matches!(err, Error::Pruned { .. }));
    store.restore(r.pruned_before, "user:test").await.unwrap();
}

#[tokio::test]
async fn policy_is_stored_in_the_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let mut store = Store::create(&path, "user:test").await.unwrap();
    assert!(store.retention().is_none()); // use the app default
    let p = limits(Some(1000), Some(500_000_000), Some(90));
    store.set_retention(Some(p.clone())).unwrap();
    drop(store);
    let store = Store::open(&path).await.unwrap();
    assert_eq!(store.retention(), Some(&p));
}
