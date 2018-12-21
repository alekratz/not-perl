use std::{
    io,
};
use failure::Fail;
use crate::syntax;

/// An error type that occurs as a result of processing a piece of code.
///
/// This may be anything from an I/O error when attempting to read a file, to a compilation error.
#[derive(Fail, Debug)]
pub enum ProcessError {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "{}", _0)]
    Syntax(#[cause] syntax::Error),
}

impl From<io::Error> for ProcessError {
    fn from(other: io::Error) -> Self {
        ProcessError::Io(other)
    }
}

impl From<syntax::Error> for ProcessError {
    fn from(other: syntax::Error) -> Self {
        ProcessError::Syntax(other)
    }
}

pub type Error = ProcessError;
pub type Result<T> = ::std::result::Result<T, Error>;
