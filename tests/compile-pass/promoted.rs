// xfail

#![feature(intrinsics, lang_items, no_core, fundamental)]
#![no_core]

#[lang = "sized"]
#[fundamental]
pub trait Sized {}

#[lang = "copy"]
pub trait Copy: Clone {}

pub trait Clone: Sized {}

fn main() {
    10.fibonacci();

    // the following two assignments create 'promoted' blocks, which are not yet implemented
    // let result = Fibonacci::fibonacci(&10);
    // let result = 10.fibonacci();
}

trait Fibonacci {
    fn fibonacci(&self) -> Self;
}

impl Fibonacci for i32 {
    fn fibonacci(&self) -> i32 {
        fibonacci_iterative(*self)
    }
}

fn fibonacci_iterative(n: i32) -> i32 {
    let mut current = 0;
    let mut next = 1;

    let mut iterator = 0;
    loop {
        if iterator == n {
            break;
        }

        let tmp = current + next;
        current = next;
        next = tmp;

        iterator += 1;
    }

    current
}

#[lang = "eq"]
pub trait PartialEq<Rhs: ?Sized = Self> {
    fn eq(&self, other: &Rhs) -> bool;

    #[inline]
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

impl PartialEq for i32 {
    #[inline]
    fn eq(&self, other: &i32) -> bool {
        (*self) == (*other)
    }
    #[inline]
    fn ne(&self, other: &i32) -> bool {
        (*self) != (*other)
    }
}

#[lang = "add"]
pub trait Add<RHS = Self> {
    type Output;
    fn add(self, rhs: RHS) -> Self::Output;
}

impl Add for i32 {
    type Output = i32;
    fn add(self, rhs: i32) -> Self::Output {
        self + rhs
    }
}

#[lang = "add_assign"]
pub trait AddAssign<Rhs = Self> {
    fn add_assign(&mut self, Rhs);
}

impl AddAssign for i32 {
    #[inline]
    fn add_assign(&mut self, other: i32) {
        *self += other
    }
}
