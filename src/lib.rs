//! An ordered map and set based on a binary search tree.

#![cfg_attr(feature = "range", feature(collections_bound))]

extern crate compare;

pub use map::Map;
pub use set::Set;

pub use balance::{Aa, Balance, Node};

#[forbid(missing_docs)]
pub mod map;
#[forbid(missing_docs)]
pub mod set;

mod balance;
mod node;

#[cfg(feature = "ordered_iter")]
mod ordered_iter;

#[cfg(feature = "quickcheck")]
mod quickcheck;
