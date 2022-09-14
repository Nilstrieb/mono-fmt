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
            fn $name:ident(&self) -> $ret:ty {
                $($default:tt)*
            }

            struct $with_name:ident$(<$(const $const_gen_name:ident: $with_ty:ty),*>)? {
                $($struct_body:tt)*
            }
        )*
    ) => {
        // FIXME: We can get rid of this Copy can't we
        pub trait FmtOpts: Copy {
            #[doc(hidden)]
            type Inner: FmtOpts;

            /// Replaces the innermost `()` with `I`
            type ReplaceInnermost<I: FmtOpts>: FmtOpts;

            fn inner(&self) -> &Self::Inner;

            /// # Example
            /// `Self` is `WithAlternate<WithFill<(), ' '>>`
            /// `Other` is WithMinus<()>
            ///
            /// This returns `WithAlternate<WithFille<WithMinus<()>, ' '>>`
            ///
            fn override_other<Other: FmtOpts>(self, other: Other) -> Self::ReplaceInnermost<Other>;

            $(
                #[inline]
                fn $name(&self) -> $ret {
                    Self::Inner::$name(Self::inner(self))
                }
            )*
        }

        impl FmtOpts for () {
            type Inner = Self;

            type ReplaceInnermost<I: FmtOpts> = I;

            fn inner(&self) -> &Self::Inner {
                self
            }

            fn override_other<Other: FmtOpts>(self, other: Other) -> Self::ReplaceInnermost::<Other> {
                other
            }

            $(
                #[inline]
                fn $name(&self) -> $ret {
                    $($default)*
                }
            )*
        }

        impl<O: FmtOpts> FmtOpts for &'_ O {
            type Inner = O::Inner;

            type ReplaceInnermost<I: FmtOpts> = O::ReplaceInnermost<I>;

            fn inner(&self) -> &Self::Inner {
                O::inner(self)
            }

            fn override_other<Other: FmtOpts>(self, other: Other) -> Self::ReplaceInnermost<Other> {
                O::override_other(*self, other)
            }

            $(
                #[inline]
                fn $name(&self) -> $ret {
                    O::$name(self)
                }
            )*
        }


        impl<W, O: FmtOpts> Formatter<W, O> {
            $(
                #[inline]
                pub fn $name(&self) -> $ret {
                    O::$name(&self.opts)
                }
            )*
        }

        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $with_name<I, $($(const $const_gen_name: $with_ty),*)?>(#[doc(hidden)] pub I);

            impl<I: FmtOpts, $($(const $const_gen_name: $with_ty),*)?> FmtOpts for $with_name<I, $($($const_gen_name),*)?> {
                type Inner = I;

                type ReplaceInnermost<Replacement: FmtOpts> = $with_name<I::ReplaceInnermost<Replacement>, $($($const_gen_name),*)?>;

                fn inner(&self) -> &Self::Inner  {
                    &self.0
                }

                fn override_other<Other: FmtOpts>(self, other: Other) -> Self::ReplaceInnermost<Other> {
                    $with_name(self.0.override_other(other))
                }

                fn $name(&self) -> $ret {
                    $($struct_body)*
                }
            }
        )*
    };
}

options!(
    fn alternate(&self) -> bool {
        false
    }
    struct WithAlternate {
        true
    }

    fn width(&self) -> Option<usize> {
        None
    }
    struct WithWidth<const A: usize> {
        Some(A)
    }

    fn align(&self) -> Alignment {
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


    fn fill(&self) -> char {
        ' '
    }
    struct WithFill<const A: char> {
        A
    }

    fn sign_plus(&self) -> bool {
        false
    }
    struct WithSignPlus {
        true
    }

    fn sign_aware_zero_pad(&self) -> bool {
        false
    }
    struct WithSignAwareZeroPad {
        true
    }

    fn sign_minus(&self) -> bool {
        false
    }
    struct WithMinus {
        true
    }

    fn precision(&self) -> Option<usize> {
        None
    }
    struct WithPrecision<const A: usize> {
        Some(A)
    }

    fn debug_lower_hex(&self) -> bool {
        false
    }
    struct WithDebugLowerHex {
        true
    }

    fn debug_upper_hex(&self) -> bool {
        false
    }
    struct WithDebugUpperHex {
        false
    }
);
