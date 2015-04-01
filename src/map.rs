//! An ordered map based on a binary search tree.

use compare::{Compare, Natural};
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::collections::Bound;
use std::default::Default;
use std::fmt::{self, Debug};
use std::hash::{self, Hash};
use std::iter::{self, IntoIterator};
use std::marker::PhantomData;
use std::mem::transmute;
use std::ops;
use super::node::{self, Find, Max, Min, Neighbor, Node, Traverse, as_node_ref};
use super::node::build::{Build, Get, GetMut, PathBuilder};

pub use super::node::{OccupiedEntry, VacantEntry};

/// An ordered map based on a binary search tree.
///
/// The behavior of this map is undefined if a key's ordering relative to any other key changes
/// while the key is in the map. This is normally only possible through `Cell`, `RefCell`, or
/// unsafe code.
#[derive(Clone)]
pub struct Map<K, V, C = Natural<K>> where C: Compare<K> {
    root: node::Link<K, V>,
    len: usize,
    cmp: C,
}

impl<K, V> Map<K, V> where K: Ord {
    /// Creates an empty map ordered according to the natural order of its keys.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// let mut it = map.iter();
    /// assert_eq!(it.next(), Some((&1, &"a")));
    /// assert_eq!(it.next(), Some((&2, &"b")));
    /// assert_eq!(it.next(), Some((&3, &"c")));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn new() -> Self { Map::with_cmp(::compare::natural()) }
}

impl<K, V, C> Map<K, V, C> where C: Compare<K> {
    /// Creates an empty map ordered according to the given comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate compare;
    /// # extern crate tree;
    /// # fn main() {
    /// use compare::{Compare, natural};
    ///
    /// let mut map = tree::Map::with_cmp(natural().rev());
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// let mut it = map.iter();
    /// assert_eq!(it.next(), Some((&3, &"c")));
    /// assert_eq!(it.next(), Some((&2, &"b")));
    /// assert_eq!(it.next(), Some((&1, &"a")));
    /// assert_eq!(it.next(), None);
    /// # }
    /// ```
    pub fn with_cmp(cmp: C) -> Self {
        Map { root: None, len: 0, cmp: cmp }
    }

    /// Checks if the map is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert!(map.is_empty());
    ///
    /// map.insert(2, "b");
    /// assert!(!map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool { self.root.is_none() }

    /// Returns the number of entries in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert(2, "b");
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn len(&self) -> usize { self.len }

    /// Returns a reference to the map's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate compare;
    /// # extern crate tree;
    /// # fn main() {
    /// use compare::{Compare, natural};
    ///
    /// let map: tree::Map<i32, &str> = tree::Map::new();
    /// assert!(map.cmp().compares_lt(&1, &2));
    ///
    /// let map: tree::Map<i32, &str, _> = tree::Map::with_cmp(natural().rev());
    /// assert!(map.cmp().compares_gt(&1, &2));
    /// # }
    /// ```
    pub fn cmp(&self) -> &C { &self.cmp }

    /// Removes all entries from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.len(), 3);
    /// assert_eq!(map.iter().next(), Some((&1, &"a")));
    ///
    /// map.clear();
    ///
    /// assert_eq!(map.len(), 0);
    /// assert_eq!(map.iter().next(), None);
    /// ```
    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

