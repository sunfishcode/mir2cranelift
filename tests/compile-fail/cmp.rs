#![feature(lang_items, no_core, fundamental, intrinsics)]
#![no_core]
#![allow(unused_variables)]

fn main() {
    let x: i32 = 0;
    let y: i32 = 1;

    let result = (x == y) as i32;
    let result = (x.eq(&y)) as i32;

    let result = (x != y) as i32;
    let result = (x.ne(&y)) as i32;

    let result = (x < y) as i32;
    let result = (x.lt(&y)) as i32;

    let result = (x <= y) as i32;
    let result = (x.le(&y)) as i32;

    let result = (x > y) as i32;
    let result = (x.gt(&y)) as i32;

    let result = (x >= y) as i32;
    let result = (x.ge(&y)) as i32;
}

pub mod marker {
    use clone::Clone;

    #[lang = "sized"]
    #[fundamental]
    pub trait Sized {}

    #[lang = "copy"]
    pub trait Copy: Clone {}
}

use cmp::*;

pub mod clone {
    use marker::Sized;

    pub trait Clone: Sized {}
}

mod option {
    pub enum Option<T> {
        None,
        Some(T),
    }
}

pub mod cmp {
    use marker::*;
    use option::Option::{self, Some};
    use self::Ordering::*;
    use clone::Clone;

    #[lang = "eq"]
    pub trait PartialEq<Rhs: ?Sized = Self> {
        fn eq(&self, other: &Rhs) -> bool;

        #[inline]
        fn ne(&self, other: &Rhs) -> bool {
            !self.eq(other)
        }
    }

    pub trait Eq: PartialEq<Self> {
        #[doc(hidden)]
        #[inline(always)]
        fn assert_receiver_is_total_eq(&self) {}
    }

    pub enum Ordering {
        Less = -1,
        Equal = 0,
        Greater = 1,
    }

    impl Ordering {
        #[inline]
        pub fn reverse(self) -> Ordering {
            match self {
                Less => Greater,
                Equal => Equal,
                Greater => Less,
            }
        }
    }

    #[inline]
    pub fn min<T: Ord>(v1: T, v2: T) -> T {
        if v1 <= v2 { v1 } else { v2 }
    }

    #[inline]
    pub fn max<T: Ord>(v1: T, v2: T) -> T {
        if v2 >= v1 { v2 } else { v1 }
    }

    pub trait Ord: Eq + PartialOrd<Self> {
        fn cmp(&self, other: &Self) -> Ordering;
    }

    impl Eq for Ordering {}

    impl Ord for Ordering {
        #[inline]
        fn cmp(&self, other: &Ordering) -> Ordering {
            (*self as i32).cmp(&(*other as i32))
        }
    }

    impl<Rhs> PartialEq<Rhs> for Ordering {
        fn eq(&self, other: &Rhs) -> bool {
            self == other
        }
    }

    impl Clone for Ordering {}
    impl Copy for Ordering {}

    impl PartialOrd for Ordering {
        #[inline]
        fn partial_cmp(&self, other: &Ordering) -> Option<Ordering> {
            (*self as i32).partial_cmp(&(*other as i32))
        }
    }

    #[lang = "ord"]
    pub trait PartialOrd<Rhs: ?Sized = Self>: PartialEq<Rhs> {
        fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;

        #[inline]
        fn lt(&self, other: &Rhs) -> bool {
            match self.partial_cmp(other) {
                Some(Less) => true,
                _ => false,
            }
        }

        #[inline]
        fn le(&self, other: &Rhs) -> bool {
            match self.partial_cmp(other) {
                Some(Less) | Some(Equal) => true,
                _ => false,
            }
        }

        #[inline]
        fn gt(&self, other: &Rhs) -> bool {
            match self.partial_cmp(other) {
                Some(Greater) => true,
                _ => false,
            }
        }

        #[inline]
        fn ge(&self, other: &Rhs) -> bool {
            match self.partial_cmp(other) {
                Some(Greater) | Some(Equal) => true,
                _ => false,
            }
        }
    }

