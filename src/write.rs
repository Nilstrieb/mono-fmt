use crate::{Result, Write};

impl<W: Write> Write for &mut W {
    fn write_str(&mut self, str: &str) -> Result {
        <W as Write>::write_str(self, str)
    }

    fn write_char(&mut self, char: char) -> Result {
        <W as Write>::write_char(self, char)
    }
}

impl Write for &'_ mut [u8] {
    fn write_str(&mut self, str: &str) -> Result {
        let data = str.as_bytes();
        let amt = core::cmp::min(str.len(), self.len());
        let (a, b) = core::mem::replace(self, &mut []).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use alloc::{boxed::Box, collections::VecDeque, string::String, vec::Vec};

    use crate::{Result, Write};

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

    impl<W: Write> Write for Box<W> {
        fn write_str(&mut self, str: &str) -> Result {
            <W as Write>::write_str(self, str)
        }

        fn write_char(&mut self, char: char) -> Result {
            <W as Write>::write_char(self, char)
        }
    }

    impl Write for Vec<u8> {
        fn write_str(&mut self, str: &str) -> Result {
            self.extend(str.as_bytes());
            Ok(())
        }
    }

    impl Write for VecDeque<u8> {
        fn write_str(&mut self, str: &str) -> Result {
            self.extend(str.as_bytes());
            Ok(())
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use std::{
        fs,
        io::{self, Write as IoWrite},
        net, process,
    };

    use crate::Result;

    trait IoWriteForwad: IoWrite {
        fn write_str(&mut self, str: &str) -> Result {
            <Self as IoWrite>::write_all(self, str.as_bytes()).map_err(|_| crate::Error)
        }

        fn write_char(&mut self, char: char) -> Result {
            let mut buf = [0; 4];

            <Self as IoWrite>::write_all(self, char.encode_utf8(&mut buf).as_bytes())
                .map_err(|_| crate::Error)
        }
    }

    macro_rules! impl_io_forward {
        ($($name:ty),* $(,)?) => {
            $(
                impl IoWriteForwad for $name {}
            )*
        };
    }

    impl_io_forward!(
        fs::File,
        net::TcpStream,
        process::ChildStdin,
        io::Cursor<&'_ mut [u8]>,
        io::Sink,
        io::Stderr,
        io::StderrLock<'_>,
        io::Stdout,
        io::StdoutLock<'_>,
        io::Cursor<&'_ mut Vec<u8>>,
        io::Cursor<Box<[u8]>>,
        io::Cursor<Vec<u8>>,
        &fs::File,
        &net::TcpStream,
        &process::ChildStdin,
        &io::Sink,
        &io::Stderr,
        &io::Stdout,
    );
}
