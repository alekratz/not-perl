use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
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

    pub fn max<'n>(&'n self, other: &'n Pos) -> &'n Self {
        match self.line.cmp(&other.line) {
            Ordering::Less => other,
            Ordering::Equal => match self.col.cmp(&other.col) {
                Ordering::Less => other,
                _ => self,
            },
            Ordering::Greater => self,
        }
    }

    pub fn min<'n>(&'n self, other: &'n Pos) -> &'n Self {
        match self.line.cmp(&other.line) {
            Ordering::Greater => other,
            Ordering::Equal => match self.col.cmp(&other.col) {
                Ordering::Greater => other,
                _ => self,
            },
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
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SrcRange(Pos, Pos);

impl SrcRange {
    pub fn new(start: Pos, end: Pos) -> Self {
        if start < end {
            SrcRange(start, end)
        } else {
            SrcRange(end, start)
        }
    }

    pub fn start(&self) -> &Pos {
        &self.0
    }

    pub fn end(&self) -> &Pos {
        &self.1
    }

    pub fn union(&self, other: &SrcRange) -> Self {
        let start = self.start().min(other.start());
        let end = self.end().max(other.end());
        SrcRange(start.clone(), end.clone())
    }

    pub fn source_text(&self) -> &str {
        let start_source = self.start().source;
        let end_source = self.end().source;
        let start = &self.0;
        &start.source_text[start_source..end_source]
    }

    pub fn source_name(&self) -> &str {
        let start = &self.0;
        &start.source_name
    }
}

impl Display for SrcRange {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        if self.0.line == self.1.line {
            write!(fmt, "{}:{} - {}", self.0.line, self.0.col, self.1.col)
        } else {
            write!(
                fmt,
                "{}:{} - {}:{}",
                self.0.line, self.0.col, self.1.line, self.1.col
            )
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Range {
    Src(SrcRange),
    Builtin,
}

impl Range {
    pub fn source_text(&self) -> &str {
        match self {
            Range::Src(range) => range.source_text(),
            Range::Builtin => "<builtin>",
        }
    }

    pub fn source_name(&self) -> &str {
        match self {
            Range::Src(range) => range.source_name(),
            Range::Builtin => "<builtin>",
        }
    }

    pub fn union(&self, other: &Range) -> Self {
        match (self, other) {
            (Range::Builtin, _) | (_, Range::Builtin) => Range::Builtin,
            (Range::Src(first), Range::Src(second)) => Range::Src(first.union(second)),
        }
    }
}

impl Display for Range {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Range::Src(s) => Display::fmt(s, fmt),
            Range::Builtin => write!(fmt, "<builtin>"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RangeWrapper<T>(pub Range, pub T)
where
    T: Sized + Clone + Debug;

impl<T> RangeWrapper<T>
where
    T: Sized + Clone + Debug,
{
    /// Makes a new ranged value.
    ///
    /// # Arguments
    /// * `range` - the range that the object takes up space in.
    /// * `value` - the wrapped value.
    pub fn new(range: Range, value: T) -> Self {
        RangeWrapper(range, value)
    }

    /// Maps the wrapped value to another value.
    pub fn map<Out>(&self, mapfn: impl FnOnce(&T) -> Out) -> RangeWrapper<Out>
    where
        Out: Clone + Debug,
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
where
    T: Sized + Clone + Debug,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T> PartialEq for RangeWrapper<T>
where
    T: Sized + Clone + Debug + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}

impl<T> Ranged for RangeWrapper<T>
where
    T: Sized + Clone + Debug,
{
    fn range(&self) -> Range {
        self.0.clone()
    }
}

pub trait Ranged: Debug {
    fn range(&self) -> Range;
}

#[macro_export]
macro_rules! impl_ranged {
    ($ty:ident :: $member:tt) => {
        impl $crate::common::pos::Ranged for $ty {
            fn range(&self) -> $crate::common::pos::Range {
                self.$member.clone()
            }
        }
    };
}
