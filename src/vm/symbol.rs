use vm::ValueIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    /// A function symbol.
    Function(ValueIndex),

    /// A constant-value symbol.
    Constant(ValueIndex),

    /// A variable symbol.
    Variable(ValueIndex),
}

impl Symbol {
    pub fn index(&self) -> usize {
        match self {
            | Symbol::Function(i)
            | Symbol::Constant(i)
            | Symbol::Variable(i) => *i
        }
    }
}
