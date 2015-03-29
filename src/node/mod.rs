mod iter;

#[cfg(test)]
mod test;

use compare::Compare;
use std::cmp::Ordering::*;
use std::mem::{self, replace, swap};

pub use self::iter::Iter;

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
        self.as_mut().map(|node| { let mut node = &mut **node; (&node.key, &mut node.value) })
    }
}

#[derive(Clone)]
pub struct Node<K, V> {
    left: Link<K, V>,
    right: Link<K, V>,
    level: usize,
    key: K,
    value: V,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Node { left: None, right: None, level: 1, key: key, value: value }
    }

    // Remove left horizontal link by rotating right
    //
    // From https://github.com/Gankro/collect-rs/tree/map.rs
    fn skew(&mut self) {
        if self.left.as_ref().map_or(false, |x| x.level == self.level) {
            let mut save = self.left.take().unwrap();
            swap(&mut self.left, &mut save.right); // save.right now None
            swap(self, &mut save);
            self.right = Some(save);
        }
    }

    // Remove dual horizontal link by rotating left and increasing level of
    // the parent
    //
    // From https://github.com/Gankro/collect-rs/tree/map.rs
    fn split(&mut self) {
        if self.right.as_ref().map_or(false,
          |x| x.right.as_ref().map_or(false, |y| y.level == self.level)) {
            let mut save = self.right.take().unwrap();
            swap(&mut self.right, &mut save.left); // save.left now None
            save.level += 1;
            swap(self, &mut save);
            self.left = Some(save);
        }
    }
}

pub fn insert<K, V, C>(link: &mut Link<K, V>, cmp: &C, key: K, value: V) -> Option<V>
    where C: Compare<K> {

    match *link {
        None => {
            *link = Some(Box::new(Node::new(key, value)));
            None
        }
        Some(ref mut node) => {
            let old_value = match cmp.compare(&key, &node.key) {
                Equal => return Some(mem::replace(&mut node.value, value)),
                Less => insert(&mut node.left, cmp, key, value),
                Greater => insert(&mut node.right, cmp, key, value),
            };

            node.skew();
            node.split();
            old_value
        },
    }
}

pub fn remove<K, V, C, Q: ?Sized>(link: &mut Link<K, V>, cmp: &C, key: &Q)
    -> Option<(K, V)> where C: Compare<Q, K> {

    let mut take = false;

    if let Some(ref mut node) = *link {
        let key_value = match cmp.compare(&key, &node.key) {
            Less => remove(&mut node.left, cmp, key),
            Greater => remove(&mut node.right, cmp, key),
            Equal => {
                let replacement = if node.left.is_some() {
                    Right::extremum(&mut node.left).take()
                } else if node.right.is_some() {
                    Left::extremum(&mut node.right).take()
                } else {
                    take = true;
                    None
                };

                replacement.map(|replacement| {
                    let replacement = *replacement; (
                        replace(&mut node.key, replacement.key),
                        replace(&mut node.value, replacement.value)
                    )
                })
            }
        };

        if key_value.is_some() {
            rebalance(node);
            return key_value;
        }
    }

    if take {
        link.take().map(|node| { let node = *node; (node.key, node.value) })
    } else {
        None
    }
}

fn rebalance<K, V>(save: &mut Node<K, V>) {
    let left_level = save.left.as_ref().map_or(0, |node| node.level);
    let right_level = save.right.as_ref().map_or(0, |node| node.level);

    // re-balance, if necessary
    if left_level < save.level - 1 || right_level < save.level - 1 {
        save.level -= 1;

        if right_level > save.level {
            let save_level = save.level;
            if let Some(ref mut x) = save.right { x.level = save_level; }
        }

        save.skew();

        if let Some(ref mut right) = save.right {
            right.skew();
            if let Some(ref mut x) = right.right { x.skew() };
        }

        save.split();
        if let Some(ref mut x) = save.right { x.split(); }
    }
}

