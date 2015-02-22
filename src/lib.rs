extern crate collect;

mod node;

use collect::compare::{self, Compare, Natural};

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
}