    /// Inserts an entry into the map, returning the previous value, if any, associated
    /// with the key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.insert(1, "a"), None);
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.insert(1, "b"), Some("a"));
    /// assert_eq!(map.get(&1), Some(&"b"));
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let old_value = node::insert(&mut self.root, &self.cmp, key, value);
        if old_value.is_none() { self.len += 1; }
        old_value
    }

    /// Removes and returns the entry whose key is equal to the given key, returning
    /// `None` if the map does not contain the key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.len(), 3);
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.remove(&1), Some((1, "a")));
    ///
    /// assert_eq!(map.len(), 2);
    /// assert_eq!(map.get(&1), None);
    /// assert_eq!(map.remove(&1), None);
    /// ```
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
        where C: Compare<Q, K> {

        let key_value = Find { key: key, cmp: &self.cmp }
            .traverse(PathBuilder::new(&mut self.root)).remove();
        if key_value.is_some() { self.len -= 1; }
        key_value
    }

    /// Returns the map's entry corresponding to the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut counts = tree::Map::new();
    ///
    /// for s in vec!["a", "b", "a", "c", "a", "b"] {
    ///     *counts.entry(s).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(counts[&"a"], 3);
    /// assert_eq!(counts[&"b"], 2);
    /// assert_eq!(counts[&"c"], 1);
    /// ```
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        Find { key: &key, cmp: &self.cmp }.traverse(PathBuilder::new(&mut self.root))
            .into_entry(&mut self.len, key)
    }

    /// Checks if the map contains the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert!(!map.contains_key(&1));
    /// map.insert(1, "a");
    /// assert!(map.contains_key(&1));
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool where C: Compare<Q, K> {
        self.get(key).is_some()
    }

    /// Returns a reference to the value associated with the given key, or `None` if the
    /// map does not contain the key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.get(&1), None);
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// ```
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V> where C: Compare<Q, K> {
        Find { key: key, cmp: &self.cmp }.traverse(Get::new(&self.root)).map(|e| e.1)
    }

    /// Returns a mutable reference to the value associated with the given key, or `None`
    /// if the map does not contain the key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.get(&1), None);
    /// map.insert(1, "a");
    ///
    /// {
    ///     let value = map.get_mut(&1).unwrap();
    ///     assert_eq!(*value, "a");
    ///     *value = "b";
    /// }
    ///
    /// assert_eq!(map.get(&1), Some(&"b"));
    /// ```
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
        where C: Compare<Q, K> {

        Find { key: key, cmp: &self.cmp }.traverse(GetMut::new(&mut self.root)).map(|e| e.1)
    }

    /// Returns a reference to the map's maximum key and a reference to its associated
    /// value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.max(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.max(), Some((&3, &"c")));
    /// ```
    pub fn max(&self) -> Option<(&K, &V)> {
        Max.traverse(Get::new(&self.root))
    }

    /// Returns a reference to the map's maximum key and a mutable reference to its
    /// associated value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.max(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// {
    ///     let max = map.max_mut().unwrap();
    ///     assert_eq!(max, (&3, &mut "c"));
    ///     *max.1 = "cc";
    /// }
    ///
    /// assert_eq!(map.max(), Some((&3, &"cc")));
    /// ```
    pub fn max_mut(&mut self) -> Option<(&K, &mut V)> {
        Max.traverse(GetMut::new(&mut self.root))
    }

    /// Removes the map's maximum key and returns it and its associated value, or `None` if the map
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.remove_max(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.remove_max(), Some((3, "c")));
    /// ```
    pub fn remove_max(&mut self) -> Option<(K, V)> {
        let key_value = Max.traverse(PathBuilder::new(&mut self.root)).remove();
        if key_value.is_some() { self.len -= 1; }
        key_value
    }

    /// Returns the map's entry corresponding to its maximum key.
    pub fn max_entry(&mut self) -> Option<OccupiedEntry<K, V>> {
        Max.traverse(PathBuilder::new(&mut self.root)).into_occupied_entry(&mut self.len)
    }

    /// Returns a reference to the map's minimum key and a reference to its associated
    /// value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.min(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.min(), Some((&1, &"a")));
    /// ```
    pub fn min(&self) -> Option<(&K, &V)> {
        Min.traverse(Get::new(&self.root))
    }

    /// Returns a reference to the map's minimum key and a mutable reference to its
    /// associated value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.min(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// {
    ///     let min = map.min_mut().unwrap();
    ///     assert_eq!(min, (&1, &mut "a"));
    ///     *min.1 = "aa";
    /// }
    ///
    /// assert_eq!(map.min(), Some((&1, &"aa")));
    /// ```
    pub fn min_mut(&mut self) -> Option<(&K, &mut V)> {
        Min.traverse(GetMut::new(&mut self.root))
    }

    /// Removes the map's minimum key and returns it and its associated value, or `None` if the map
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    /// assert_eq!(map.remove_min(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.remove_min(), Some((1, "a")));
    /// ```
    pub fn remove_min(&mut self) -> Option<(K, V)> {
        let key_value = Min.traverse(PathBuilder::new(&mut self.root)).remove();
        if key_value.is_some() { self.len -= 1; }
        key_value
    }

    /// Returns the map's entry corresponding to its minimum key.
    pub fn min_entry(&mut self) -> Option<OccupiedEntry<K, V>> {
        Min.traverse(PathBuilder::new(&mut self.root)).into_occupied_entry(&mut self.len)
    }

    /// Returns a reference to the predecessor of the given key and a
    /// reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// If `inclusive` is `false`, this method finds the greatest key that is strictly less than
    /// the given key. If `inclusive` is `true`, this method finds the greatest key that is less
    /// than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.pred(&0, false), None);
    /// assert_eq!(map.pred(&1, false), None);
    /// assert_eq!(map.pred(&2, false), Some((&1, &"a")));
    /// assert_eq!(map.pred(&3, false), Some((&2, &"b")));
    /// assert_eq!(map.pred(&4, false), Some((&3, &"c")));
    ///
    /// assert_eq!(map.pred(&0, true), None);
    /// assert_eq!(map.pred(&1, true), Some((&1, &"a")));
    /// assert_eq!(map.pred(&2, true), Some((&2, &"b")));
    /// assert_eq!(map.pred(&3, true), Some((&3, &"c")));
    /// assert_eq!(map.pred(&4, true), Some((&3, &"c")));
    /// ```
    pub fn pred<Q: ?Sized>(&self, key: &Q, inclusive: bool) -> Option<(&K, &V)>
        where C: Compare<Q, K> {

        Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Min }
            .traverse(Get::new(&self.root))
    }

    /// Returns a reference to the predecessor of the given key and a
    /// mutable reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// If `inclusive` is `false`, this method finds the greatest key that is strictly less than
    /// the given key. If `inclusive` is `true`, this method finds the greatest key that is less
    /// than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// {
    ///     let pred = map.pred_mut(&2, false).unwrap();
    ///     assert_eq!(pred, (&1, &mut "a"));
    ///     *pred.1 = "aa";
    /// }
    ///
    /// assert_eq!(map.pred(&2, false), Some((&1, &"aa")));
    ///
    /// {
    ///     let pred_or_eq = map.pred_mut(&1, true).unwrap();
    ///     assert_eq!(pred_or_eq, (&1, &mut "aa"));
    ///     *pred_or_eq.1 = "aaa";
    /// }
    ///
    /// {
    ///     let pred_or_eq = map.pred_mut(&4, true).unwrap();
    ///     assert_eq!(pred_or_eq, (&3, &mut "c"));
    ///     *pred_or_eq.1 = "cc";
    /// }
    ///
    /// assert_eq!(map.pred(&1, true), Some((&1, &"aaa")));
    /// assert_eq!(map.pred(&4, true), Some((&3, &"cc")));
    /// ```
    pub fn pred_mut<Q: ?Sized>(&mut self, key: &Q, inclusive: bool) -> Option<(&K, &mut V)>
        where C: Compare<Q, K> {

        Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Min }
            .traverse(GetMut::new(&mut self.root))
    }

    /// Removes the predecessor of the given key from the map and returns it and its associated
    /// value, or `None` if no such key is present in the map.
    ///
    /// If `inclusive` is `false`, this method removes the greatest key that is strictly less than
    /// the given key. If `inclusive` is `true`, this method removes the greatest key that is less
    /// than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    pub fn remove_pred<Q: ?Sized>(&mut self, key: &Q, inclusive: bool) -> Option<(K, V)>
        where C: Compare<Q, K> {

        let key_value = Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Min }
            .traverse(PathBuilder::new(&mut self.root)).remove();
        if key_value.is_some() { self.len -= 1; }
        key_value
    }

    /// Returns the entry corresponding to the predecessor of the given key.
    ///
    /// If `inclusive` is `false`, this method returns the entry corresponding to the greatest key
    /// that is strictly less than the given key. If `inclusive` is `true`, this method returns
    /// the entry corresponding to the greatest key that is less than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    pub fn pred_entry<Q: ?Sized>(&mut self, key: &Q, inclusive: bool)
        -> Option<OccupiedEntry<K, V>> where C: Compare<Q, K> {

        Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Min }
            .traverse(PathBuilder::new(&mut self.root)).into_occupied_entry(&mut self.len)
    }

    /// Returns a reference to the successor of the given key and a
    /// reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// If `inclusive` is `false`, this method finds the smallest key that is strictly greater than
    /// the given key. If `inclusive` is `true`, this method finds the smallest key that is greater
    /// than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.succ(&0, false), Some((&1, &"a")));
    /// assert_eq!(map.succ(&1, false), Some((&2, &"b")));
    /// assert_eq!(map.succ(&2, false), Some((&3, &"c")));
    /// assert_eq!(map.succ(&3, false), None);
    /// assert_eq!(map.succ(&4, false), None);
    ///
    /// assert_eq!(map.succ(&0, true), Some((&1, &"a")));
    /// assert_eq!(map.succ(&1, true), Some((&1, &"a")));
    /// assert_eq!(map.succ(&2, true), Some((&2, &"b")));
    /// assert_eq!(map.succ(&3, true), Some((&3, &"c")));
    /// assert_eq!(map.succ(&4, true), None);
    /// ```
    pub fn succ<Q: ?Sized>(&self, key: &Q, inclusive: bool) -> Option<(&K, &V)>
        where C: Compare<Q, K> {

        Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Max }
            .traverse(Get::new(&self.root))
    }

    /// Returns a reference to the successor of the given key and a
    /// mutable reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// If `inclusive` is `false`, this method finds the smallest key that is strictly greater than
    /// the given key. If `inclusive` is `true`, this method finds the smallest key that is greater
    /// than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// {
    ///     let succ = map.succ_mut(&2, false).unwrap();
    ///     assert_eq!(succ, (&3, &mut "c"));
    ///     *succ.1 = "cc";
    /// }
    ///
    /// assert_eq!(map.succ(&2, false), Some((&3, &"cc")));
    ///
    /// {
    ///     let succ_or_eq = map.succ_mut(&0, true).unwrap();
    ///     assert_eq!(succ_or_eq, (&1, &mut "a"));
    ///     *succ_or_eq.1 = "aa";
    /// }
    ///
    /// {
    ///     let succ_or_eq = map.succ_mut(&3, true).unwrap();
    ///     assert_eq!(succ_or_eq, (&3, &mut "cc"));
    ///     *succ_or_eq.1 = "ccc";
    /// }
    ///
    /// assert_eq!(map.succ(&0, true), Some((&1, &"aa")));
    /// assert_eq!(map.succ(&3, true), Some((&3, &"ccc")));
    /// ```
    pub fn succ_mut<Q: ?Sized>(&mut self, key: &Q, inclusive: bool) -> Option<(&K, &mut V)>
        where C: Compare<Q, K> {

        Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Max }
            .traverse(GetMut::new(&mut self.root))
    }

    /// Removes the successor of the given key from the map and returns it and its associated
    /// value, or `None` if no such key is present in the map.
    ///
    /// If `inclusive` is `false`, this method removes the smallest key that is strictly greater
    /// than the given key. If `inclusive` is `true`, this method removes the smallest key that is
    /// greater than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    pub fn remove_succ<Q: ?Sized>(&mut self, key: &Q, inclusive: bool) -> Option<(K, V)>
        where C: Compare<Q, K> {

        let key_value = Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Max }
            .traverse(PathBuilder::new(&mut self.root)).remove();
        if key_value.is_some() { self.len -= 1; }
        key_value
    }

    /// Returns the entry corresponding to the successor of the given key.
    ///
    /// If `inclusive` is `false`, this method returns the entry corresponding to the smallest key
    /// that is strictly greater than the given key. If `inclusive` is `true`, this method returns
    /// the entry corresponding to the smallest key that is greater than or equal to the given key.
    ///
    /// The given key need not itself be present in the map.
    pub fn succ_entry<Q: ?Sized>(&mut self, key: &Q, inclusive: bool)
        -> Option<OccupiedEntry<K, V>> where C: Compare<Q, K> {

        Neighbor { key: key, cmp: &self.cmp, inc: inclusive, ext: Max }
            .traverse(PathBuilder::new(&mut self.root)).into_occupied_entry(&mut self.len)
    }

    /// Returns an iterator that consumes the map.
    ///
    /// The iterator yields the entries in ascending order according to the map's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// let mut it = map.into_iter();
    /// assert_eq!(it.next(), Some((1, "a")));
    /// assert_eq!(it.next(), Some((2, "b")));
    /// assert_eq!(it.next(), Some((3, "c")));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn into_iter(mut self) -> IntoIter<K, V> {
        IntoIter(node::Iter::new(self.root.take(), self.len))
    }

    /// Returns an iterator over the map's entries with immutable references to the values.
    ///
    /// The iterator yields the entries in ascending order according to the map's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// let mut it = map.iter();
    /// assert_eq!(it.next(), Some((&1, &"a")));
    /// assert_eq!(it.next(), Some((&2, &"b")));
    /// assert_eq!(it.next(), Some((&3, &"c")));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<K, V> {
        Iter(node::Iter::new(as_node_ref(&self.root), self.len))
    }

    /// Returns an iterator over the map's entries with mutable references to the values.
    ///
    /// The iterator yields the entries in ascending order according to the map's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut map = tree::Map::new();
    ///
    /// map.insert("b", 2);
    /// map.insert("a", 1);
    /// map.insert("c", 3);
    ///
    /// let mut i = 1;
    ///
    /// for (_, value) in map.iter_mut() {
    ///     assert_eq!(i, *value);
    ///     *value *= 2;
    ///     i += 1;
    /// }
    ///
    /// assert_eq!(map[&"a"], 2);
    /// assert_eq!(map[&"b"], 4);
    /// assert_eq!(map[&"c"], 6);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut { iter: self.iter(), _mut: PhantomData }
    }

    /// Returns an iterator that consumes the map, yielding only those entries whose keys lie in
    /// the given range.
    ///
    /// The iterator yields the entries in ascending order according to the map's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(collections)]
    /// # extern crate tree;
    /// # fn main() {
    /// use std::collections::Bound::{Excluded, Unbounded};
    ///
    /// let mut map = tree::Map::new();
    ///
    /// map.insert("b", 2);
    /// map.insert("a", 1);
    /// map.insert("c", 3);
    ///
    /// assert_eq!(map.into_range(Excluded(&"a"), Unbounded).collect::<Vec<_>>(),
    ///     [("b", 2), ("c", 3)]);
    /// # }
    /// ```
    pub fn into_range<Min: ?Sized, Max: ?Sized>(mut self, min: Bound<&Min>, max: Bound<&Max>)
        -> IntoRange<K, V> where C: Compare<Min, K> + Compare<Max, K> {

        IntoRange(node::Iter::range(self.root.take(), self.len, &self.cmp, min, max))
    }

    /// Returns an iterator over the map's entries whose keys lie in the given range with immutable
    /// references to the values.
    ///
    /// The iterator yields the entries in ascending order according to the map's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(collections)]
    /// # extern crate tree;
    /// # fn main() {
    /// use std::collections::Bound::{Included, Excluded, Unbounded};
    ///
    /// let mut map = tree::Map::new();
    ///
    /// map.insert("b", 2);
    /// map.insert("a", 1);
    /// map.insert("c", 3);
    ///
    /// assert_eq!(map.range(Unbounded, Unbounded).collect::<Vec<_>>(),
    ///     [(&"a", &1), (&"b", &2), (&"c", &3)]);
    /// assert_eq!(map.range(Excluded(&"a"), Included(&"f")).collect::<Vec<_>>(),
    ///     [(&"b", &2), (&"c", &3)]);
    /// assert_eq!(map.range(Included(&"a"), Excluded(&"b")).collect::<Vec<_>>(),
    ///     [(&"a", &1)]);
    /// # }
    /// ```
    pub fn range<Min: ?Sized, Max: ?Sized>(&self, min: Bound<&Min>, max: Bound<&Max>)
        -> Range<K, V> where C: Compare<Min, K> + Compare<Max, K> {

        Range(node::Iter::range(as_node_ref(&self.root), self.len, &self.cmp, min, max))
    }

    /// Returns an iterator over the map's entries whose keys lie in the given range with mutable
    /// references to the values.
    ///
    /// The iterator yields the entries in ascending order according to the map's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(collections)]
    /// # extern crate tree;
    /// # fn main() {
    /// use std::collections::Bound;
    ///
    /// let mut map = tree::Map::new();
    ///
    /// map.insert("b", 2);
    /// map.insert("a", 1);
    /// map.insert("c", 3);
    ///
    /// let mut i = 1;
    ///
    /// for (_, value) in map.range_mut(Bound::Unbounded, Bound::Excluded(&"c")) {
    ///     assert_eq!(i, *value);
    ///     *value *= 2;
    ///     i += 1;
    /// }
    ///
    /// assert_eq!(map[&"a"], 2);
    /// assert_eq!(map[&"b"], 4);
    /// assert_eq!(map[&"c"], 3);
    /// # }
    /// ```
    pub fn range_mut<Min: ?Sized, Max: ?Sized>(&mut self, min: Bound<&Min>, max: Bound<&Max>)
        -> RangeMut<K, V> where C: Compare<Min, K> + Compare<Max, K> {

        RangeMut { iter: self.range(min, max), _mut: PhantomData }
    }

    #[cfg(test)]
    pub fn root(&self) -> &node::Link<K, V> { &self.root }
}

