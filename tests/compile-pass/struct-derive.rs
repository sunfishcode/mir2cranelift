#![feature(intrinsics, lang_items, no_core, fundamental)]
#![no_core]
#![allow(dead_code)]
#![allow(unused_variables)]

#[derive(Clone, Copy)]
struct Rectangle {
    w: i32,
    h: i32,
}

impl Rectangle {
    fn area(&self) -> i32 {
        self.w * self.h
    }
}

fn main() {
    use clone::Clone;

    let mut r = Rectangle { w: 2, h: 5 };
    r.w = 3;

    let mut r = r.clone();
    r.w = 4;
}

pub mod marker {
    use clone::Clone;

    #[lang = "sized"]
    #[fundamental]
    pub trait Sized {}

    #[lang = "copy"]
    pub trait Copy: Clone {}

    #[lang = "phantom_data"]
    pub struct PhantomData;
}

pub mod clone {
    use marker::Sized;

    pub trait Clone: Sized {
        fn clone(&self) -> Self;
    }

    pub struct AssertParamIsClone<T: Clone + ?Sized> {
        _field: ::marker::PhantomData<T>,
    }

    // should this #[derive] requirement exist and have to be translated in our case ?
    pub fn assert_receiver_is_clone<T: Clone + ?Sized>(_: &T) {}

    macro_rules! clone_impl {
        ($t:ty) => {
            impl Clone for $t {
                /// Returns a deep copy of the value.
                #[inline]
                fn clone(&self) -> $t { *self }
            }
        }
    }

    clone_impl! { i32 }
}

#[lang = "mul"]
pub trait Mul<RHS = Self> {
    type Output;
    fn mul(self, rhs: RHS) -> Self::Output;
}

impl Mul for i32 {
    type Output = i32;
    fn mul(self, rhs: i32) -> Self::Output {
        self * rhs
    }
}
