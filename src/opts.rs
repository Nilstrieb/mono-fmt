pub enum Alignment {
    Left,
    Center,
    Right,
}

pub trait FmtOpts {
    #[doc(hidden)]
    type Inner: FmtOpts;

    fn alternate() -> bool {
        Self::Inner::alternate()
    }

    fn width() -> Option<usize> {
        Self::Inner::width()
    }

    fn align() -> Option<Alignment> {
        Self::Inner::align()
    }

    fn fill() -> Option<char> {
        Self::Inner::fill()
    }
}

pub struct Default;

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

impl FmtOpts for () {
    type Inner = never::Never;

    fn alternate() -> bool {
        false
    }

    fn width() -> Option<usize> {
        None
    }

    fn align() -> Option<Alignment> {
        None
    }

    fn fill() -> Option<char> {
        None
    }
}

macro_rules! with_fmts {
    (@ty $ty:ty, $($tt:tt)*) => { $ty };
    ($(struct $name:ident $(<const A: $param:ty>)? {
        fn $override:ident() -> $override_ret:ty {
            $($override_body:tt)*
        }
    })*) => {
        $(
            pub struct $name<I, $(const A: $param)?>(I);

            impl<I: FmtOpts, $(const A: $param)?> FmtOpts for $name<I, $(with_fmts!(@ty A, $param))?> {
                type Inner = I;

                fn $override() -> $override_ret {
                    $($override_body)*
                }
            }
        )*
    };

    (struct $name:ident) => {};
}

with_fmts! {
    struct WithAlternate {
        fn alternate() -> bool {
            true
        }
    }
    struct WithWidth<const A: usize> {
        fn width() -> Option<usize> {
            Some(A)
        }
    }
    struct WithLeftAlign {
        fn align() -> Option<Alignment> {
            Some(Alignment::Left)
        }
    }
    struct WithRightAlign {
        fn align() -> Option<Alignment> {
            Some(Alignment::Right)
        }
    }
    struct WithCenterAlign {
        fn align() -> Option<Alignment> {
            Some(Alignment::Center)
        }
    }
    struct WithFill<const A: char> {    
        fn fill() -> Option<char> {
            Some(A)
        }
    }
}
