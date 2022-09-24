//! Copied modified stuff from core

mod num;

use crate::{opts::Alignment, Error, FmtOpts, Formatter, Result, Write};

mod numfmt {
    //! Shared utilities used by both float and integer formatting.

    /// Formatted parts.
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Part<'a> {
        /// Given number of zero digits.
        Zero(usize),
        /// A literal number up to 5 digits.
        Num(u16),
        /// A verbatim copy of given bytes.
        Copy(&'a [u8]),
    }

    impl<'a> Part<'a> {
        /// Returns the exact byte length of given part.
        pub fn len(&self) -> usize {
            match *self {
                Part::Zero(nzeroes) => nzeroes,
                Part::Num(v) => {
                    if v < 1_000 {
                        if v < 10 {
                            1
                        } else if v < 100 {
                            2
                        } else {
                            3
                        }
                    } else if v < 10_000 {
                        4
                    } else {
                        5
                    }
                }
                Part::Copy(buf) => buf.len(),
            }
        }

        /// Writes a part into the supplied buffer.
        /// Returns the number of written bytes, or `None` if the buffer is not enough.
        /// (It may still leave partially written bytes in the buffer; do not rely on that.)
        pub fn write(&self, out: &mut [u8]) -> Option<usize> {
            let len = self.len();
            if out.len() >= len {
                match *self {
                    Part::Zero(nzeroes) => {
                        for c in &mut out[..nzeroes] {
                            *c = b'0';
                        }
                    }
                    Part::Num(mut v) => {
                        for c in out[..len].iter_mut().rev() {
                            *c = b'0' + (v % 10) as u8;
                            v /= 10;
                        }
                    }
                    Part::Copy(buf) => {
                        out[..buf.len()].copy_from_slice(buf);
                    }
                }
                Some(len)
            } else {
                None
            }
        }
    }

    /// Formatted result containing one or more parts.
    /// This can be written to the byte buffer or converted to the allocated string.
    #[allow(missing_debug_implementations)]
    #[derive(Clone)]
    pub struct Formatted<'a> {
        /// A byte slice representing a sign, either `""`, `"-"` or `"+"`.
        pub sign: &'static str,
        /// Formatted parts to be rendered after a sign and optional zero padding.
        pub parts: &'a [Part<'a>],
    }

    impl<'a> Formatted<'a> {
        /// Returns the exact byte length of combined formatted result.
        pub fn len(&self) -> usize {
            let mut len = self.sign.len();
            for part in self.parts {
                len += part.len();
            }
            len
        }

        /// Writes all formatted parts into the supplied buffer.
        /// Returns the number of written bytes, or `None` if the buffer is not enough.
        /// (It may still leave partially written bytes in the buffer; do not rely on that.)
        pub fn write(&self, out: &mut [u8]) -> Option<usize> {
            if out.len() < self.sign.len() {
                return None;
            }
            out[..self.sign.len()].copy_from_slice(self.sign.as_bytes());

            let mut written = self.sign.len();
            for part in self.parts {
                let len = part.write(&mut out[written..])?;
                written += len;
            }
            Some(written)
        }
    }
}

/// Padding after the end of something. Returned by `Formatter::padding`.
#[must_use = "don't forget to write the post padding"]
pub(crate) struct PostPadding {
    fill: char,
    padding: usize,
}

impl PostPadding {
    fn new(fill: char, padding: usize) -> PostPadding {
        PostPadding { fill, padding }
    }

    /// Write this post padding.
    pub(crate) fn write<W: Write, O>(self, f: &mut Formatter<W, O>) -> Result {
        for _ in 0..self.padding {
            f.buf.write_char(self.fill)?;
        }
        Ok(())
    }
}

impl<W: Write, O: FmtOpts> Formatter<W, O> {
    fn pad_integral(&mut self, is_nonnegative: bool, prefix: &str, buf: &str) -> Result {
        let mut width = buf.len();

        let mut sign = None;
        if !is_nonnegative {
            sign = Some('-');
            width += 1;
        } else if self.sign_plus() {
            sign = Some('+');
            width += 1;
        }

        let prefix = if self.alternate() {
            width += prefix.chars().count();
            Some(prefix)
        } else {
            None
        };

        // Writes the sign if it exists, and then the prefix if it was requested
        #[inline(never)]
        fn write_prefix<W: Write, O>(
            f: &mut Formatter<W, O>,
            sign: Option<char>,
            prefix: Option<&str>,
        ) -> Result {
            if let Some(c) = sign {
                f.buf.write_char(c)?;
            }
            if let Some(prefix) = prefix {
                f.buf.write_str(prefix)
            } else {
                Ok(())
            }
        }

        // The `width` field is more of a `min-width` parameter at this point.
        match self.width() {
            // If there's no minimum length requirements then we can just
            // write the bytes.
            None => {
                write_prefix(self, sign, prefix)?;
                self.buf.write_str(buf)
            }
            // Check if we're over the minimum width, if so then we can also
            // just write the bytes.
            Some(min) if width >= min => {
                write_prefix(self, sign, prefix)?;
                self.buf.write_str(buf)
            }
            // The sign and prefix goes before the padding if the fill character
            // is zero
            Some(min) if self.sign_aware_zero_pad() => {
                write_prefix(self, sign, prefix)?;
                let post_padding =
                    self.padding(min - width, Alignment::Right, '0', Alignment::Right)?;
                self.buf.write_str(buf)?;
                post_padding.write(self)?;
                Ok(())
            }
            // Otherwise, the sign and prefix goes after the padding
            Some(min) => {
                let post_padding =
                    self.padding(min - width, Alignment::Right, self.fill(), self.align())?;
                write_prefix(self, sign, prefix)?;
                self.buf.write_str(buf)?;
                post_padding.write(self)
            }
        }
    }

