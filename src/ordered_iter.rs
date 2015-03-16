extern crate ordered_iter;

use self::ordered_iter::{OrderedMapIterator, OrderedSetIterator};
use super::{map, set};

impl<K, V> OrderedMapIterator for map::IntoIter<K, V> where K: Ord {
    type Key = K;
    type Val = V;
}

impl<'a, K, V> OrderedMapIterator for map::Iter<'a, K, V> where K: Ord {
    type Key = &'a K;
    type Val = &'a V;
}

impl<'a, K, V> OrderedMapIterator for map::IterMut<'a, K, V> where K: Ord {
    type Key = &'a K;
    type Val = &'a mut V;
}

impl<K, V> OrderedMapIterator for map::IntoRange<K, V> where K: Ord {
    type Key = K;
    type Val = V;
}

impl<'a, K, V> OrderedMapIterator for map::Range<'a, K, V> where K: Ord {
    type Key = &'a K;
    type Val = &'a V;
}

impl<'a, K, V> OrderedMapIterator for map::RangeMut<'a, K, V> where K: Ord {
    type Key = &'a K;
    type Val = &'a mut V;
}

impl<T> OrderedSetIterator for set::IntoIter<T> where T: Ord {}

impl<'a, T> OrderedSetIterator for set::Iter<'a, T> where T: Ord {}

impl<T> OrderedSetIterator for set::IntoRange<T> where T: Ord {}

impl<'a, T> OrderedSetIterator for set::Range<'a, T> where T: Ord {}
