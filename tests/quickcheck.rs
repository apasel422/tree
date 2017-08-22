#![cfg_attr(feature = "range", feature(collections_bound))]

extern crate compare;
extern crate quickcheck;
extern crate tree;

use compare::Compare;
use quickcheck::{Arbitrary, Gen};
use tree::map::{self, Map};

pub trait OccupiedEntry<K, C> where C: Compare<K> {
    fn entry<'a, V>(&self, map: &'a mut Map<K, V, C>) -> Option<map::OccupiedEntry<'a, K, V>>;
}

#[derive(Clone, Debug)]
pub struct RemoveEntry<R>(R);

impl<R> Arbitrary for RemoveEntry<R> where R: Arbitrary {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self { RemoveEntry(R::arbitrary(gen)) }
    fn shrink(&self) -> Box<Iterator<Item=Self>> { Box::new(self.0.shrink().map(RemoveEntry)) }
}

impl<R, K, C> Remove<K, C> for RemoveEntry<R> where R: OccupiedEntry<K, C>, C: Compare<K> {
    fn remove<V>(&self, map: &mut Map<K, V, C>) -> Option<(K, V)> {
        self.0.entry(map).map(map::OccupiedEntry::remove)
    }
}

macro_rules! occupied_entry {
    ($K:ty, $V:ty, $R:ty) => {
        mod occupied_entry {
            remove!{$K, $V, ::RemoveEntry<$R>}
        }
    }
}

pub trait Remove<K, C> where C: Compare<K> {
    fn remove<V>(&self, map: &mut Map<K, V, C>) -> Option<(K, V)>;
}

