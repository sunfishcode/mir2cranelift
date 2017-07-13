#![feature(lang_items, no_core, start)]
#![allow(dead_code)]
#![no_core]

#[lang="sized"]
trait Sized {}

#[lang="copy"]
trait Copy {}

enum Tag {
    A(i32),
    B(i32),
}

#[start]
fn main(_i: isize, _: *const *const u8) -> isize {
    let a = Tag::A(5);
    match a {
        Tag::A(i) => i,
        Tag::B(i) => i,
    };
    0
}
