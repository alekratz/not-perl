use vm::{BlockSymbol, Symbolic, Symbol};

pub type LabelIndex = usize;

/// A VM label, which points to a location in code.
#[derive(Debug, Clone)]
pub struct Label {
    /// The symbol that describes this label.
    pub symbol: BlockSymbol,

    /// The index in the current function that this label is pointing at.
    pub pc: LabelIndex,

    /// The name for this label.
    pub name: String,
}

impl Label {
    pub fn new(symbol: BlockSymbol, pc: LabelIndex) -> Self {
        Label {
            symbol,
            pc,
            name: format!("Label#{:x}:{:x}", symbol.index(), pc),
        }
    }
}

impl Symbolic for Label {
    type Symbol = BlockSymbol;
    fn symbol(&self) -> BlockSymbol {
        self.symbol
    }

    fn name(&self) -> &str {
        &self.name
    }
}
