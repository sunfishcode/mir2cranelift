//! This is a minimal libcore-like library that can be used by tests
//! before we support the actual libcore.

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


#[link(name = "c")]
extern "C" {}

//extern { fn puts(s: *const u8); }
//extern "rust-intrinsic" { fn transmute<T, U>(t: T) -> U; }

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "eh_unwind_resume"]
extern "C" fn eh_unwind_resume() {}
#[lang = "panic_fmt"]
fn panic_fmt() -> ! {
    loop {}
}
#[no_mangle]
pub extern "C" fn rust_eh_register_frames() {}
#[no_mangle]
pub extern "C" fn rust_eh_unregister_frames() {}
