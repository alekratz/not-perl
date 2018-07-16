use syntax::tree;
use ir::{
    Ir,
    Action, Symbol, Ty, Value, Block,
};

#[derive(Debug)]
pub struct Function<'n> {
    pub symbol: Symbol,
    pub params: Vec<FunctionParam<'n>>,
    pub return_ty: Ty,
    pub body: Block<'n>,
}

impl<'n> Function<'n> {
    pub fn new(symbol: Symbol, params: Vec<FunctionParam<'n>>, return_ty: Ty, body: Block<'n>) -> Self {
        Function { symbol, params, return_ty, body }
    }

    pub fn name(&self) -> &str { &self.symbol.name() }
}

impl<'n> Ir<tree::Function<'n>> for Function<'n> {
    fn from_syntax(tree::Function { name, params, return_ty, body }: &tree::Function<'n>) -> Self {
        let symbol = Symbol::Function(name.clone());
        let params = params.iter()
            .map(FunctionParam::from_syntax)
            .collect();
        let return_ty = if let Some(return_ty) = return_ty {
            Ty::Definite(return_ty.to_string())
        } else {
            Ty::None
        };
        let body = body.iter()
            .map(Action::from_syntax)
            .collect();
        Function { symbol, params, return_ty, body }
    }
}

#[derive(Debug)]
pub struct FunctionParam<'n> {
    pub name: Symbol,
    pub ty: Ty,
    pub default: Option<Value<'n>>,
}

impl<'n> FunctionParam<'n> {
    pub fn new(name: Symbol, ty: Ty, default: Option<Value<'n>>) -> Self {
        FunctionParam { name, ty, default, }
    }
}

impl<'n> Ir<tree::FunctionParam<'n>> for FunctionParam<'n> {
    fn from_syntax(tree::FunctionParam { name, ty, default }: &tree::FunctionParam<'n>) -> Self {
        let name = Symbol::Variable(name.to_string());
        let ty = if let Some(ty) = ty {
            Ty::Definite(ty.to_string())
        } else {
            // variables, by default, have a type of "any"
            Ty::Any
        };
        let default = default.as_ref().map(Value::from_syntax);
        FunctionParam::new(name, ty, default)
    }
}
