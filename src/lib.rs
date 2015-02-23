#![feature(box_patterns, box_syntax)]
#![feature(core)]

extern crate collect;

mod node;

#[cfg(feature = "quickcheck")]
mod quickcheck;

use collect::compare::{self, Compare, Natural};
use node::{Left, LinkExt, Node, Right};
use std::default::Default;
use std::fmt::{self, Debug};
use std::iter::{self, IntoIterator};
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

    /// Removes all entries from the map.
    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }

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

    /// Returns a reference to the map's maximum key and a reference to its associated
    /// value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    /// assert_eq!(map.max(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.max(), Some((&3, &"c")));
    /// ```
    pub fn max(&self) -> Option<(&K, &V)> {
        node::extremum::<_, Right>(&self.root).key_value()
    }

    /// Returns a reference to the map's maximum key and a mutable reference to its
    /// associated value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
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
        node::extremum::<_, Right>(&mut self.root).key_value_mut()
    }

    /// Returns a reference to the map's minimum key and a reference to its associated
    /// value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    /// assert_eq!(map.min(), None);
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.min(), Some((&1, &"a")));
    /// ```
    pub fn min(&self) -> Option<(&K, &V)> {
        node::extremum::<_, Left>(&self.root).key_value()
    }

    /// Returns a reference to the map's minimum key and a mutable reference to its
    /// associated value, or `None` if the map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
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
        node::extremum::<_, Left>(&mut self.root).key_value_mut()
    }

    /// Returns a reference to the greatest key that is strictly less than the given key and a
    /// reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.pred(&0), None);
    /// assert_eq!(map.pred(&1), None);
    /// assert_eq!(map.pred(&2), Some((&1, &"a")));
    /// assert_eq!(map.pred(&3), Some((&2, &"b")));
    /// assert_eq!(map.pred(&4), Some((&3, &"c")));
    /// ```
    pub fn pred<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)> where C: Compare<Q, K> {
        node::closest::<_, _, _, Left>(&self.root, &self.cmp, key, false).key_value()
    }

    /// Returns a reference to the greatest key that is strictly less than the given key and a
    /// mutable reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// The given key need not itself be present in the map.
    pub fn pred_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<(&K, &mut V)> where C: Compare<Q, K> {
        node::closest::<_, _, _, Left>(&mut self.root, &self.cmp, key, false).key_value_mut()
    }

    /// Returns a reference to the greatest key that is less than or equal to the given key and a
    /// reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.pred_or_eq(&0), None);
    /// assert_eq!(map.pred_or_eq(&1), Some((&1, &"a")));
    /// assert_eq!(map.pred_or_eq(&2), Some((&2, &"b")));
    /// assert_eq!(map.pred_or_eq(&3), Some((&3, &"c")));
    /// assert_eq!(map.pred_or_eq(&4), Some((&3, &"c")));
    /// ```
    pub fn pred_or_eq<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)> where C: Compare<Q, K> {
        node::closest::<_, _, _, Left>(&self.root, &self.cmp, key, true).key_value()
    }

    /// Returns a reference to the greatest key that is less than or equal to the given key and a
    /// mutable reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// The given key need not itself be present in the map.
    pub fn pred_or_eq_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<(&K, &mut V)>
        where C: Compare<Q, K> {

        node::closest::<_, _, _, Left>(&mut self.root, &self.cmp, key, true).key_value_mut()
    }

    /// Returns a reference to the smallest key that is strictly greater than the given key and a
    /// reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.succ(&0), Some((&1, &"a")));
    /// assert_eq!(map.succ(&1), Some((&2, &"b")));
    /// assert_eq!(map.succ(&2), Some((&3, &"c")));
    /// assert_eq!(map.succ(&3), None);
    /// assert_eq!(map.succ(&4), None);
    /// ```
    pub fn succ<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)> where C: Compare<Q, K> {
        node::closest::<_, _, _, Right>(&self.root, &self.cmp, key, false).key_value()
    }

    /// Returns a reference to the smallest key that is strictly greater than the given key and a
    /// mutable reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// The given key need not itself be present in the map.
    pub fn succ_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<(&K, &mut V)> where C: Compare<Q, K> {
        node::closest::<_, _, _, Right>(&mut self.root, &self.cmp, key, false).key_value_mut()
    }

    /// Returns a reference to the smallest key that is greater than or equal to the given key and
    /// a reference to its associated value, or `None` if no such key is present in the map.
    ///
    /// The given key need not itself be present in the map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// assert_eq!(map.succ_or_eq(&0), Some((&1, &"a")));
    /// assert_eq!(map.succ_or_eq(&1), Some((&1, &"a")));
    /// assert_eq!(map.succ_or_eq(&2), Some((&2, &"b")));
    /// assert_eq!(map.succ_or_eq(&3), Some((&3, &"c")));
    /// assert_eq!(map.succ_or_eq(&4), None);
    /// ```
    pub fn succ_or_eq<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)> where C: Compare<Q, K> {
        node::closest::<_, _, _, Right>(&self.root, &self.cmp, key, true).key_value()
    }

    /// Returns a reference to the smallest key that is greater than or equal to the given key and
    /// a mutable reference to its associated value, or `None` if no such key is present in the
    /// map.
    ///
    /// The given key need not itself be present in the map.
    pub fn succ_or_eq_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<(&K, &mut V)>
        where C: Compare<Q, K> {

        node::closest::<_, _, _, Right>(&mut self.root, &self.cmp, key, true).key_value_mut()
    }

    /// Returns an iterator that consumes the map in ascending order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
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

    /// Returns an iterator over the map's entries in ascending order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
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
        Iter(node::Iter::new(self.root.as_node_ref(), self.len))
    }

    /// Returns an iterator over the map's entries in ascending order with mutable
    /// references to the values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
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
    /// assert_eq!(map["a"], 2);
    /// assert_eq!(map["b"], 4);
    /// assert_eq!(map["c"], 6);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut(node::IterMut::new(&mut self.root, self.len))
    }

    /// Returns an iterator over the map's entries in descending order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    ///
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    ///
    /// let mut it = map.rev_iter();
    /// assert_eq!(it.next(), Some((&3, &"c")));
    /// assert_eq!(it.next(), Some((&2, &"b")));
    /// assert_eq!(it.next(), Some((&1, &"a")));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn rev_iter(&self) -> RevIter<K, V> {
        RevIter(node::Iter::new(self.root.as_node_ref(), self.len))
    }

    /// Returns an iterator over the map's entries in descending order with mutable
    /// references to the values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeMap;
    ///
    /// let mut map = TreeMap::new();
    ///
    /// map.insert("b", 2);
    /// map.insert("a", 1);
    /// map.insert("c", 3);
    ///
    /// let mut i = 3;
    ///
    /// for (_, value) in map.rev_iter_mut() {
    ///     assert_eq!(i, *value);
    ///     *value *= 2;
    ///     i -= 1;
    /// }
    ///
    /// assert_eq!(map["a"], 2);
    /// assert_eq!(map["b"], 4);
    /// assert_eq!(map["c"], 6);
    /// ```
    pub fn rev_iter_mut(&mut self) -> RevIterMut<K, V> {
        RevIterMut(node::IterMut::new(&mut self.root, self.len))
    }
}

