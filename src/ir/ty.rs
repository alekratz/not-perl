use crate::{
    common::prelude::*,
    syntax::tree::UserTy,
    ir::Fun,
};

#[derive(Debug)]
pub struct Ty {
    pub name: String,
    pub parents: Vec<String>,
    pub functions: Vec<Fun>,
    pub range: Range,
}

impl From<UserTy> for Ty {
    fn from(UserTy { name, parents, functions, range, }: UserTy) -> Self {
        let functions = functions.into_iter()
            .map(From::from)
            .collect();
        Ty {
            name,
            parents,
            functions,
            range,
        }
    }
}

#[derive(Debug)]
pub struct TyExpr(String);
