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

pub struct FmtSpecParser<'a> {
    chars: &'a mut Peekable<Chars<'a>>,
    state: State,
    argument: FmtSpec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn parse(mut self) -> Result<FmtSpec, ()> {
        while self.state != State::Done {
            self.step()?;
        }
        Ok(self.argument)
    }

    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn eat(&mut self, char: char) -> bool {
        if self.peek() == Some(char) {
            self.next();
            return true;
        }
        false
    }

    fn eat_until(&mut self, char: char) -> Option<String> {
        let mut string = String::new();
        let mut has_char = false;
        while self.peek() != Some(char) {
            self.next();
            string.push(char);
            has_char = true;
        }
        has_char.then_some(string)
    }

    fn step(&mut self) -> Result<(), ()> {
        match self.state {
            State::Initial => {
                let argument = if let Some(arg) = self.eat_until(':') {
                    if let Ok(num) = arg.parse() {
                        Argument::PositionalExplicit(num)
                    } else {
                        Argument::Keyword(arg)
                    }
                } else {
                    Argument::Positional
                };

                self.argument.arg = argument;
                self.state = State::Argument;

                if !self.eat(':') {
                    return Err(());
                }

                Ok(())
            }
            State::Argument => todo!(),
            State::Fill => todo!(),
            State::Align => todo!(),
            State::Sign => todo!(),
            State::Zero => todo!(),
            State::Width => todo!(),
            State::Precision => todo!(),
            State::Type => todo!(),
            State::Done => unreachable!(),
        }
    }
}
