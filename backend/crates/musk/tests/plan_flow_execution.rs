//! plan_flow_execution.rs — PLAN-096 V03: execution-scope tests against real
//! temp Git repos + worktrees. T-03 subset: concurrent plan allocation
//! (CreateNew, no overwrite, >999 boundary). T-04+ extends with worktree
//! scope / main-root protection / Canvas path cases.
//!
//! Serial: shares process state (global allocation mutex is per-store, but
//! the >999 case touches plan dirs inside its own tempdir).

use std::sync::Arc;

use musk::plans::PlansStore;

fn tmp_store() -> (tempfile::TempDir, PlansStore) {
    let td = tempfile::tempdir().unwrap();
    let store = PlansStore::new(td.path().join("docs/plans"));
    (td, store)
}

/// AC-16：并发 create 同仓唯一分配——CreateNew 不覆盖既有计划，撞号重分配，
/// N 个并发创建得到 N 个互异序号、字节级独立文件。
#[test]
fn concurrent_create_allocates_unique_seqs_without_overwrite() {
    let (_td, store) = tmp_store();
    let store = Arc::new(store);
    const N: usize = 24;
    let mut handles = Vec::new();
    for i in 0..N {
        let s = store.clone();
        handles.push(std::thread::spawn(move || {
            s.create(&format!("Feature {i}"), "").unwrap()
        }));
    }
    let mut seqs = Vec::new();
    for h in handles {
        let pf = h.join().unwrap();
        seqs.push(pf.seq);
    }
    seqs.sort();
    let unique: std::collections::BTreeSet<u32> = seqs.iter().copied().collect();
    assert_eq!(unique.len(), N, "seqs must be unique: {seqs:?}");
    assert_eq!(seqs[0], 1, "contiguous from 1");
    assert_eq!(*seqs.last().unwrap(), N as u32);
    // 每个文件独立存在（无覆盖：总数 == N）。
    let all = store.list(true);
    assert_eq!(all.len(), N);
    // 回读 id 与文件名前缀一致。
    for pf in all {
        assert_eq!(pf.id, format!("PLAN-{:03}", pf.seq));
    }
}

/// AC-16：>999 明确阻断——不截断、不环绕、不覆盖旧计划。
#[test]
fn create_beyond_999_is_blocked_loudly() {
    let (_td, store) = tmp_store();
    // 手工放一个 999 号文件（绕过 create，模拟历史满号仓）。
    std::fs::write(
        store.plans_dir.join("999-full.md"),
        "---\nplan_id: PLAN-999\nstatus: drafting\n---\n# full\n",
    )
    .unwrap();
    let err = store.create("One Too Many", "").unwrap_err();
    assert!(err.contains("999"), "{err}");
    assert!(err.contains("blocked") || err.contains("exceeds"), "{err}");
    // 旧计划字节不变。
    let bytes = std::fs::read_to_string(store.plans_dir.join("999-full.md")).unwrap();
    assert_eq!(bytes, "---\nplan_id: PLAN-999\nstatus: drafting\n---\n# full\n");
    assert_eq!(store.list(true).len(), 1);
}

/// AC-16：撞号重分配——目标路径已被占（同号文件）时 create_new 冲突 →
/// 自动顺延下一个空号，绝不覆盖。
#[test]
fn create_on_collision_reallocates_instead_of_overwriting() {
    let (_td, store) = tmp_store();
    store.create("alpha", "").unwrap(); // 001
    // 手工占用 002 号位（模拟并发写者抢先落盘）。
    let squatter = store.plans_dir.join("002-squatter.md");
    std::fs::write(&squatter, "---\nplan_id: PLAN-002\n---\n# squatted\n").unwrap();
    let pf = store.create("beta", "").unwrap();
    assert_eq!(pf.seq, 3, "collided 002 → reallocates to 003");
    // squatter 字节不变。
    let bytes = std::fs::read_to_string(&squatter).unwrap();
    assert_eq!(bytes, "---\nplan_id: PLAN-002\n---\n# squatted\n");
}
