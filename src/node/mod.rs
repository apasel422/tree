mod iter;

use collect::compare::Compare;
use std::cmp::Ordering::*;
use std::mem;

pub use self::iter::{Iter, IterMut};

pub type Link<K, V> = Option<Box<Node<K, V>>>;

pub trait LinkExt: Sized {
    type K;
    type V;
    fn as_node_ref(&self) -> Option<&Node<Self::K, Self::V>>;
    fn key_value(&self) -> Option<(&Self::K, &Self::V)>;
    fn key_value_mut(&mut self) -> Option<(&Self::K, &mut Self::V)>;
}

impl<K, V> LinkExt for Link<K, V> {
    type K = K;
    type V = V;

    fn as_node_ref(&self) -> Option<&Node<K, V>> {
        self.as_ref().map(|node| &**node)
    }

    fn key_value(&self) -> Option<(&K, &V)> {
        self.as_ref().map(|node| (&node.key, &node.value))
    }

    fn key_value_mut(&mut self) -> Option<(&K, &mut V)> {
        self.as_mut().map(|&mut box ref mut node| (&node.key, &mut node.value))
    }
}

#[derive(Clone)]
pub struct Node<K, V> {
    left: Link<K, V>,
    right: Link<K, V>,
    key: K,
    value: V,
}

pub fn insert<K, V, C>(link: &mut Link<K, V>, cmp: &C, key: K, value: V) -> Option<V>
    where C: Compare<K> {

    match *link {
        None => {
            *link = Some(box Node { left: None, right: None, key: key, value: value });
            None
        }
        Some(ref mut node) => match cmp.compare(&key, &node.key) {
            Equal => {
                node.key = key;
                Some(mem::replace(&mut node.value, value))
            }
            Less => insert(&mut node.left, cmp, key, value),
            Greater => insert(&mut node.right, cmp, key, value),
        },
    }
}

trait LinkRef<'a>: Sized {
    type K: 'a;
    type V: 'a;
    fn as_ref(self) -> &'a Link<Self::K, Self::V>;
    unsafe fn from_ref(link: &'a Link<Self::K, Self::V>) -> Self;

    fn with<F>(self, f: F) -> Self
        where F: FnOnce(&'a Link<Self::K, Self::V>) -> &'a Link<Self::K, Self::V> {

        let link = f(self.as_ref());
        unsafe { LinkRef::from_ref(link) }
    }
}

impl<'a, K: 'a, V: 'a> LinkRef<'a> for &'a Link<K, V> {
    type K = K;
    type V = V;

    fn as_ref(self) -> &'a Link<K, V> { self }

    fn from_ref(link: &'a Link<K, V>) -> &'a Link<K, V> { link }
}

impl<'a, K: 'a, V: 'a> LinkRef<'a> for &'a mut Link<K, V> {
    type K = K;
    type V = V;

    fn as_ref(self) -> &'a Link<K, V> { self }

    unsafe fn from_ref(link: &'a Link<K, V>) -> &'a mut Link<K, V> {
        mem::transmute(link)
    }
}

pub fn get<'a, L: LinkRef<'a>, C, Q: ?Sized>(link: L, cmp: &C, key: &Q) -> L
    where C: Compare<Q, L::K> {

    link.with(|mut link| loop {
        match *link {
            None => return link,
            Some(ref node) => match cmp.compare(key, &node.key) {
                Equal => return link,
                Less => link = &node.left,
                Greater => link = &node.right,
            },
        }
    })
}

trait Dir: ::std::marker::MarkerTrait {
    type Opposite: Dir;

    fn left() -> bool;

    fn forward<T, L, R, U>(t: T, l: L, r: R) -> U
        where L: FnOnce(T) -> U, R: FnOnce(T) -> U;

    fn reverse<T, L, R, U>(t: T, l: L, r: R) -> U
        where L: FnOnce(T) -> U, R: FnOnce(T) -> U;
}

pub enum Left {}

impl Dir for Left {
    type Opposite = Right;

    fn left() -> bool { true }

    fn forward<T, L, R, U>(t: T, l: L, _r: R) -> U
        where L: FnOnce(T) -> U, R: FnOnce(T) -> U { l(t) }

    fn reverse<T, L, R, U>(t: T, _l: L, r: R) -> U
        where L: FnOnce(T) -> U, R: FnOnce(T) -> U { r(t) }
}

pub enum Right {}

impl Dir for Right {
    type Opposite = Left;

    fn left() -> bool { false }

    fn forward<T, L, R, U>(t: T, _l: L, r: R) -> U
        where L: FnOnce(T) -> U, R: FnOnce(T) -> U { r(t) }

    fn reverse<T, L, R, U>(t: T, l: L, _r: R) -> U
        where L: FnOnce(T) -> U, R: FnOnce(T) -> U { l(t) }
}

pub fn extremum<'a, L, D>(link: L) -> L where L: LinkRef<'a>, D: Dir {
    link.with(|mut link| {
        while let Some(ref node) = *link {
            let child = <D as Dir>::forward(node, |node| &node.left, |node| &node.right);
            if child.is_some() { link = child; } else { break; }
        }

        link
    })
}

pub fn closest<'a, L: LinkRef<'a>, C, Q: ?Sized, D>(link: L, cmp: &C, key: &Q, inc: bool) -> L
    where C: Compare<Q, L::K>, D: Dir {

    link.with(|mut link| {
        let mut closest_ancstr = None;

        loop {
            match *link {
                None => break,
                Some(ref node) => match cmp.compare(key, &node.key) {
                    Equal => return
                        if inc {
                            link
                        } else {
                            let child =
                                <D as Dir>::forward(node, |node| &node.left, |node| &node.right);

                            match closest_ancstr {
                                Some(ancstr) if child.is_none() => ancstr,
                                _ => extremum::<_, <D as Dir>::Opposite>(child),
                            }
                        },
                    order =>
                        if <D as Dir>::left() == (order == Less) {
                            link =
                                <D as Dir>::forward(node, |node| &node.left, |node| &node.right);
                        } else {
                            closest_ancstr = Some(link);
                            link =
                                <D as Dir>::reverse(node, |node| &node.left, |node| &node.right);
                        },
                },
            }
        }

        closest_ancstr.unwrap_or(link)
    })
}
