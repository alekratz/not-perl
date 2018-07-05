use vm::{
    Symbol,
    bc::Bc,
};

#[derive(Debug, Clone)]
pub struct Function {
    symbol: Symbol,
    body: Vec<Bc>,
    locals: Vec<Symbol>,
}

impl Function {
    pub fn new(symbol: Symbol, body: Vec<Bc>, locals: Vec<Symbol>) -> Self {
        Function {
            symbol,
            body,
            locals,
        }
    }

    pub fn body(&self) -> &[Bc] {
        &self.body
    }

    pub fn locals(&self) -> &[Symbol] {
        &self.locals
    }

    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn name(&self) -> &str {
        &self.symbol.name()
    }
}

