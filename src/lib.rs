//! An ordered map and set based on a binary search tree.

#![cfg_attr(feature = "range", feature(collections))]

extern crate compare;

pub use map::Map;
pub use set::Set;

#[forbid(missing_docs)]
pub mod map;
#[forbid(missing_docs)]
pub mod set;

mod node;

#[cfg(feature = "ordered_iter")]
mod ordered_iter;

#[cfg(feature = "quickcheck")]
mod quickcheck;

/// Data that can augment a binary search tree.
pub trait Augment: Sized {
    /// Returns a new augment for a leaf node.
    fn new() -> Self;

    /// Updates the augment in bottom-up fashion using the augments of its children.
    fn bottom_up(&mut self, left: Option<&Self>, right: Option<&Self>);
}

impl Augment for () {
    fn new() {}
    fn bottom_up(&mut self, _: Option<&Self>, _: Option<&Self>) {}
}

/// An augment that allows a map or set to support efficient access by in-order index.
///
/// See [`Map::select`](map/struct.Map.html#method.select) or [`Set::select`]
/// (set/struct.Set.html#method.select) for an example.
#[derive(Clone)]
pub struct OrderStat(usize);

impl Augment for OrderStat {
    fn new() -> Self { OrderStat(1) }

    fn bottom_up(&mut self, left: Option<&Self>, right: Option<&Self>) {
        self.0 = left.map_or(0, |left| left.0) + right.map_or(0, |right| right.0) + 1;
    }
}
