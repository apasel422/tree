#![feature(box_syntax)]
#![feature(collections, core)]
#![feature(custom_attribute)]
#![feature(plugin)]
#![plugin(quickcheck_macros)]

extern crate compare;
extern crate quickcheck;
extern crate rand;
extern crate tree;

use compare::Compare;
use quickcheck::{Arbitrary, Gen, TestResult};
use std::collections::Bound as StdBound;
use std::iter::order;

type K = u32;
type V = u16;
type M = tree::Map<K, V>;

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

#[derive(Clone, Debug)]
enum Bound<T> {
    Included(T),
    Excluded(T),
    Unbounded,
}

impl<T> Bound<T> {
    fn as_ref(&self) -> Bound<&T> {
        match *self {
            Bound::Included(ref t) => Bound::Included(t),
            Bound::Excluded(ref t) => Bound::Excluded(t),
            Bound::Unbounded => Bound::Unbounded,
        }
    }

    fn to_std_bound(self) -> StdBound<T> {
        match self {
            Bound::Included(t) => StdBound::Included(t),
            Bound::Excluded(t) => StdBound::Excluded(t),
            Bound::Unbounded => StdBound::Unbounded,
        }
    }
}

impl<T> Arbitrary for Bound<T> where T: Arbitrary {
    fn arbitrary<G: Gen>(gen: &mut G) -> Bound<T> {
        match gen.gen_range(0, 3) {
            0 => Bound::Included(Arbitrary::arbitrary(gen)),
            1 => Bound::Excluded(Arbitrary::arbitrary(gen)),
            _ => Bound::Unbounded,
        }
    }

    fn shrink(&self) -> Box<Iterator<Item=Bound<T>>> {
        match *self {
            Bound::Included(ref t) => box t.shrink().map(Bound::Included),
            Bound::Excluded(ref t) => box t.shrink().map(Bound::Excluded),
            Bound::Unbounded => box None.into_iter(),
        }
    }
}

#[quickcheck]
fn range(m: M, min: Bound<K>, max: Bound<K>) -> bool {
    let r = m.range(min.as_ref().to_std_bound(), max.as_ref().to_std_bound());

    let i = m.iter()
        .skip_while(|e| match min {
            Bound::Included(ref t) => e.0 < t,
            Bound::Excluded(ref t) => e.0 <= t,
            Bound::Unbounded => false,
        })
        .take_while(|e| match max {
            Bound::Included(ref t) => e.0 <= t,
            Bound::Excluded(ref t) => e.0 < t,
            Bound::Unbounded => true,
        });

    order::equals(r, i)
}

#[quickcheck]
fn range_rev(m: M, min: Bound<K>, max: Bound<K>) -> bool {
    let r = m.range(min.as_ref().to_std_bound(), max.as_ref().to_std_bound()).rev();

    let i = m.iter().rev()
        .skip_while(|e| match max {
            Bound::Included(ref t) => e.0 > t,
            Bound::Excluded(ref t) => e.0 >= t,
            Bound::Unbounded => false,
        })
        .take_while(|e| match min {
            Bound::Included(ref t) => e.0 >= t,
            Bound::Excluded(ref t) => e.0 > t,
            Bound::Unbounded => true,
        });

    order::equals(r, i)
}