impl<K, V, C> Debug for Map<K, V, C> where K: Debug, V: Debug, C: Compare<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));

        let mut it = self.iter();

        if let Some((k, v)) = it.next() {
            try!(write!(f, "{:?}: {:?}", k, v));
            for (k, v) in it { try!(write!(f, ", {:?}: {:?}", k, v)); }
        }

        write!(f, "}}")
    }
}

impl<K, V, C> Default for Map<K, V, C> where C: Compare<K> + Default {
    fn default() -> Self { Map::with_cmp(Default::default()) }
}

impl<K, V, C> Extend<(K, V)> for Map<K, V, C> where C: Compare<K> {
    fn extend<I: IntoIterator<Item=(K, V)>>(&mut self, it: I) {
        for (k, v) in it { self.insert(k, v); }
    }
}

impl<K, V, C> iter::FromIterator<(K, V)> for Map<K, V, C>
    where C: Compare<K> + Default {

    fn from_iter<I: IntoIterator<Item=(K, V)>>(it: I) -> Self {
        let mut map: Self = Default::default();
        map.extend(it);
        map
    }
}

impl<K, V, C> Hash for Map<K, V, C> where K: Hash, V: Hash, C: Compare<K> {
    fn hash<H: hash::Hasher>(&self, h: &mut H) {
        for e in self.iter() { e.hash(h); }
    }
}

