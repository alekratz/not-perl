use vm::{
    Symbolic,
    VariableSymbol,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable(pub String, pub VariableSymbol);

impl Symbolic for Variable {
    type Symbol = VariableSymbol;

    fn name(&self) -> &str {
        &self.0
    }

    fn symbol(&self) -> VariableSymbol {
        self.1
    }
}
