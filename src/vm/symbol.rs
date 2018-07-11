use vm::ValueIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// A function symbol.
    Function(ValueIndex, String),

    /// A constant-value symbol.
    Constant(ValueIndex, String),

    /// A variable symbol.
    Variable(ValueIndex, String),
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            | Symbol::Function(_, s)
            | Symbol::Constant(_, s)
            | Symbol::Variable(_, s) => s
        }
    }

    pub fn index(&self) -> usize {
        match self {
            | Symbol::Function(i, _)
            | Symbol::Constant(i, _)
            | Symbol::Variable(i, _) => *i
        }
    }
}