impl<'a, K, V, C, Q: ?Sized> ops::Index<&'a Q> for Map<K, V, C>
    where C: Compare<K> + Compare<Q, K> {

    type Output = V;
    fn index(&self, key: &Q) -> &V { self.get(key).expect("key not found") }
}

impl<'a, K, V, C> IntoIterator for &'a Map<K, V, C> where C: Compare<K> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> { self.iter() }
}

impl<'a, K, V, C> IntoIterator for &'a mut Map<K, V, C> where C: Compare<K> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> IterMut<'a, K, V> { self.iter_mut() }
}

impl<K, V, C> IntoIterator for Map<K, V, C> where C: Compare<K> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> IntoIter<K, V> { self.into_iter() }
}

impl<K, V, C> PartialEq for Map<K, V, C> where V: PartialEq, C: Compare<K> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(l, r)| {
            self.cmp.compares_eq(&l.0, &r.0) && l.1 == r.1
        })
    }
}

impl<K, V, C> Eq for Map<K, V, C> where V: Eq, C: Compare<K> {}

impl<K, V, C> PartialOrd for Map<K, V, C> where V: PartialOrd, C: Compare<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut l = self.iter();
        let mut r = other.iter();

        loop {
            match (l.next(), r.next()) {
                (None, None) => return Some(Equal),
                (None, Some(_)) => return Some(Less),
                (Some(_), None) => return Some(Greater),
                (Some(l), Some(r)) => match self.cmp.compare(&l.0, &r.0) {
                    Equal => match l.1.partial_cmp(&r.1) {
                        Some(Equal) => {}
                        non_eq => return non_eq,
                    },
                    non_eq => return Some(non_eq),
                },
            }
        }
    }
}

