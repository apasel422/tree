mod iter;

#[cfg(test)]
mod test;

use compare::Compare;
use self::build::{Build, PathBuilder};
use std::cmp::Ordering::*;
use std::default::Default;
use std::mem::{self, replace, swap};
use super::map::Entry;

pub use self::iter::Iter;

pub fn as_node_ref<K, V>(link: &Link<K, V>) -> Option<&Node<K, V>> {
    link.as_ref().map(|node| &**node)
}

pub type Link<K, V> = Option<Box<Node<K, V>>>;

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

    fn rebalance(node: &mut Box<Node<K, V>>) {
        let left_level = node.left.as_ref().map_or(0, |node| node.level);
        let right_level = node.right.as_ref().map_or(0, |node| node.level);

        // re-balance, if necessary
        if left_level < node.level - 1 || right_level < node.level - 1 {
            node.level -= 1;

            if right_level > node.level {
                let node_level = node.level;
                if let Some(ref mut x) = node.right { x.level = node_level; }
            }

            Node::skew(node);

            if let Some(ref mut right) = node.right {
                Node::skew(right);
                if let Some(ref mut x) = right.right { Node::skew(x); };
            }

            Node::split(node);
            if let Some(ref mut x) = node.right { Node::split(x); }
        }
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

pub mod build {
    use std::default::Default;
    use std::marker::PhantomData;
    use super::{Link, Node, Path};

    pub struct Closed<'a, K: 'a, V: 'a> {
        link: *const Link<K, V>,
        _marker: PhantomData<&'a Link<K, V>>,
    }

    pub trait Build<'a>: Sized + Default {
        type Key: 'a;
        type Value: 'a;
        type Node: ::std::ops::Deref<Target = Box<Node<Self::Key, Self::Value>>>;
        type Link;
        type Output;

        fn closed(link: &Self::Link) -> Closed<'a, Self::Key, Self::Value>;

        fn into_option(link: Self::Link) -> Option<Self::Node>;

        fn left(&mut self, node: Self::Node) -> Self::Link;

        fn right(&mut self, node: Self::Node) -> Self::Link;

        fn build_open(self, link: Self::Link) -> Self::Output;

        fn build_closed(self, link: Closed<'a, Self::Key, Self::Value>) -> Self::Output;
    }

    pub struct Get<'a, K: 'a, V: 'a>(PhantomData<fn(&'a Link<K, V>)>);

    impl<'a, K, V> Default for Get<'a, K, V> {
        fn default() -> Self { Get(PhantomData) }
    }

    impl<'a, K: 'a, V: 'a> Build<'a> for Get<'a, K, V> {
        type Key = K;
        type Value = V;
        type Node = &'a Box<Node<K, V>>;
        type Link = &'a Link<K, V>;
        type Output = Option<(&'a K, &'a V)>;

        fn closed(link: & &'a Link<K, V>) -> Closed<'a, K, V> {
            Closed { link: *link, _marker: PhantomData }
        }

        fn into_option(link: &'a Link<K, V>) -> Option<&'a Box<Node<K, V>>> { link.as_ref() }

        fn left(&mut self, node: &'a Box<Node<K, V>>) -> &'a Link<K, V> { &node.left }

        fn right(&mut self, node: &'a Box<Node<K, V>>) -> &'a Link<K, V> { &node.right }

        fn build_open(self, link: &'a Link<K, V>) -> Option<(&'a K, &'a V)> {
            link.as_ref().map(|node| (&node.key, &node.value))
        }

        fn build_closed(self, link: Closed<'a, K, V>) -> Option<(&'a K, &'a V)> {
            self.build_open(unsafe { &*link.link })
        }
    }

    pub struct GetMut<'a, K: 'a, V: 'a>(PhantomData<&'a mut Link<K, V>>);

    impl<'a, K, V> Default for GetMut<'a, K, V> {
        fn default() -> Self { GetMut(PhantomData) }
    }

    impl<'a, K: 'a, V: 'a> Build<'a> for GetMut<'a, K, V> {
        type Key = K;
        type Value = V;
        type Node = &'a mut Box<Node<K, V>>;
        type Link = &'a mut Link<K, V>;
        type Output = Option<(&'a K, &'a mut V)>;

        fn closed(link: & &'a mut Link<K, V>) -> Closed<'a, K, V> {
            Closed { link: *link, _marker: PhantomData }
        }

        fn into_option(link: &'a mut Link<K, V>) -> Option<&'a mut Box<Node<K, V>>> {
            link.as_mut()
        }

        fn left(&mut self, node: &'a mut Box<Node<K, V>>) -> &'a mut Link<K, V> { &mut node.left }

        fn right(&mut self, node: &'a mut Box<Node<K, V>>) -> &'a mut Link<K, V> { &mut node.right }

        fn build_open(self, link: &'a mut Link<K, V>) -> Option<(&'a K, &'a mut V)> {
            link.as_mut().map(|node| { let node = &mut **node; (&node.key, &mut node.value) })
        }

        fn build_closed(self, link: Closed<'a, K, V>) -> Option<(&'a K, &'a mut V)> {
            self.build_open(unsafe { &mut *(link.link as *mut _) })
        }
    }

    pub struct PathBuilder<'a, K: 'a, V: 'a> {
        path: Vec<*mut Box<Node<K, V>>>,
        _marker: PhantomData<&'a mut Box<Node<K, V>>>,
    }

    impl<'a, K, V> Default for PathBuilder<'a, K, V> {
        fn default() -> Self { PathBuilder { path: vec![], _marker: PhantomData } }
    }

    impl<'a, K: 'a, V: 'a> Build<'a> for PathBuilder<'a, K, V> {
        type Key = K;
        type Value = V;
        type Node = &'a mut Box<Node<K, V>>;
        type Link = &'a mut Link<K, V>;
        type Output = Path<'a, K, V>;

        fn closed(link: & &'a mut Link<K, V>) -> Closed<'a, K, V> {
            Closed { link: *link, _marker: PhantomData }
        }

        fn into_option(link: &'a mut Link<K, V>) -> Option<&'a mut Box<Node<K, V>>> {
            link.as_mut()
        }

        fn left(&mut self, node: &'a mut Box<Node<K, V>>) -> &'a mut Link<K, V> {
            self.path.push(node);
            &mut node.left
        }

        fn right(&mut self, node: &'a mut Box<Node<K, V>>) -> &'a mut Link<K, V> {
            self.path.push(node);
            &mut node.right
        }

        fn build_open(self, link: &'a mut Link<K, V>) -> Path<'a, K, V> {
            Path { path: self.path, link: link }
        }

        fn build_closed(self, link: Closed<'a, K, V>) -> Path<'a, K, V> {
            Path {
                path: self.path.into_iter().take_while(|l| *l as *const _ != link.link).collect(),
                link: unsafe { &mut *(link.link as *mut _) },
            }
        }
    }
}

pub fn find<'a, B, C: ?Sized, Q: ?Sized>(mut link: B::Link, mut build: B, cmp: &C, key: &Q)
    -> B::Output where B: Build<'a>, C: Compare<Q, B::Key> {

    loop {
        let closed = B::closed(&link);

        link = match B::into_option(link) {
            None => return build.build_closed(closed),
            Some(node) => match cmp.compare(key, &node.key) {
                Less => build.left(node),
                Equal => return build.build_closed(closed),
                Greater => build.right(node),
            },
        };
    }
}

pub trait Extreme: Sized {
    type Opposite: Extreme<Opposite = Self>;

    fn min() -> bool;
    fn has_forward<K, V>(node: &Node<K, V>) -> bool;
    fn forward<'a, B>(node: B::Node, build: &mut B) -> B::Link where B: Build<'a>;

    fn extreme<'a, B>(mut link: B::Link, mut build: B) -> B::Output where B: Build<'a> {
        loop {
            let closed = B::closed(&link);

            link = match B::into_option(link) {
                None => return build.build_closed(closed),
                Some(node) =>
                    if Self::has_forward(&*node) {
                        Self::forward(node, &mut build)
                    } else {
                        return build.build_closed(closed);
                    },
            };
        }
    }

    fn closest<'a, B, C: ?Sized, Q: ?Sized>(mut link: B::Link, mut build: B, cmp: &C, key: &Q,
                                            inclusive: bool)
        -> B::Output where B: Build<'a>, C: Compare<Q, B::Key> {

        let mut save = None;

        loop {
            let closed = B::closed(&link);

            link = match B::into_option(link) {
                None => return build.build_closed(save.unwrap_or(closed)),
                Some(node) => match cmp.compare(key, &node.key) {
                    Equal => return
                        if inclusive {
                            build.build_closed(closed)
                        } else if Self::has_forward(&*node) {
                            let forward = Self::forward(node, &mut build);
                            Self::Opposite::extreme(forward, build)
                        } else {
                            match save {
                                None => {
                                    let forward = Self::forward(node, &mut build);
                                    build.build_open(forward)
                                }
                                Some(save) => build.build_closed(save),
                            }
                        },
                    order =>
                        if Self::min() == (order == Less) {
                            Self::forward(node, &mut build)
                        } else {
                            save = Some(closed);
                            Self::Opposite::forward(node, &mut build)
                        },
                },
            }
        }
    }
}

#[allow(dead_code)] // FIXME: rust-lang/rust#23808
pub enum Max {}

impl Extreme for Max {
    type Opposite = Min;
    fn min() -> bool { false }
    fn has_forward<K, V>(node: &Node<K, V>) -> bool { node.right.is_some() }
    fn forward<'a, B>(node: B::Node, build: &mut B) -> B::Link where B: Build<'a> {
        build.right(node)
    }
}

#[allow(dead_code)] // FIXME: rust-lang/rust#23808
pub enum Min {}

impl Extreme for Min {
    type Opposite = Max;
    fn min() -> bool { true }
    fn has_forward<K, V>(node: &Node<K, V>) -> bool { node.left.is_some() }
    fn forward<'a, B>(node: B::Node, build: &mut B) -> B::Link where B: Build<'a> {
        build.left(node)
    }
}

pub struct Path<'a, K: 'a, V: 'a> {
    path: Vec<*mut Box<Node<K, V>>>,
    link: &'a mut Link<K, V>,
}

impl<'a, K, V> Path<'a, K, V> {
    pub fn into_entry(self, len: &'a mut usize, key: K) -> Entry<'a, K, V> {
        if self.link.is_some() {
            Entry::Occupied(OccupiedEntry { path: self, len: len })
        } else {
            Entry::Vacant(VacantEntry { path: self, len: len, key: key })
        }
    }

    pub fn into_occupied_entry(self, len: &'a mut usize) -> Option<OccupiedEntry<'a, K, V>> {
        if self.link.is_some() {
            Some(OccupiedEntry { path: self, len: len })
        } else {
            None
        }
    }

    fn remove_(self) -> Option<(K, V)> {
        let key_value = match *self.link {
            None => return None,
            Some(ref mut node) => {
                let replacement = if node.left.is_some() {
                    Max::extreme(&mut node.left, PathBuilder::default()).remove_()
                } else if node.right.is_some() {
                    Min::extreme(&mut node.right, PathBuilder::default()).remove_()
                } else {
                    None
                };

                replacement.map(|replacement| {
                    let key_value = (replace(&mut node.key, replacement.0),
                                     replace(&mut node.value, replacement.1));
                    Node::rebalance(node);
                    key_value
                })
            }
        }.or_else(|| self.link.take().map(|node| { let node = *node; (node.key, node.value) }));

        for node in self.path.into_iter().rev() { Node::rebalance(unsafe { &mut *node }); }
        key_value
    }

    pub fn remove(self, len: &mut usize) -> Option<(K, V)> {
        let key_value = self.remove_();
        if key_value.is_some() { *len -= 1; }
        key_value
    }
}

unsafe impl<'a, K, V> Send for Path<'a, K, V> where K: Send, V: Send {}
unsafe impl<'a, K, V> Sync for Path<'a, K, V> where K: Sync, V: Sync {}

/// An occupied entry.
///
/// See [`Map::entry`](struct.Map.html#method.entry) for an example.
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    path: Path<'a, K, V>,
    len: &'a mut usize,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Returns a reference to the entry's key.
    pub fn key(&self) -> &K { &self.path.link.as_ref().unwrap().key }

    /// Returns a reference to the entry's value.
    pub fn get(&self) -> &V { &self.path.link.as_ref().unwrap().value }

    /// Returns a mutable reference to the entry's value.
    pub fn get_mut(&mut self) -> &mut V { &mut self.path.link.as_mut().unwrap().value }

    /// Returns a mutable reference to the entry's value with the same lifetime as the map.
    pub fn into_mut(self) -> &'a mut V { &mut self.path.link.as_mut().unwrap().value }

    /// Replaces the entry's value with the given value, returning the old one.
    pub fn insert(&mut self, value: V) -> V { replace(self.get_mut(), value) }

    /// Removes the entry from the map and returns its key and value.
    pub fn remove(self) -> (K, V) {
        self.path.remove(self.len).unwrap()
    }
}

/// A vacant entry.
///
/// See [`Map::entry`](struct.Map.html#method.entry) for an example.
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    path: Path<'a, K, V>,
    len: &'a mut usize,
    key: K,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    /// Inserts the entry into the map with its key and the given value, returning a mutable
    /// reference to the value with the same lifetime as the map.
    pub fn insert(self, value: V) -> &'a mut V {
        *self.len += 1;

        *self.path.link = Some(Box::new(Node::new(self.key, value)));
        let value = &mut self.path.link.as_mut().unwrap().value;

        for node in self.path.path.into_iter().rev() {
            unsafe {
                Node::skew(&mut *node);
                Node::split(&mut *node);
            }
        }

        value
    }
}
