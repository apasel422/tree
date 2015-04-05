//! An ordered set based on a binary search tree.

use compare::{Compare, Natural};
use std::cmp::Ordering;
#[cfg(feature = "range")] use std::collections::Bound;
use std::fmt::{self, Debug};
use std::hash::{self, Hash};
use std::iter;
use super::{Augment, OrderStat};
use super::map::{self, Map};

/// An ordered set based on a binary search tree.
///
/// The behavior of this set is undefined if an item's ordering relative to any other item changes
/// while the item is in the set. This is normally only possible through `Cell`, `RefCell`, or
/// unsafe code.
#[derive(Clone)]
pub struct Set<T, A = (), C = Natural<T>> where A: Augment, C: Compare<T> {
    map: Map<T, (), A, C>,
}

impl<T> Set<T> where T: Ord {
    /// Creates an empty set ordered according to the natural order of its items.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
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
    pub fn new() -> Self { Set { map: Map::new() } }
}

impl<T, C> Set<T, (), C> where C: Compare<T> {
    /// Creates an empty set ordered according to the given comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate compare;
    /// # extern crate tree;
    /// # fn main() {
    /// use compare::{Compare, natural};
    ///
    /// let mut set = tree::Set::with_cmp(natural().rev());
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
    pub fn with_cmp(cmp: C) -> Self { Set { map: Map::with_cmp(cmp) } }
}

impl<T, A> Set<T, A> where T: Ord, A: Augment {
    /// Creates an augmented empty set ordered according to the natural order of its items.
    pub fn with_augment() -> Self { Set { map: Map::with_augment() } }
}

impl<T, A, C> Set<T, A, C> where A: Augment, C: Compare<T> {
    /// Creates an augmented empty set ordered according to the given comparator.
    pub fn with_cmp_and_augment(cmp: C) -> Self { Set { map: Map::with_cmp_and_augment(cmp) } }

    /// Checks if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
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
    /// ```
    /// let mut set = tree::Set::new();
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
    /// ```
    /// # extern crate compare;
    /// # extern crate tree;
    /// # fn main() {
    /// use compare::{Compare, natural};
    ///
    /// let set = tree::Set::new();
    /// assert!(set.cmp().compares_lt(&1, &2));
    ///
    /// let set: tree::Set<i32, (), _> = tree::Set::with_cmp(natural().rev());
    /// assert!(set.cmp().compares_gt(&1, &2));
    /// # }
    /// ```
    pub fn cmp(&self) -> &C { self.map.cmp() }

    /// Removes all items from the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
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
    /// ```
    /// let mut set = tree::Set::new();
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
    /// ```
    /// let mut set = tree::Set::new();
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

    /// Returns the set's entry corresponding to the given item.
    ///
    /// # Examples
    ///
    /// ```
    /// use tree::set::Entry;
    ///
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// match set.entry(1) {
    ///     Entry::Occupied(e) => {
    ///         assert_eq!(*e.get(), 1);
    ///         assert_eq!(e.remove(), 1);
    ///     }
    ///     Entry::Vacant(_) => panic!("expected an occupied entry"),
    /// }
    ///
    /// assert!(!set.contains(&1));
    ///
    /// match set.entry(4) {
    ///     Entry::Occupied(_) => panic!("expected a vacant entry"),
    ///     Entry::Vacant(e) => e.insert(),
    /// }
    ///
    /// assert!(set.contains(&4));
    /// ```
    pub fn entry(&mut self, item: T) -> Entry<T, A> {
        match self.map.entry(item) {
            map::Entry::Occupied(e) => Entry::Occupied(OccupiedEntry(e)),
            map::Entry::Vacant(e) => Entry::Vacant(VacantEntry(e)),
        }
    }

