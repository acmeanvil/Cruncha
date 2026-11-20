// SPDX-License-Identifier: GPL-3.0-or-later
//! Crash safety: a writer process is SIGKILLed at arbitrary points (mid-commit, mid-blob write,
//! mid-checkpoint);
//! the container must reopen, keep a complete history, and never expose a partial commit.
//! The same holds while history is being pruned (ADR-0011).

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use cruncha_store::{Cell, CellKind, Change, Retention, Store};

const DIR_VAR: &str = "CRUNCHA_CRASH_DIR";

fn tight_policy() -> Retention {
    Retention {
        saves: Some(12),
        floor: 10,
        ..Retention::keep_everything()
    }
}

fn pair(n: u64) -> [Change; 2] {
    let mut a = Cell::new(CellKind::Calc, "a0", format!("a = {n}"));
    a.id = "PAIR_A".into();
    // `b` is large enough to be stored out of line (ADR-0013), so kills also land around blob
    // writes and blob cleanup.
    let pad = "lorem ipsum dolor ".repeat(80);
    let mut b = Cell::new(CellKind::Prose, "a1", format!("b = {n} {pad}"));
    b.id = "PAIR_B".into();
    [Change::Upsert(a), Change::Upsert(b)]
}

/// The child: commits `a = n`, `b = n` pairs forever. Run only by `survives_sigkill_mid_commit`.
#[test]
#[ignore = "helper process for survives_sigkill_mid_commit"]
fn crash_child() {
    let Ok(dir) = std::env::var(DIR_VAR) else {
        return;
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut store = Store::open(&dir).await.expect("child opens store");
        loop {
            let n = store.version() + 1;
            store.commit(&pair(n), "user:child", "pair").await.unwrap();
        }
    });
}

/// The pruning child: edits, tags every so often, and prunes, forever.
#[test]
#[ignore = "helper process for survives_sigkill_mid_prune"]
fn prune_child() {
    let Ok(dir) = std::env::var(DIR_VAR) else {
        return;
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut store = Store::open(&dir).await.expect("child opens store");
        store.set_squash_interval(5);
        loop {
            for _ in 0..4 {
                let n = store.version() + 1;
                store.commit(&pair(n), "user:child", "pair").await.unwrap();
            }
            let v = store.version();
            if v % 3 == 0 {
                store.tag(&format!("t{v}"), "", "user:child").await.unwrap();
            }
            store.prune(&tight_policy(), "system").await.unwrap();
        }
    });
}

async fn check(path: &Path, min_version: u64) -> u64 {
    let store = Store::open(path)
        .await
        .expect("reopen after SIGKILL (lock must be released)");
    let v = store.version();
    let earliest = store.earliest_version();
    assert!(
        v >= min_version,
        "version went backwards: {v} < {min_version}"
    );
    assert_eq!(
        store.history().await.unwrap().len() as u64,
        v - earliest + 1,
        "history incomplete"
    );
    // Atomicity: every version shows both cells from the same commit.
    for version in [v, v.saturating_sub(1), (earliest + v) / 2, earliest]
        .into_iter()
        .filter(|x| *x >= 1)
    {
        let cells = store.cells_at(version).await.unwrap();
        let nums: Vec<&str> = cells
            .iter()
            .map(|c| {
                let rest = c.source.split(" = ").nth(1).unwrap();
                rest.split_whitespace().next().unwrap()
            })
            .collect();
        assert_eq!(cells.len(), 2, "v{version}: {cells:?}");
        assert_eq!(nums[0], nums[1], "torn commit visible at v{version}");
    }
    // Named versions stay readable, pruned or not.
    for r in store.refs().await.unwrap() {
        let cells = store
            .cells_at_ref(&r.name)
            .await
            .unwrap_or_else(|e| panic!("ref {} (v{}): {e}", r.name, r.doc_version));
        assert_eq!(cells.len(), 2, "ref {}", r.name);
    }
    v
}

fn kill_after(exe: &Path, child: &str, path: &Path, delay_ms: u64) {
    let mut child = Command::new(exe)
        .args([child, "--exact", "--ignored", "--nocapture", "--test-threads=1"])
        .env(DIR_VAR, path)
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(delay_ms));
    child.kill().unwrap(); // SIGKILL on Unix
    child.wait().unwrap();
}

#[tokio::test]
async fn survives_sigkill_mid_commit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("crash.cruncha");
    {
        let mut store = Store::create(&path, "user:test").await.unwrap();
        store.commit(&pair(1), "user:test", "init").await.unwrap();
    }
    let exe = std::env::current_exe().unwrap();
    let mut version = 1;
    let mut kills = 0;
    // Varied delays so kills land in different phases (merge, log write, checkpoint at v%100).
    for delay_ms in [150, 420, 777, 1300, 2100, 3300, 5200, 6100] {
        kill_after(&exe, "crash_child", &path, delay_ms);
        kills += 1;
        version = check(&path, version).await;
    }
    assert!(
        version > 20,
        "child made too little progress ({version}) for the test to mean much"
    );
    eprintln!("survived {kills} SIGKILLs; final version {version}");
}

#[tokio::test]
async fn survives_sigkill_mid_prune() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("prune.cruncha");
    {
        let mut store = Store::create(&path, "user:test").await.unwrap();
        store.commit(&pair(1), "user:test", "init").await.unwrap();
    }
    let exe = std::env::current_exe().unwrap();
    let mut version = 1;
    for delay_ms in [600, 950, 1400, 2300, 3100, 4700, 5900, 7300] {
        kill_after(&exe, "prune_child", &path, delay_ms);
        version = check(&path, version).await;
    }
    // A prune interrupted after its point of no return is finished by the next call.
    let mut store = Store::open(&path).await.unwrap();
    store.prune(&tight_policy(), "system").await.unwrap();
    drop(store);
    check(&path, version).await;
    let store = Store::open(&path).await.unwrap();
    assert!(
        store.earliest_version() > 0,
        "the child never pruned (version {version}); the test means little"
    );
    eprintln!(
        "survived 8 SIGKILLs while pruning; final version {}, earliest {}",
        store.version(),
        store.earliest_version()
    );
}
