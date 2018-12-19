use crate::syntax;
use failure::Fail;
use std::io;

pub mod lang;
pub mod strings;
pub mod value;
#[macro_use]
pub mod pos;

pub mod prelude {
    pub use super::lang::*;
    pub use super::pos::*;
}

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