    /// Checks if the set contains the given item.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
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
    /// ```
    /// let mut set = tree::Set::new();
    /// assert_eq!(set.max(), None);
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.max(), Some(&3));
    /// ```
    pub fn max(&self) -> Option<&T> { self.map.max().map(|e| e.0) }

    /// Removes and returns the set's maximum item, or `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    /// assert_eq!(set.remove_max(), None);
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.remove_max(), Some(3));
    /// ```
    pub fn remove_max(&mut self) -> Option<T> { self.map.remove_max().map(|e| e.0) }

    /// Returns the entry corresponding to the set's maximum item.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    /// assert!(set.max_entry().is_none());
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// {
    ///     let mut e = set.max_entry().unwrap();
    ///     assert_eq!(*e.get(), 3);
    ///     assert_eq!(e.remove(), 3);
    /// }
    ///
    /// assert!(!set.contains(&3));
    /// ```
    pub fn max_entry(&mut self) -> Option<OccupiedEntry<T, A>> {
        self.map.max_entry().map(OccupiedEntry)
    }

    /// Returns a reference to the set's minimum item, or `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    /// assert_eq!(set.min(), None);
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.min(), Some(&1));
    /// ```
    pub fn min(&self) -> Option<&T> { self.map.min().map(|e| e.0) }

    /// Removes and returns the set's minimum item, or `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    /// assert_eq!(set.remove_min(), None);
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.remove_min(), Some(1));
    /// ```
    pub fn remove_min(&mut self) -> Option<T> { self.map.remove_min().map(|e| e.0) }

    /// Returns the entry corresponding to the set's minimum item.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    /// assert!(set.min_entry().is_none());
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// {
    ///     let mut e = set.min_entry().unwrap();
    ///     assert_eq!(*e.get(), 1);
    ///     assert_eq!(e.remove(), 1);
    /// }
    ///
    /// assert!(!set.contains(&1));
    /// ```
    pub fn min_entry(&mut self) -> Option<OccupiedEntry<T, A>> {
        self.map.min_entry().map(OccupiedEntry)
    }

    /// Returns a reference to the predecessor of the given item, or
    /// `None` if no such item is present in the set.
    ///
    /// If `inclusive` is `false`, this method finds the greatest item that is strictly less than
    /// the given item. If `inclusive` is `true`, this method finds the greatest item that is less
    /// than or equal to the given item.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.pred(&0, false), None);
    /// assert_eq!(set.pred(&1, false), None);
    /// assert_eq!(set.pred(&2, false), Some(&1));
    /// assert_eq!(set.pred(&3, false), Some(&2));
    /// assert_eq!(set.pred(&4, false), Some(&3));
    ///
    /// assert_eq!(set.pred(&0, true), None);
    /// assert_eq!(set.pred(&1, true), Some(&1));
    /// assert_eq!(set.pred(&2, true), Some(&2));
    /// assert_eq!(set.pred(&3, true), Some(&3));
    /// assert_eq!(set.pred(&4, true), Some(&3));
    /// ```
    pub fn pred<Q: ?Sized>(&self, item: &Q, inclusive: bool) -> Option<&T> where C: Compare<Q, T> {
        self.map.pred(item, inclusive).map(|e| e.0)
    }

    /// Removes the predecessor of the given item from the set and returns it, or `None` if no such
    /// item present in the set.
    ///
    /// If `inclusive` is `false`, this method removes the greatest item that is strictly less than
    /// the given item. If `inclusive` is `true`, this method removes the greatest item that is
    /// less than or equal to the given item.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.remove_pred(&1, false), None);
    /// assert!(set.contains(&1));
    ///
    /// assert_eq!(set.remove_pred(&2, false), Some(1));
    /// assert!(!set.contains(&1));
    ///
    /// assert_eq!(set.remove_pred(&2, true), Some(2));
    /// assert!(!set.contains(&2));
    /// ```
    pub fn remove_pred<Q: ?Sized>(&mut self, item: &Q, inclusive: bool) -> Option<T>
        where C: Compare<Q, T> {

        self.map.remove_pred(item, inclusive).map(|e| e.0)
    }

    /// Returns the entry corresponding to the predecessor of the given item.
    ///
    /// If `inclusive` is `false`, this method returns the entry corresponding to the greatest item
    /// that is strictly less than the given item. If `inclusive` is `true`, this method returns
    /// the entry corresponding to the greatest item that is less than or equal to the given item.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert!(set.pred_entry(&1, false).is_none());
    ///
    /// {
    ///     let mut e = set.pred_entry(&4, true).unwrap();
    ///     assert_eq!(*e.get(), 3);
    /// }
    ///
    /// {
    ///     let e = set.pred_entry(&3, false).unwrap();
    ///     assert_eq!(e.remove(), 2);
    /// }
    ///
    /// assert!(!set.contains(&2));
    /// ```
    pub fn pred_entry<Q: ?Sized>(&mut self, item: &Q, inclusive: bool)
        -> Option<OccupiedEntry<T, A>> where C: Compare<Q, T> {

        self.map.pred_entry(item, inclusive).map(OccupiedEntry)
    }

    /// Returns a reference to the successor of the given item, or
    /// `None` if no such item is present in the set.
    ///
    /// If `inclusive` is `false`, this method finds the smallest item that is strictly greater
    /// than the given item. If `inclusive` is `true`, this method finds the smallest item that is
    /// greater than or equal to the given item.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.succ(&0, false), Some(&1));
    /// assert_eq!(set.succ(&1, false), Some(&2));
    /// assert_eq!(set.succ(&2, false), Some(&3));
    /// assert_eq!(set.succ(&3, false), None);
    /// assert_eq!(set.succ(&4, false), None);
    ///
    /// assert_eq!(set.succ(&0, true), Some(&1));
    /// assert_eq!(set.succ(&1, true), Some(&1));
    /// assert_eq!(set.succ(&2, true), Some(&2));
    /// assert_eq!(set.succ(&3, true), Some(&3));
    /// assert_eq!(set.succ(&4, true), None);
    /// ```
    pub fn succ<Q: ?Sized>(&self, item: &Q, inclusive: bool) -> Option<&T> where C: Compare<Q, T> {
        self.map.succ(item, inclusive).map(|e| e.0)
    }

    /// Removes the successor of the given item from the set and returns it, or `None` if no such
    /// item present in the set.
    ///
    /// If `inclusive` is `false`, this method removes the smallest item that is strictly greater
    /// than the given item. If `inclusive` is `true`, this method removes the smallest item that
    /// is greater than or equal to the given item.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.remove_succ(&3, false), None);
    /// assert!(set.contains(&3));
    ///
    /// assert_eq!(set.remove_succ(&2, false), Some(3));
    /// assert!(!set.contains(&3));
    ///
    /// assert_eq!(set.remove_succ(&2, true), Some(2));
    /// assert!(!set.contains(&2));
    /// ```
    pub fn remove_succ<Q: ?Sized>(&mut self, item: &Q, inclusive: bool) -> Option<T>
        where C: Compare<Q, T> {

        self.map.remove_succ(item, inclusive).map(|e| e.0)
    }

    /// Returns the entry corresponding to the successor of the given item.
    ///
    /// If `inclusive` is `false`, this method returns the entry corresponding to the smallest item
    /// that is strictly greater than the given item. If `inclusive` is `true`, this method returns
    /// the entry corresponding to the smallest item that is greater than or equal to the given
    /// item.
    ///
    /// The given item need not itself be present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert!(set.succ_entry(&3, false).is_none());
    ///
    /// {
    ///     let mut e = set.succ_entry(&0, true).unwrap();
    ///     assert_eq!(*e.get(), 1);
    /// }
    ///
    /// {
    ///     let e = set.succ_entry(&1, false).unwrap();
    ///     assert_eq!(e.remove(), 2);
    /// }
    ///
    /// assert!(!set.contains(&2));
    /// ```
    pub fn succ_entry<Q: ?Sized>(&mut self, item: &Q, inclusive: bool)
        -> Option<OccupiedEntry<T, A>> where C: Compare<Q, T> {

        self.map.succ_entry(item, inclusive).map(OccupiedEntry)
    }

    /// Returns an iterator that consumes the set.
    ///
    /// The iterator yields the items in ascending order according to the set's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
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
    pub fn into_iter(self) -> IntoIter<T, A> { IntoIter(self.map.into_iter()) }

    /// Returns an iterator over the set.
    ///
    /// The iterator yields the items in ascending order according to the set's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::new();
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
    pub fn iter(&self) -> Iter<T, A> { Iter(self.map.iter()) }
}

