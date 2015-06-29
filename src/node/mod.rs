mod iter;

#[cfg(test)]
mod test;

use compare::Compare;
use self::build::{Build, PathBuilder};
use std::cmp::Ordering::*;
use std::mem::{self, replace, swap};
use super::{Aa, Balance};
use super::map::Entry;

pub use self::iter::{Iter, MarkedNode, MutMarkedNode};
#[cfg(feature = "range")] pub use self::iter::Range;

pub type Link<K, V, B> = Option<Box<Node<K, V, B>>>;

#[derive(Clone)]
pub struct Node<K, V, B> {
    left: Link<K, V, B>,
    right: Link<K, V, B>,
    balance: B,
    key: K,
    value: V,
}

impl<K, V, B> super::Node for Box<Node<K, V, B>> where B: Balance {
    type Balance = B;

    fn balance(&self) -> &Self::Balance { &self.balance }
    fn balance_mut(&mut self) -> &mut Self::Balance { &mut self.balance }

    fn left(&self) -> Option<&Self> { self.left.as_ref() }
    fn left_mut(&mut self) -> Option<&mut Self> { self.left.as_mut() }

    fn right(&self) -> Option<&Self> { self.right.as_ref() }
    fn right_mut(&mut self) -> Option<&mut Self> { self.right.as_mut() }

    fn rotate_right(&mut self) {
        if let Some(mut save) = self.left.take() {
            swap(&mut self.left, &mut save.right); // save.right now None
            swap(self, &mut save);
            self.right = Some(save);
        }
    }

    fn rotate_left(&mut self) {
        if let Some(mut save) = self.right.take() {
            swap(&mut self.right, &mut save.left); // save.left now None
            swap(self, &mut save);
            self.left = Some(save);
        }
    }
}

impl<K, V, B> Node<K, V, B> where B: Balance {
    fn new(key: K, value: V) -> Self {
        Node { left: None, right: None, balance: B::default(), key: key, value: value }
    }
}

pub fn insert<K, V, C, B>(link: &mut Link<K, V, B>, cmp: &C, key: K, value: V) -> Option<V>
    where C: Compare<K>, B: Balance {

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

            B::rebalance_insert(node);
            old_value
        },
    }
}

pub mod build {
    use std::marker::PhantomData;
    use super::{Link, Node, Path};

    pub struct Closed<'a, K: 'a, V: 'a, B: 'a> {
        link: *const Link<K, V, B>,
        _marker: PhantomData<&'a Link<K, V, B>>,
    }

    pub trait Build<'a>: Sized + Default {
        type Key: 'a;
        type Value: 'a;
        type Balance: 'a;
        type Node: ::std::ops::Deref<Target = Box<Node<Self::Key, Self::Value, Self::Balance>>>;
        type Link;
        type Output;

        fn closed(link: &Self::Link) -> Closed<'a, Self::Key, Self::Value, Self::Balance>;

        fn into_option(link: Self::Link) -> Option<Self::Node>;

        fn left(&mut self, node: Self::Node) -> Self::Link;

        fn right(&mut self, node: Self::Node) -> Self::Link;

        fn build_open(self, link: Self::Link) -> Self::Output;

        fn build_closed(self, link: Closed<'a, Self::Key, Self::Value, Self::Balance>)
            -> Self::Output;
    }

