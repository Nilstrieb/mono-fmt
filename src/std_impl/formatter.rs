use crate::{FmtOpts, Formatter, Result, Write};

impl<W: Write, O: FmtOpts> Formatter<W, O> {
    pub fn pad_integral(&mut self, is_nonnegative: bool, prefix: &str, buf: &str) -> Result {
        todo!()
    }
}
