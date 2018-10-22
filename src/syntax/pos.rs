use std::{
    cmp::Ordering,
    fmt::{self, Formatter, Display, Debug},
    ops::Deref,
};

/// A position in a character stream.
#[derive(Clone, Copy)]
#[cfg_attr(not(test), derive(PartialEq))]
pub struct Pos<'n> {
    pub source: usize,
    pub line: usize,
    pub col: usize,
    pub source_name: Option<&'n str>,
    pub source_text: &'n str,
}

impl<'n> Debug for Pos<'n> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Pos")
            .field("source", &self.source)
            .field("line", &self.line)
            .field("col", &self.col)
            .field("source_name", &self.source_name)
            .finish()
    }
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

    pub fn new(source_name: Option<&'n str>, source_text: &'n str) -> Self {
        Pos {
            source_name,
            source_text,
            ..Default::default()
        }
    }

    pub fn max(self, other: Pos<'n>) -> Self {
        match self.line.cmp(&other.line) {
            Ordering::Less => other,
            Ordering::Equal => match self.col.cmp(&other.col) {
                Ordering::Less => other,
                _ => self
            }
            Ordering::Greater => self,
        }
    }

    pub fn min(self, other: Pos<'n>) -> Self {
        match self.line.cmp(&other.line) {
            Ordering::Greater => other,
            Ordering::Equal => match self.col.cmp(&other.col) {
                Ordering::Greater => other,
                _ => self
            }
            Ordering::Less => self,
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
            source_text: "",
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

    pub fn union(&self, other: &Range<'n>) -> Self {
        let start = self.start().min(other.start());
        let end = self.end().max(other.end());
        Range(start, end)
    }

    pub fn text(&self) -> &str {
        let start_source = self.start().source;
        let end_source = self.end().source;
        &self.start().source_text[start_source .. end_source]
    }
}

#[derive(Clone, Debug)]
pub struct Ranged<'n, T>(pub Range<'n>, pub T)
    where T: Sized + Clone + Debug;

impl<'n, T> Ranged<'n, T>
    where T: Sized + Clone + Debug
{
    /// Makes a new ranged value.
    ///
    /// # Arguments
    /// * `range` - the range that the object takes up space in.
    /// * `value` - the wrapped value.
    pub fn new(range: Range<'n>, value: T) -> Self {
        Ranged(range, value)
    }

    pub fn map<Out>(&self, mapfn: impl FnOnce(&T) -> Out) -> Ranged<'n, Out>
        where Out: Clone + Debug
    {
        let Ranged(range, ref inner) = self;
        let inner = (mapfn)(inner);
        Ranged(*range, inner)
    }

    pub fn into_inner(self) -> T {
        self.1
    }

    pub fn as_inner(&self) -> &T {
        &self.1
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
