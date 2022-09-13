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
            type Inner = never::Never;

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
    };
}

options!(
    fn alternate() -> bool {
        false
    }

    fn width() -> Option<usize> {
        None
    }

    fn align() -> Alignment {
        Alignment::Unknown
    }

    fn fill() -> char {
        ' '
    }

    fn sign_plus() -> bool {
        false
    }

    fn sign_aware_zero_pad() -> bool {
        false
    }

    fn sign_minus() -> bool {
        false
    }

    fn precision() -> Option<usize> {
        None
    }

    fn debug_lower_hex() -> bool {
        false
    }

    fn debug_upper_hex() -> bool {
        false
    }
);

mod never {
    use crate::FmtOpts;

    pub trait Func {
        type Output;
    }

    impl<T> Func for fn() -> T {
        type Output = T;
    }

    pub type Never = <fn() -> ! as Func>::Output;

    impl FmtOpts for Never {
        type Inner = Self;
    }
}

pub struct WithAlternate<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithAlternate<I> {
    type Inner = I;
    #[inline]
    fn alternate() -> bool {
        true
    }
}
pub struct WithWidth<I, const A: usize>(pub I);

impl<I: FmtOpts, const A: usize> FmtOpts for WithWidth<I, A> {
    type Inner = I;
    #[inline]
    fn width() -> Option<usize> {
        Some(A)
    }
}
pub struct WithLeftAlign<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithLeftAlign<I> {
    type Inner = I;
    #[inline]
    fn align() -> Alignment {
        Alignment::Left
    }
}
pub struct WithRightAlign<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithRightAlign<I> {
    type Inner = I;
    #[inline]
    fn align() -> Alignment {
        Alignment::Right
    }
}
pub struct WithCenterAlign<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithCenterAlign<I> {
    type Inner = I;
    #[inline]
    fn align() -> Alignment {
        Alignment::Center
    }
}
pub struct WithFill<I, const A: char>(pub I);

impl<I: FmtOpts, const A: char> FmtOpts for WithFill<I, A> {
    type Inner = I;
    #[inline]
    fn fill() -> char {
        A
    }
}
