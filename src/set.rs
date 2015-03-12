//! An ordered set based on a binary search tree.

use compare::{Compare, Natural};
use std::cmp::Ordering;
use std::collections::Bound;
use std::default::Default;
use std::fmt::{self, Debug};
use std::hash::{self, Hash};
use std::iter::{self, IntoIterator};
use super::map::{self, TreeMap};

/// An ordered set based on a binary search tree.
#[derive(Clone)]
pub struct TreeSet<T, C = Natural<T>> where C: Compare<T> {
    map: TreeMap<T, (), C>,
}

impl<T> TreeSet<T> where T: Ord {
    /// Creates an empty set ordered according to the natural order of its items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// let mut it = set.iter();
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&3));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn new() -> TreeSet<T> { TreeSet { map: TreeMap::new() } }
}

impl<T, C> TreeSet<T, C> where C: Compare<T> {
    /// Creates an empty set ordered according to the given comparator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate compare;
    /// # extern crate tree;
    /// # fn main() {
    /// use compare::{Compare, natural};
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::with_cmp(natural().rev());
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// let mut it = set.iter();
    /// assert_eq!(it.next(), Some(&3));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), None);
    /// # }
    /// ```
    pub fn with_cmp(cmp: C) -> TreeSet<T, C> { TreeSet { map: TreeMap::with_cmp(cmp) } }

    /// Checks if the set is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    /// assert!(set.is_empty());
    ///
    /// set.insert(2);
    /// assert!(!set.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool { self.map.is_empty() }

    /// Returns the number of items in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    /// assert_eq!(set.len(), 0);
    ///
    /// set.insert(2);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn len(&self) -> usize { self.map.len() }

    /// Returns a reference to the set's comparator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate compare;
    /// # extern crate tree;
    /// # fn main() {
    /// use compare::{Compare, natural};
    /// use tree::TreeSet;
    ///
    /// let set = TreeSet::new();
    /// assert!(set.cmp().compares_lt(&1, &2));
    ///
    /// let set: TreeSet<_, _> = TreeSet::with_cmp(natural().rev());
    /// assert!(set.cmp().compares_gt(&1, &2));
    /// # }
    /// ```
    pub fn cmp(&self) -> &C { self.map.cmp() }

    /// Removes all items from the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.len(), 3);
    /// assert_eq!(set.iter().next(), Some(&1));
    ///
    /// set.clear();
    ///
    /// assert_eq!(set.len(), 0);
    /// assert_eq!(set.iter().next(), None);
    /// ```
    pub fn clear(&mut self) { self.map.clear(); }

    /// Inserts an item into the set, returning `true` if the set did not already contain the item.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    /// assert!(!set.contains(&1));
    /// assert!(set.insert(1));
    /// assert!(set.contains(&1));
    /// assert!(!set.insert(1));
    /// ```
    pub fn insert(&mut self, item: T) -> bool { self.map.insert(item, ()).is_none() }

    /// Removes the given item from the set, returning `true` if the set contained the item.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.len(), 3);
    /// assert!(set.contains(&1));
    /// assert!(set.remove(&1));
    ///
    /// assert_eq!(set.len(), 2);
    /// assert!(!set.contains(&1));
    /// assert!(!set.remove(&1));
    /// ```
    pub fn remove<Q: ?Sized>(&mut self, item: &Q) -> bool where C: Compare<Q, T> {
        self.map.remove(item).is_some()
    }

    /// Checks if the set contains the given item.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    /// assert!(!set.contains(&1));
    /// set.insert(1);
    /// assert!(set.contains(&1));
    /// ```
    pub fn contains<Q: ?Sized>(&self, item: &Q) -> bool where C: Compare<Q, T> {
        self.map.contains_key(item)
    }

