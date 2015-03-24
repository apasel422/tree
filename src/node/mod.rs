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
        self.as_mut().map(|&mut box ref mut node| (&node.key, &mut node.value))
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
            *link = Some(box Node { left: None, right: None, level: 1, key: key, value: value });
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

// Adapted from https://github.com/Gankro/collect-rs/tree/map.rs
pub fn remove<K, V, C, Q: ?Sized>(node: &mut Link<K, V>, cmp: &C, key: &Q)
    -> Option<(K, V)> where C: Compare<Q, K> {

    fn heir_swap<K, V>(node: &mut Node<K, V>, child: &mut Link<K, V>) {
        if let Some(ref mut x) = *child {
            let mut x = x;

            loop {
                let x_curr = x;

                x = match x_curr.right {
                    None => break,
                    Some(ref mut right) => right,
                };
            }

            swap(&mut node.key, &mut x.key);
            swap(&mut node.value, &mut x.value);
        }
    }

    if let Some(ref mut save) = *node {
        let (old, rebalance) = match cmp.compare(key, &save.key) {
            Less => (remove(&mut save.left, cmp, key), true),
            Greater => (remove(&mut save.right, cmp, key), true),
            Equal => {
                if let Some(mut left) = save.left.take() {
                    if save.right.is_some() {
                        if left.right.is_some() {
                            heir_swap(save, &mut left.right);
                        } else {
                            swap(&mut save.key, &mut left.key);
                            swap(&mut save.value, &mut left.value);
                        }

                        save.left = Some(left);
                        (remove(&mut save.left, cmp, key), true)
                    } else {
                        let box Node { key, value, .. } = replace(save, left);
                        *save = save.left.take().unwrap();
                        (Some((key, value)), true)
                    }
                } else if let Some(new) = save.right.take() {
                    let box Node { key, value, .. } = replace(save, new);
                    (Some((key, value)), true)
                } else {
                    (None, false)
                }
            }
        };

        if rebalance {
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

            return old;
        }
    }

    node.take().map(|box node| (node.key, node.value))
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

pub trait Dir {
    type Opposite: Dir<Opposite=Self>;

    fn left() -> bool;

    fn forward<K, V>(node: &Node<K, V>) -> &Link<K, V>;

    fn reverse<K, V>(node: &Node<K, V>) -> &Link<K, V>;
}

pub enum Left {}

impl Dir for Left {
    type Opposite = Right;

    fn left() -> bool { true }

    fn forward<K, V>(node: &Node<K, V>) -> &Link<K, V> { &node.left }

    fn reverse<K, V>(node: &Node<K, V>) -> &Link<K, V> { &node.right }
}

pub enum Right {}

impl Dir for Right {
    type Opposite = Left;

    fn left() -> bool { false }

    fn forward<K, V>(node: &Node<K, V>) -> &Link<K, V> { &node.right }

    fn reverse<K, V>(node: &Node<K, V>) -> &Link<K, V> { &node.left }
}

pub fn remove_extremum<K, V, D>(link: &mut Link<K, V>) -> Option<(K, V)> where D: Dir {
    extremum::<_, D>(link).take().map(|box node| (node.key, node.value))
}

pub fn extremum<'a, L, D>(link: L) -> L where L: LinkRef<'a>, D: Dir {
    link.with(|mut link| {
        while let Some(ref node) = *link {
            let child = D::forward(node);
            if child.is_some() { link = child; } else { break; }
        }

        link
    })
}

pub fn closest<'a, L, C, Q: ?Sized, D>(link: L, cmp: &C, key: &Q, inc: bool) -> L
    where L: LinkRef<'a>, C: Compare<Q, L::K>, D: Dir {

    link.with(|mut link| {
        let mut closest_ancstr = None;

        while let Some(ref node) = *link {
            match cmp.compare(key, &node.key) {
                Equal => return
                    if inc {
                        link
                    } else {
                        let child = D::forward(node);

                        match closest_ancstr {
                            Some(ancstr) if child.is_none() => ancstr,
                            _ => extremum::<_, D::Opposite>(child),
                        }
                    },
                order => link =
                    if D::left() == (order == Less) {
                        D::forward(node)
                    } else {
                        closest_ancstr = Some(link);
                        D::reverse(node)
                    },
            }
        }

        closest_ancstr.unwrap_or(link)
    })
}