#[cfg(feature = "range")]
impl<T, A, C> Set<T, A, C> where A: Augment, C: Compare<T> {
    /// Returns an iterator that consumes the set, yielding only those items that lie in the given
    /// range.
    ///
    /// The iterator yields the items in ascending order according to the set's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(collections)]
    /// # extern crate tree;
    /// # fn main() {
    /// use std::collections::Bound::{Excluded, Unbounded};
    ///
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.into_range(Excluded(&1), Unbounded).collect::<Vec<_>>(), [2, 3]);
    /// # }
    /// ```
    pub fn into_range<Min: ?Sized, Max: ?Sized>(self, min: Bound<&Min>, max: Bound<&Max>)
        -> IntoRange<T, A> where C: Compare<Min, T> + Compare<Max, T> {

        IntoRange(self.map.into_range(min, max))
    }

    /// Returns an iterator over the set's items that lie in the given range.
    ///
    /// The iterator yields the items in ascending order according to the set's comparator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(collections)]
    /// # extern crate tree;
    /// # fn main() {
    /// use std::collections::Bound::{Included, Excluded, Unbounded};
    ///
    /// let mut set = tree::Set::new();
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.range(Unbounded, Unbounded).collect::<Vec<_>>(), [&1, &2, &3]);
    /// assert_eq!(set.range(Excluded(&1), Included(&5)).collect::<Vec<_>>(), [&2, &3]);
    /// assert_eq!(set.range(Included(&1), Excluded(&2)).collect::<Vec<_>>(), [&1]);
    /// # }
    /// ```
    pub fn range<Min: ?Sized, Max: ?Sized>(&self, min: Bound<&Min>, max: Bound<&Max>)
        -> Range<T, A> where C: Compare<Min, T> + Compare<Max, T> {

        Range(self.map.range(min, max))
    }
}

