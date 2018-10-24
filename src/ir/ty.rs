use std::fmt::{self, Display, Formatter};
use crate::syntax::tree;
use crate::ir::{Fun, Ir};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TyExpr {
    Any,
    Definite(String),
    None,
}

impl Display for TyExpr {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            TyExpr::Any => write!(fmt, "Any"),
            TyExpr::Definite(t) => write!(fmt, "{}", t),
            TyExpr::None => write!(fmt, "None"),
        }
    }
}

/// An intermediate representation of a user-defined type.
#[derive(Debug)]
pub struct UserTy<'n> {
    pub name: String,
    pub parents: Vec<String>,
    pub functions: Vec<Fun<'n>>,
}

impl<'n> Ir<tree::UserTy<'n>> for UserTy<'n> {
    fn from_syntax(ty: &tree::UserTy<'n>) -> Self {
        UserTy {
            name: ty.name.clone(),
            parents: ty.parents.clone(),
            functions: ty.functions
                .iter()
                .map(Fun::from_syntax)
                .collect(),
        }
    }
}