impl<K, V, C> Debug for TreeMap<K, V, C> where K: Debug, V: Debug, C: Compare<K> {
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

impl<K, V, C> Default for TreeMap<K, V, C> where C: Compare<K> + Default {
    fn default() -> TreeMap<K, V, C> { TreeMap::with_cmp(Default::default()) }
}

impl<K, V, C> Extend<(K, V)> for TreeMap<K, V, C> where C: Compare<K> {
    fn extend<I: IntoIterator<Item=(K, V)>>(&mut self, it: I) {
        for (k, v) in it { self.insert(k, v); }
    }
}

impl<K, V, C> iter::FromIterator<(K, V)> for TreeMap<K, V, C>
    where C: Compare<K> + Default {

    fn from_iter<I: IntoIterator<Item=(K, V)>>(it: I) -> TreeMap<K, V, C> {
        let mut map: TreeMap<K, V, C> = Default::default();
        map.extend(it);
        map
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

impl<'a, K, V, C> IntoIterator for &'a TreeMap<K, V, C> where C: Compare<K> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> { self.iter() }
}

impl<'a, K, V, C> IntoIterator for &'a mut TreeMap<K, V, C> where C: Compare<K> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> IterMut<'a, K, V> { self.iter_mut() }
}

impl<K, V, C> IntoIterator for TreeMap<K, V, C> where C: Compare<K> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> IntoIter<K, V> { self.into_iter() }
}

/// An iterator that consumes the map in ascending order.
pub struct IntoIter<K, V>(node::Iter<Box<Node<K, V>>, Left>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}

/// An iterator over the map's entries in ascending order.
pub struct Iter<'a, K: 'a, V: 'a>(node::Iter<&'a Node<K, V>, Left>);

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Iter<'a, K, V> { Iter(self.0.clone()) }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<(&'a K, &'a V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}

/// An iterator over the map's entries in ascending order with mutable references to the
/// values.
pub struct IterMut<'a, K: 'a, V: 'a>(node::IterMut<'a, K, V, Left>);

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}

/// An iterator over the map's entries in descending order.
pub struct RevIter<'a, K: 'a, V: 'a>(node::Iter<&'a Node<K, V>, Right>);

impl<'a, K, V> Clone for RevIter<'a, K, V> {
    fn clone(&self) -> RevIter<'a, K, V> { RevIter(self.0.clone()) }
}

impl<'a, K, V> Iterator for RevIter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<(&'a K, &'a V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, K, V> ExactSizeIterator for RevIter<'a, K, V> {}

/// An iterator over the map's entries in descending order with mutable references to the
/// values.
pub struct RevIterMut<'a, K: 'a, V: 'a>(node::IterMut<'a, K, V, Right>);

impl<'a, K, V> Iterator for RevIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> { self.0.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, K, V> ExactSizeIterator for RevIterMut<'a, K, V> {}
