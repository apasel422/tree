#![feature(plugin)]
#![plugin(quickcheck_macros)]

extern crate collect;
extern crate quickcheck;
extern crate tree;

use collect::compare::Compare;
use quickcheck::TestResult;

type K = u32;
type V = u16;
type M = tree::TreeMap<K, V>;

#[quickcheck]
fn insert_incs_len(mut m: M, k: K, v: V) -> TestResult {
    let old_len = m.len();
    if m.insert(k, v).is_some() { return TestResult::discard(); }
    TestResult::from_bool(m.len() == old_len + 1)
}

#[quickcheck]
fn insert_returns_none(mut m: M, k: K, v: V) -> TestResult {
    if m.get(&k).is_some() { return TestResult::discard(); }
    TestResult::from_bool(m.insert(k, v).is_none())
}

#[quickcheck]
fn insert_sets_val(mut m: M, k: K, v: V) -> TestResult {
    if m.insert(k, v).is_some() { return TestResult::discard(); }
    TestResult::from_bool(m[k] == v)
}

#[quickcheck]
fn reinsert_changes_val(mut m: M, k: K, v1: V, v2: V) -> bool {
    m.insert(k, v1);
    m.insert(k, v2);
    m[k] == v2
}

#[quickcheck]
fn reinsert_keeps_len(mut m: M, k: K, v1: V, v2: V) -> bool {
    m.insert(k, v1);
    let old_len = m.len();
    m.insert(k, v2);
    m.len() == old_len
}

#[quickcheck]
fn reinsert_returns_old_val(mut m: M, k: K, v1: V, v2: V) -> bool {
    m.insert(k, v1);
    m.insert(k, v2) == Some(v1)
}

#[quickcheck]
fn remove_returns_entry(mut m: M, k: K, v: V) -> bool {
    m.insert(k, v);
    m.remove(&k) == Some((k, v))
}

#[quickcheck]
fn remove_decs_len(mut m: M, k: K, v: V) -> bool {
    m.insert(k, v);
    let old_len = m.len();
    m.remove(&k);
    m.len() == old_len - 1
}

#[quickcheck]
fn remove_removes(mut m: M, k: K, v: V) -> bool {
    m.insert(k, v);
    m.remove(&k);
    m.get(&k).is_none()
}

#[quickcheck]
fn max_consistent_with_iter(m: M) -> bool {
    m.max() == m.iter().next_back()
}

#[quickcheck]
fn min_consistent_with_iter(m: M) -> bool {
    m.min() == m.iter().next()
}

#[quickcheck]
fn iter_ascends(m: M) -> bool {
    m.iter().zip(m.iter().skip(1)).all(|(e1, e2)| m.cmp().compares_lt(e1.0, e2.0))
}

#[quickcheck]
fn iter_rev_descends(m: M) -> bool {
    m.iter().rev().zip(m.iter().rev().skip(1)).all(|(e2, e1)| m.cmp().compares_gt(e2.0, e1.0))
}

#[quickcheck]
fn pred_consistent_with_iter_rev(m: M, k: K) -> bool {
    m.pred(&k) == m.iter().rev().find(|e| m.cmp().compares_lt(e.0, &k))
}

#[quickcheck]
fn pred_or_eq_consistent_with_iter_rev(m: M, k: K) -> bool {
    m.pred_or_eq(&k) == m.iter().rev().find(|e| m.cmp().compares_le(e.0, &k))
}

#[quickcheck]
fn succ_consistent_with_iter(m: M, k: K) -> bool {
    m.succ(&k) == m.iter().find(|e| m.cmp().compares_gt(e.0, &k))
}

#[quickcheck]
fn succ_or_eq_consistent_with_iter(m: M, k: K) -> bool {
    m.succ_or_eq(&k) == m.iter().find(|e| m.cmp().compares_ge(e.0, &k))
}

#[quickcheck]
fn clear_empties(mut m: M) -> bool {
    m.clear();
    m.is_empty()
}

#[quickcheck]
fn clear_zeroes_len(mut m: M) -> bool {
    m.clear();
    m.len() == 0
}

#[quickcheck]
fn clear_clears(mut m: M) -> bool {
    m.clear();
    m.iter().next().is_none()
}
