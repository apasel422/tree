use compare::Compare;
use std::cmp::Ordering::*;
use std::collections::{Bound, VecDeque};
use std::marker::PhantomData;
use self::visit::{Seen, Visit};
use super::{Link, LinkExt, Node};

trait NodeRef {
    type Key;
    type Item;
    fn key(&self) -> &Self::Key;
    fn item(self) -> Self::Item;
    fn left(&mut self) -> Option<Self>;
    fn right(&mut self) -> Option<Self>;
}

impl<'a, K, V> NodeRef for &'a Node<K, V> {
    type Key = K;
    type Item = (&'a K, &'a V);
    fn key(&self) -> &K { &self.key }
    fn item(self) -> (&'a K, &'a V) { (&self.key, &self.value) }
    fn left(&mut self) -> Option<&'a Node<K, V>> { self.left.as_node_ref() }
    fn right(&mut self) -> Option<&'a Node<K, V>> { self.right.as_node_ref() }
}

impl<K, V> NodeRef for Box<Node<K, V>> {
    type Key = K;
    type Item = (K, V);
    fn key(&self) -> &K { &self.key }
    fn item(self) -> (K, V) { let node = *self; (node.key, node.value) }
    fn left(&mut self) -> Link<K, V> { self.left.take() }
    fn right(&mut self) -> Link<K, V> { self.right.take() }
}

#[derive(Clone)]
pub struct Iter<N> where N: NodeRef {
    visits: VecDeque<Visit<N>>,
    size: usize,
}

macro_rules! bound {
    ($iter:expr,
     $cmp:expr,
     $bound:expr,
     $ordering_pre:ident,
     $ordering_post:ident,
     $pre:ident,
     $post:ident,
     $mut_:ident,
     $pop:ident,
     $push:ident
    ) => {
        if let Some((key, inc)) = bound_to_opt($bound) {
            loop {
                let op = match $iter.visits.$mut_() {
                    None => break,
                    Some(visit) => match $cmp.compare(key, visit.key()) {
                        Equal =>
                            if inc {
                                if visit.$pre().is_some() { $iter.size -= 1; }
                                break;
                            } else {
                                Op::PopPush(visit.$post(), true)
                            },
                        $ordering_post => Op::PopPush(visit.$post(), false),
                        $ordering_pre => Op::Push(visit.$pre()),
                    },
                };

                match op {
                    Op::Push(node_ref) => match node_ref {
                        None => break,
                        Some(node) => $iter.visits.$push(Visit::new(node)),
                    },
                    Op::PopPush(node_ref, terminate) => {
                        $iter.visits.$pop();
                        $iter.size -= 1;
                        if let Some(node) = node_ref { $iter.visits.$push(Visit::new(node)); }
                        if terminate { break; }
                    }
                }
            }
        }
    }
}

impl<N> Iter<N> where N: NodeRef {
    pub fn new(root: Option<N>, size: usize) -> Iter<N> {
        Iter { visits: root.into_iter().map(Visit::new).collect(), size: size }
    }

    pub fn range<C, Min: ?Sized, Max: ?Sized>(root: Option<N>, size: usize, cmp: &C,
                                              min: Bound<&Min>, max: Bound<&Max>)
        -> Iter<N> where C: Compare<Min, N::Key> + Compare<Max, N::Key> {

        fn bound_to_opt<T>(bound: Bound<T>) -> Option<(T, bool)> {
            match bound {
                Bound::Unbounded => None,
                Bound::Included(bound) => Some((bound, true)),
                Bound::Excluded(bound) => Some((bound, false)),
            }
        }

        enum Op<T> {
            PopPush(Option<T>, bool),
            Push(Option<T>),
        }

        let mut it = Iter::new(root, size);

        bound!(it, cmp, min, Less, Greater, left, right, back_mut, pop_back, push_back);
        bound!(it, cmp, max, Greater, Less, right, left, front_mut, pop_front, push_front);

        it
    }

    pub fn range_size_hint(&self) -> (usize, Option<usize>) {
        (self.visits.len(), Some(self.size))
    }
}

macro_rules! next {
    ($iter:expr,
     $seen_pre:ident,
     $seen_post:ident,
     $pre:ident,
     $post:ident,
     $mut_:ident,
     $pop:ident,
     $push:ident) => {
        loop {
            let (pop, push) = match $iter.visits.$mut_() {
                None => return None,
                Some(visit) => match visit.seen() {
                    Seen::N | Seen::$seen_post => (false, visit.$pre()),
                    Seen::$seen_pre => (true, visit.$post()),
                    Seen::B => (true, None),
                }
            };

            let item = if pop {
                $iter.size -= 1;
                $iter.visits.$pop().map(Visit::item)
            } else {
                None
            };

            if let Some(node) = push { $iter.visits.$push(Visit::new(node)); }
            if item.is_some() { return item; }
        }
    }
}

impl<N> Iterator for Iter<N> where N: NodeRef {
    type Item = N::Item;

    fn next(&mut self) -> Option<N::Item> {
        next!(self, L, R, left, right, back_mut, pop_back, push_back)
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (self.size, Some(self.size)) }
}

impl<N> DoubleEndedIterator for Iter<N> where N: NodeRef {
    fn next_back(&mut self) -> Option<N::Item> {
        next!(self, R, L, right, left, front_mut, pop_front, push_front)
    }
}

mod visit {
    #[derive(Clone)]
    pub struct Visit<N> where N: super::NodeRef {
        node: N,
        seen: Seen,
    }

    impl<N> Visit<N> where N: super::NodeRef {
        pub fn new(node: N) -> Visit<N> { Visit { node: node, seen: Seen::N } }

        pub fn left(&mut self) -> Option<N> {
            match self.seen {
                Seen::N => { self.seen = Seen::L; self.node.left() }
                Seen::R => { self.seen = Seen::B; self.node.left() }
                Seen::L | Seen::B => None,
            }
        }

        pub fn right(&mut self) -> Option<N> {
            match self.seen {
                Seen::N => { self.seen = Seen::R; self.node.right() }
                Seen::L => { self.seen = Seen::B; self.node.right() }
                Seen::R | Seen::B => None,
            }
        }

        pub fn key(&self) -> &N::Key { self.node.key() }

        pub fn item(self) -> N::Item { self.node.item() }

        pub fn seen(&self) -> Seen { self.seen }
    }

    #[derive(Clone, Copy)]
    pub enum Seen {
        N,
        L,
        R,
        B,
    }
}

pub struct IterMut<'a, K: 'a, V: 'a> {
    iter: Iter<&'a Node<K, V>>,
    _mut: PhantomData<&'a mut V>,
}

impl<'a, K, V> IterMut<'a, K, V> {
    pub fn new(node: &'a mut Link<K, V>, size: usize) -> IterMut<'a, K, V> {
        IterMut { iter: Iter::new(node.as_node_ref(), size), _mut: PhantomData }
    }

    pub fn range<C, Min: ?Sized, Max: ?Sized>(node: &'a mut Link<K, V>, size: usize, cmp: &C,
                                              min: Bound<&Min>, max: Bound<&Max>)
        -> IterMut<'a, K, V> where C: Compare<Min, K> + Compare<Max, K> {

        IterMut { iter: Iter::range(node.as_node_ref(), size, cmp, min, max), _mut: PhantomData }
    }

    pub fn range_size_hint(&self) -> (usize, Option<usize>) { self.iter.range_size_hint() }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        let next = self.iter.next();
        unsafe { ::std::mem::transmute(next) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        let next_back = self.iter.next_back();
        unsafe { ::std::mem::transmute(next_back) }
    }
}
