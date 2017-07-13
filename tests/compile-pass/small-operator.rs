#![feature(intrinsics, lang_items, no_core, fundamental)]
#![no_core]
#![allow(unused_variables)]

#[lang = "sized"]
#[fundamental]
pub trait Sized {}

#[lang = "copy"]
pub trait Copy: Clone {}

pub trait Clone: Sized {}

#[lang = "add"]
pub trait Add<RHS = Self> {
    type Output;
    fn add(self, rhs: RHS) -> Self::Output;
}

impl Add for isize {
    type Output = isize;
    fn add(self, rhs: isize) -> Self::Output {
        self + rhs
    }
}


fn main() {
    let _i = 1 + 2;
}
