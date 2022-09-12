// for the test macro expansion
#[cfg(test)]
extern crate self as mono_fmt;

pub use mono_fmt_macro::format_args;

use crate::arguments::Arguments;

pub type Result = std::result::Result<(), Error>;

#[derive(Debug)]
pub struct Error;

pub trait Write {
    fn write_str(&mut self, str: &str) -> Result;
    fn write_char(&mut self, char: char) -> Result;
}

impl Write for String {
    fn write_str(&mut self, str: &str) -> Result {
        self.push_str(str);
        Ok(())
    }

    fn write_char(&mut self, char: char) -> Result {
        self.push(char);
        Ok(())
    }
}

impl<W: Write> Write for &mut W {
    fn write_str(&mut self, str: &str) -> Result {
        <W as Write>::write_str(self, str)
    }

    fn write_char(&mut self, char: char) -> Result {
        <W as Write>::write_char(self, char)
    }
}

trait Debug {
    fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result;
}

trait Display {
    fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result;
}

impl Debug for &'_ str {
    fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result {
        f.write_char('"')?;
        f.write_str(self)?;
        f.write_char('"')
    }
}

impl Display for &'_ str {
    fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result {
        f.write_str(self)
    }
}

pub struct Formatter<W> {
    buf: W,
}

impl<W> Formatter<W> {
    fn new(buf: W) -> Self {
        Self { buf }
    }
}

impl<W: Write> Formatter<W> {
    pub fn write_char(&mut self, char: char) -> Result {
        self.buf.write_char(char)
    }

    pub fn write_str(&mut self, str: &str) -> Result {
        self.buf.write_str(str)
    }
}

pub fn format<A: Arguments>(args: A) -> String {
    let mut string = String::new();
    let mut fmt = Formatter::new(&mut string);
    args.fmt(&mut fmt).unwrap();
    string
}

mod arguments {
    use crate::{Debug, Display, Formatter, Result, Write};
    pub trait Arguments {
        fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result;
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
                fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result {
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
        fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result {
            f.write_str(self.0)
        }
    }

    pub struct DebugArg<T>(pub T);

    impl<T: Debug> Arguments for DebugArg<T> {
        fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result {
            Debug::fmt(&self.0, f)
        }
    }

    pub struct DisplayArg<T>(pub T);

    impl<T: Display> Arguments for DisplayArg<T> {
        fn fmt<W: Write>(&self, f: &mut Formatter<W>) -> Result {
            Display::fmt(&self.0, f)
        }
    }
}

mod _private {
    pub use super::arguments::{DebugArg, DisplayArg, Str};
}

#[macro_export]
macro_rules! format {
    ($($tt:tt)*) => {
        $crate::format($crate::format_args!($($tt)*))
    };
}

#[cfg(test)]
mod tests {
    use crate::format;

    #[test]
    fn hello_world() {
        let result = format!("Hello, World");
        assert_eq!(result, "Hello, World");
    }

    #[test]
    fn display() {
        let result = format!("{}", "uwu");
        assert_eq!(result, "uwu");
    }

    #[test]
    fn display_with_strings() {
        let result = format!("oow{} omg", "uwu");
        assert_eq!(result, "oowuwu omg");
    }

    #[test]
    fn debug() {
        let result = format!("test {:?} hello", "uwu");
        assert_eq!(result, r#"test "uwu" hello"#);
    }
}
