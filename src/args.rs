use std::marker::PhantomData;

use crate::{Debug, Display, FmtOpts, Formatter, Result, Write};
pub trait Arguments {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result;
}

macro_rules! impl_arguments {
        () => {};
        ($first:ident $($rest:ident)*) => {
            impl<$first, $($rest),*> Arguments for ($first, $($rest),*)
            where
               $first: Arguments,
               $($rest: Arguments),*
            {
                #[allow(non_snake_case)]
                fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
                    let ($first, $($rest),*) = self;
                    Arguments::fmt($first, f)?;
                    $(
                        Arguments::fmt($rest, f)?;
                    )*
                    Ok(())
                }
            }

            impl_arguments!($($rest)*);
        };
    }

#[rustfmt::skip]
    impl_arguments!(
        A1  A2  A3  A4  A5  A6  A7  A8  A9  A10
        // A11 A12 A13 A14 A15 A16 A17 A18 A19 A20
        // A21 A22 A23 A24 A25 A26 A27 A28 A29 A30
        // A31 A32 A33 A34 A35 A36 A37 A38 A39 A40
        // A41 A42 A43 A44 A45 A46 A47 A48 A49 A50
        // A51 A52 A53 A54 A55 A56 A57 A58 A59 A60
        // A61 A62 A63 A64 A65 A66 A67 A68 A69 A70
        // A71 A72 A73 A74 A75 A76 A77 A78 A79 A80
        // A81 A82 A83 A84 A85 A86 A87 A88 A89 A90
        // A91 A92 A93 A94 A95 A96 A97 A98 A99 A100
    );

pub struct Str(pub &'static str);

impl Arguments for Str {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        f.write_str(self.0)
    }
}

pub struct DebugArg<T>(pub T);

impl<T: Debug> Arguments for DebugArg<T> {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        Debug::fmt(&self.0, f)
    }
}

pub struct DisplayArg<T>(pub T);

impl<T: Display> Arguments for DisplayArg<T> {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        Display::fmt(&self.0, f)
    }
}

pub struct ConstWidthArg<T, const WIDTH: usize> {
    value: T,
    _boo: PhantomData<[(); WIDTH]>,
}

#[allow(non_snake_case)]
pub fn ConstWidthArg<T, const WIDTH: usize>(value: T) -> ConstWidthArg<T, WIDTH> {
    ConstWidthArg {
        value,
        _boo: PhantomData,
    }
}

impl<T: Display, const WIDTH: usize> Arguments for ConstWidthArg<T, WIDTH> {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        todo!()
    }
}
