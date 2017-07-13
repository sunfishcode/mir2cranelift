#![feature(lang_items, no_core)]

#![no_core]

#[no_mangle]
pub fn main() {}

#[lang = "sized"]
trait Sized {}

#[lang = "copy"]
pub trait Copy {}