impl<T, C> Set<T, OrderStat, C> where C: Compare<T> {
    /// Returns a reference to the item at the given in-order index in the set, or `None` if the
    /// index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = tree::Set::<_, tree::OrderStat>::with_augment();
    /// assert_eq!(set.select(0), None);
    ///
    /// set.insert(2);
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// assert_eq!(set.select(0), Some(&1));
    /// assert_eq!(set.select(1), Some(&2));
    /// assert_eq!(set.select(2), Some(&3));
    /// assert_eq!(set.select(3), None);
    /// ```
    pub fn select(&self, index: usize) -> Option<&T> { self.map.select(index).map(|e| e.0) }

    /// Removes the item at the given in-order index in the set, or `None` if the index is out of
    /// bounds.
    pub fn remove_select(&mut self, index: usize) -> Option<T> {
        self.map.remove_select(index).map(|e| e.0)
    }

    /// Returns the entry corresponding to the item at the given in-order index.
    pub fn select_entry(&mut self, index: usize) -> Option<OccupiedEntry<T, OrderStat>> {
        self.map.select_entry(index).map(OccupiedEntry)
    }
}

impl<T, A, C> Debug for Set<T, A, C> where T: Debug, A: Augment, C: Compare<T> {
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

impl<T, A, C> Default for Set<T, A, C> where A: Augment, C: Compare<T> + Default {
    fn default() -> Self { Set { map: Map::default() } }
}

impl<T, A, C> Extend<T> for Set<T, A, C> where A: Augment, C: Compare<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, it: I) {
        for item in it { self.insert(item); }
    }
}

impl<T, A, C> iter::FromIterator<T> for Set<T, A, C> where A: Augment, C: Compare<T> + Default {
    fn from_iter<I: IntoIterator<Item=T>>(it: I) -> Self {
        let mut set = Set::default();
        set.extend(it);
        set
    }
}

impl<T, A, C> Hash for Set<T, A, C> where T: Hash, A: Augment, C: Compare<T> {
    fn hash<H: hash::Hasher>(&self, h: &mut H) { self.map.hash(h); }
}

impl<T, C> ::std::ops::Index<usize> for Set<T, OrderStat, C> where C: Compare<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T { self.select(index).expect("index out of bounds") }
}

impl<'a, T, A, C> IntoIterator for &'a Set<T, A, C> where A: Augment, C: Compare<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, A>;
    fn into_iter(self) -> Iter<'a, T, A> { self.iter() }
}

impl<T, A, C> IntoIterator for Set<T, A, C> where A: Augment, C: Compare<T> {
    type Item = T;
    type IntoIter = IntoIter<T, A>;
    fn into_iter(self) -> IntoIter<T, A> { self.into_iter() }
}

impl<T, A, C> PartialEq for Set<T, A, C> where A: Augment, C: Compare<T> {
    fn eq(&self, other: &Self) -> bool { self.map == other.map }
}

impl<T, A, C> Eq for Set<T, A, C> where A: Augment, C: Compare<T> {}

impl<T, A, C> PartialOrd for Set<T, A, C> where A: Augment, C: Compare<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.map.partial_cmp(&other.map)
    }
}

impl<T, A, C> Ord for Set<T, A, C> where A: Augment, C: Compare<T> {
    fn cmp(&self, other: &Self) -> Ordering { Ord::cmp(&self.map, &other.map) }
}

