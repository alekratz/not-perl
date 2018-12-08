use crate::common::pos::Pos;
use failure::{Backtrace, Context, Fail};
use std::{
    fmt::{self, Display, Formatter},
    result,
};

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "unexpected {}", _0)]
    Unexpected(String),

    #[fail(display = "expected {}; got {} at {}", _0, _1, _2)]
    ExpectedGot(String, String, Pos),

    #[fail(display = "reached {} while insode of string literal", _0)]
    EarlyStringEnd(String),

    #[fail(display = "{}", _0)]
    Message(String),
}

#[derive(Debug)]
pub struct Error
where
    ErrorKind: 'static,
{
    pos: Pos,
    kind: Context<ErrorKind>,
}

impl Error {
    pub fn new(pos: Pos, kind: ErrorKind) -> Self {
        Error {
            pos,
            kind: Context::new(kind),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        self.kind.get_context()
    }

    pub fn pos(&self) -> Pos {
        self.pos.clone()
    }
}

impl Fail for Error
where
    Self: 'static,
{
    fn cause(&self) -> Option<&Fail> {
        self.kind.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.kind.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.kind, fmt)
    }
}

// An alias for a `SyntaxError`.
//pub type Error = SyntaxError;

/// An alias for a result with an error type of `Error`.
pub type Result<T> = result::Result<T, Error>;
