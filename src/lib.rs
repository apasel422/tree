//! A collection based on a binary search tree.

#![feature(box_patterns, box_syntax)]
#![feature(collections)]
#![feature(core)]

extern crate compare;

pub use map::TreeMap;
pub use set::TreeSet;

pub mod map;
pub mod set;
