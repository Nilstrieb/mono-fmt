// for the test macro expansion
#[cfg(test)]
extern crate self as mono_fmt;

mod args;
mod opts;
mod write;

pub use mono_fmt_macro::format_args;
use opts::{WithAlternate, WithCenterAlign, WithFill, WithLeftAlign, WithRightAlign, WithWidth};

pub use crate::args::Arguments;
pub use crate::opts::FmtOpts;

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

pub trait Debug {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result;
}

pub trait Display {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result;
}

pub struct Formatter<W, O> {
    buf: W,
    opts: O,
}

impl<W: Write, O: FmtOpts> core::fmt::Write for Formatter<W, O> {
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.buf.write_char(c).map_err(|_| std::fmt::Error)
    }
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.write_str(s).map_err(|_| std::fmt::Error)
    }
}

impl<W> Formatter<W, ()> {
    fn new(buf: W) -> Self {
        Self { buf, opts: () }
    }
}

impl<W: Write, O: FmtOpts> Formatter<W, O> {
    pub fn write_char(&mut self, char: char) -> Result {
        self.buf.write_char(char)
    }

    pub fn write_str(&mut self, str: &str) -> Result {
        self.buf.write_str(str)
    }
}

pub fn write<W: Write, A: Arguments>(buffer: W, args: A) -> Result {
    let mut fmt = Formatter::new(buffer);
    args.fmt(&mut fmt)
}

pub fn format<A: Arguments>(args: A) -> String {
    let mut string = String::new();
    write(&mut string, args).unwrap();
    string
}

mod _private {
    pub use crate::args::{ConstWidthArg, DebugArg, DisplayArg, Str};

    pub use crate::opts::{
        WithAlternate, WithCenterAlign, WithFill, WithLeftAlign, WithRightAlign, WithWidth,
    };
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
