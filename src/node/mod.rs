mod iter;

#[cfg(test)]
mod test;

use compare::Compare;
use self::build::{Build, LinkRef, NodeRef, Open, PathBuilder};
use std::cmp::Ordering::*;
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
    use std::marker::PhantomData;
    use std::mem::transmute;
    use super::{Link, Node, Path};

    #[derive(Copy)]
    pub struct Open;
    #[derive(Copy)]
    pub struct Closed;

    pub struct LinkRef<'a, K: 'a, V: 'a, S> {
        link: &'a Link<K, V>,
        state: S,
    }

    impl<'a, K, V, S> LinkRef<'a, K, V, S> where S: Copy {
        pub fn into_option(self) -> Option<NodeRef<'a, K, V, S>> {
            self.link.as_ref().map(|node| NodeRef { node: node, _state: self.state })
        }
    }

    impl<'a, K, V> LinkRef<'a, K, V, Open> {
        pub fn closed(&self) -> LinkRef<'a, K, V, Closed> {
            LinkRef { link: self.link, state: Closed }
        }
    }

    pub struct NodeRef<'a, K: 'a, V: 'a, S> {
        node: &'a Box<Node<K, V>>,
        _state: S,
    }

    impl<'a, K, V, S> ::std::ops::Deref for NodeRef<'a, K, V, S> {
        type Target = Box<Node<K, V>>;
        fn deref(&self) -> &Box<Node<K, V>> { self.node }
    }

    pub trait Build<'a, K, V>: Sized {
        type Link;
        type Output;
        fn new(link: Self::Link) -> (LinkRef<'a, K, V, Open>, Self);
        fn left(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open>;
        fn right(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open>;
        fn build_open(self, link: LinkRef<'a, K, V, Open>) -> Self::Output;
        fn build_closed(self, link: LinkRef<'a, K, V, Closed>) -> Self::Output;
    }

    pub struct Get;

    impl<'a, K, V> Build<'a, K, V> for Get {
        type Link = &'a Link<K, V>;
        type Output = Option<(&'a K, &'a V)>;

        fn new(link: &'a Link<K, V>) -> (LinkRef<'a, K, V, Open>, Self) {
            (LinkRef { link: link, state: Open }, Get)
        }

        fn left(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open> {
            LinkRef { link: &node.node.left, state: Open }
        }

        fn right(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open> {
            LinkRef { link: &node.node.right, state: Open }
        }

        fn build_open(self, link: LinkRef<'a, K, V, Open>) -> Option<(&'a K, &'a V)> {
            link.link.as_ref().map(|node| (&node.key, &node.value))
        }

        fn build_closed(self, link: LinkRef<'a, K, V, Closed>) -> Option<(&'a K, &'a V)> {
            link.link.as_ref().map(|node| (&node.key, &node.value))
        }
    }

    pub struct GetMut;

    impl<'a, K, V> Build<'a, K, V> for GetMut {
        type Link = &'a mut Link<K, V>;
        type Output = Option<(&'a K, &'a mut V)>;

        fn new(link: &'a mut Link<K, V>) -> (LinkRef<'a, K, V, Open>, Self) {
            (LinkRef { link: link, state: Open }, GetMut)
        }

        fn left(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open> {
            Build::left(&mut Get, node)
        }

        fn right(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open> {
            Build::right(&mut Get, node)
        }

        fn build_open(self, link: LinkRef<'a, K, V, Open>) -> Option<(&'a K, &'a mut V)> {
            let key_value = Build::build_open(Get, link);
            unsafe { transmute(key_value) }
        }

        fn build_closed(self, link: LinkRef<'a, K, V, Closed>) -> Option<(&'a K, &'a mut V)> {
            let key_value = Build::build_closed(Get, link);
            unsafe { transmute(key_value) }
        }
    }

    pub struct PathBuilder<'a, K: 'a, V: 'a> {
        path: Vec<*mut Box<Node<K, V>>>,
        _marker: PhantomData<&'a mut Box<Node<K, V>>>,
    }

    impl<'a, K, V> Build<'a, K, V> for PathBuilder<'a, K, V> {
        type Link = &'a mut Link<K, V>;
        type Output = Path<'a, K, V>;

        fn new(link: &'a mut Link<K, V>) -> (LinkRef<'a, K, V, Open>, Self) {
            (LinkRef { link: link, state: Open }, PathBuilder { path: vec![], _marker: PhantomData })
        }

        fn left(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open> {
            self.path.push(node.node as *const _ as *mut _);
            Build::left(&mut Get, node)
        }

        fn right(&mut self, node: NodeRef<'a, K, V, Open>) -> LinkRef<'a, K, V, Open> {
            self.path.push(node.node as *const _ as *mut _);
            Build::right(&mut Get, node)
        }

        fn build_open(self, link: LinkRef<'a, K, V, Open>) -> Path<'a, K, V> {
            Path { path: self.path, link: unsafe { transmute(link.link) } }
        }

        fn build_closed(self, link: LinkRef<'a, K, V, Closed>) -> Path<'a, K, V> {
            Path {
                path: self.path.into_iter().take_while(|l| *l as *const _ != link.link as *const _)
                    .skip(1).collect(),
                link: unsafe { transmute(link.link) },
            }
        }
    }
}

pub trait Traverse<K>: Sized {
    fn traverse<'a, V, B>(self, (LinkRef<'a, K, V, Open>, B)) -> B::Output
        where B: Build<'a, K, V>;
}

pub struct Find<'q, Q: 'q + ?Sized, C> {
    pub key: &'q Q,
    pub cmp: C,
}

impl<'q, Q: ?Sized, C, K> Traverse<K> for Find<'q, Q, C> where C: Compare<Q, K> {
    fn traverse<'a, V, B>(self, (mut link, mut build): (LinkRef<'a, K, V, Open>, B)) -> B::Output
        where B: Build<'a, K, V> {

        loop {
            link = match link.into_option() {
                None => break,
                Some(node) => match self.cmp.compare(self.key, &node.key) {
                    Less => build.left(node),
                    Equal => break,
                    Greater => build.right(node),
                },
            };
        }

        build.build_open(link)
    }
}

pub trait Extreme: Sized {
    type Opposite: Extreme<Opposite = Self>;
    fn new() -> Self;
    fn min() -> bool;
    fn has_forward<K, V>(node: &Node<K, V>) -> bool;
    fn forward<'a, K, V, B>(node: NodeRef<'a, K, V, Open>, build: &mut B)
        -> LinkRef<'a, K, V, Open> where B: Build<'a, K, V>;
}

impl<E, K> Traverse<K> for E where E: Extreme {
    fn traverse<'a, V, B>(self, (mut link, mut build): (LinkRef<'a, K, V, Open>, B)) -> B::Output
        where B: Build<'a, K, V> {

        loop {
            link = match link.into_option() {
                None => break,
                Some(node) =>
                    if E::has_forward(&*node) {
                        E::forward(node, &mut build)
                    } else {
                        break;
                    },
            };
        }

        build.build_open(link)
    }
}

