mod iter;

#[cfg(test)]
mod test;

use compare::Compare;
use std::cmp::Ordering::*;
use std::mem::{self, replace, swap};
use super::map::Entry;

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
    fn skew(node: &mut Box<Self>) {
        if node.left.as_ref().map_or(false, |x| x.level == node.level) {
            let mut save = node.left.take().unwrap();
            swap(&mut node.left, &mut save.right); // save.right now None
            swap(node, &mut save);
            node.right = Some(save);
        }
    }

    // Remove dual horizontal link by rotating left and increasing level of
    // the parent
    //
    // From https://github.com/Gankro/collect-rs/tree/map.rs
    fn split(node: &mut Box<Self>) {
        if node.right.as_ref().map_or(false,
          |x| x.right.as_ref().map_or(false, |y| y.level == node.level)) {
            let mut save = node.right.take().unwrap();
            swap(&mut node.right, &mut save.left); // save.left now None
            save.level += 1;
            swap(node, &mut save);
            node.left = Some(save);
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

            Node::skew(node);
            Node::split(node);
            old_value
        },
    }
}

fn do_remove<K, V>(link: &mut Link<K, V>, path: Vec<*mut Box<Node<K, V>>>) -> Option<(K, V)> {
    let key_value = match *link {
        None => return None,
        Some(ref mut node) => {
            let mut path: Vec<*mut Box<Node<K, V>>> = vec![];

            let replacement = if node.left.is_some() {
                Right::extremum_f(&mut node.left, |node| path.push(node as *const _ as *mut _)).take()
            } else if node.right.is_some() {
                Left::extremum_f(&mut node.right, |node| path.push(node as *const _ as *mut _)).take()
            } else {
                None
            };

            replacement.map(|replacement| {
                for node in path.into_iter().rev() { rebalance(unsafe { &mut *node }); }
                let replacement = *replacement;
                let key_value = (replace(&mut node.key, replacement.key),
                                 replace(&mut node.value, replacement.value));
                rebalance(node);
                key_value
            })
        }
    }.or_else(|| link.take().map(|node| { let node = *node; (node.key, node.value) }));

    for node in path.into_iter().rev() { rebalance(unsafe { &mut *node }); }
    key_value
}

pub fn remove<K, V, C, Q: ?Sized>(link: &mut Link<K, V>, cmp: &C, key: &Q)
    -> Option<(K, V)> where C: Compare<Q, K> {

    let mut path: Vec<*mut Box<Node<K, V>>> = vec![];
    let link = get_f(link, cmp, key, |node| path.push(node as *const _ as *mut _));
    do_remove(link, path)
}

fn rebalance<K, V>(save: &mut Box<Node<K, V>>) {
    let left_level = save.left.as_ref().map_or(0, |node| node.level);
    let right_level = save.right.as_ref().map_or(0, |node| node.level);

    // re-balance, if necessary
    if left_level < save.level - 1 || right_level < save.level - 1 {
        save.level -= 1;

        if right_level > save.level {
            let save_level = save.level;
            if let Some(ref mut x) = save.right { x.level = save_level; }
        }

        Node::skew(save);

        if let Some(ref mut right) = save.right {
            Node::skew(right);
            if let Some(ref mut x) = right.right { Node::skew(x); };
        }

        Node::split(save);
        if let Some(ref mut x) = save.right { Node::split(x); }
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

    get_f(link, cmp, key, |_| ())
}

fn get_f<'a, L, C, Q: ?Sized, F>(link: L, cmp: &C, key: &Q, mut f: F) -> L
    where L: LinkRef<'a>, C: Compare<Q, L::K>, F: FnMut(&'a Box<Node<L::K, L::V>>) {

    link.with(|mut link| loop {
        match *link {
            None => return link,
            Some(ref node) => {
                match cmp.compare(key, &node.key) {
                    Equal => return link,
                    Less => link = &node.left,
                    Greater => link = &node.right,
                }

                f(node);
            }
        }
    })
}

pub trait Dir: Sized {
    type Opposite: Dir<Opposite=Self>;

    fn left() -> bool;

    fn forward<K, V>(node: &Node<K, V>) -> &Link<K, V>;
    fn forward_mut<K, V>(node: &mut Node<K, V>) -> &mut Link<K, V>;

    fn extremum<'a, L>(link: L) -> L where L: LinkRef<'a> { Self::extremum_f(link, |_| ()) }

    fn extremum_f<'a, L, F>(link: L, mut f: F) -> L
        where L: LinkRef<'a>, F: FnMut(&'a Box<Node<L::K, L::V>>) {

        link.with(|mut link| {
            while let Some(ref node) = *link {
                let child = Self::forward(node);
                if child.is_none() { break; }
                link = child;
                f(node);
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

pub fn entry<'a, K, V, C>(link: &'a mut Link<K, V>, cmp: &C, key: K, len: &'a mut usize)
    -> Entry<'a, K, V> where C: Compare<K> {

    let mut path = vec![];
    let link = get_f(link, cmp, &key, |node| path.push(node as *const _ as *mut _));

    if link.is_some() {
        Entry::Occupied(OccupiedEntry { path: path, link: link, len: len })
    } else {
        Entry::Vacant(VacantEntry { path: path, link: link, len: len, key: key })
    }
}

/// An occupied entry.
///
/// See [`Map::entry`](struct.Map.html#method.entry) for an example.
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    path: Vec<*mut Box<Node<K, V>>>,
    link: &'a mut Link<K, V>,
    len: &'a mut usize,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Returns a reference to the entry's key.
    pub fn key(&self) -> &K { &self.link.as_ref().unwrap().key }

    /// Returns a reference to the entry's value.
    pub fn get(&self) -> &V { &self.link.as_ref().unwrap().value }

    /// Returns a mutable reference to the entry's value.
    pub fn get_mut(&mut self) -> &mut V { &mut self.link.as_mut().unwrap().value }

    /// Returns a mutable reference to the entry's value with the same lifetime as the map.
    pub fn into_mut(self) -> &'a mut V { &mut self.link.as_mut().unwrap().value }

    /// Replaces the entry's value with the given value, returning the old one.
    pub fn insert(&mut self, value: V) -> V { replace(self.get_mut(), value) }

    /// Removes the entry from the map and returns its key and value.
    pub fn remove(self) -> (K, V) {
        *self.len -= 1;
        do_remove(self.link, self.path).unwrap()
    }
}

unsafe impl<'a, K, V> Send for OccupiedEntry<'a, K, V> where K: Send, V: Send {}
unsafe impl<'a, K, V> Sync for OccupiedEntry<'a, K, V> where K: Sync, V: Sync {}

/// A vacant entry.
///
/// See [`Map::entry`](struct.Map.html#method.entry) for an example.
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    path: Vec<*mut Box<Node<K, V>>>,
    link: &'a mut Link<K, V>,
    len: &'a mut usize,
    key: K,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    /// Inserts the entry into the map with its key and the given value, returning a mutable
    /// reference to the value with the same lifetime as the map.
    #[allow(trivial_casts)]
    pub fn insert(self, value: V) -> &'a mut V {
        *self.len += 1;

        *self.link = Some(Box::new(Node::new(self.key, value)));
        let value = &mut self.link.as_mut().unwrap().value;

        for node in self.path.into_iter().rev() {
            unsafe {
                Node::skew(&mut *node);
                Node::split(&mut *node);
            }
        }

        value
    }
}

unsafe impl<'a, K, V> Send for VacantEntry<'a, K, V> where K: Send, V: Send {}
unsafe impl<'a, K, V> Sync for VacantEntry<'a, K, V> where K: Sync, V: Sync {}
