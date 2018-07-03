use std::{
    cmp::Ordering,
    fmt::{self, Formatter, Display, Debug},
    ops::Deref,
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
    fn eq(&self, _other: &Self) -> bool { true }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(not(test), derive(PartialEq))]
pub struct Range<'n>(Pos<'n>, Pos<'n>);

impl<'n> Range<'n> {
    pub fn new(start: Pos<'n>, end: Pos<'n>) -> Self {
        Range(start, end)
    }

    pub fn start(&self) -> Pos<'n> {
        self.0
    }

    pub fn end(&self) -> Pos<'n> {
        self.1
    }
}

#[derive(Clone, Debug)]
pub struct Ranged<'n, T>(Range<'n>, pub (in syntax) T)
    where T: Sized + Clone + Debug;

impl<'n, T> Ranged<'n, T>
    where T: Sized + Clone + Debug
{
    pub fn new(range: Range<'n>, value: T) -> Self {
        Ranged(range, value)
    }
}

impl<'n, T> Deref for Ranged<'n, T>
    where T: Sized + Clone + Debug
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<'n, T> PartialEq for Ranged<'n, T>
    where T: Sized + Clone + Debug + PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}
