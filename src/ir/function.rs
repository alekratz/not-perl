use crate::syntax::tree::{self, Stmt};
use crate::ir::{
    Ir,
    Action, Symbol, TyExpr, Value, Block,
};

#[derive(Debug)]
pub struct Fun<'n> {
    pub symbol: Symbol,
    pub params: Vec<FunParam<'n>>,
    pub return_ty: TyExpr,
    pub body: Block<'n>,
    pub inner_functions: Vec<Fun<'n>>,
}

impl<'n> Fun<'n> {
    pub fn new(symbol: Symbol, params: Vec<FunParam<'n>>, return_ty: TyExpr, body: Block<'n>,
               inner_functions: Vec<Fun<'n>>) -> Self {
        Fun { symbol, params, return_ty, body, inner_functions }
    }

    pub fn name(&self) -> &str { &self.symbol.name() }
}

impl<'n> Ir<tree::Fun<'n>> for Fun<'n> {
    fn from_syntax(tree::Fun { name, params, return_ty, body }: &tree::Fun<'n>) -> Self {
        let symbol = Symbol::Fun(name.clone());
        let params = params.iter()
            .map(FunParam::from_syntax)
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
            .partition(|s| matches!(s, Stmt::Fun(_)));
        let body = syntax_body
            .into_iter()
            .map(Action::from_syntax)
            .collect();
        let inner_functions = inner_functions
            .into_iter()
            .map(|s| if let Stmt::Fun(f) = s { Fun::from_syntax(f) } else { unreachable!() })
            .collect();
        Fun { symbol, params, return_ty, body, inner_functions }
    }
}

#[derive(Debug)]
pub enum FunParam<'n> {
    SelfKw,
    Variable {
        symbol: Symbol,
        ty: TyExpr,
        default: Option<Value<'n>>,
    },
}

impl<'n> FunParam<'n> {
    pub fn name(&self) -> &str {
        match self {
            FunParam::SelfKw => "self",
            FunParam::Variable { symbol, ty: _, default: _ } => symbol.name(),
        }
    }
}

impl<'n> Ir<tree::FunParam<'n>> for FunParam<'n> {
    fn from_syntax(param: &tree::FunParam<'n>) -> Self {
        match param {
            tree::FunParam::Variable { name, ty, default } => {
                let symbol = Symbol::Variable(name.to_string());
                let ty = if let Some(ty) = ty {
                    TyExpr::Definite(ty.to_string())
                } else {
                    // variables, by default, have a type of "any"
                    TyExpr::Any
                };
                let default = default.as_ref().map(Value::from_syntax);
                FunParam::Variable { symbol, ty, default }
            }
            tree::FunParam::SelfKw => FunParam::SelfKw,
        }
    }
}
