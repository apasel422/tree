//! An ordered map and set based on a binary search tree.

#![feature(box_patterns, box_syntax)]
#![feature(collections)]

extern crate compare;

pub use map::Map;
pub use set::Set;

pub mod map;
pub mod set;

mod node;

#[cfg(feature = "ordered_iter")]
mod ordered_iter;

#[cfg(feature = "quickcheck")]
mod quickcheck;