    pub struct Get<'a, K: 'a, V: 'a, B: 'a>(PhantomData<fn(&'a Link<K, V, B>)>);

    impl<'a, K, V, B> Default for Get<'a, K, V, B> {
        fn default() -> Self { Get(PhantomData) }
    }

    impl<'a, K: 'a, V: 'a, B: 'a> Build<'a> for Get<'a, K, V, B> {
        type Key = K;
        type Value = V;
        type Balance = B;
        type Node = &'a Box<Node<K, V, B>>;
        type Link = &'a Link<K, V, B>;
        type Output = Option<(&'a K, &'a V)>;

        fn closed(link: &Self::Link) -> Closed<'a, K, V, B> {
            Closed { link: *link, _marker: PhantomData }
        }

        fn into_option(link: Self::Link) -> Option<Self::Node> { link.as_ref() }

        fn left(&mut self, node: Self::Node) -> Self::Link { &node.left }

        fn right(&mut self, node: Self::Node) -> Self::Link { &node.right }

        fn build_open(self, link: Self::Link) -> Self::Output {
            link.as_ref().map(|node| (&node.key, &node.value))
        }

        fn build_closed(self, link: Closed<'a, Self::Key, Self::Value, Self::Balance>)
            -> Self::Output {

            self.build_open(unsafe { &*link.link })
        }
    }

    pub struct GetMut<'a, K: 'a, V: 'a, B: 'a>(PhantomData<&'a mut Link<K, V, B>>);

    impl<'a, K, V, B> Default for GetMut<'a, K, V, B> {
        fn default() -> Self { GetMut(PhantomData) }
    }

    impl<'a, K: 'a, V: 'a, B: 'a> Build<'a> for GetMut<'a, K, V, B> {
        type Key = K;
        type Value = V;
        type Balance = B;
        type Node = &'a mut Box<Node<K, V, B>>;
        type Link = &'a mut Link<K, V, B>;
        type Output = Option<(&'a K, &'a mut V)>;

        fn closed(link: &Self::Link) -> Closed<'a, K, V, B> {
            Closed { link: *link, _marker: PhantomData }
        }

        fn into_option(link: Self::Link) -> Option<Self::Node> {
            link.as_mut()
        }

        fn left(&mut self, node: Self::Node) -> Self::Link { &mut node.left }

        fn right(&mut self, node: Self::Node) -> Self::Link { &mut node.right }

        fn build_open(self, link: Self::Link) -> Self::Output {
            link.as_mut().map(|node| { let node = &mut **node; (&node.key, &mut node.value) })
        }

        fn build_closed(self, link: Closed<'a, Self::Key, Self::Value, Self::Balance>)
            -> Self::Output {

            self.build_open(unsafe { &mut *(link.link as *mut _) })
        }
    }

    pub struct PathBuilder<'a, K: 'a, V: 'a, B: 'a> {
        path: Vec<*mut Box<Node<K, V, B>>>,
        _marker: PhantomData<&'a mut Box<Node<K, V, B>>>,
    }

    impl<'a, K, V, B> Default for PathBuilder<'a, K, V, B> {
        fn default() -> Self { PathBuilder { path: vec![], _marker: PhantomData } }
    }

    impl<'a, K: 'a, V: 'a, B: 'a> Build<'a> for PathBuilder<'a, K, V, B> {
        type Key = K;
        type Value = V;
        type Balance = B;
        type Node = &'a mut Box<Node<K, V, B>>;
        type Link = &'a mut Link<K, V, B>;
        type Output = Path<'a, K, V, B>;

        fn closed(link: &Self::Link) -> Closed<'a, K, V, B> {
            Closed { link: *link, _marker: PhantomData }
        }

        fn into_option(link: Self::Link) -> Option<Self::Node> {
            link.as_mut()
        }

        fn left(&mut self, node: Self::Node) -> Self::Link {
            self.path.push(node);
            &mut node.left
        }

        fn right(&mut self, node: Self::Node) -> Self::Link {
            self.path.push(node);
            &mut node.right
        }

        fn build_open(self, link: Self::Link) -> Self::Output {
            Path { path: self.path, link: link }
        }

        fn build_closed(self, link: Closed<'a, K, V, B>) -> Self::Output {
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
    fn has_forward<K, V, B>(node: &Node<K, V, B>) -> bool;
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
    fn has_forward<K, V, B>(node: &Node<K, V, B>) -> bool { node.right.is_some() }
    fn forward<'a, B>(node: B::Node, build: &mut B) -> B::Link where B: Build<'a> {
        build.right(node)
    }
}

#[allow(dead_code)] // FIXME: rust-lang/rust#23808
pub enum Min {}

impl Extreme for Min {
    type Opposite = Max;
    fn min() -> bool { true }
    fn has_forward<K, V, B>(node: &Node<K, V, B>) -> bool { node.left.is_some() }
    fn forward<'a, B>(node: B::Node, build: &mut B) -> B::Link where B: Build<'a> {
        build.left(node)
    }
}

pub struct Path<'a, K: 'a, V: 'a, B: 'a> {
    path: Vec<*mut Box<Node<K, V, B>>>,
    link: &'a mut Link<K, V, B>,
}

impl<'a, K, V, B> Path<'a, K, V, B> where B: Balance {
    pub fn into_entry(self, len: &'a mut usize, key: K) -> Entry<'a, K, V, B> {
        if self.link.is_some() {
            Entry::Occupied(OccupiedEntry { path: self, len: len })
        } else {
            Entry::Vacant(VacantEntry { path: self, len: len, key: key })
        }
    }

    pub fn into_occupied_entry(self, len: &'a mut usize) -> Option<OccupiedEntry<'a, K, V, B>> {
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
                    B::rebalance_remove(node);
                    key_value
                })
            }
        }.or_else(|| self.link.take().map(|node| { let node = *node; (node.key, node.value) }));

        for node in self.path.into_iter().rev() { B::rebalance_remove(unsafe { &mut *node }); }
        key_value
    }

    pub fn remove(self, len: &mut usize) -> Option<(K, V)> {
        let key_value = self.remove_();
        if key_value.is_some() { *len -= 1; }
        key_value
    }
}

unsafe impl<'a, K, V, B> Send for Path<'a, K, V, B> where K: Send, V: Send, B: Send {}
unsafe impl<'a, K, V, B> Sync for Path<'a, K, V, B> where K: Sync, V: Sync, B: Sync {}

/// An occupied entry.
///
/// See [`Map::entry`](struct.Map.html#method.entry) for an example.
pub struct OccupiedEntry<'a, K: 'a, V: 'a, B: 'a = Aa> where B: Balance {
    path: Path<'a, K, V, B>,
    len: &'a mut usize,
}

impl<'a, K, V, B> OccupiedEntry<'a, K, V, B> where B: Balance {
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
pub struct VacantEntry<'a, K: 'a, V: 'a, B: 'a = Aa> where B: Balance {
    path: Path<'a, K, V, B>,
    len: &'a mut usize,
    key: K,
}

impl<'a, K, V, B> VacantEntry<'a, K, V, B> where B: Balance {
    /// Inserts the entry into the map with its key and the given value, returning a mutable
    /// reference to the value with the same lifetime as the map.
    pub fn insert(self, value: V) -> &'a mut V {
        *self.len += 1;

        *self.path.link = Some(Box::new(Node::new(self.key, value)));
        let value = &mut self.path.link.as_mut().unwrap().value;

        for node in self.path.path.into_iter().rev() {
            B::rebalance_insert(unsafe { &mut *node });
        }

        value
    }
}
