#![feature(box_patterns, box_syntax)]
#![feature(core)]

extern crate collect;

mod node;

use collect::compare::{self, Compare, Natural};
use node::LinkExt;
use std::default::Default;
use std::ops;

/// An ordered map based on a binary search tree.
#[derive(Clone)]
pub struct TreeMap<K, V, C = Natural<K>> where C: Compare<K> {
    root: node::Link<K, V>,
    len: usize,
    cmp: C,
}

impl<K, V> TreeMap<K, V> where K: Ord {
    /// Creates an empty map ordered according to the natural order of its keys.
    pub fn new() -> TreeMap<K, V> { TreeMap::with_cmp(compare::natural()) }
}

impl<K, V, C> TreeMap<K, V, C> where C: Compare<K> {
    /// Creates an empty map ordered according to the given comparator.
    pub fn with_cmp(cmp: C) -> TreeMap<K, V, C> {
        TreeMap { root: None, len: 0, cmp: cmp }
    }

    /// Checks if the map is empty.
    pub fn is_empty(&self) -> bool { self.root.is_none() }

    /// Returns the number of entries in the map.
    pub fn len(&self) -> usize { self.len }

    /// Returns a reference to the map's comparator.
    pub fn cmp(&self) -> &C { &self.cmp }

    /// Inserts an entry into the map, returning the previous value, if any, associated
    /// with the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
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

    /// Returns a reference to the value associated with the given key, or `None` if the
    /// map does not contain the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    /// assert_eq!(map.get(&1), None);
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// ```
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V> where C: Compare<Q, K> {
        node::get(&self.root, &self.cmp, key).key_value().map(|e| e.1)
    }

    /// Returns a mutable reference to the value associated with the given key, or `None`
    /// if the map does not contain the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
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

        node::get(&mut self.root, &self.cmp, key).key_value_mut().map(|e| e.1)
    }
}

impl<K, V, C> Default for TreeMap<K, V, C> where C: Compare<K> + Default {
    fn default() -> TreeMap<K, V, C> { TreeMap::with_cmp(Default::default()) }
}

impl<K, V, C> Extend<(K, V)> for TreeMap<K, V, C> where C: Compare<K> {
    fn extend<I: ::std::iter::IntoIterator<Item=(K, V)>>(&mut self, it: I) {
        for (k, v) in it { self.insert(k, v); }
    }
}

impl<K, V, C, Q: ?Sized> ops::Index<Q> for TreeMap<K, V, C>
    where C: Compare<K> + Compare<Q, K> {

    type Output = V;
    fn index(&self, key: &Q) -> &V { self.get(key).expect("key not found") }
}

impl<K, V, C, Q: ?Sized> ops::IndexMut<Q> for TreeMap<K, V, C>
    where C: Compare<K> + Compare<Q, K> {

    fn index_mut(&mut self, key: &Q) -> &mut V {
        self.get_mut(key).expect("key not found")
    }
}
