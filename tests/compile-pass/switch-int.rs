// xfail
#![feature(lang_items, no_core)]
#![no_core]

#[lang="sized"]
trait Sized {}

fn main() {
    let x = 6;
    match x {
        i @ 1...5 => i,
        _ => -2,
    };

    match x {
        1 | 2 => -1,
        3 => -2,
        _ => -3,
    };
}