impl<K, V, C> Ord for Map<K, V, C> where V: Ord, C: Compare<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut l = self.iter();
        let mut r = other.iter();

        loop {
            match (l.next(), r.next()) {
                (None, None) => return Equal,
                (None, Some(_)) => return Less,
                (Some(_), None) => return Greater,
                (Some(l), Some(r)) => match self.cmp.compare(&l.0, &r.0) {
                    Equal => match l.1.cmp(&r.1) {
                        Equal => {}
                        non_eq => return non_eq,
                    },
                    non_eq => return non_eq,
                },
            }
        }
    }
}

/// An iterator that consumes the map.
///
/// The iterator yields the entries in ascending order according to the map's comparator.
///
/// # Examples
///
/// Acquire through [`Map::into_iter`](struct.Map.html#method.into_iter) or the
/// `IntoIterator` trait:
///
/// ```
/// let mut map = tree::Map::new();
///
/// map.insert(2, "b");
/// map.insert(1, "a");
/// map.insert(3, "c");
///
/// for (key, value) in map {
///     println!("{:?}: {:?}", key, value);
/// }
/// ```
#[derive(Clone)]
pub struct IntoIter<K, V>(node::Iter<Box<Node<K, V>>>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<(K, V)> { self.0.next_back() }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}

/// An iterator over the map's entries with immutable references to the values.
///
/// The iterator yields the entries in ascending order according to the map's comparator.
///
/// # Examples
///
/// Acquire through [`Map::iter`](struct.Map.html#method.iter) or the `IntoIterator` trait:
///
/// ```
/// let mut map = tree::Map::new();
///
/// map.insert(2, "b");
/// map.insert(1, "a");
/// map.insert(3, "c");
///
/// for (key, value) in &map {
///     println!("{:?}: {:?}", key, value);
/// }
/// ```
pub struct Iter<'a, K: 'a, V: 'a>(node::Iter<&'a Node<K, V>>);

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Iter<'a, K, V> { Iter(self.0.clone()) }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<(&'a K, &'a V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> { self.0.next_back() }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}

