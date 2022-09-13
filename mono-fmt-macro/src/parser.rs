use std::str::Chars;

use peekmore::PeekMoreIterator;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Alignment {
    fn from_char(char: char) -> Result<Self> {
        match char {
            '<' => Ok(Self::Left),
            '^' => Ok(Self::Center),
            '>' => Ok(Self::Right),
            _ => Err(Error::new(format!("Invalid alignment specifier {char}"))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Align {
    kind: Alignment,
    fill: Option<char>,
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

#[derive(Debug, PartialEq)]
enum Precision {
    Num(usize),
    Asterisk,
}

#[derive(Debug, PartialEq, Default)]
pub struct FmtSpec {
    arg: Argument,
    align: Option<Align>,
    sign: Option<char>,
    alternate: bool,
    zero: bool,
    width: Option<usize>,
    precision: Option<Precision>,
    kind: FmtType,
}

pub struct FmtSpecParser<'a, 'b> {
    chars: &'a mut PeekMoreIterator<Chars<'b>>,
    /// The last state of the parser.
    state: State,
    argument: FmtSpec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Initial,
    Argument,
    // : here
    Align,
    Sign,
    Alternate,
    Zero,
    Width,
    Precision,
    Done,
}

impl<'a, 'b> FmtSpecParser<'a, 'b> {
    pub fn new(chars: &'a mut PeekMoreIterator<Chars<'b>>) -> Self {
        Self {
            chars,
            state: State::Initial,
            argument: FmtSpec::default(),
        }
    }

    pub fn parse(mut self) -> Result<FmtSpec> {
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

    fn expect(&mut self, char: char) -> Result<()> {
        if !self.eat(char) {
            return Err(Error::new(format!(
                "Expected {char}, found {}",
                self.peek()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "end of input".to_string())
            )));
        }
        Ok(())
    }

    fn eat_until(&mut self, should_stop: impl Fn(char) -> bool) -> Option<String> {
        let mut string = String::new();
        let mut has_char = false;
        // let_chains would be neat here
        while self.peek().is_some() && !should_stop(self.peek().unwrap()) {
            let next = self.next().unwrap();
            string.push(next);
            has_char = true;
        }
        has_char.then_some(string)
    }

    fn step(&mut self) -> Result<()> {
        match self.state {
            State::Initial => {
                let argument = if let Some(arg) = self.eat_until(|c| matches!(c, ':' | '}')) {
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

                if self.argument.arg != Argument::Positional {
                    self.expect(':')?;
                }
            }
            State::Argument => match self
                .peek()
                .ok_or_else(|| Error::new("unexpected end of input".to_string()))?
            {
                c @ ('>' | '^' | '<') => {
                    self.next();
                    self.argument.align = Some(Align {
                        kind: Alignment::from_char(c)?,
                        fill: None,
                    });
                    self.state = State::Align;
                }
                other => {
                    // peek2
                    if let Some(c @ ('>' | '^' | '<')) = self.peek() {
                        self.argument.align = Some(Align {
                            kind: Alignment::from_char(c).unwrap(),
                            fill: Some(other),
                        });
                    }

                    self.state = State::Align;
                }
            },
            State::Align => {
                if let Some(c @ ('+' | '-')) = self.peek() {
                    self.next();
                    self.argument.sign = Some(c);
                }
                self.state = State::Sign;
            }
            State::Sign => {
                if self.eat('#') {
                    self.argument.alternate = true;
                }
                self.state = State::Alternate;
            }
            State::Alternate => {
                if self.eat('0') {
                    self.argument.zero = true;
                }
                self.state = State::Zero;
            }
            State::Zero => {
                if let Some(width) = self.eat_until(|c| !c.is_ascii_digit()) {
                    let width = width
                        .parse()
                        .map_err(|_| Error::new("width specified too long".to_string()))?;
                    self.argument.width = Some(width);
                }
                self.state = State::Width;
            }
            State::Width => {
                if self.eat('.') {
                    if let Some(precision) = self.eat_until(|c| c != '*' && !c.is_ascii_digit()) {
                        let precision = if precision == "*" {
                            Precision::Asterisk
                        } else {
                            Precision::Num(precision.parse().map_err(|_| {
                                Error::new("precision specified too long".to_string())
                            })?)
                        };
                        self.argument.precision = Some(precision);
                    }
                }
                self.state = State::Precision;
            }
            State::Precision => match self.next() {
                Some('?') => {
                    self.argument.kind = FmtType::Debug;
                    self.expect('}')?;
                }
                Some('x') => {
                    self.expect('?')?;
                    self.argument.kind = FmtType::LowerHex;
                    self.expect('}')?;
                }
                Some('X') => {
                    self.expect('?')?;
                    self.argument.kind = FmtType::UpperHex;
                    self.expect('}')?;
                }
                Some('}') | None => {}
                Some(other) => {
                    if let Some(kind) = self.eat_until(|c| c == '}') {
                        self.argument.kind = FmtType::Other(format!("{other}{kind}"));
                        self.expect('}')?;
                    } else {
                        self.argument.kind = FmtType::Default;
                        self.expect('}')?;
                    }
                }
            },
            State::Done => unreachable!(),
        }

        Ok(())
    }
}
