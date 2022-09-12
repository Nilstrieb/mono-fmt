use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Default)]
pub struct Align {
    amount: usize,
    char: Option<char>,
}

#[derive(Debug, PartialEq, Default)]
pub enum Argument {
    #[default]
    Positional,
    PositionalExplicit(usize),
    Keyword(String),
}

#[derive(Debug, PartialEq, Default)]
pub enum FmtType {
    #[default]
    Default,
    Debug,
    LowerHex,
    UpperHex,
    Other(String),
}

#[derive(Debug, PartialEq, Default)]
pub struct FmtSpec {
    arg: Argument,
    align: Option<Align>,
    sign: Option<char>,
    alternate: bool,
    zero: bool,
    width: Option<usize>,
    precision: Option<usize>,
    kind: FmtType,
}

struct FmtSpecParser<'a> {
    chars: &'a mut Peekable<Chars<'a>>,
    state: State,
    argument: FmtSpec,
}

#[derive(PartialEq)]
enum State {
    Initial,
    Argument,
    // : here
    Fill,
    Align,
    Sign,
    Zero,
    Width,
    Precision,
    Type,
    Done,
}

impl<'a> FmtSpecParser<'a> {
    pub fn new(chars: &'a mut Peekable<Chars<'a>>) -> Self {
        Self {
            chars,
            state: State::Initial,
            argument: FmtSpec::default(),
        }
    }

    fn parse(mut self) -> Result<FmtSpec, ()> {
        while self.state != State::Done {
            self.step()?;
        }
        Ok(self.argument)
    }

    fn step(&mut self) -> Result<(), ()> {
        todo!()
    }
}
