use std::{
    result,
    fmt::{self, Formatter, Display},
};
use crate::common::pos::Pos;

#[derive(Debug, Clone)]
pub struct SyntaxError {
    reason: String,
    location: Pos,
}

impl SyntaxError {
    pub fn new(reason: String, location: Pos) -> Self {
        SyntaxError {
            reason,
            location,
        }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{} : {}", self.location, self.reason)
    }
}

/// An alias for a `SyntaxError`.
pub type Error = SyntaxError;

/// An alias for a result with an error type of `SyntaxError`.
pub type Result<T> = result::Result<T, SyntaxError>;
