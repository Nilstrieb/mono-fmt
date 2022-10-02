#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

extern crate alloc;

mod args;
mod formatter;
mod opts;
mod rust_core_impl;
mod write;

#[macro_export]
macro_rules! format_args {
    ($($tt:tt)*) => {
        $crate::_private::__format_args!($crate $($tt)*)
    };
}

pub use crate::{
    args::{pub_exports::*, Arguments},
    formatter::{DebugList, DebugMap, DebugSet, DebugStruct, DebugTuple, Formatter},
    opts::FmtOpts,
};

pub type Result = core::result::Result<(), Error>;

#[derive(Debug, Clone, Copy)]
pub struct Error;

pub trait Write {
    fn write_str(&mut self, str: &str) -> Result;

    fn write_char(&mut self, char: char) -> Result {
        let mut buf = [0; 4];
        self.write_str(char.encode_utf8(&mut buf))
    }
}

pub mod helpers {
    #[cfg(feature = "alloc")]
    use alloc::string::String;

    use crate::{Arguments, Formatter, Result, Write};

    pub fn write<W: Write, A: Arguments>(buffer: W, args: A) -> Result {
        let mut fmt = Formatter::new(buffer);
        args.fmt(&mut fmt)
    }

    #[cfg(feature = "alloc")]
    pub fn format<A: Arguments>(args: A) -> String {
        let mut string = String::new();
        write(&mut string, args).unwrap();
        string
    }
}

/// Not part of the public API.
#[doc(hidden)]
pub mod _private {
    pub use mono_fmt_macro::__format_args;

    pub use crate::{
        args::{macro_exports::*, Str},
        opts::exports::*,
    };
}

#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! format {
    ($($tt:tt)*) => {
        $crate::helpers::format($crate::format_args!($($tt)*))
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

    #[test]
    fn number() {
        let result = format!("a: {}", 32523532u64);
        assert_eq!(result, "a: 32523532");
    }

    #[test]
    fn escape() {
        let result = format!("a: {{{}}}", 6);
        assert_eq!(result, "a: {6}");
    }
}

pub mod uwu {
    use std::cell::Cell;

    fn test_expansion() {
        let evil = Cell::new(0);
        format!(
            "{0}{0}",
            {
                evil.set(evil.get() + 1);
                0
            },
        );
    }
}
