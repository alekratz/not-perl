use std::{
    cmp::Ordering,
    fmt::{self, Formatter, Display, Debug},
    ops::Deref,
    sync::Arc,
};

/// A position in a character stream.
#[derive(Clone)]
#[cfg_attr(not(test), derive(PartialEq))]
pub struct Pos {
    pub source: usize,
    pub line: usize,
    pub col: usize,
    pub source_name: Arc<String>,
    pub source_text: Arc<String>,
}

impl Debug for Pos {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("Pos")
            .field("source", &self.source)
            .field("line", &self.line)
            .field("col", &self.col)
            .field("source_name", &self.source_name)
            .finish()
    }
}

impl Pos {
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

    pub fn new(source_name: Arc<String>, source_text: Arc<String>) -> Self {
        Pos {
            source_name,
            source_text,
            ..Default::default()
        }
    }

    pub fn max(self, other: Pos) -> Self {
        match self.line.cmp(&other.line) {
            Ordering::Less => other,
            Ordering::Equal => match self.col.cmp(&other.col) {
                Ordering::Less => other,
                _ => self
            }
            Ordering::Greater => self,
        }
    }

    pub fn min(self, other: Pos) -> Self {
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

impl Display for Pos {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.line + 1, self.col + 1)
    }
}

impl Default for Pos {
    fn default() -> Self {
        Pos {
            source: 0,
            line: 0,
            col: 0,
            source_name: Arc::new(String::new()),
            source_text: Arc::new(String::new()),
        }
    }
}

impl PartialOrd for Pos {
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
impl PartialEq for Pos {
    fn eq(&self, _other: &Self) -> bool { true }
}

#[derive(Clone, Debug)]
#[cfg_attr(not(test), derive(PartialEq))]
pub struct Range(Pos, Pos);

impl Range {
    pub fn new(start: Pos, end: Pos) -> Self {
        Range(start, end)
    }

    pub fn start(&self) -> Pos {
        self.0.clone()
    }

    pub fn end(&self) -> Pos {
        self.1.clone()
    }

    pub fn union(&self, other: &Range) -> Self {
        let start = self.start().min(other.start());
        let end = self.end().max(other.end());
        Range(start, end)
    }
}

impl Display for Range {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        let start_source = self.start().source;
        let end_source = self.end().source;
        write!(fmt, "{}", &self.start().source_text[start_source .. end_source])
    }
}

#[derive(Clone, Debug)]
pub struct RangeWrapper<T>(pub Range, pub T)
    where T: Sized + Clone + Debug;

impl<T> RangeWrapper<T>
    where T: Sized + Clone + Debug
{
    /// Makes a new ranged value.
    ///
    /// # Arguments
    /// * `range` - the range that the object takes up space in.
    /// * `value` - the wrapped value.
    pub fn new(range: Range, value: T) -> Self {
        RangeWrapper(range, value)
    }

    pub fn map<Out>(&self, mapfn: impl FnOnce(&T) -> Out) -> RangeWrapper<Out>
        where Out: Clone + Debug
    {
        let RangeWrapper(range, ref inner) = self;
        let inner = (mapfn)(inner);
        RangeWrapper(range.clone(), inner)
    }

    pub fn into_inner(self) -> T {
        self.1
    }

    pub fn as_inner(&self) -> &T {
        &self.1
    }
}

impl<T> Deref for RangeWrapper<T>
    where T: Sized + Clone + Debug
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T> PartialEq for RangeWrapper<T>
    where T: Sized + Clone + Debug + PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}

impl<T> Ranged for RangeWrapper<T>
    where T: Sized + Clone + Debug
{
    fn range(&self) -> Range {
        self.0.clone()
    }
}

pub trait Ranged: Clone + Debug {
    fn range(&self) -> Range;
}

#[macro_export]
macro_rules! impl_ranged {
    ($ty:ident :: $member:tt) => {
        impl $crate::common::pos::Ranged for $ty  {
            fn range(&self) -> Range {
                self.$member.clone()
            }
        }
    };
}