macro_rules! remove {
    ($K:ty, $V:ty, $R:ty) => {
        mod remove {
            use Remove;
            use quickcheck::{TestResult, quickcheck};
            use tree::Map;

            #[test]
            fn removes_key() {
                fn test(mut map: Map<$K, $V>, removal: $R) -> TestResult {
                    match removal.remove(&mut map) {
                        None => TestResult::discard(),
                        Some((ref key, _)) => TestResult::from_bool(
                            !map.contains_key(key) &&
                            map.get(key).is_none() &&
                            map.get_mut(key).is_none() &&
                            map.iter().find(|e| e.0 == key).is_none()
                        ),
                    }
                }

                quickcheck(test as fn(Map<$K, $V>, $R) -> TestResult);
            }

            #[test]
            fn affects_no_others() {
                fn test(mut map: Map<$K, $V>, removal: $R) -> bool {
                    let old_map = map.clone();

                    match removal.remove(&mut map) {
                        None => map == old_map,
                        Some((ref key, _)) =>
                            map.iter().collect::<Vec<_>>() ==
                               old_map.iter().filter(|e| e.0 != key).collect::<Vec<_>>()
                    }
                }

                quickcheck(test as fn(Map<$K, $V>, $R) -> bool);
            }

            #[test]
            fn sets_len() {
                fn test(mut map: Map<$K, $V>, removal: $R) -> bool {
                    let old_len = map.len();

                    match removal.remove(&mut map) {
                        None => map.len() == old_len,
                        Some(_) => map.len() == old_len - 1,
                    }
                }

                quickcheck(test as fn(Map<$K, $V>, $R) -> bool);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Find<Q>(Q);

impl<Q> Arbitrary for Find<Q> where Q: Arbitrary {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self { Find(Q::arbitrary(gen)) }
    fn shrink(&self) -> Box<Iterator<Item=Self>> { Box::new(self.0.shrink().map(Find)) }
}

impl<Q, K, C> Remove<K, C> for Find<Q> where C: Compare<K> + Compare<Q, K> {
    fn remove<V>(&self, map: &mut Map<K, V, C>) -> Option<(K, V)> { map.remove(&self.0) }
}

impl<K, C> OccupiedEntry<K, C> for Find<K> where K: Clone, C: Compare<K> {
    fn entry<'a, V>(&self, map: &'a mut Map<K, V, C>) -> Option<map::OccupiedEntry<'a, K, V>> {
        match map.entry(self.0.clone()) {
            map::Entry::Occupied(e) => Some(e),
            map::Entry::Vacant(_) => None,
        }
    }
}

pub trait Insert<K> {
    fn key(&self) -> K;
    fn insert<V, C>(self, map: &mut Map<K, V, C>, value: V) -> Option<V> where C: Compare<K>;
}

impl<K> Insert<K> for Find<K> where K: Clone {
    fn key(&self) -> K { self.0.clone() }

    fn insert<V, C>(self, map: &mut Map<K, V, C>, value: V) -> Option<V> where C: Compare<K> {
        map.insert(self.0, value)
    }
}

#[derive(Clone, Debug)]
pub struct FindEntry<K>(K);

impl<K> Arbitrary for FindEntry<K> where K: Arbitrary {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self { FindEntry(K::arbitrary(gen)) }
    fn shrink(&self) -> Box<Iterator<Item=Self>> { Box::new(self.0.shrink().map(FindEntry)) }
}

impl<K> Insert<K> for FindEntry<K> where K: Clone {
    fn key(&self) -> K { self.0.clone() }

    fn insert<V, C>(self, map: &mut Map<K, V, C>, value: V) -> Option<V> where C: Compare<K> {
        use tree::map::Entry;

        match map.entry(self.0) {
            Entry::Occupied(mut e) => Some(e.insert(value)),
            Entry::Vacant(e) => { e.insert(value); None }
        }
    }
}

macro_rules! insert {
    ($K:ty, $V:ty, $R:ty) => {
        mod insert {
            use Insert;
            use quickcheck::quickcheck;
            use tree::Map;

            #[test]
            fn sets_len() {
                fn test(mut map: Map<$K, $V>, r: $R, value: $V) -> bool {
                    let old_len = map.len();

                    if r.insert(&mut map, value).is_some() {
                        map.len() == old_len
                    } else {
                        map.len() == old_len + 1
                    }
                }

                quickcheck(test as fn(Map<$K, $V>, $R, $V) -> bool);
            }

            #[test]
            fn inserts_key() {
                fn test(mut map: Map<$K, $V>, r: $R, mut value: $V) -> bool {
                    let key = r.key();
                    r.insert(&mut map, value);

                    map.contains_key(&key) &&
                    map.get(&key) == Some(&value) &&
                    map.get_mut(&key) == Some(&mut value) &&
                    map.iter().filter(|e| *e.0 == key).collect::<Vec<_>>() == [(&key, &value)]
                }

                quickcheck(test as fn(Map<$K, $V>, $R, $V) -> bool);
            }

            #[test]
            fn affects_no_others() {
                fn test(mut map: Map<$K, $V>, r: $R, value: $V) -> bool {
                    let old_map = map.clone();
                    let key = r.key();
                    r.insert(&mut map, value);

                    map.iter().filter(|e| *e.0 != key).collect::<Vec<_>>() ==
                        old_map.iter().filter(|e| *e.0 != key).collect::<Vec<_>>()
                }

                quickcheck(test as fn(Map<$K, $V>, $R, $V) -> bool);
            }

            #[test]
            fn returns_old_value() {
                fn test(mut map: Map<$K, $V>, r: $R, value: $V) -> bool {
                    let key = r.key();
                    map.get(&key).cloned() == r.insert(&mut map, value)
                }

                quickcheck(test as fn(Map<$K, $V>, $R, $V) -> bool);
            }
        }
    }
}

mod find {
    mod entry {
        use quickcheck::quickcheck;
        use tree::map::{Entry, Map};

        #[test]
        fn agrees_with_get() {
            fn test(mut map: Map<u32, u16>, key: u32) -> bool {
                let value = map.get(&key).cloned();

                match map.entry(key) {
                    Entry::Occupied(e) => value == Some(*e.get()),
                    Entry::Vacant(_) => value.is_none(),
                }
            }

            quickcheck(test as fn(Map<u32, u16>, u32) -> bool);
        }

        insert!{u32, u16, ::FindEntry<u32>}
    }

    insert!{u32, u16, ::Find<u32>}
    occupied_entry!{u32, u16, ::Find<u32>}
    remove!{u32, u16, ::Find<u32>}
}

#[derive(Clone, Debug)]
struct Max;

impl Arbitrary for Max { fn arbitrary<G: Gen>(_gen: &mut G) -> Self { Max } }

impl<K, C> Remove<K, C> for Max where C: Compare<K> {
    fn remove<V>(&self, map: &mut Map<K, V, C>) -> Option<(K, V)> { map.remove_last() }
}

impl<K, C> OccupiedEntry<K, C> for Max where C: Compare<K> {
    fn entry<'a, V>(&self, map: &'a mut Map<K, V, C>) -> Option<map::OccupiedEntry<'a, K, V>> {
        map.last_entry()
    }
}

mod last {
    use quickcheck::quickcheck;
    use tree::Map;