    /// Returns a reference to the set's maximum item, or `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    /// assert_eq!(set.max(), None);
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.max(), Some(&3));
    /// ```
    pub fn max(&self) -> Option<&T> { self.map.max().map(|e| e.0) }

    /// Returns a reference to the set's minimum item, or `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    /// assert_eq!(set.min(), None);
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.min(), Some(&1));
    /// ```
    pub fn min(&self) -> Option<&T> { self.map.min().map(|e| e.0) }

    /// Returns a reference to the greatest item that is strictly less than the given item, or
    /// `None` if no such item is present in the set.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.pred(&0), None);
    /// assert_eq!(set.pred(&1), None);
    /// assert_eq!(set.pred(&2), Some(&1));
    /// assert_eq!(set.pred(&3), Some(&2));
    /// assert_eq!(set.pred(&4), Some(&3));
    /// ```
    pub fn pred<Q: ?Sized>(&self, item: &Q) -> Option<&T> where C: Compare<Q, T> {
        self.map.pred(item).map(|e| e.0)
    }

    /// Returns a reference to the greatest item that is less than or equal to the given item, or
    /// `None` if no such item is present in the ste.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.pred_or_eq(&0), None);
    /// assert_eq!(set.pred_or_eq(&1), Some(&1));
    /// assert_eq!(set.pred_or_eq(&2), Some(&2));
    /// assert_eq!(set.pred_or_eq(&3), Some(&3));
    /// assert_eq!(set.pred_or_eq(&4), Some(&3));
    /// ```
    pub fn pred_or_eq<Q: ?Sized>(&self, item: &Q) -> Option<&T> where C: Compare<Q, T> {
        self.map.pred_or_eq(item).map(|e| e.0)
    }

    /// Returns a reference to the smallest item that is strictly greater than the given item, or
    /// `None` if no such item is present in the set.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.succ(&0), Some(&1));
    /// assert_eq!(set.succ(&1), Some(&2));
    /// assert_eq!(set.succ(&2), Some(&3));
    /// assert_eq!(set.succ(&3), None);
    /// assert_eq!(set.succ(&4), None);
    /// ```
    pub fn succ<Q: ?Sized>(&self, item: &Q) -> Option<&T> where C: Compare<Q, T> {
        self.map.succ(item).map(|e| e.0)
    }

    /// Returns a reference to the smallest item that is greater than or equal to the given item,
    /// or `None` if no such item is present in the set.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.succ_or_eq(&0), Some(&1));
    /// assert_eq!(set.succ_or_eq(&1), Some(&1));
    /// assert_eq!(set.succ_or_eq(&2), Some(&2));
    /// assert_eq!(set.succ_or_eq(&3), Some(&3));
    /// assert_eq!(set.succ_or_eq(&4), None);
    /// ```
    pub fn succ_or_eq<Q: ?Sized>(&self, item: &Q) -> Option<&T> where C: Compare<Q, T> {
        self.map.succ_or_eq(item).map(|e| e.0)
    }

    /// Returns an iterator that consumes the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// let mut it = set.into_iter();
    /// assert_eq!(it.next(), Some(1));
    /// assert_eq!(it.next(), Some(2));
    /// assert_eq!(it.next(), Some(3));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn into_iter(self) -> IntoIter<T> { IntoIter(self.map.into_iter()) }

    /// Returns an iterator over the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// let mut it = set.iter();
    /// assert_eq!(it.next(), Some(&1));
    /// assert_eq!(it.next(), Some(&2));
    /// assert_eq!(it.next(), Some(&3));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> { Iter(self.map.iter()) }

    /// Returns an iterator over the set's items that lie in the given range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::Bound::{Included, Excluded, Unbounded};
    /// use tree::TreeSet;
    ///
    /// let mut set = TreeSet::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.range(Unbounded, Unbounded).collect::<Vec<_>>(), [&1, &2, &3]);
    /// assert_eq!(set.range(Excluded(&1), Included(&5)).collect::<Vec<_>>(), [&2, &3]);
    /// assert_eq!(set.range(Included(&1), Excluded(&2)).collect::<Vec<_>>(), [&1]);
    /// ```
    pub fn range<Min: ?Sized, Max: ?Sized>(&self, min: Bound<&Min>, max: Bound<&Max>)
        -> Range<T> where C: Compare<Min, T> + Compare<Max, T> {

        Range(self.map.range(min, max))
    }
}