    fn pad_formatted_parts(&mut self, formatted: &numfmt::Formatted<'_>) -> Result {
        // WARN(mono-fmt): This was changed heavily, there might be a bug here
        if let Some(mut width) = self.width() {
            // for the sign-aware zero padding, we render the sign first and
            // behave as if we had no sign from the beginning.
            let mut formatted = formatted.clone();

            let mut the_fill = self.fill();
            let mut the_align = self.align();
            if self.sign_aware_zero_pad() {
                // a sign always goes first
                let sign = formatted.sign;
                self.buf.write_str(sign)?;

                // remove the sign from the formatted parts
                formatted.sign = "";
                width = width.saturating_sub(sign.len());
                the_fill = '0';
                the_align = Alignment::Right;
            }

            // remaining parts go through the ordinary padding process.
            let len = formatted.len();

            if width <= len {
                // no padding
                self.write_formatted_parts(&formatted)
            } else {
                let post_padding = self.padding(width - len, the_align, the_fill, the_align)?;
                self.write_formatted_parts(&formatted)?;
                post_padding.write(self)
            }
        } else {
            // this is the common case and we take a shortcut
            self.write_formatted_parts(formatted)
        }
    }

    pub(crate) fn padding(
        &mut self,
        padding: usize,
        default: Alignment,
        actual_fill: char,
        actual_align: Alignment,
    ) -> std::result::Result<PostPadding, Error> {
        // WARN: We might have `self` in an invalid state, don't touch `self` opts
        let align = match actual_align {
            Alignment::Unknown => default,
            _ => actual_align,
        };

        let (pre_pad, post_pad) = match align {
            Alignment::Left => (0, padding),
            Alignment::Right | Alignment::Unknown => (padding, 0),
            Alignment::Center => (padding / 2, (padding + 1) / 2),
        };

        for _ in 0..pre_pad {
            self.buf.write_char(actual_fill)?;
        }

        Ok(PostPadding::new(actual_fill, post_pad))
    }

    fn write_formatted_parts(&mut self, formatted: &numfmt::Formatted<'_>) -> Result {
        fn write_bytes<W: Write>(buf: &mut W, s: &[u8]) -> Result {
            // SAFETY: This is used for `numfmt::Part::Num` and `numfmt::Part::Copy`.
            // It's safe to use for `numfmt::Part::Num` since every char `c` is between
            // `b'0'` and `b'9'`, which means `s` is valid UTF-8.
            // It's also probably safe in practice to use for `numfmt::Part::Copy(buf)`
            // since `buf` should be plain ASCII, but it's possible for someone to pass
            // in a bad value for `buf` into `numfmt::to_shortest_str` since it is a
            // public function.
            // FIXME: Determine whether this could result in UB.
            buf.write_str(unsafe { std::str::from_utf8_unchecked(s) })
        }

        if !formatted.sign.is_empty() {
            self.buf.write_str(formatted.sign)?;
        }
        for part in formatted.parts {
            match *part {
                numfmt::Part::Zero(mut nzeroes) => {
                    const ZEROES: &str = // 64 zeroes
                        "0000000000000000000000000000000000000000000000000000000000000000";
                    while nzeroes > ZEROES.len() {
                        self.buf.write_str(ZEROES)?;
                        nzeroes -= ZEROES.len();
                    }
                    if nzeroes > 0 {
                        self.buf.write_str(&ZEROES[..nzeroes])?;
                    }
                }
                numfmt::Part::Num(mut v) => {
                    let mut s = [0; 5];
                    let len = part.len();
                    for c in s[..len].iter_mut().rev() {
                        *c = b'0' + (v % 10) as u8;
                        v /= 10;
                    }
                    write_bytes(&mut self.buf, &s[..len])?;
                }
                numfmt::Part::Copy(buf) => {
                    write_bytes(&mut self.buf, buf)?;
                }
            }
        }
        Ok(())
    }
}
