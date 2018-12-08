use crate::common::pos::{Range, Ranged};
use crate::ir::{Fun, Ir};
use crate::syntax::tree;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TyExpr {
    Definite(String),
    None,
}

impl Display for TyExpr {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            TyExpr::Definite(t) => write!(fmt, "{}", t),
            TyExpr::None => write!(fmt, "None"),
        }
    }
}

/// An intermediate representation of a user-defined type.
#[derive(Debug)]
pub struct UserTy {
    pub name: String,
    pub parents: Vec<String>,
    pub functions: Vec<Fun>,
    pub range: Range,
}

impl Ir<tree::UserTy> for UserTy {
    fn from_syntax(ty: &tree::UserTy) -> Self {
        UserTy {
            name: ty.name.clone(),
            parents: ty.parents.clone(),
            functions: ty.functions.iter().map(Fun::from_syntax).collect(),
            range: ty.range(),
        }
    }
}

impl Ranged for UserTy {
    fn range(&self) -> Range {
        self.range.clone()
    }
}