    // Implementation of PartialEq, Eq, PartialOrd and Ord for primitive types
    mod impls {
        use super::{PartialOrd, Ord, PartialEq, Eq, Ordering};
        use super::Ordering::{Less, Greater, Equal};

        use option::Option::{self, Some};
        use marker::Sized;

        macro_rules! partial_eq_impl {
            ($($t:ty)*) => ($(

                impl PartialEq for $t {
                    #[inline]
                    fn eq(&self, other: &$t) -> bool { (*self) == (*other) }
                    #[inline]
                    fn ne(&self, other: &$t) -> bool { (*self) != (*other) }
                }
            )*)
        }

        impl PartialEq for () {
            #[inline]
            fn eq(&self, _other: &()) -> bool {
                true
            }
            #[inline]
            fn ne(&self, _other: &()) -> bool {
                false
            }
        }

        partial_eq_impl! {
            bool i32 u8 isize //char usize u16 u32 u64 i8 i16 i64 f32 f64
        }

        macro_rules! eq_impl {
            ($($t:ty)*) => ($(

                impl Eq for $t {}
            )*)
        }

        eq_impl! {
            () bool i32 u8 isize //char usize u16 u32 u64 i8 i16 i64
        }

        macro_rules! partial_ord_impl {
            ($($t:ty)*) => ($(

                impl PartialOrd for $t {
                    #[inline]
                    fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                        match (self <= other, self >= other) {
                            (false, false) => None,
                            (false, true) => Some(Greater),
                            (true, false) => Some(Less),
                            (true, true) => Some(Equal),
                        }
                    }
                    #[inline]
                    fn lt(&self, other: &$t) -> bool { (*self) < (*other) }
                    #[inline]
                    fn le(&self, other: &$t) -> bool { (*self) <= (*other) }
                    #[inline]
                    fn ge(&self, other: &$t) -> bool { (*self) >= (*other) }
                    #[inline]
                    fn gt(&self, other: &$t) -> bool { (*self) > (*other) }
                }
            )*)
        }


        impl PartialOrd for () {
            #[inline]
            fn partial_cmp(&self, _: &()) -> Option<Ordering> {
                Some(Equal)
            }
        }


        impl PartialOrd for bool {
            #[inline]
            fn partial_cmp(&self, other: &bool) -> Option<Ordering> {
                (*self as u8).partial_cmp(&(*other as u8))
            }
        }