/// An iterator over the map's entries with mutable references to the values.
///
/// The iterator yields the entries in ascending order according to the map's comparator.
///
/// # Examples
///
/// Acquire through [`Map::iter_mut`](struct.Map.html#method.iter_mut) or the
/// `IntoIterator` trait:
///
/// ```
/// let mut map = tree::Map::new();
///
/// map.insert(2, "b");
/// map.insert(1, "a");
/// map.insert(3, "c");
///
/// for (key, value) in &mut map {
///     println!("{:?}: {:?}", key, value);
/// }
/// ```
pub struct IterMut<'a, K: 'a, V: 'a> {
    iter: Iter<'a, K, V>,
    _mut: PhantomData<&'a mut V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        let next = self.iter.next();
        unsafe { transmute(next)  }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        let next_back = self.iter.next_back();
        unsafe { transmute(next_back) }
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}

/// An iterator that consumes the map, yielding only those entries whose keys lie in a given range.
///
/// The iterator yields the entries in ascending order according to the map's comparator.
///
/// Acquire through [`Map::into_range`](struct.Map.html#method.into_range).
#[derive(Clone)]
pub struct IntoRange<K, V>(node::Iter<Box<Node<K, V>>>);

impl<K, V> Iterator for IntoRange<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.range_size_hint() }
}

