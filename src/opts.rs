use crate::Formatter;

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Unknown,
}

macro_rules! options {
    (
        $(
            fn $name:ident() -> $ret:ty {
                $($default:tt)*
            }

            struct $with_name:ident$(<$(const $const_gen_name:ident: $with_ty:ty),*>)? {
                $($struct_body:tt)*
            }
        )*
    ) => {
        pub trait FmtOpts {
            #[doc(hidden)]
            type Inner: FmtOpts;

            $(
                #[inline]
                fn $name() -> $ret {
                    Self::Inner::$name()
                }
            )*
        }

        impl FmtOpts for () {
            type Inner = Self;

            $(
                #[inline]
                fn $name() -> $ret {
                    $($default)*
                }
            )*
        }

        impl<W, O: FmtOpts> Formatter<W, O> {
            $(
                #[inline]
                pub fn $name(&self) -> $ret {
                    O::$name()
                }
            )*
        }

        $(
            pub struct $with_name<I, $($(const $const_gen_name: $with_ty),*)?>(#[doc(hidden)] pub I);

            impl<I: FmtOpts, $($(const $const_gen_name: $with_ty),*)?> FmtOpts for $with_name<I, $($($const_gen_name),*)?> {
                type Inner = I;

                fn $name() -> $ret {
                    $($struct_body)*
                }
            }
        )*
    };
}

options!(
    fn alternate() -> bool {
        false
    }
    struct WithAlternate {
        true
    }

    fn width() -> Option<usize> {
        None
    }
    struct WithWidth<const A: usize> {
        Some(A)
    }

    fn align() -> Alignment {
        Alignment::Unknown
    }
    struct WithAlign<const A: usize> {
        match A {
            0 => Alignment::Unknown,
            1 => Alignment::Left,
            2 => Alignment::Center,
            3 => Alignment::Right,
            _ => panic!("invalid alignment number {A}")
        }
    }


    fn fill() -> char {
        ' '
    }
    struct WithFill<const A: char> {
        A
    }

    fn sign_plus() -> bool {
        false
    }
    struct WithSignPlus {
        true
    }

    fn sign_aware_zero_pad() -> bool {
        false
    }
    struct WithSignAwareZeroPad {
        true
    }

    fn sign_minus() -> bool {
        false
    }
    struct WithMinus {
        true
    }

    fn precision() -> Option<usize> {
        None
    }
    struct WithPrecision<const A: usize> {
        Some(A)
    }

    fn debug_lower_hex() -> bool {
        false
    }
    struct WithDebugLowerHex {
        true
    }

    fn debug_upper_hex() -> bool {
        false
    }
    struct WithDebugUpperHex {
        false
    }
);
