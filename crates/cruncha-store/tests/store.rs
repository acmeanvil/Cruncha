// SPDX-License-Identifier: GPL-3.0-or-later

use cruncha_store::{Cell, CellDiff, CellKind, Change, CommitKind, Error, Provenance, Store};
use serde_json::json;

fn calc(order: &str, src: &str, defines: &[&str], reads: &[&str]) -> Cell {
    let mut c = Cell::new(CellKind::Calc, order, src);
    c.defines = defines.iter().map(|s| s.to_string()).collect();
    c.reads = reads.iter().map(|s| s.to_string()).collect();
    c.attrs = json!({"display": "?"});
    c
}

fn sources(cells: &[Cell]) -> Vec<&str> {
    cells.iter().map(|c| c.source.as_str()).collect()
}

async fn new_store(dir: &tempfile::TempDir) -> Store {
    Store::create(dir.path().join("t.cruncha"), "user:test")
        .await
        .unwrap()
}

#[tokio::test]
async fn create_is_empty_at_version_zero() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(&dir).await;
    assert_eq!(store.version(), 0);
    assert!(store.cells().await.unwrap().is_empty());
    let h = store.history().await.unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].kind, CommitKind::Create);
}

#[tokio::test]
async fn create_refuses_existing_path() {
    let dir = tempfile::tempdir().unwrap();
    let err = Store::create(dir.path(), "user:test").await.err().unwrap();
    assert!(matches!(err, Error::AlreadyExists(_)));
}

#[tokio::test]
async fn commit_round_trips_all_fields_and_orders_by_key() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;

    let mut prose = Cell::new(CellKind::Prose, "a0", r#"{"type":"paragraph"}"#);
    prose.provenance = Provenance::AiProposed {
        session: "s1".into(),
    };
    let mut inline = calc("a1", "L = 3.1 m", &["L"], &[]);
    inline.parent_id = Some(prose.id.clone());
    let m = calc("a2", "M = w L^2 / 8 = ?? kip·ft", &["M"], &["w", "L"]);
    let w = calc("a1V", "w = 100 lbf/ft", &["w"], &[]);

    let v = store
        .commit(
            &[
                Change::Upsert(m.clone()),
                Change::Upsert(prose.clone()),
                Change::Upsert(inline.clone()),
                Change::Upsert(w.clone()),
            ],
            "user:test",
            "first",
        )
        .await
        .unwrap();
    assert_eq!(v, 1);

    let cells = store.cells().await.unwrap();
    assert_eq!(
        sources(&cells),
        vec![
            prose.source.as_str(),
            "L = 3.1 m",
            "w = 100 lbf/ft",
            m.source.as_str()
        ]
    );
    let got_m = cells.iter().find(|c| c.id == m.id).unwrap();
    assert!(got_m.same_content(&m));
    assert_eq!(got_m.modified_by, "user:test");
    assert!(got_m.modified_at_us > 0);
    let got_inline = cells.iter().find(|c| c.id == inline.id).unwrap();
    assert_eq!(got_inline.parent_id.as_deref(), Some(prose.id.as_str()));
    let got_prose = cells.iter().find(|c| c.id == prose.id).unwrap();
    assert_eq!(got_prose.provenance, prose.provenance);
    assert!(got_prose.defines.is_empty());
}

#[tokio::test]
async fn update_delete_and_time_travel() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    let b = calc("a1", "y = x + 1 = ?", &["y"], &["x"]);
    let v1 = store
        .commit(
            &[Change::Upsert(a.clone()), Change::Upsert(b.clone())],
            "user:test",
            "add",
        )
        .await
        .unwrap();

    let mut a2 = a.clone();
    a2.source = "x = 2".into();
    let v2 = store
        .commit(
            &[Change::Upsert(a2.clone()), Change::Delete(b.id.clone())],
            "ai:s1",
            "edit",
        )
        .await
        .unwrap();
    assert_eq!(v2, v1 + 1);

    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 2"]);
    assert_eq!(
        sources(&store.cells_at(v1).await.unwrap()),
        vec!["x = 1", "y = x + 1 = ?"]
    );
    assert!(store.cells_at(0).await.unwrap().is_empty());
    assert!(matches!(
        store.cells_at(v2 + 1).await,
        Err(Error::NoSuchVersion(_))
    ));

    let diff = store.diff(v1, v2).await.unwrap();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().any(|d| matches!(d, CellDiff::Modified { before, after } if before.source == "x = 1" && after.source == "x = 2")));
    assert!(
        diff.iter()
            .any(|d| matches!(d, CellDiff::Removed(c) if c.id == b.id))
    );

    let h = store.history().await.unwrap();
    assert_eq!(h[0].version, v2);
    assert_eq!(h[0].actor, "ai:s1");
    assert_eq!(h[0].message, "edit");
    assert_eq!(h[0].kind, CommitKind::Edit);
}

