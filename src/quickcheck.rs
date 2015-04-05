extern crate quickcheck;

use compare::Compare;
use self::quickcheck::{Arbitrary, Gen};
use super::{Augment, Map, Set};

impl<K, V, A, C> Arbitrary for Map<K, V, A, C>
    where K: Arbitrary, V: Arbitrary, A: 'static + Augment + Clone + Send,
          C: 'static + Clone + Compare<K> + Default + Send {

    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        Vec::<(K, V)>::arbitrary(gen).into_iter().collect()
    }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        let vec: Vec<(K, V)> = self.clone().into_iter().collect();
        Box::new(vec.shrink().map(|vec| vec.into_iter().collect()))
    }
}

impl<T, A, C> Arbitrary for Set<T, A, C>
    where T: Arbitrary, A: 'static + Augment + Clone + Send,
          C: 'static + Clone + Compare<T> + Default + Send {

    fn arbitrary<G: Gen>(gen: &mut G) -> Self { Vec::<T>::arbitrary(gen).into_iter().collect() }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        let vec: Vec<T> = self.clone().into_iter().collect();
        Box::new(vec.shrink().map(|vec| vec.into_iter().collect()))
    }
}
