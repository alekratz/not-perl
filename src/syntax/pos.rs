use std::{
    cmp::Ordering,
    fmt::{self, Formatter, Display},
};

/// A position in a character stream.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(not(test), derive(PartialEq))]
pub struct Pos<'n> {
    pub source: usize,
    pub line: usize,
    pub col: usize,
    pub source_name: Option<&'n str>,
}

impl<'n> Pos<'n> {
    /// Increments the source index and the column index.
    pub fn adv(&mut self) {
        self.source += 1;
        self.col += 1;
    }

    /// Resets the column index, and increments the line index.
    pub fn line(&mut self) {
        self.line += 1;
        self.col = 0;
    }

    pub fn new(source_name: Option<&'n str>) -> Self {
        Pos {
            source_name,
            ..Default::default()
        }
    }
}

impl<'n> Display for Pos<'n> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.line + 1, self.col + 1)
    }
}

impl<'n> Default for Pos<'n> {
    fn default() -> Self {
        Pos {
            source: 0,
            line: 0,
            col: 0,
            source_name: None,
        }
    }
}

impl<'n> PartialOrd for Pos<'n> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.source_name != other.source_name {
            None
        } else {
            self.source.partial_cmp(&other.source)
        }
    }
}


// Pos is only equal during testing
#[cfg(test)]
impl<'n> PartialEq for Pos<'n> {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}
