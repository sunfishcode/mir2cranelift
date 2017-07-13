#![feature(lang_items, no_core)]

#![no_core]
#![no_std]

#[no_mangle]
pub fn fact(_n: i32) -> i32 {
    120
}

#[lang = "sized"]
//#[fundamental]
trait Sized {}
