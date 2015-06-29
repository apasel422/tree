extern crate ordered_iter;

use self::ordered_iter::{OrderedMapIterator, OrderedSetIterator};
use super::{map, set};

impl<K, V, B> OrderedMapIterator for map::IntoIter<K, V, B> where K: Ord {
    type Key = K;
    type Val = V;
}

impl<'a, K, V, B> OrderedMapIterator for map::Iter<'a, K, V, B> where K: Ord {
    type Key = &'a K;
    type Val = &'a V;
}

impl<'a, K, V, B> OrderedMapIterator for map::IterMut<'a, K, V, B> where K: Ord {
    type Key = &'a K;
    type Val = &'a mut V;
}

#[cfg(feature = "range")]
impl<K, V, B> OrderedMapIterator for map::IntoRange<K, V, B> where K: Ord {
    type Key = K;
    type Val = V;
}

#[cfg(feature = "range")]
impl<'a, K, V, B> OrderedMapIterator for map::Range<'a, K, V, B> where K: Ord {
    type Key = &'a K;
    type Val = &'a V;
}

#[cfg(feature = "range")]
impl<'a, K, V, B> OrderedMapIterator for map::RangeMut<'a, K, V, B> where K: Ord {
    type Key = &'a K;
    type Val = &'a mut V;
}

impl<T, B> OrderedSetIterator for set::IntoIter<T, B> where T: Ord {}

impl<'a, T, B> OrderedSetIterator for set::Iter<'a, T, B> where T: Ord {}

#[cfg(feature = "range")]
impl<T, B> OrderedSetIterator for set::IntoRange<T, B> where T: Ord {}

#[cfg(feature = "range")]
impl<'a, T, B> OrderedSetIterator for set::Range<'a, T, B> where T: Ord {}
