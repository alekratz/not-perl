use std::io;
use failure::Fail;
use crate::{compile, syntax, vm};

pub mod scope;
pub mod strings;
pub mod value;
pub mod lang;
#[macro_use] pub mod pos;

pub mod prelude {
    pub use super::lang::*;
    pub use super::pos::*;
    pub use super::scope::*;
}

/// An error type that occurs as a result of processing a piece of code.
///
/// This may be anything from an I/O error when attempting to read a file, to a compilation error.
#[derive(Fail, Debug)]
pub enum ProcessError {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "{}", _0)]
    Compile(#[cause] compile::Error),
    #[fail(display = "{}", _0)]
    Syntax(#[cause] syntax::Error),
    #[fail(display = "{}", _0)]
    Vm(#[cause] vm::Error),
}

impl From<io::Error> for ProcessError {
    fn from(other: io::Error) -> Self { ProcessError::Io(other) }
}

impl From<compile::Error> for ProcessError {
    fn from(other: compile::Error) -> Self { ProcessError::Compile(other) }
}

impl From<syntax::Error> for ProcessError {
    fn from(other: syntax::Error) -> Self { ProcessError::Syntax(other) }
}

impl From<vm::Error> for ProcessError {
    fn from(other: vm::Error) -> Self { ProcessError::Vm(other) }
}

