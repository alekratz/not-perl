use std::{
    fmt::{self, Display, Formatter},
};
use vm::{Symbolic, TySymbol};

#[derive(Debug, Clone)]
pub enum Ty {
    User(UserTy),
    Builtin(BuiltinTy, TySymbol),
}

#[derive(Debug, Clone)]
pub struct UserTy {
    pub name: String,
    pub symbol: TySymbol,
}

#[derive(Debug, Clone)]
pub enum BuiltinTy {
    Str,
    Int,
    Float,
    Bool,
    None,
}

impl BuiltinTy {
    pub fn name(&self) -> &str {
        match self {
            BuiltinTy::Str => "Str",
            BuiltinTy::Int => "Int",
            BuiltinTy::Float => "Float",
            BuiltinTy::Bool => "Bool",
            BuiltinTy::None => "None",
        }
    }
}

impl Display for BuiltinTy {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name())
    }
}

impl Symbolic for Ty {
    type Symbol = TySymbol;

    fn name(&self) -> &str {
        match self {
            Ty::User(u) => &u.name,
            Ty::Builtin(b, _) => &b.name(),
        }
    }

    fn symbol(&self) -> TySymbol {
        match self {
            Ty::User(u) => u.symbol,
            Ty::Builtin(_, s) => *s,
        }
    }
}
