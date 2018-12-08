use crate::common::pos::{Range, Ranged};
use crate::ir::{Action, Block, Ir, Symbol, TyExpr, UserTy, Value};
use crate::syntax::tree::{self, Stmt};

#[derive(Debug)]
pub struct Fun {
    pub symbol: Symbol,
    pub params: Vec<FunParam>,
    pub return_ty: TyExpr,
    pub body: Block,
    pub inner_types: Vec<UserTy>,
    pub inner_functions: Vec<Fun>,
    pub range: Range,
}

impl Fun {
    pub fn name(&self) -> &str {
        &self.symbol.name()
    }
}

impl Ir<tree::Fun> for Fun {
    fn from_syntax(
        tree::Fun {
            name,
            params,
            return_ty,
            body,
            range,
        }: &tree::Fun,
    ) -> Self {
        let symbol = Symbol::Fun(name.clone());
        let params = params.iter().map(FunParam::from_syntax).collect();
        let return_ty = if let Some(return_ty) = return_ty {
            TyExpr::Definite(return_ty.to_string())
        } else {
            TyExpr::None
        };
        let (inner_functions, syntax_body): (Vec<_>, Vec<_>) =
            body.iter().partition(|s| matches!(s, Stmt::Fun(_)));
        let (inner_types, syntax_body): (Vec<_>, Vec<_>) = syntax_body
            .iter()
            .partition(|s| matches!(s, Stmt::UserTy(_)));
        let body = syntax_body.into_iter().map(Action::from_syntax).collect();
        let inner_types = inner_types
            .into_iter()
            .map(|s| {
                if let Stmt::UserTy(t) = s {
                    UserTy::from_syntax(t)
                } else {
                    unreachable!()
                }
            })
            .collect();
        let inner_functions = inner_functions
            .into_iter()
            .map(|s| {
                if let Stmt::Fun(f) = s {
                    Fun::from_syntax(f)
                } else {
                    unreachable!()
                }
            })
            .collect();
        Fun {
            symbol,
            params,
            return_ty,
            body,
            inner_types,
            inner_functions,
            range: range.clone(),
        }
    }
}

impl_ranged!(Fun::range);

#[derive(Debug)]
pub struct FunParam {
    pub symbol: Symbol,
    pub ty: TyExpr,
    pub default: Option<Value>,
    pub range: Range,
}

impl FunParam {
    pub fn new(symbol: Symbol, ty: TyExpr, default: Option<Value>, range: Range) -> Self {
        FunParam {
            symbol,
            ty,
            default,
            range,
        }
    }
}

impl FunParam {
    pub fn name(&self) -> &str {
        self.symbol.name()
    }
}

impl Ranged for FunParam {
    fn range(&self) -> Range {
        self.range.clone()
    }
}

impl Ir<tree::FunParam> for FunParam {
    fn from_syntax(param: &tree::FunParam) -> Self {
        let symbol = Symbol::Variable(param.name.to_string());
        let ty = if let Some(ty) = &param.ty {
            TyExpr::Definite(ty.to_string())
        } else {
            TyExpr::None
        };
        let default = param.default.as_ref().map(Value::from_syntax);
        FunParam::new(symbol, ty, default, param.range.clone())
    }
}
