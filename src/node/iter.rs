use std::marker::PhantomData;
use std::collections::VecDeque;
use super::{Dir, Link, LinkExt, Node};

trait NodeRef {
    type Item;
    fn item(self) -> Self::Item;
    fn left(&mut self) -> Option<Self>;
    fn right(&mut self) -> Option<Self>;
}

impl<'a, K, V> NodeRef for &'a Node<K, V> {
    type Item = (&'a K, &'a V);
    fn item(self) -> (&'a K, &'a V) { (&self.key, &self.value) }
    fn left(&mut self) -> Option<&'a Node<K, V>> { self.left.as_node_ref() }
    fn right(&mut self) -> Option<&'a Node<K, V>> { self.right.as_node_ref() }
}

impl<K, V> NodeRef for Box<Node<K, V>> {
    type Item = (K, V);
    fn item(self) -> (K, V) { let node = *self; (node.key, node.value) }
    fn left(&mut self) -> Link<K, V> { self.left.take() }
    fn right(&mut self) -> Link<K, V> { self.right.take() }
}

#[derive(Clone)]
pub struct Iter<N> where N: NodeRef {
    visits: VecDeque<Visit<N>>,
    size: usize,
}

impl<N> Iter<N> where N: NodeRef {
    pub fn new(root: Option<N>, size: usize) -> Iter<N> {
        Iter { visits: root.into_iter().map(Visit::new).collect(), size: size }
    }
}

impl<N> Iterator for Iter<N> where N: NodeRef {
    type Item = N::Item;

    fn next(&mut self) -> Option<N::Item> {
        loop {
            let op = match self.visits.back_mut() {
                None => return None,
                Some(visit) => match visit.seen {
                    Seen::N => { visit.seen = Seen::L; Op::Push(visit.node.left()) }
                    Seen::L => { visit.seen = Seen::B; Op::PopPush(visit.node.right()) }
                    Seen::R => { visit.seen = Seen::B; Op::Push(visit.node.left()) }
                    Seen::B => Op::Pop,
                }
            };

            match op {
                Op::Push(node_ref) =>
                    if let Some(node) = node_ref { self.visits.push_back(Visit::new(node)); },
                Op::PopPush(node_ref) => {
                    self.size -= 1;
                    let visit = self.visits.pop_back().unwrap();
                    if let Some(node) = node_ref { self.visits.push_back(Visit::new(node)); }
                    return Some(visit.node.item());
                }
                Op::Pop => {
                    self.size -= 1;
                    let visit = self.visits.pop_back().unwrap();
                    return Some(visit.node.item());
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (self.size, Some(self.size)) }
}

impl<N> DoubleEndedIterator for Iter<N> where N: NodeRef {
    fn next_back(&mut self) -> Option<N::Item> {
        loop {
            let op = match self.visits.front_mut() {
                None => return None,
                Some(visit) => match visit.seen {
                    Seen::N => { visit.seen = Seen::R; Op::Push(visit.node.right()) }
                    Seen::R => { visit.seen = Seen::B; Op::PopPush(visit.node.left()) }
                    Seen::L => { visit.seen = Seen::B; Op::Push(visit.node.right()) }
                    Seen::B => Op::Pop,
                }
            };

            match op {
                Op::Push(node_ref) =>
                    if let Some(node) = node_ref { self.visits.push_front(Visit::new(node)); },
                Op::PopPush(node_ref) => {
                    self.size -= 1;
                    let visit = self.visits.pop_front().unwrap();
                    if let Some(node) = node_ref { self.visits.push_front(Visit::new(node)); }
                    return Some(visit.node.item());
                }
                Op::Pop => {
                    self.size -= 1;
                    let visit = self.visits.pop_front().unwrap();
                    return Some(visit.node.item());
                }
            }
        }
    }
}

#[derive(Clone)]
struct Visit<N> where N: NodeRef {
    node: N,
    seen: Seen,
}

impl<N> Visit<N> where N: NodeRef {
    fn new(node: N) -> Visit<N> { Visit { node: node, seen: Seen::N } }
}

#[derive(Clone)]
enum Seen {
    N,
    L,
    R,
    B,
}

enum Op<T> {
    Push(Option<T>),
    PopPush(Option<T>),
    Pop,
}

pub struct IterMut<'a, K: 'a, V: 'a> {
    iter: Iter<&'a Node<K, V>>,
    _mut: PhantomData<&'a mut V>,
}

impl<'a, K, V> IterMut<'a, K, V> {
    pub fn new(node: &'a mut Link<K, V>, size: usize) -> IterMut<'a, K, V> {
        IterMut { iter: Iter::new(node.as_node_ref(), size), _mut: PhantomData }
    }
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

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}