    #[test]
    fn agrees_with_iter() {
        fn test(map: Map<u32, u16>) -> bool {
            map.last() == map.iter().rev().next()
        }

        quickcheck(test as fn(Map<u32, u16>) -> bool);
    }

    occupied_entry!{u32, u16, ::Max}
    remove!{u32, u16, ::Max}
}

#[derive(Clone, Debug)]
struct Min;

impl Arbitrary for Min { fn arbitrary<G: Gen>(_gen: &mut G) -> Self { Min } }

impl<K, C> Remove<K, C> for Min where C: Compare<K> {
    fn remove<V>(&self, map: &mut Map<K, V, C>) -> Option<(K, V)> { map.remove_first() }
}

impl<K, C> OccupiedEntry<K, C> for Min where C: Compare<K> {
    fn entry<'a, V>(&self, map: &'a mut Map<K, V, C>) -> Option<map::OccupiedEntry<'a, K, V>> {
        map.first_entry()
    }
}

mod first {
    use quickcheck::quickcheck;
    use tree::Map;

    #[test]
    fn agrees_with_iter() {
        fn test(map: Map<u32, u16>) -> bool {
            map.first() == map.iter().next()
        }

        quickcheck(test as fn(Map<u32, u16>) -> bool);
    }

    occupied_entry!{u32, u16, ::Min}
    remove!{u32, u16, ::Min}
}

#[derive(Clone, Debug)]
struct Succ<Q>(Q, bool);

impl<Q> Arbitrary for Succ<Q> where Q: Arbitrary {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self { Succ(Q::arbitrary(gen), bool::arbitrary(gen)) }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        Box::new((self.0.clone(), self.1).shrink().map(|(key, inc)| Succ(key, inc)))
    }
}

impl<Q, K, C> Remove<K, C> for Succ<Q> where C: Compare<K> + Compare<Q, K> {
    fn remove<V>(&self, map: &mut Map<K, V, C>) -> Option<(K, V)> {
        map.remove_succ(&self.0, self.1)
    }
}

impl<Q, K, C> OccupiedEntry<K, C> for Succ<Q> where C: Compare<K> + Compare<Q, K> {
    fn entry<'a, V>(&self, map: &'a mut Map<K, V, C>) -> Option<map::OccupiedEntry<'a, K, V>> {
        map.succ_entry(&self.0, self.1)
    }
}

mod succ {
    use quickcheck::quickcheck;
    use tree::Map;

    #[test]
    fn exclusive_agrees_with_iter() {
        fn test(map: Map<u32, u16>, key: u32) -> bool {
            map.succ(&key, false) == map.iter().find(|e| *e.0 > key)
        }

        quickcheck(test as fn(Map<u32, u16>, u32) -> bool);
    }

    #[test]
    fn inclusive_agrees_with_iter() {
        fn test(map: Map<u32, u16>, key: u32) -> bool {
            map.succ(&key, true) == map.iter().find(|e| *e.0 >= key)
        }

        quickcheck(test as fn(Map<u32, u16>, u32) -> bool);
    }

    occupied_entry!{u32, u16, ::Succ<u32>}
    remove!{u32, u16, ::Succ<u32>}
}

#[derive(Clone, Debug)]
struct Pred<Q>(Q, bool);

impl<Q> Arbitrary for Pred<Q> where Q: Arbitrary {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self { Pred(Q::arbitrary(gen), bool::arbitrary(gen)) }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        Box::new((self.0.clone(), self.1).shrink().map(|(key, inc)| Pred(key, inc)))
    }
}

impl<Q, K, C> Remove<K, C> for Pred<Q> where C: Compare<K> + Compare<Q, K> {
    fn remove<V>(&self, map: &mut Map<K, V, C>) -> Option<(K, V)> {
        map.remove_pred(&self.0, self.1)
    }
}

impl<Q, K, C> OccupiedEntry<K, C> for Pred<Q> where C: Compare<K> + Compare<Q, K> {
    fn entry<'a, V>(&self, map: &'a mut Map<K, V, C>) -> Option<map::OccupiedEntry<'a, K, V>> {
        map.pred_entry(&self.0, self.1)
    }
}

mod pred {
    use quickcheck::quickcheck;
    use tree::Map;

    #[test]
    fn exclusive_agrees_with_iter() {
        fn test(map: Map<u32, u16>, key: u32) -> bool {
            map.pred(&key, false) == map.iter().rev().find(|e| *e.0 < key)
        }

        quickcheck(test as fn(Map<u32, u16>, u32) -> bool);
    }