#[tokio::test]
async fn deleting_a_missing_id_is_harmless_and_empty_commit_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let v = store.commit(&[], "user:test", "nothing").await.unwrap();
    assert_eq!(v, 0);
    store
        .commit(&[Change::Delete("nope".into())], "user:test", "del")
        .await
        .unwrap();
    assert!(store.cells().await.unwrap().is_empty());
}

#[tokio::test]
async fn duplicate_ids_in_one_commit_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    let err = store
        .commit(
            &[Change::Upsert(a.clone()), Change::Delete(a.id.clone())],
            "user:test",
            "dup",
        )
        .await
        .err()
        .unwrap();
    assert!(matches!(err, Error::DuplicateId(_)));
    assert_eq!(store.version(), 0);
}

#[tokio::test]
async fn persists_across_reopen_and_lock_is_exclusive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let a = calc("a0", "x = 1", &["x"], &[]);
    {
        let mut store = Store::create(&path, "user:test").await.unwrap();
        store
            .commit(&[Change::Upsert(a.clone())], "user:test", "add")
            .await
            .unwrap();
        assert!(matches!(Store::open(&path).await, Err(Error::Locked(_))));
    }
    let store = Store::open(&path).await.unwrap();
    assert_eq!(store.version(), 1);
    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 1"]);
}

#[tokio::test]
async fn open_rejects_non_container() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Store::open(dir.path()).await,
        Err(Error::NotAContainer(_))
    ));
}

#[tokio::test]
async fn restore_is_a_new_version_and_can_be_undone() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    let b = calc("a1", "y = 2", &["y"], &[]);
    let v1 = store
        .commit(&[Change::Upsert(a.clone())], "user:test", "a")
        .await
        .unwrap();
    let mut a2 = a.clone();
    a2.source = "x = 10".into();
    let v2 = store
        .commit(
            &[Change::Upsert(a2), Change::Upsert(b)],
            "user:test",
            "a2+b",
        )
        .await
        .unwrap();

    let v3 = store.restore(v1, "user:test").await.unwrap();
    assert_eq!(v3, v2 + 1);
    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 1"]);
    let h = store.history().await.unwrap();
    assert_eq!(h[0].kind, CommitKind::Restore);
    assert_eq!(h[0].restored_from, Some(v1));

    // Undo the restore by restoring to v2.
    store.restore(v2, "user:test").await.unwrap();
    assert_eq!(
        sources(&store.cells().await.unwrap()),
        vec!["x = 10", "y = 2"]
    );
    // Restoring to the current state is a no-op.
    let v = store.version();
    assert_eq!(store.restore(v, "user:test").await.unwrap(), v);
}

#[tokio::test]
async fn tags_pin_versions() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    store
        .commit(&[Change::Upsert(a.clone())], "user:test", "a")
        .await
        .unwrap();
    let r = store
        .tag("rev A", "issued for review", "user:test")
        .await
        .unwrap();
    assert!(matches!(
        store.tag("rev A", "", "user:test").await,
        Err(Error::RefExists(_))
    ));

    let mut a2 = a.clone();
    a2.source = "x = 2".into();
    store
        .commit(&[Change::Upsert(a2)], "user:test", "edit")
        .await
        .unwrap();

    assert_eq!(
        sources(&store.cells_at_ref("rev A").await.unwrap()),
        vec!["x = 1"]
    );
    let refs = store.refs().await.unwrap();
    assert_eq!(refs, vec![r]);
    assert!(matches!(
        store.cells_at_ref("rev B").await,
        Err(Error::NoSuchRef(_))
    ));
}