impl<T, C> Debug for TreeSet<T, C> where T: Debug, C: Compare<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));

        let mut it = self.iter();

        if let Some(item) = it.next() {
            try!(write!(f, "{:?}", item));
            for item in it { try!(write!(f, ", {:?}", item)); }
        }

        write!(f, "}}")
    }
}

impl<T, C> Default for TreeSet<T, C> where C: Compare<T> + Default {
    fn default() -> TreeSet<T, C> { TreeSet::with_cmp(Default::default()) }
}

impl<T, C> Extend<T> for TreeSet<T, C> where C: Compare<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, it: I) {
        for item in it { self.insert(item); }
    }
}

impl<T, C> iter::FromIterator<T> for TreeSet<T, C> where C: Compare<T> + Default {
    fn from_iter<I: IntoIterator<Item=T>>(it: I) -> TreeSet<T, C> {
        let mut set: TreeSet<T, C> = Default::default();
        set.extend(it);
        set
    }
}

impl<T, C> Hash for TreeSet<T, C> where T: Hash, C: Compare<T> {
    fn hash<H: hash::Hasher>(&self, h: &mut H) { self.map.hash(h); }
}

impl<T> PartialEq for TreeSet<T> where T: Ord {
    fn eq(&self, other: &TreeSet<T>) -> bool { self.map == other.map }
}

impl<T> Eq for TreeSet<T> where T: Ord {}

impl<T> PartialOrd for TreeSet<T> where T: Ord {
    fn partial_cmp(&self, other: &TreeSet<T>) -> Option<Ordering> {
        self.map.partial_cmp(&other.map)
    }
}

impl<T> Ord for TreeSet<T> where T: Ord {
    fn cmp(&self, other: &TreeSet<T>) -> Ordering { Ord::cmp(&self.map, &other.map) }
}

/// An iterator that consumes the set.
///
/// # Examples
///
/// Acquire through [`TreeSet::into_iter`](struct.TreeSet.html#method.into_iter) or the
/// `IntoIterator` trait:
///
/// ```rust
/// use tree::TreeSet;
///
/// let mut set = TreeSet::new();
///
/// set.insert(2);
/// set.insert(1);
/// set.insert(3);
///
/// for item in set {
///     println!("{:?}", item);
/// }
/// ```
pub struct IntoIter<T>(map::IntoIter<T, ()>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.0.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> { self.0.next_back().map(|e| e.0) }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

/// An iterator over the set.
///
/// # Examples
///
/// Acquire through [`TreeSet::iter`](struct.TreeSet.html#method.iter) or the `IntoIterator` trait:
///
/// ```rust
/// use tree::TreeSet;
///
/// let mut set = TreeSet::new();
///
/// set.insert(2);
/// set.insert(1);
/// set.insert(3);
///
/// for item in &set {
///     println!("{:?}", item);
/// }
/// ```
pub struct Iter<'a, T: 'a>(map::Iter<'a, T, ()>);

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Iter<'a, T> { Iter(self.0.clone()) }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { self.0.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> { self.0.next_back().map(|e| e.0) }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T, C> IntoIterator for &'a TreeSet<T, C> where C: Compare<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> { self.iter() }
}

impl<T, C> IntoIterator for TreeSet<T, C> where C: Compare<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> { self.into_iter() }
}

/// An iterator over the set's items that lie in a given range.
///
/// Acquire through [`TreeSet::range`](struct.TreeSet.html#method.range).
pub struct Range<'a, T: 'a>(map::Range<'a, T, ()>);

impl<'a, T> Clone for Range<'a, T> {
    fn clone(&self) -> Range<'a, T> { Range(self.0.clone()) }
}

impl<'a, T> Iterator for Range<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { self.0.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, T> DoubleEndedIterator for Range<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> { self.0.next_back().map(|e| e.0) }
}