/// An iterator that consumes the set.
///
/// The iterator yields the items in ascending order according to the set's comparator.
///
/// # Examples
///
/// Acquire through [`Set::into_iter`](struct.Set.html#method.into_iter) or the
/// `IntoIterator` trait:
///
/// ```
/// let mut set = tree::Set::new();
///
/// set.insert(2);
/// set.insert(1);
/// set.insert(3);
///
/// for item in set {
///     println!("{:?}", item);
/// }
/// ```
#[derive(Clone)]
pub struct IntoIter<T, A = ()>(map::IntoIter<T, (), A>);

impl<T, A> Iterator for IntoIter<T, A> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.0.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<T, A> DoubleEndedIterator for IntoIter<T, A> {
    fn next_back(&mut self) -> Option<T> { self.0.next_back().map(|e| e.0) }
}

impl<T, A> ExactSizeIterator for IntoIter<T, A> {}

/// An iterator over the set.
///
/// The iterator yields the items in ascending order according to the set's comparator.
///
/// # Examples
///
/// Acquire through [`Set::iter`](struct.Set.html#method.iter) or the `IntoIterator` trait:
///
/// ```
/// let mut set = tree::Set::new();
///
/// set.insert(2);
/// set.insert(1);
/// set.insert(3);
///
/// for item in &set {
///     println!("{:?}", item);
/// }
/// ```
pub struct Iter<'a, T: 'a, A: 'a = ()>(map::Iter<'a, T, (), A>);

impl<'a, T, A> Clone for Iter<'a, T, A> {
    fn clone(&self) -> Self { Iter(self.0.clone()) }
}

impl<'a, T, A> Iterator for Iter<'a, T, A> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { self.0.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, T, A> DoubleEndedIterator for Iter<'a, T, A> {
    fn next_back(&mut self) -> Option<&'a T> { self.0.next_back().map(|e| e.0) }
}

impl<'a, T, A> ExactSizeIterator for Iter<'a, T, A> {}

/// An iterator that consumes the set, yielding only those items that lie in a given range.
///
/// The iterator yields the items in ascending order according to the set's comparator.
///
/// Acquire through [`Set::into_range`](struct.Set.html#method.into_range).
#[cfg(feature = "range")]
#[derive(Clone)]
pub struct IntoRange<T, A = ()>(map::IntoRange<T, (), A>);

#[cfg(feature = "range")]
impl<T, A> Iterator for IntoRange<T, A> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.0.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

#[cfg(feature = "range")]
impl<T, A> DoubleEndedIterator for IntoRange<T, A> {
    fn next_back(&mut self) -> Option<T> { self.0.next_back().map(|e| e.0) }
}

/// An iterator over the set's items that lie in a given range.
///
/// The iterator yields the items in ascending order according to the set's comparator.
///
/// Acquire through [`Set::range`](struct.Set.html#method.range).
#[cfg(feature = "range")]
pub struct Range<'a, T: 'a, A: 'a = ()>(map::Range<'a, T, (), A>);

#[cfg(feature = "range")]
impl<'a, T, A> Clone for Range<'a, T, A> {
    fn clone(&self) -> Self { Range(self.0.clone()) }
}

#[cfg(feature = "range")]
impl<'a, T, A> Iterator for Range<'a, T, A> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { self.0.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

#[cfg(feature = "range")]
impl<'a, T, A> DoubleEndedIterator for Range<'a, T, A> {
    fn next_back(&mut self) -> Option<&'a T> { self.0.next_back().map(|e| e.0) }
}

/// An entry in the set.
pub enum Entry<'a, T: 'a, A: 'a = ()> where A: Augment {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, T, A>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, T, A>),
}

/// An occupied entry.
pub struct OccupiedEntry<'a, T: 'a, A: 'a = ()>(map::OccupiedEntry<'a, T, (), A>) where A: Augment;

impl<'a, T, A> OccupiedEntry<'a, T, A> where A: Augment {
    /// Returns a reference to the entry's item.
    pub fn get(&self) -> &T { self.0.key() }

    /// Removes the entry from the set and returns its item.
    pub fn remove(self) -> T { self.0.remove().0 }
}

/// A vacant entry.
pub struct VacantEntry<'a, T: 'a, A: 'a = ()>(map::VacantEntry<'a, T, (), A>) where A: Augment;

impl<'a, T, A> VacantEntry<'a, T, A> where A: Augment {
    /// Inserts the entry into the set with its item.
    pub fn insert(self) { self.0.insert(()); }
}