#[tokio::test]
async fn compact_preserves_content_and_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let mut cells: Vec<Cell> = (0..20)
        .map(|i| calc(&format!("a{i:03}"), &format!("x{i} = {i}"), &[], &[]))
        .collect();
    for c in &cells {
        store
            .commit(&[Change::Upsert(c.clone())], "user:test", "add")
            .await
            .unwrap();
    }
    cells[3].source = "x3 = 33".into();
    let before_compact = store
        .commit(&[Change::Upsert(cells[3].clone())], "user:test", "edit")
        .await
        .unwrap();

    let stats = store.compact("system").await.unwrap();
    assert!(
        stats.files_before > 1 && stats.files_after == 1,
        "{stats:?}"
    );
    assert_eq!(store.version(), before_compact + 1);

    let now = store.cells().await.unwrap();
    assert_eq!(now.len(), 20);
    assert_eq!(now[3].source, "x3 = 33");
    assert_eq!(store.cells_at(5).await.unwrap().len(), 5);
    let h = store.history().await.unwrap();
    assert_eq!(h[0].kind, CommitKind::Maintenance);
    assert_eq!(h.len() as u64, store.version() + 1);

    drop(store);
    let reopened = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    assert_eq!(reopened.cells().await.unwrap().len(), 20);
}

// ---- revision-log layout (ADR-0010) ----

async fn snapshot_all(store: &Store) -> Vec<Vec<Cell>> {
    let mut out = Vec::new();
    for v in 0..=store.version() {
        out.push(store.cells_at(v).await.unwrap());
    }
    out
}

fn same(a: &[Cell], b: &[Cell]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.same_content(y))
}

#[tokio::test]
async fn squash_leaves_every_earlier_version_identical() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    store.set_squash_interval(1_000); // squash only when we say so
    let mut cells: Vec<Cell> = (0..6)
        .map(|i| calc(&format!("a{i}"), &format!("x{i} = {i}"), &[], &[]))
        .collect();
    for c in &cells {
        store
            .commit(&[Change::Upsert(c.clone())], "user:test", "add")
            .await
            .unwrap();
    }
    cells[2].source = "x2 = 22".into();
    store
        .commit(
            &[
                Change::Upsert(cells[2].clone()),
                Change::Delete(cells[4].id.clone()),
            ],
            "user:test",
            "edit",
        )
        .await
        .unwrap();
    let before = snapshot_all(&store).await;

    store.compact("system").await.unwrap();
    let after = snapshot_all(&store).await;
    assert_eq!(
        after.len(),
        before.len() + 1,
        "squash adds exactly one version"
    );
    for (v, (b, a)) in before.iter().zip(&after).enumerate() {
        assert!(same(b, a), "version {v} changed after squash");
    }
    assert!(
        same(after.last().unwrap(), before.last().unwrap()),
        "squash changed content"
    );

    // Edits after a squash still win over squashed rows.
    cells[2].source = "x2 = 222".into();
    store
        .commit(&[Change::Upsert(cells[2].clone())], "user:test", "edit")
        .await
        .unwrap();
    let now = store.cells().await.unwrap();
    assert_eq!(now.len(), 5);
    assert!(now.iter().any(|c| c.source == "x2 = 222"));
}

#[tokio::test]
async fn automatic_squash_keeps_live_files_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    store.set_squash_interval(5);
    let c = calc("a0", "x = 0", &["x"], &[]);
    for i in 0..12 {
        let mut c = c.clone();
        c.source = format!("x = {i}");
        store
            .commit(&[Change::Upsert(c)], "user:test", "edit")
            .await
            .unwrap();
    }
    let h = store.history().await.unwrap();
    let squashes = h
        .iter()
        .filter(|v| v.kind == CommitKind::Maintenance)
        .count();
    assert_eq!(squashes, 2, "squash after every 5 edits");
    assert_eq!(store.cells().await.unwrap()[0].source, "x = 11");
    // 12 edits + 2 squashes + create.
    assert_eq!(store.version(), 14);
}

