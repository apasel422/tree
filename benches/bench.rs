// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(test)]

extern crate rand;
extern crate test;
extern crate tree;

use rand::{Rng, weak_rng};
use test::{Bencher, black_box};
use tree::Map;

macro_rules! map_insert_rand_bench {
    ($name: ident, $n: expr) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let n: usize = $n;
            let mut map = Map::new();
            // setup
            let mut rng = weak_rng();

            for _ in 0..n {
                let i = rng.gen::<usize>() % n;
                map.insert(i, i);
            }

            // measure
            b.iter(|| {
                let k = rng.gen::<usize>() % n;
                map.insert(k, k);
                //map.remove(&k);
            });
            black_box(map);
        }
    )
}

macro_rules! map_insert_seq_bench {
    ($name: ident, $n: expr) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let mut map = Map::new();
            let n: usize = $n;
            // setup
            for i in 0..n {
                map.insert(i * 2, i * 2);
            }

            // measure
            let mut i = 1;
            b.iter(|| {
                map.insert(i, i);
                //map.remove(&i);
                i = (i + 2) % n;
            });
            black_box(map);
        }
    )
}

macro_rules! map_find_rand_bench {
    ($name: ident, $n: expr) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let mut map = Map::new();
            let n: usize = $n;

            // setup
            let mut rng = weak_rng();
            let mut keys: Vec<_> = (0..n).map(|_| rng.gen::<usize>() % n).collect();

            for &k in &keys {
                map.insert(k, k);
            }

            rng.shuffle(&mut keys);

            // measure
            let mut i = 0;
            b.iter(|| {
                let t = map.get(&keys[i]);
                i = (i + 1) % n;
                black_box(t);
            })
        }
    )
}

macro_rules! map_find_seq_bench {
    ($name: ident, $n: expr) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let mut map = Map::new();
            let n: usize = $n;

            // setup
            for i in 0..n {
                map.insert(i, i);
            }

            // measure
            let mut i = 0;
            b.iter(|| {
                let x = map.get(&i);
                i = (i + 1) % n;
                black_box(x);
            })
        }
    )
}

macro_rules! map_iter_bench {
    ($name: ident, $n: expr) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let mut map = Map::<u32, u32>::new();
            let n: usize = $n;
            let mut rng = weak_rng();

            for _ in 0..n {
                map.insert(rng.gen(), rng.gen());
            }

            b.iter(|| {
                for entry in map.iter() {
                    black_box(entry);
                }
            });
        }
    )
}

map_insert_rand_bench!{insert_rand_100,    100}
map_insert_rand_bench!{insert_rand_10_000, 10_000}

map_insert_seq_bench!{insert_seq_100,    100}
map_insert_seq_bench!{insert_seq_10_000, 10_000}

map_find_rand_bench!{find_rand_100,    100}
map_find_rand_bench!{find_rand_10_000, 10_000}

map_find_seq_bench!{find_seq_100,    100}
map_find_seq_bench!{find_seq_10_000, 10_000}

map_iter_bench!{iter_100,     100}
map_iter_bench!{iter_1000,    1000}
map_iter_bench!{iter_100_000, 100_000}