        macro_rules! ord_impl {
            ($($t:ty)*) => ($(

                impl PartialOrd for $t {
                    #[inline]
                    fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                        Some(self.cmp(other))
                    }
                    #[inline]
                    fn lt(&self, other: &$t) -> bool { (*self) < (*other) }
                    #[inline]
                    fn le(&self, other: &$t) -> bool { (*self) <= (*other) }
                    #[inline]
                    fn ge(&self, other: &$t) -> bool { (*self) >= (*other) }
                    #[inline]
                    fn gt(&self, other: &$t) -> bool { (*self) > (*other) }
                }


                impl Ord for $t {
                    #[inline]
                    fn cmp(&self, other: &$t) -> Ordering {
                        if *self == *other { Equal }
                        else if *self < *other { Less }
                        else { Greater }
                    }
                }
            )*)
        }


        impl Ord for () {
            #[inline]
            fn cmp(&self, _other: &()) -> Ordering {
                Equal
            }
        }


        impl Ord for bool {
            #[inline]
            fn cmp(&self, other: &bool) -> Ordering {
                (*self as u8).cmp(&(*other as u8))
            }
        }

        ord_impl! {
            i32 u8 isize // char usize u16 u32 u64 i8 i16 i64
        }

        // & pointers


        impl<'a, 'b, A: ?Sized, B: ?Sized> PartialEq<&'b B> for &'a A
        where
            A: PartialEq<B>,
        {
            #[inline]
            fn eq(&self, other: &&'b B) -> bool {
                PartialEq::eq(*self, *other)
            }
            #[inline]
            fn ne(&self, other: &&'b B) -> bool {
                PartialEq::ne(*self, *other)
            }
        }

        impl<'a, 'b, A: ?Sized, B: ?Sized> PartialOrd<&'b B> for &'a A
        where
            A: PartialOrd<B>,
        {
            #[inline]
            fn partial_cmp(&self, other: &&'b B) -> Option<Ordering> {
                PartialOrd::partial_cmp(*self, *other)
            }
            #[inline]
            fn lt(&self, other: &&'b B) -> bool {
                PartialOrd::lt(*self, *other)
            }
            #[inline]
            fn le(&self, other: &&'b B) -> bool {
                PartialOrd::le(*self, *other)
            }
            #[inline]
            fn ge(&self, other: &&'b B) -> bool {
                PartialOrd::ge(*self, *other)
            }
            #[inline]
            fn gt(&self, other: &&'b B) -> bool {
                PartialOrd::gt(*self, *other)
            }
        }

        impl<'a, A: ?Sized> Ord for &'a A
        where
            A: Ord,
        {
            #[inline]
            fn cmp(&self, other: &&'a A) -> Ordering {
                Ord::cmp(*self, *other)
            }
        }

        impl<'a, A: ?Sized> Eq for &'a A
        where
            A: Eq,
        {
        }

        // &mut pointers


        impl<'a, 'b, A: ?Sized, B: ?Sized> PartialEq<&'b mut B> for &'a mut A
        where
            A: PartialEq<B>,
        {
            #[inline]
            fn eq(&self, other: &&'b mut B) -> bool {
                PartialEq::eq(*self, *other)
            }
            #[inline]
            fn ne(&self, other: &&'b mut B) -> bool {
                PartialEq::ne(*self, *other)
            }
        }

        impl<'a, 'b, A: ?Sized, B: ?Sized> PartialOrd<&'b mut B> for &'a mut A
        where
            A: PartialOrd<B>,
        {
            #[inline]
            fn partial_cmp(&self, other: &&'b mut B) -> Option<Ordering> {
                PartialOrd::partial_cmp(*self, *other)
            }
            #[inline]
            fn lt(&self, other: &&'b mut B) -> bool {
                PartialOrd::lt(*self, *other)
            }
            #[inline]
            fn le(&self, other: &&'b mut B) -> bool {
                PartialOrd::le(*self, *other)
            }
            #[inline]
            fn ge(&self, other: &&'b mut B) -> bool {
                PartialOrd::ge(*self, *other)
            }
            #[inline]
            fn gt(&self, other: &&'b mut B) -> bool {
                PartialOrd::gt(*self, *other)
            }
        }

        impl<'a, A: ?Sized> Ord for &'a mut A
        where
            A: Ord,
        {
            #[inline]
            fn cmp(&self, other: &&'a mut A) -> Ordering {
                Ord::cmp(*self, *other)
            }
        }

        impl<'a, A: ?Sized> Eq for &'a mut A
        where
            A: Eq,
        {
        }


        impl<'a, 'b, A: ?Sized, B: ?Sized> PartialEq<&'b mut B> for &'a A
        where
            A: PartialEq<B>,
        {
            #[inline]
            fn eq(&self, other: &&'b mut B) -> bool {
                PartialEq::eq(*self, *other)
            }
            #[inline]
            fn ne(&self, other: &&'b mut B) -> bool {
                PartialEq::ne(*self, *other)
            }
        }


        impl<'a, 'b, A: ?Sized, B: ?Sized> PartialEq<&'b B> for &'a mut A
        where
            A: PartialEq<B>,
        {
            #[inline]
            fn eq(&self, other: &&'b B) -> bool {
                PartialEq::eq(*self, *other)
            }
            #[inline]
            fn ne(&self, other: &&'b B) -> bool {
                PartialEq::ne(*self, *other)
            }
        }
    }
}