    #[test]
    fn inclusive_agrees_with_iter() {
        fn test(map: Map<u32, u16>, key: u32) -> bool {
            map.pred(&key, true) == map.iter().rev().find(|e| *e.0 <= key)
        }

        quickcheck(test as fn(Map<u32, u16>, u32) -> bool);
    }

    occupied_entry!{u32, u16, ::Pred<u32>}
    remove!{u32, u16, ::Pred<u32>}
}

mod iter {
    use quickcheck::quickcheck;
    use tree::Map;

    #[test]
    fn ascends() {
        fn test(map: Map<u32, u16>) -> bool {
            map.iter().zip(map.iter().skip(1)).all(|(e1, e2)| e1.0 < e2.0)
        }

        quickcheck(test as fn(Map<u32, u16>) -> bool);
    }

    #[test]
    fn descends_when_reversed() {
        fn test(map: Map<u32, u16>) -> bool {
            map.iter().rev().zip(map.iter().rev().skip(1)).all(|(e2, e1)| e2.0 > e1.0)
        }

        quickcheck(test as fn(Map<u32, u16>) -> bool);
    }

    #[test]
    fn size_hint_is_exact() {
        fn test(map: Map<u32, u16>) -> bool {
            let mut len = map.len();
            let mut it = map.iter();

            loop {
                if it.size_hint() != (len, Some(len)) { return false; }
                if it.next().is_none() { break; }
                len -= 1;
            }

            len == 0 && it.size_hint() == (0, Some(0))
        }

        quickcheck(test as fn(Map<u32, u16>) -> bool);
    }
}

#[cfg(feature = "range")]
mod range {
    use quickcheck::{Arbitrary, Gen, quickcheck};
    use std::collections::Bound::*;
    use tree::Map;

    #[derive(Clone, Debug)]
    struct Bound<T>(::std::collections::Bound<T>);

    impl<T> Bound<T> {
        fn as_ref(&self) -> Bound<&T> {
            Bound(match self.0 {
                Included(ref t) => Included(t),
                Excluded(ref t) => Excluded(t),
                Unbounded => Unbounded,
            })
        }
    }

    impl<T> Arbitrary for Bound<T> where T: Arbitrary {
        fn arbitrary<G: Gen>(gen: &mut G) -> Self {
            Bound(match gen.gen_range(0, 3) {
                0 => Included(T::arbitrary(gen)),
                1 => Excluded(T::arbitrary(gen)),
                _ => Unbounded,
            })
        }

        fn shrink(&self) -> Box<Iterator<Item=Self>> {
            match self.0 {
                Included(ref t) => Box::new(t.shrink().map(|t| Bound(Included(t)))),
                Excluded(ref t) => Box::new(t.shrink().map(|t| Bound(Excluded(t)))),
                Unbounded => Box::new(None.into_iter()),
            }
        }
    }

    #[test]
    fn range() {
        fn test(map: Map<u32, u16>, min: Bound<u32>, max: Bound<u32>) -> bool {
            let r = map.range(min.as_ref().0, max.as_ref().0);

            let i = map.iter()
                .skip_while(|e| match min.0 {
                    Included(ref t) => e.0 < t,
                    Excluded(ref t) => e.0 <= t,
                    Unbounded => false,
                })
            .take_while(|e| match max.0 {
                Included(ref t) => e.0 <= t,
                Excluded(ref t) => e.0 < t,
                Unbounded => true,
            });

            r.collect::<Vec<_>>() == i.collect::<Vec<_>>()
        }

        quickcheck(test as fn(Map<u32, u16>, Bound<u32>, Bound<u32>) -> bool);
    }

    #[test]
    fn range_rev() {
        fn test(map: Map<u32, u16>, min: Bound<u32>, max: Bound<u32>) -> bool {
            let r = map.range(min.as_ref().0, max.as_ref().0).rev();

            let i = map.iter().rev()
                .skip_while(|e| match max.0 {
                    Included(ref t) => e.0 > t,
                    Excluded(ref t) => e.0 >= t,
                    Unbounded => false,
                })
            .take_while(|e| match min.0 {
                Included(ref t) => e.0 >= t,
                Excluded(ref t) => e.0 > t,
                Unbounded => true,
            });

            r.collect::<Vec<_>>() == i.collect::<Vec<_>>()
        }

        quickcheck(test as fn(Map<u32, u16>, Bound<u32>, Bound<u32>) -> bool);
    }
}
