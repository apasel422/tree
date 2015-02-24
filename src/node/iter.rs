use std::marker::PhantomData;
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

pub struct Iter<N, D> where N: NodeRef, D: Dir {
    stack: Vec<N>,
    node: Option<N>,
    size: usize,
    _dir: PhantomData<*mut D>,
}

impl<N, D> Iter<N, D> where N: NodeRef, D: Dir {
    pub fn new(node: Option<N>, size: usize) -> Iter<N, D> {
        Iter { stack: vec![], node: node, size: size, _dir: PhantomData }
    }
}

impl<N, D> Clone for Iter<N, D> where N: Clone + NodeRef, D: Dir {
    fn clone(&self) -> Iter<N, D> {
        Iter {
            stack: self.stack.clone(),
            node: self.node.clone(),
            size: self.size,
            _dir: PhantomData,
        }
    }
}

impl<N: NodeRef, D> Iterator for Iter<N, D> where D: Dir {
    type Item = N::Item;

    fn next(&mut self) -> Option<N::Item> {
        while let Some(mut node) = self.node.take() {
            self.node =
                <D as Dir>::forward(&mut node, |node| node.left(), |node| node.right());
            self.stack.push(node);
        }

        self.stack.pop().map(|mut node| {
            self.size -= 1;
            self.node =
                <D as Dir>::reverse(&mut node, |node| node.left(), |node| node.right());
            node.item()
        })
    }
}

pub struct IterMut<'a, K: 'a, V: 'a, D> where D: Dir {
    iter: Iter<&'a Node<K, V>, D>,
    _mut: PhantomData<&'a mut V>,
}

impl<'a, K, V, D> IterMut<'a, K, V, D> where D: Dir {
    pub fn new(node: &'a mut Link<K, V>, size: usize) -> IterMut<'a, K, V, D> {
        IterMut { iter: Iter::new(node.as_node_ref(), size), _mut: PhantomData }
    }
}

impl<'a, K, V, D> Iterator for IterMut<'a, K, V, D> where D: Dir {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        let next = self.iter.next();
        unsafe { ::std::mem::transmute(next) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, K, V, D> ExactSizeIterator for IterMut<'a, K, V, D> where D: Dir {}
