use std::{
    result,
    fmt::{self, Formatter, Display},
};
use syntax::Pos;

#[derive(Debug, Clone)]
pub struct SyntaxError<'n> {
    reason: String,
    location: Pos<'n>,
}

impl<'n> SyntaxError<'n> {
    pub fn new(reason: String, location: Pos<'n>) -> Self {
        SyntaxError {
            reason,
            location,
        }
    }
}

impl<'n> Display for SyntaxError<'n> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{} : {}", self.location, self.reason)
    }
}

/// An alias for a `SyntaxError`.
pub type Error<'n> = SyntaxError<'n>;

/// An alias for a result with an error type of `SyntaxError`.
pub type Result<'n, T> = result::Result<T, SyntaxError<'n>>;
