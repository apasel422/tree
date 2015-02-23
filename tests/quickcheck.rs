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
fn max_consistent_with_rev_iter(m: M) -> bool {
    m.max() == m.rev_iter().next()
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
fn rev_iter_descends(m: M) -> bool {
    m.rev_iter().zip(m.rev_iter().skip(1)).all(|(e2, e1)| m.cmp().compares_gt(e2.0, e1.0))
}
