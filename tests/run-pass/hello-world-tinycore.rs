#![feature(intrinsics, lang_items, start, no_core, fundamental)]
#![no_core]
#![allow(unused_imports)]

pub mod tinycore;
use tinycore::*;

// access to the wasm "spectest" module test printing functions
mod wasm {
    pub fn print_i32(i: isize) {
        unsafe {
            _print_i32(i);
        }
    }

    extern "C" {
        fn _print_i32(i: isize);
    }
}

fn real_main() -> isize {
    let i = 1;
    let j = i + 2;
    j
}

#[start]
fn main(_i: isize, _: *const *const u8) -> isize {
    let result = real_main() + 3;
    wasm::print_i32(result); //~ (i32.const 6)
    result
}