pub trait LinkRef<'a>: Sized {
    type K: 'a;
    type V: 'a;
    fn into_ref(self) -> &'a Link<Self::K, Self::V>;
    unsafe fn from_ref(link: &'a Link<Self::K, Self::V>) -> Self;

    fn with<F>(self, f: F) -> Self
        where F: FnOnce(&'a Link<Self::K, Self::V>) -> &'a Link<Self::K, Self::V> {

        let link = f(self.into_ref());
        unsafe { LinkRef::from_ref(link) }
    }
}

impl<'a, K: 'a, V: 'a> LinkRef<'a> for &'a Link<K, V> {
    type K = K;
    type V = V;

    fn into_ref(self) -> &'a Link<K, V> { self }

    unsafe fn from_ref(link: &'a Link<K, V>) -> &'a Link<K, V> { link }
}

impl<'a, K: 'a, V: 'a> LinkRef<'a> for &'a mut Link<K, V> {
    type K = K;
    type V = V;

    fn into_ref(self) -> &'a Link<K, V> { self }

    unsafe fn from_ref(link: &'a Link<K, V>) -> &'a mut Link<K, V> {
        mem::transmute(link)
    }
}

pub fn get<'a, L, C, Q: ?Sized>(link: L, cmp: &C, key: &Q) -> L
    where L: LinkRef<'a>, C: Compare<Q, L::K> {

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

pub trait Dir: Sized {
    type Opposite: Dir<Opposite=Self>;

    fn left() -> bool;

    fn forward<K, V>(node: &Node<K, V>) -> &Link<K, V>;
    fn forward_mut<K, V>(node: &mut Node<K, V>) -> &mut Link<K, V>;

    fn extremum<'a, L>(link: L) -> L where L: LinkRef<'a> {
        link.with(|mut link| {
            while let Some(ref node) = *link {
                let child = Self::forward(node);
                if child.is_some() { link = child; } else { break; }
            }

            link
        })
    }

    fn remove_extremum<K, V>(link: &mut Link<K, V>) -> Option<(K, V)> {
        match *link {
            Some(ref mut node) if Self::forward(node).is_some() => {
                let key_value = Self::remove_extremum(Self::forward_mut(node));
                rebalance(node);
                key_value
            }
            _ => link.take().map(|node| {
                let mut node = *node;
                *link = Self::Opposite::forward_mut(&mut node).take();
                (node.key, node.value)
            }),
        }
    }

    fn closest<'a, L, C, Q: ?Sized>(link: L, cmp: &C, key: &Q, inc: bool) -> L
        where L: LinkRef<'a>, C: Compare<Q, L::K> {

        link.with(|mut link| {
            let mut closest_ancstr = None;

            while let Some(ref node) = *link {
                match cmp.compare(key, &node.key) {
                    Equal => return
                        if inc {
                            link
                        } else {
                            let child = Self::forward(node);

                            match closest_ancstr {
                                Some(ancstr) if child.is_none() => ancstr,
                                _ => Self::Opposite::extremum(child),
                            }
                        },
                    order => link =
                        if Self::left() == (order == Less) {
                            Self::forward(node)
                        } else {
                            closest_ancstr = Some(link);
                            Self::Opposite::forward(node)
                        },
                }
            }

            closest_ancstr.unwrap_or(link)
        })
    }
}

#[allow(unused)] // FIXME: rust-lang/rust#23808
pub enum Left {}

impl Dir for Left {
    type Opposite = Right;

    fn left() -> bool { true }

    fn forward<K, V>(node: &Node<K, V>) -> &Link<K, V> { &node.left }
    fn forward_mut<K, V>(node: &mut Node<K, V>) -> &mut Link<K, V> { &mut node.left }
}

#[allow(unused)] // FIXME: rust-lang/rust#23808
pub enum Right {}

impl Dir for Right {
    type Opposite = Left;

    fn left() -> bool { false }

    fn forward<K, V>(node: &Node<K, V>) -> &Link<K, V> { &node.right }
    fn forward_mut<K, V>(node: &mut Node<K, V>) -> &mut Link<K, V> { &mut node.right }
}
