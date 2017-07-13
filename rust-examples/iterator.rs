#![feature(intrinsics, lang_items, no_core, fundamental)]
#![no_core]

fn main() {
    for i in 0..10 {
        wasm::print_i32(i);
    }
}

pub mod ops {
    pub struct Range<Idx> {
        pub start: Idx,
        pub end: Idx,
    }

    #[lang = "add"]
    pub trait Add<RHS = Self> {
        type Output;
        fn add(self, rhs: RHS) -> Self::Output;
    }

    impl Add for i32 {
        type Output = i32;
        fn add(self, rhs: i32) -> Self::Output {
            self + rhs
        }
    }
}

pub mod iter {
    use option::Option::{self, Some, None};
    use ops::*;

    pub trait IntoIterator {
        type Item;
        type IntoIter: Iterator<Item = Self::Item>;
        fn into_iter(self) -> Self::IntoIter;
    }

    pub trait Iterator {
        type Item;
        fn next(&mut self) -> Option<Self::Item>;
    }

    impl IntoIterator for Range<i32> {
        type Item = i32;
        type IntoIter = RangeIterator<i32>;
        fn into_iter(self) -> Self::IntoIter {
            RangeIterator {
                idx: self.start,
                end: self.end,
            }
        }
    }

    pub struct RangeIterator<T> {
        idx: T,
        end: T,
    }

    impl Iterator for RangeIterator<i32> {
        type Item = i32;
        fn next(&mut self) -> Option<Self::Item> {
            if self.idx == self.end {
                return None;
            }

            let result = self.idx;
            self.idx = self.idx + 1;
            Some(result)
        }
    }
}

mod cmp {
    use marker::Sized;

    #[lang = "eq"]
    pub trait PartialEq<Rhs: ?Sized = Self> {
        fn eq(&self, other: &Rhs) -> bool;

        #[inline]
        fn ne(&self, other: &Rhs) -> bool {
            !self.eq(other)
        }
    }

    impl PartialEq for i32 {
        #[inline]
        fn eq(&self, other: &i32) -> bool {
            (*self) == (*other)
        }
        #[inline]
        fn ne(&self, other: &i32) -> bool {
            (*self) != (*other)
        }
    }
}

pub mod marker {
    use clone::Clone;

    #[lang = "sized"]
    #[fundamental]
    pub trait Sized {}

    #[lang = "copy"]
    pub trait Copy: Clone {}
}

pub mod clone {
    use marker::Sized;

    pub trait Clone: Sized {
        fn clone(&self) -> Self;
    }
}

mod option {
    pub enum Option<T> {
        None,
        Some(T),
    }
}

// access to the wasm "spectest" module test printing functions
mod wasm {
    pub fn print_i32(i: i32) {
        unsafe {
            _print_i32(i);
        }
    }

    extern "C" {
        fn _print_i32(i: i32);
    }
}
