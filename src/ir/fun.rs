use crate::{
    ir::{
        Block,
        TyExpr,
    },
    common::prelude::*,
    syntax::tree,
};

#[derive(Debug)]
pub struct Fun {
    pub name: String,
    pub params: Vec<FunParam>,
    pub return_ty: Option<TyExpr>,
    pub body: Block,
    pub range: Range,
}

impl_ranged!(Fun::range);

#[derive(Debug)]
pub struct FunParam {
    pub name: String,
    pub ty: Option<TyExpr>,
    pub range: Range,
}

impl From<tree::Fun> for Fun {
    fn from(tree::Fun { name, params, return_ty, body, range, }: tree::Fun) -> Self {
        unimplemented!()
    }
}

impl_ranged!(FunParam::range);
