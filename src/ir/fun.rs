use crate::{
    ir::{
        Action,
        Value,
        TyExpr,
    },
    common::prelude::*,
    syntax::tree,
};

#[derive(Debug, Clone)]
pub struct Fun {
    pub name: String,
    pub params: Vec<FunParam>,
    pub return_ty: Option<TyExpr>,
    pub body: Action,
    pub range: Range,
}

impl_ranged!(Fun::range);

impl From<tree::Fun> for Fun {
    fn from(tree::Fun { name, params, return_ty, body, range, }: tree::Fun) -> Self {
        Fun {
            name,
            params: params.into_iter().map(From::from).collect(),
            return_ty: return_ty.map(|s| TyExpr(s)),
            body: body.into(),
            range,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunParam {
    pub name: String,
    pub ty: Option<TyExpr>,
    pub default: Option<Value>,
    pub range: Range,
}

impl From<tree::FunParam> for FunParam {
    fn from(tree::FunParam { name, ty, default, range, }: tree::FunParam) -> Self {
        FunParam {
            name,
            ty: ty.map(|s| TyExpr(s)),
            default: default.map(From::from),
            range,
        }
    }
}

impl_ranged!(FunParam::range);
