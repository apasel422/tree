//! An ordered map and set based on a binary search tree.

#![feature(collections)]

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
