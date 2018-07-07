use vm::{
    Symbol,
    Bc,
    Label,
    Ty,
};

#[derive(Debug, Clone)]
pub struct Function {
    pub symbol: Symbol,
    pub params: Vec<FunctionParam>,
    pub return_ty: Ty,
    pub locals: Vec<Symbol>,
    pub body: Vec<Bc>,
    pub labels: Vec<Label>,
}

impl Function {
    pub fn new(symbol: Symbol, params: Vec<FunctionParam>, return_ty: Ty, locals: Vec<Symbol>, body: Vec<Bc>,
               labels: Vec<Label>) -> Self {
        Function {
            symbol,
            params,
            return_ty,
            locals,
            body,
            labels,
        }
    }
    pub fn name(&self) -> &str {
        &self.symbol.name()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub symbol: Symbol,
    pub ty: Ty,
}

impl FunctionParam {

    pub fn name(&self) -> &str {
        self.symbol.name()
    }
}