impl<K, V> DoubleEndedIterator for IntoRange<K, V> {
    fn next_back(&mut self) -> Option<(K, V)> { self.0.next_back() }
}

/// An iterator over the map's entries whose keys lie in a given range with immutable references to
/// the values.
///
/// The iterator yields the entries in ascending order according to the map's comparator.
///
/// Acquire through [`Map::range`](struct.Map.html#method.range).
pub struct Range<'a, K: 'a, V: 'a>(node::Iter<&'a Node<K, V>>);

impl<'a, K, V> Clone for Range<'a, K, V> {
    fn clone(&self) -> Range<'a, K, V> { Range(self.0.clone()) }
}

impl<'a, K, V> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<(&'a K, &'a V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.range_size_hint() }
}

impl<'a, K, V> DoubleEndedIterator for Range<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> { self.0.next_back() }
}

/// An iterator over the map's entries whose keys lie in a given range with mutable references to
/// the values.
///
/// The iterator yields the entries in ascending order according to the map's comparator.
///
/// Acquire through [`Map::range_mut`](struct.Map.html#method.range_mut).
pub struct RangeMut<'a, K: 'a, V: 'a> {
    iter: Range<'a, K, V>,
    _mut: PhantomData<&'a mut V>,
}

impl<'a, K, V> Iterator for RangeMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        let next = self.iter.next();
        unsafe { transmute(next) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, K, V> DoubleEndedIterator for RangeMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        let next_back = self.iter.next_back();
        unsafe { transmute(next_back) }
    }
}

/// An entry in the map.
///
/// See [`Map::entry`](struct.Map.html#method.entry) for an example.
pub enum Entry<'a, K: 'a, V: 'a> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V> {
    /// Returns the entry's value, inserting the given default if the entry is vacant.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(default),
        }
    }

    /// Returns the entry's value, inserting the given function's result if the entry is vacant.
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V where F: FnOnce() -> V {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(default()),
        }
    }
}
