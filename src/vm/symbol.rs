use vm::ValueIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    /// A function symbol.
    Function(ValueIndex),

    /// A constant-value symbol.
    Constant(ValueIndex),

    /// A variable symbol, with both a global and local index.
    Variable(ValueIndex, ValueIndex),
}

impl Symbol {
    pub fn index(&self) -> ValueIndex {
        match self {
            | Symbol::Function(i)
            | Symbol::Constant(i)
            | Symbol::Variable(i, _) => *i
        }
    }

    pub fn local_index(&self) -> ValueIndex {
        match self {
            | Symbol::Function(i)
            | Symbol::Constant(i)
            | Symbol::Variable(_, i) => *i
        }
    }
}