#[tokio::test]
async fn empty_document_squashes_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let mut store = Store::create(&path, "user:test").await.unwrap();
    let a = calc("a0", "x = 1", &["x"], &[]);
    store
        .commit(&[Change::Upsert(a.clone())], "user:test", "add")
        .await
        .unwrap();
    store
        .commit(&[Change::Delete(a.id.clone())], "user:test", "del")
        .await
        .unwrap();
    store.compact("system").await.unwrap();
    assert!(store.cells().await.unwrap().is_empty());
    assert_eq!(store.cells_at(1).await.unwrap().len(), 1);
    drop(store);
    let store = Store::open(&path).await.unwrap();
    assert!(store.cells().await.unwrap().is_empty());
}

/// Build a format-1 container by hand (one row per cell, no rev/deleted columns), the way
/// pre-ADR-0010 builds wrote it.
async fn make_v1_container(path: &std::path::Path, versions: &[Vec<Cell>]) {
    use cruncha_store::schema::{DOC_SCHEMA, REFS_SCHEMA, changes_to_batch, conform};
    use deltalake::DeltaTable;
    use deltalake::kernel::StructType;
    use deltalake::kernel::engine::arrow_conversion::TryIntoKernel as _;
    use deltalake::protocol::SaveMode;

    std::fs::create_dir_all(path).unwrap();
    for (name, schema) in [("doc", &*DOC_SCHEMA), ("refs", &*REFS_SCHEMA)] {
        let p = path.join(name);
        std::fs::create_dir_all(&p).unwrap();
        let kernel: StructType = schema.as_ref().try_into_kernel().unwrap();
        let url = url::Url::from_directory_path(p.canonicalize().unwrap()).unwrap();
        let mut t = DeltaTable::try_from_url(url)
            .await
            .unwrap()
            .create()
            .with_columns(kernel.fields().cloned())
            .with_configuration([("delta.enableExpiredLogCleanup", Some("false"))])
            .await
            .unwrap();
        if name == "doc" {
            // Each v1 version here is a full overwrite with that version's cells.
            for cells in versions {
                let changes: Vec<Change> = cells.iter().cloned().map(Change::Upsert).collect();
                let batch = conform(
                    &changes_to_batch(&changes, "user:v1", 1).unwrap(),
                    &DOC_SCHEMA,
                )
                .unwrap();
                t = t
                    .write([batch])
                    .with_save_mode(SaveMode::Overwrite)
                    .await
                    .unwrap();
            }
        }
    }
    let manifest = serde_json::json!({
        "format": "cruncha-container", "format_version": 1,
        "id": "01JV1CONTAINER000000000000", "created_at_us": 1
    });
    std::fs::write(
        path.join("cruncha.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
}

#[tokio::test]
async fn format_1_container_migrates_with_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.cruncha");
    let a = calc("a0", "x = 1", &["x"], &[]);
    let b = calc("a1", "y = 2", &["y"], &[]);
    let mut a2 = a.clone();
    a2.source = "x = 10".into();
    make_v1_container(&path, &[vec![a.clone()], vec![a2.clone(), b.clone()]]).await;

    let mut store = Store::open(&path).await.unwrap();
    assert_eq!(store.manifest().format_version, cruncha_store::FORMAT_VERSION);
    // v1 history is still readable: v1 = [a], v2 = [a2, b]; v3 = the migration.
    assert_eq!(sources(&store.cells_at(1).await.unwrap()), vec!["x = 1"]);
    assert_eq!(
        sources(&store.cells_at(2).await.unwrap()),
        vec!["x = 10", "y = 2"]
    );
    assert_eq!(
        sources(&store.cells().await.unwrap()),
        vec!["x = 10", "y = 2"]
    );

    // New-format edits win over migrated rows; deletes work on them too.
    let mut a3 = a.clone();
    a3.source = "x = 100".into();
    store
        .commit(
            &[Change::Upsert(a3), Change::Delete(b.id.clone())],
            "user:test",
            "edit",
        )
        .await
        .unwrap();
    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 100"]);
    store.compact("system").await.unwrap();
    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 100"]);
    assert_eq!(
        sources(&store.cells_at(2).await.unwrap()),
        vec!["x = 10", "y = 2"]
    );

    // Reopening a migrated container doesn't migrate again.
    let v = store.version();
    drop(store);
    let store = Store::open(&path).await.unwrap();
    assert_eq!(store.version(), v);
}
