#![feature(intrinsics, lang_items, main, no_core, fundamental)]
#![no_core]

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

#[lang = "sub"]
pub trait Sub<RHS = Self> {
    type Output;
    fn sub(self, rhs: RHS) -> Self::Output;
}

impl Sub for isize {
    type Output = isize;
    fn sub(self, rhs: isize) -> Self::Output {
        self - rhs
    }
}

#[lang = "mul"]
pub trait Mul<RHS = Self> {
    type Output;
    fn mul(self, rhs: RHS) -> Self::Output;
}

impl Mul for isize {
    type Output = isize;
    fn mul(self, rhs: isize) -> Self::Output {
        self * rhs
    }
}

#[lang = "div"]
pub trait Div<RHS = Self> {
    type Output;
    fn div(self, rhs: RHS) -> Self::Output;
}

impl Div for isize {
    type Output = isize;
    fn div(self, rhs: isize) -> Self::Output {
        self / rhs
    }
}

#[lang = "add_assign"]
pub trait AddAssign<Rhs = Self> {
    fn add_assign(&mut self, Rhs);
}

impl AddAssign for isize {
    #[inline]
    fn add_assign(&mut self, other: isize) {
        *self += other
    }
}

#[lang = "sub_assign"]
pub trait SubAssign<Rhs = Self> {
    fn sub_assign(&mut self, Rhs);
}

impl SubAssign for isize {
    #[inline]
    fn sub_assign(&mut self, other: isize) {
        *self -= other
    }
}

#[lang = "mul_assign"]
pub trait MulAssign<Rhs = Self> {
    fn mul_assign(&mut self, Rhs);
}

impl MulAssign for isize {
    #[inline]
    fn mul_assign(&mut self, other: isize) {
        *self *= other
    }
}

#[lang = "div_assign"]
pub trait DivAssign<Rhs = Self> {
    fn div_assign(&mut self, Rhs);
}

impl DivAssign for isize {
    #[inline]
    fn div_assign(&mut self, other: isize) {
        *self /= other
    }
}

#[lang = "eq"]
pub trait PartialEq<Rhs: ?Sized = Self> {
    fn eq(&self, other: &Rhs) -> bool;

    #[inline]
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

impl PartialEq for isize {
    #[inline]
    fn eq(&self, other: &isize) -> bool {
        (*self) == (*other)
    }
    #[inline]
    fn ne(&self, other: &isize) -> bool {
        (*self) != (*other)
    }
}

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

fn test(i: isize) -> isize {
    ((i + 3) * 2 - 2) / 3
}

#[main]
fn main() {
    let mut i = 0;
    i += 3;
    i *= 4;
    i /= 6;
    i -= 1;

    let _j = i == 1;
    // TODO: assert j is true

    let result = test(i);
    wasm::print_i32(result); // (i32.const 2)
}
