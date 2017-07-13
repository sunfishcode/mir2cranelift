#![feature(fundamental, lang_items, no_core)]
#![allow(dead_code)]
#![no_core]

enum Tag {
    A(i32),
    B(i32),
    C,
}

enum CTag {
    A = -1,
    B = 0,
    C = 1,
    D = 12,
}

fn main() {
    let t = Tag::B(17);
    let i = match t {
        Tag::A(_) => 1,
        Tag::B(i) => i,
        _ => 3,
    };
    wasm::print_i32(i); // (i32.const 17)

    let t = CTag::A;
    let i = match t {
        i @ _ => i as i32,
    };
    wasm::print_i32(i as i32); // (i32.const -1)
    wasm::print_i32(CTag::D as i32); // (i32.const 12)
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

mod wasm {
    // access to the wasm "spectest" module test printing functions
    pub fn print_i32(i: i32) {
        unsafe {
            _print_i32(i);
        }
    }

    extern "C" {
        fn _print_i32(i: i32);
    }
}