pub struct Max;

impl Extreme for Max {
    type Opposite = Min;
    fn new() -> Self { Max }
    fn min() -> bool { false }
    fn has_forward<K, V>(node: &Node<K, V>) -> bool { node.right.is_some() }
    fn forward<'a, K, V, B>(node: NodeRef<'a, K, V, Open>, build: &mut B)
        -> LinkRef<'a, K, V, Open> where B: Build<'a, K, V> { build.right(node) }
}

pub struct Min;

impl Extreme for Min {
    type Opposite = Max;
    fn new() -> Self { Min }
    fn min() -> bool { true }
    fn has_forward<K, V>(node: &Node<K, V>) -> bool { node.left.is_some() }
    fn forward<'a, K, V, B>(node: NodeRef<'a, K, V, Open>, build: &mut B)
        -> LinkRef<'a, K, V, Open> where B: Build<'a, K, V> { build.left(node) }
}

pub struct Neighbor<'q, Q: 'q + ?Sized, C, E> where E: Extreme {
    pub key: &'q Q,
    pub cmp: C,
    pub inc: bool,
    pub ext: E,
}

impl<'q, Q: ?Sized, C, E, K> Traverse<K> for Neighbor<'q, Q, C, E>
    where C: Compare<Q, K>, E: Extreme {

    fn traverse<'a, V, B>(self, (mut link, mut build): (LinkRef<'a, K, V, Open>, B)) -> B::Output
        where B: Build<'a, K, V> {

        let mut save = None;

        loop {
            let closed = link.closed();

            link = match link.into_option() {
                None => return build.build_closed(save.unwrap_or(closed)),
                Some(node) => match self.cmp.compare(self.key, &node.key) {
                    Equal => return
                        if self.inc {
                            build.build_closed(closed)
                        } else if E::has_forward(&*node) {
                            let forward = E::forward(node, &mut build);
                            E::Opposite::new().traverse((forward, build))
                        } else {
                            match save {
                                None => {
                                    let forward = E::forward(node, &mut build);
                                    build.build_open(forward)
                                }
                                Some(save) => build.build_closed(save),
                            }
                        },
                    order =>
                        if E::min() == (order == Less) {
                            E::forward(node, &mut build)
                        } else {
                            save = Some(closed);
                            E::Opposite::forward(node, &mut build)
                        },
                },
            }
        }
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

    pub fn remove(self) -> Option<(K, V)> {
        let key_value = match *self.link {
            None => return None,
            Some(ref mut node) => {
                let replacement = if node.left.is_some() {
                    Max.traverse(PathBuilder::new(&mut node.left)).remove()
                } else if node.right.is_some() {
                    Min.traverse(PathBuilder::new(&mut node.right)).remove()
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
        *self.len -= 1;
        self.path.remove().unwrap()
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
    #[allow(trivial_casts)]
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
