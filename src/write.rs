use crate::{Debug, Display, FmtOpts, Formatter, Result, Write};

impl Debug for &'_ str {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        f.write_char('"')?;
        f.write_str(self)?;
        f.write_char('"')
    }
}

impl Display for &'_ str {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        f.write_str(self)
    }
}

impl Debug for String {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        f.write_char('"')?;
        f.write_str(self)?;
        f.write_char('"')
    }
}

impl Display for String {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        f.write_str(self)
    }
}

macro_rules! naive_fmt {
    ($($int:ty)*) => {
        $(
            impl Debug for $int {
                fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
                    // FIXME lol
                    let string = format!("{:?}", self);
                    f.write_str(&string)
                }
            }

            impl Display for $int {
                fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
                    // FIXME lol
                    let string = self.to_string();
                    f.write_str(&string)
                }
            }
        )*
    };
}

naive_fmt!(
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize

    f32 f64
);
