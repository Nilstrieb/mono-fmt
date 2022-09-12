use crate::Formatter;

pub enum Alignment {
    Left,
    Center,
    Right,
}

pub trait FmtOpts {
    #[doc(hidden)]
    type Inner: FmtOpts;

    fn alternate() -> bool {
        Self::Inner::alternate()
    }

    fn width() -> Option<usize> {
        Self::Inner::width()
    }

    fn align() -> Option<Alignment> {
        Self::Inner::align()
    }

    fn fill() -> Option<char> {
        Self::Inner::fill()
    }
}

mod never {
    use crate::FmtOpts;

    pub trait Func {
        type Output;
    }

    impl<T> Func for fn() -> T {
        type Output = T;
    }

    pub type Never = <fn() -> ! as Func>::Output;

    impl FmtOpts for Never {
        type Inner = Self;
    }
}

impl FmtOpts for () {
    type Inner = never::Never;

    fn alternate() -> bool {
        false
    }

    fn width() -> Option<usize> {
        None
    }

    fn align() -> Option<Alignment> {
        None
    }

    fn fill() -> Option<char> {
        None
    }
}
pub struct WithAlternate<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithAlternate<I> {
    type Inner = I;
    fn alternate() -> bool {
        true
    }
}
pub struct WithWidth<I, const A: usize>(pub I);

impl<I: FmtOpts, const A: usize> FmtOpts for WithWidth<I, A> {
    type Inner = I;
    fn width() -> Option<usize> {
        Some(A)
    }
}
pub struct WithLeftAlign<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithLeftAlign<I> {
    type Inner = I;
    fn align() -> Option<Alignment> {
        Some(Alignment::Left)
    }
}
pub struct WithRightAlign<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithRightAlign<I> {
    type Inner = I;
    fn align() -> Option<Alignment> {
        Some(Alignment::Right)
    }
}
pub struct WithCenterAlign<I>(pub I);

impl<I: FmtOpts> FmtOpts for WithCenterAlign<I> {
    type Inner = I;
    fn align() -> Option<Alignment> {
        Some(Alignment::Center)
    }
}
pub struct WithFill<I, const A: char>(pub I);

impl<I: FmtOpts, const A: char> FmtOpts for WithFill<I, A> {
    type Inner = I;
    fn fill() -> Option<char> {
        Some(A)
    }
}

impl<W, O: FmtOpts> Formatter<W, O> {
    pub fn alternate(&self) -> bool {
        O::alternate()
    }

    pub fn width() -> Option<usize> {
        O::width()
    }

    pub fn align() -> Option<Alignment> {
        O::align()
    }

    pub fn fill() -> Option<char> {
        O::fill()
    }

    pub fn with_alternate(self) -> Formatter<W, WithAlternate<O>> {
        Formatter {
            buf: self.buf,
            opts: WithAlternate(self.opts),
        }
    }

    pub fn with_width<const WIDTH: usize>(self) -> Formatter<W, WithWidth<O, WIDTH>> {
        Formatter {
            buf: self.buf,
            opts: WithWidth(self.opts),
        }
    }

    pub fn with_left_align(self) -> Formatter<W, WithLeftAlign<O>> {
        Formatter {
            buf: self.buf,
            opts: WithLeftAlign(self.opts),
        }
    }

    pub fn with_right_align(self) -> Formatter<W, WithRightAlign<O>> {
        Formatter {
            buf: self.buf,
            opts: WithRightAlign(self.opts),
        }
    }

    pub fn with_center_align(self) -> Formatter<W, WithCenterAlign<O>> {
        Formatter {
            buf: self.buf,
            opts: WithCenterAlign(self.opts),
        }
    }

    pub fn with_fill<const FILL: char>(self) -> Formatter<W, WithFill<O, FILL>> {
        Formatter {
            buf: self.buf,
            opts: WithFill(self.opts),
        }
    }
}
