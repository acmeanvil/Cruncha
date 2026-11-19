// SPDX-License-Identifier: GPL-3.0-or-later
//! Crash safety: a writer process is SIGKILLed at arbitrary points (mid-MERGE, mid-checkpoint);
//! the container must reopen, keep a complete history, and never expose a partial commit.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use cruncha_store::{Cell, CellKind, Change, Store};

const DIR_VAR: &str = "CRUNCHA_CRASH_DIR";

fn pair(n: u64) -> [Change; 2] {
    let mut a = Cell::new(CellKind::Calc, "a0", format!("a = {n}"));
    a.id = "PAIR_A".into();
    let mut b = Cell::new(CellKind::Calc, "a1", format!("b = {n}"));
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

async fn check(path: &Path, min_version: u64) -> u64 {
    let store = Store::open(path)
        .await
        .expect("reopen after SIGKILL (lock must be released)");
    let v = store.version();
    assert!(
        v >= min_version,
        "version went backwards: {v} < {min_version}"
    );
    assert_eq!(
        store.history().await.unwrap().len() as u64,
        v + 1,
        "history incomplete"
    );
    // Atomicity: every version shows both cells from the same commit.
    for version in [v, v.saturating_sub(1), v / 2]
        .into_iter()
        .filter(|x| *x >= 1)
    {
        let cells = store.cells_at(version).await.unwrap();
        let nums: Vec<&str> = cells
            .iter()
            .map(|c| c.source.split(" = ").nth(1).unwrap())
            .collect();
        assert_eq!(cells.len(), 2, "v{version}: {cells:?}");
        assert_eq!(nums[0], nums[1], "torn commit visible at v{version}");
    }
    v
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
        let mut child = Command::new(&exe)
            .args([
                "crash_child",
                "--exact",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(DIR_VAR, &path)
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_millis(delay_ms));
        child.kill().unwrap(); // SIGKILL on Unix
        child.wait().unwrap();
        kills += 1;
        version = check(&path, version).await;
    }
    assert!(
        version > 20,
        "child made too little progress ({version}) for the test to mean much"
    );
    eprintln!("survived {kills} SIGKILLs; final version {version}");
}
