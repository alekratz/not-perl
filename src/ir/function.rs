use crate::common::pos::{Range, Ranged};
use crate::syntax::tree::{self, Stmt};
use crate::ir::{
    Ir,
    Action, Symbol, TyExpr, Value, Block,
};

#[derive(Debug)]
pub struct Fun {
    pub symbol: Symbol,
    pub params: Vec<FunParam>,
    pub return_ty: TyExpr,
    pub body: Block,
    pub inner_functions: Vec<Fun>,
    pub range: Range,
}

impl Fun {
    // TODO(range) remove this
    /*
    pub fn new(symbol: Symbol, params: Vec<FunParam>, return_ty: TyExpr, body: Block,
               inner_functions: Vec<Fun>) -> Self {
        Fun { symbol, params, return_ty, body, inner_functions }
    }
    */

    pub fn name(&self) -> &str { &self.symbol.name() }
}

impl Ir<tree::Fun> for Fun {
    fn from_syntax(tree::Fun { name, params, return_ty, body, range, }: &tree::Fun) -> Self {
        let symbol = Symbol::Fun(name.clone());
        let params = params.iter()
            .map(FunParam::from_syntax)
            .collect();
        let return_ty = if let Some(return_ty) = return_ty {
            TyExpr::Definite(return_ty.to_string())
        } else if body.iter().any(|stmt| matches!(stmt, Stmt::Return(Some(_), _))) {
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
        Fun { symbol, params, return_ty, body, inner_functions, range: range.clone(), }
    }
}

impl_ranged!(Fun::range);

#[derive(Debug)]
pub enum FunParam {
    SelfKw(Range),
    Variable {
        symbol: Symbol,
        ty: TyExpr,
        default: Option<Value>,
        range: Range,
    },
}

impl FunParam {
    pub fn name(&self) -> &str {
        match self {
            FunParam::SelfKw(_) => "self",
            FunParam::Variable { symbol, ty: _, default: _, range: _ } => symbol.name(),
        }
    }
}

impl Ranged for FunParam {
    fn range(&self) -> Range {
        match self {
            | FunParam::SelfKw(range)
            | FunParam::Variable { symbol: _, ty: _, default: _, range, } => range.clone(),
        }
    }
}


impl Ir<tree::FunParam> for FunParam {
    fn from_syntax(param: &tree::FunParam) -> Self {
        match param {
            // TODO(range) ir::FunParam range
            tree::FunParam::Variable { name, ty, default, range, } => {
                let symbol = Symbol::Variable(name.to_string());
                let ty = if let Some(ty) = ty {
                    TyExpr::Definite(ty.to_string())
                } else {
                    // variables, by default, have a type of "any"
                    TyExpr::Any
                };
                let default = default.as_ref().map(Value::from_syntax);
                FunParam::Variable { symbol, ty, default, range: range.clone(), }
            }
            // TODO(range) ir::FunParam range
            tree::FunParam::SelfKw(range) => FunParam::SelfKw(range.clone()),
        }
    }
}
