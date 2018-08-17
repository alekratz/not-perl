use syntax::tree::{self, Stmt};
use ir::{
    Ir,
    Action, Symbol, TyExpr, Value, Block,
};

#[derive(Debug)]
pub struct Function<'n> {
    pub symbol: Symbol,
    pub params: Vec<FunctionParam<'n>>,
    pub return_ty: TyExpr,
    pub body: Block<'n>,
    pub inner_functions: Vec<Function<'n>>,
}

impl<'n> Function<'n> {
    pub fn new(symbol: Symbol, params: Vec<FunctionParam<'n>>, return_ty: TyExpr, body: Block<'n>,
               inner_functions: Vec<Function<'n>>) -> Self {
        Function { symbol, params, return_ty, body, inner_functions }
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
            TyExpr::Definite(return_ty.to_string())
        } else if body.iter().any(|stmt| matches!(stmt, Stmt::Return(Some(_)))) {
            // search for at least one return statement that has a value
            TyExpr::Any
        } else {
            TyExpr::None
        };
        let (inner_functions, syntax_body): (Vec<_>, Vec<_>) = body.iter()
            .partition(|s| matches!(s, Stmt::Function(_)));
        let body = syntax_body
            .into_iter()
            .map(Action::from_syntax)
            .collect();
        let inner_functions = inner_functions
            .into_iter()
            .map(|s| if let Stmt::Function(f) = s { Function::from_syntax(f) } else { unreachable!() })
            .collect();
        Function { symbol, params, return_ty, body, inner_functions }
    }
}

#[derive(Debug)]
pub enum FunctionParam<'n> {
    SelfKw,
    Variable {
        symbol: Symbol,
        ty: TyExpr,
        default: Option<Value<'n>>,
    },
}

impl<'n> FunctionParam<'n> {
    pub fn name(&self) -> &str {
        match self {
            FunctionParam::SelfKw => "self",
            FunctionParam::Variable { symbol, ty: _, default: _ } => symbol.name(),
        }
    }
}

impl<'n> Ir<tree::FunctionParam<'n>> for FunctionParam<'n> {
    fn from_syntax(param: &tree::FunctionParam<'n>) -> Self {
        match param {
            tree::FunctionParam::Variable { name, ty, default } => {
                let symbol = Symbol::Variable(name.to_string());
                let ty = if let Some(ty) = ty {
                    TyExpr::Definite(ty.to_string())
                } else {
                    // variables, by default, have a type of "any"
                    TyExpr::Any
                };
                let default = default.as_ref().map(Value::from_syntax);
                FunctionParam::Variable { symbol, ty, default }
            }
            tree::FunctionParam::SelfKw => FunctionParam::SelfKw,
        }
    }
}
