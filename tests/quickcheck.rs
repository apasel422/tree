#![feature(collections, core, custom_attribute, plugin)]
#![plugin(quickcheck_macros)]

extern crate quickcheck;
extern crate tree;

type K = u32;
type V = u16;
type Map = tree::Map<K, V>;

mod insert {
    use quickcheck::TestResult;
    use super::{K, Map, V};

    #[quickcheck]
    fn increments_len_when_key_is_absent(mut map: Map, key: K, value: V) -> TestResult {
        if map.contains_key(&key) {
            TestResult::discard()
        } else {
            let old_len = map.len();
            map.insert(key, value);
            TestResult::from_bool(map.len() == old_len + 1)
        }
    }

    #[quickcheck]
    fn maintains_len_when_key_is_present(mut map: Map, key: K, value: V) -> TestResult {
        if map.contains_key(&key) {
            let old_len = map.len();
            map.insert(key, value);
            TestResult::from_bool(map.len() == old_len)
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn returns_some_when_key_is_present(mut map: Map, key: K, value: V) -> TestResult {
        if map.contains_key(&key) {
            let old_value = map[&key];
            TestResult::from_bool(map.insert(key, value) == Some(old_value))
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn returns_none_when_key_is_absent(mut map: Map, key: K, value: V) -> TestResult {
        if map.contains_key(&key) {
            TestResult::discard()
        } else {
            TestResult::from_bool(map.insert(key, value).is_none())
        }
    }

    #[quickcheck]
    fn inserts_key_when_key_is_absent(mut map: Map, key: K, mut value: V) -> TestResult {
        if map.contains_key(&key) {
            TestResult::discard()
        } else {
            map.insert(key, value);
            TestResult::from_bool(
                map.contains_key(&key) &&
                map.get(&key) == Some(&value) &&
                map.get_mut(&key) == Some(&mut value) &&
                map[&key] == value &&
                map.iter().filter(|e| *e.0 == key).collect::<Vec<_>>() == [(&key, &value)]
            )
        }
    }

    #[quickcheck]
    fn updates_value_when_key_is_absent(mut map: Map, key: K, mut value: V) -> TestResult {
        if map.contains_key(&key) {
            map.insert(key, value);
            TestResult::from_bool(
                map.contains_key(&key) &&
                map.get(&key) == Some(&value) &&
                map.get_mut(&key) == Some(&mut value) &&
                map.iter().filter(|e| *e.0 == key).collect::<Vec<_>>() == [(&key, &value)]
            )
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn affects_no_other_keys(mut map: Map, key: K, value: V) -> bool {
        let others: Vec<_> = map.iter()
            .filter_map(|(k, v)| if *k == key { None } else { Some((k.clone(), v.clone())) })
            .collect();
        map.insert(key, value);
        ::std::iter::order::equals(map.into_iter().filter(|e| e.0 != key), others.into_iter())
    }
}

mod remove {
    use quickcheck::TestResult;
    use super::{K, Map};

    #[quickcheck]
    fn decrements_len_when_key_is_present(mut map: Map, key: K) -> TestResult {
        if map.contains_key(&key) {
            let old_len = map.len();
            map.remove(&key);
            TestResult::from_bool(map.len() == old_len - 1)
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn maintains_len_when_key_is_absent(mut map: Map, key: K) -> TestResult {
        if map.contains_key(&key) {
            TestResult::discard()
        } else {
            let old_len = map.len();
            map.remove(&key);
            TestResult::from_bool(map.len() == old_len)
        }
    }

    #[quickcheck]
    fn returns_some_when_key_is_present(mut map: Map, key: K) -> TestResult {
        if map.contains_key(&key) {
            let value = map[&key];
            TestResult::from_bool(map.remove(&key) == Some((key, value)))
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn returns_none_when_key_is_absent(mut map: Map, key: K) -> TestResult {
        if map.contains_key(&key) {
            TestResult::discard()
        } else {
            TestResult::from_bool(map.remove(&key).is_none())
        }
    }

    #[quickcheck]
    fn removes_key(mut map: Map, key: K) -> TestResult {
        if map.contains_key(&key) {
            map.remove(&key);
            TestResult::from_bool(
                !map.contains_key(&key) &&
                map.get(&key).is_none() &&
                map.get_mut(&key).is_none() &&
                map.iter().find(|e| *e.0 == key).is_none()
            )
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn removes_no_other_keys_when_key_is_present(mut map: Map, key: K) -> TestResult {
        if map.contains_key(&key) {
            let others: Vec<_> = map.iter()
                .filter_map(|(k, v)| if *k == key { None } else { Some((k.clone(), v.clone())) })
                .collect();
            map.remove(&key);
            TestResult::from_bool(::std::iter::order::equals(map.into_iter(), others.into_iter()))
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn removes_no_other_keys_when_key_is_absent(mut map: Map, key: K) -> TestResult {
        if map.contains_key(&key) {
            TestResult::discard()
        } else {
            let old_map = map.clone();
            map.remove(&key);
            TestResult::from_bool(map == old_map)
        }
    }
}

#[quickcheck]
fn max_agrees_with_iter(map: Map) -> bool {
    map.max() == map.iter().rev().next()
}

#[quickcheck]
fn min_agrees_with_iter(map: Map) -> bool {
    map.min() == map.iter().next()
}

mod remove_max {
    use super::Map;

    #[quickcheck]
    fn returns_max(mut map: Map) -> bool {
        let max = map.max().map(|(k, v)| (k.clone(), v.clone()));
        map.remove_max() == max
    }

    #[quickcheck]
    fn affects_len(mut map: Map) -> bool {
        let old_len = map.len();
        let removed = map.remove_max().is_some();
        map.len() == if removed { old_len - 1 } else { 0 }
    }

    #[quickcheck]
    fn removes_key(mut map: Map) -> bool {
        let key = match map.max() {
            None => return true,
            Some((key, _)) => key.clone(),
        };

        map.remove_max();

        !map.contains_key(&key) &&
        map.get(&key).is_none() &&
        map.get_mut(&key).is_none() &&
        map.iter().find(|e| *e.0 == key).is_none()
    }

    #[quickcheck]
    fn removes_no_other_keys(mut map: Map) -> bool {
        let old_map = map.clone();
        map.remove_max();
        ::std::iter::order::equals(map.iter(), old_map.iter().take(map.len()))
    }
}

mod remove_min {
    use super::Map;

    #[quickcheck]
    fn returns_min(mut map: Map) -> bool {
        let min = map.min().map(|(k, v)| (k.clone(), v.clone()));
        map.remove_min() == min
    }

    #[quickcheck]
    fn affects_len(mut map: Map) -> bool {
        let old_len = map.len();
        let removed = map.remove_min().is_some();
        map.len() == if removed { old_len - 1 } else { 0 }
    }

    #[quickcheck]
    fn removes_key(mut map: Map) -> bool {
        let key = match map.min() {
            None => return true,
            Some((key, _)) => key.clone(),
        };

        map.remove_min();

        !map.contains_key(&key) &&
        map.get(&key).is_none() &&
        map.get_mut(&key).is_none() &&
        map.iter().find(|e| *e.0 == key).is_none()
    }

    #[quickcheck]
    fn removes_no_other_keys(mut map: Map) -> bool {
        let old_map = map.clone();
        map.remove_min();
        ::std::iter::order::equals(map.iter(), old_map.iter().skip(1))
    }
}

mod pred {
    use super::{K, Map};

    #[quickcheck]
    fn agrees_with_iter(map: Map, key: K) -> bool {
        map.pred(&key) == map.iter().rev().find(|e| *e.0 < key)
    }

    #[quickcheck]
    fn or_eq_agrees_with_iter(map: Map, key: K) -> bool {
        map.pred_or_eq(&key) == map.iter().rev().find(|e| *e.0 <= key)
    }
}

mod succ {
    use super::{K, Map};

    #[quickcheck]
    fn agrees_with_iter(map: Map, key: K) -> bool {
        map.succ(&key) == map.iter().find(|e| *e.0 > key)
    }

    #[quickcheck]
    fn or_eq_agrees_with_iter(map: Map, key: K) -> bool {
        map.succ_or_eq(&key) == map.iter().find(|e| *e.0 >= key)
    }
}

mod iter {
    use super::Map;

    #[quickcheck]
    fn ascends(map: Map) -> bool {
        map.iter().zip(map.iter().skip(1)).all(|(e1, e2)| e1.0 < e2.0)
    }

    #[quickcheck]
    fn descends_when_reversed(map: Map) -> bool {
        map.iter().rev().zip(map.iter().rev().skip(1)).all(|(e2, e1)| e2.0 > e1.0)
    }
}

mod range {
    extern crate rand;

    use quickcheck::{Arbitrary, Gen};
    use self::rand::Rng;
    use std::collections::Bound::*;
    use std::iter::order;
    use super::{K, Map};

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
                0 => Included(Arbitrary::arbitrary(gen)),
                1 => Excluded(Arbitrary::arbitrary(gen)),
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

    #[quickcheck]
    fn range(map: Map, min: Bound<K>, max: Bound<K>) -> bool {
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

        order::equals(r, i)
    }

    #[quickcheck]
    fn range_rev(map: Map, min: Bound<K>, max: Bound<K>) -> bool {
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

        order::equals(r, i)
    }
}
