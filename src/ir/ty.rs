use std::fmt::{self, Display, Formatter};
use vm;
use syntax::tree;
use ir::{Function, Ir};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TyExpr {
    Any,
    Definite(String),
    Builtin(vm::BuiltinTy),
    None,
}

impl Display for TyExpr {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            TyExpr::Any => write!(fmt, "Any"),
            TyExpr::Definite(t) => write!(fmt, "{}", t),
            TyExpr::Builtin(b) => write!(fmt, "Builtin type {}", b),
            TyExpr::None => write!(fmt, "None"),
        }
    }
}

impl From<vm::Ty> for TyExpr {
    fn from(other: vm::Ty) -> Self {
        match other {
            vm::Ty::Builtin(vm::BuiltinTy::Any) => TyExpr::Any,
            vm::Ty::Builtin(vm::BuiltinTy::None) => TyExpr::None,
            vm::Ty::Builtin(b) => TyExpr::Builtin(b),
            vm::Ty::User(_) => panic!("vm::Ty::User type cannot be converted to a type expression"),
        }
    }
}

// Type alias for a user-defined type.
//
// Since the syntax and IR would effectively be the same, it would be more work to keep two
// different structures in tandem with one another.
//pub type UserTy<'n> = tree::UserTy<'n>;

/// An intermediate representation of a user-defined type.
#[derive(Debug)]
pub struct UserTy<'n> {
    pub name: String,
    pub parents: Vec<String>,
    pub functions: Vec<Function<'n>>,
}

impl<'n> Ir<tree::UserTy<'n>> for UserTy<'n> {
    fn from_syntax(ty: &tree::UserTy<'n>) -> Self {
        UserTy {
            name: ty.name.clone(),
            parents: ty.parents.clone(),
            functions: ty.functions
                .iter()
                .map(Function::from_syntax)
                .collect(),
        }
    }
}
