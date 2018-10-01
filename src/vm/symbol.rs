use vm::ValueIndex;

pub trait SymbolIndex {
    #[deprecated]
    fn index(&self) -> ValueIndex;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FunctionSymbol(pub ValueIndex);

impl SymbolIndex for FunctionSymbol {
    fn index(&self) -> ValueIndex {
        self.0
    }
}

pub struct ConstantSymbol(pub ValueIndex);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VariableSymbol {
    pub global: ValueIndex,
    pub local: ValueIndex,
}

impl SymbolIndex for VariableSymbol {
    fn index(&self) -> ValueIndex {
        self.global
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TySymbol(pub ValueIndex);

impl SymbolIndex for TySymbol {
    fn index(&self) -> ValueIndex {
        self.0
    }
}
