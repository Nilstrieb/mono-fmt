//! A bunch of impls copied from all over the place

use impl_prelude::*;

mod impl_prelude {
    pub use crate::*;
}

impl<T: Debug, const N: usize> Debug for [T; N] {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        <[T] as Debug>::fmt(&self[..], f)
    }
}

impl<T: Debug> Debug for [T] {
    fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

// pointers
mod pointers {
    use super::impl_prelude::*;

    impl<T: ?Sized> Pointer for *const T {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            pointer_fmt_inner((*self as *const ()) as usize, f)
        }
    }

    impl<T: ?Sized> Pointer for *mut T {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            pointer_fmt_inner((*self as *mut ()) as usize, f)
        }
    }

    impl<T: ?Sized> Pointer for &T {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            Pointer::fmt(&(*self as *const T), f)
        }
    }

    impl<T: ?Sized> Pointer for &mut T {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            Pointer::fmt(&(&**self as *const T), f)
        }
    }

    pub(crate) fn pointer_fmt_inner<W: Write, O: FmtOpts>(
        ptr_addr: usize,
        f: &mut Formatter<W, O>,
    ) -> Result {
        fn tail<W: Write, O: FmtOpts>(f: &mut Formatter<W, O>, ptr_addr: usize) -> Result {
            let mut f = f.wrap_with(&crate::opts::WithAlternate(()));
            LowerHex::fmt(&ptr_addr, &mut f)
        }

        // The alternate flag is already treated by LowerHex as being special-
        // it denotes whether to prefix with 0x. We use it to work out whether
        // or not to zero extend, and then unconditionally set it to get the
        // prefix.
        if f.alternate() {
            let mut f = f.wrap_with(&crate::opts::WithSignAwareZeroPad(()));

            if f.width().is_none() {
                const WIDTH: usize = (usize::BITS / 4) as usize + 2;

                let mut f = f.wrap_with(&crate::opts::WithWidth::<(), WIDTH>(()));

                tail(&mut f, ptr_addr)
            } else {
                tail(&mut f, ptr_addr)
            }
        } else {
            tail(f, ptr_addr)
        }
    }
}

mod char {
    use super::impl_prelude::*;

    impl Display for char {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            if f.width().is_none() && f.precision().is_none() {
                f.write_char(*self)
            } else {
                f.pad(self.encode_utf8(&mut [0; 4]))
            }
        }
    }
}

mod strings {
    #[cfg(feature = "alloc")]
    use alloc::string::String;

    use super::impl_prelude::*;

    #[cfg(feature = "alloc")]
    impl Debug for String {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            f.write_char('"')?;
            f.write_str(self)?;
            f.write_char('"')
        }
    }

    #[cfg(feature = "alloc")]
    impl Display for String {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            f.write_str(self)
        }
    }

    impl Display for str {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            f.write_str(self)
        }
    }

    impl Debug for str {
        fn fmt<W: Write, O: FmtOpts>(&self, f: &mut Formatter<W, O>) -> Result {
            f.write_char('"')?;
            f.write_str(self)?;
            f.write_char('"')
        }
    }
}
