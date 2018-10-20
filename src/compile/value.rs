use vm;

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub symbol: vm::RegSymbol,
}

impl Var {
    pub fn new(name: String, symbol: vm::RegSymbol) -> Self {
        Var { name, symbol }
    }
}

impl vm::Symbolic for Var {
    type Symbol = vm::RegSymbol;

    fn name(&self) -> &str {
        &self.name
    }

    fn symbol(&self) -> vm::RegSymbol {
        self.symbol
    }
}
