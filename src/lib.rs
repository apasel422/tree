//! A collection based on a binary search tree.

#![feature(box_patterns, box_syntax)]
#![feature(collections)]
#![feature(core)]

extern crate compare;

pub use map::Map;
pub use set::Set;

pub mod map;
pub mod set;

#[cfg(feature = "ordered_iter")]
mod ordered_iter;

#[cfg(feature = "quickcheck")]
mod quickcheck;
