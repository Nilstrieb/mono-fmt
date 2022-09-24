//! A bunch of impls copied from all over the place

use crate::{Debug, FmtOpts, Formatter, Result, Write};

impl<T: Debug + ?Sized> Debug for &T {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        <T as Debug>::fmt(&self, f)
    }
}

impl<T: Debug, const N: usize> Debug for [T; N] {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        <[T] as Debug>::fmt(&&self[..], f)
    }
}

impl<T: Debug> Debug for [T] {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
