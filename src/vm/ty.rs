use std::fmt::{self, Display, Formatter};
use vm::{FunctionSymbol, TySymbol};

#[derive(EnumIsA, Debug, Clone, PartialEq)]
pub enum Ty {
    Builtin(BuiltinTy, TySymbol),
    User(UserTy),
}

impl Ty {
    pub fn symbol(&self) -> TySymbol {
        match self {
            | Ty::Builtin(_, sym) => *sym,
            | Ty::User(u) => u.symbol,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Ty::Builtin(b, _) => b.name(),
            Ty::User(u) => &u.name,
        }
    }
}

impl Display for Ty {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name())
    }
}

#[derive(EnumIsA, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum BuiltinTy {
    Float,
    Bool,
    Int,
    Array,
    Str,
    Any,
    None,
}

impl BuiltinTy {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltinTy::Float => "Float",
            BuiltinTy::Bool => "Bool",
            BuiltinTy::Int => "Int",
            BuiltinTy::Array => "Array",
            BuiltinTy::Str => "Str",
            BuiltinTy::Any => "Any",
            BuiltinTy::None => "None",
        }
    }
}

impl Display for BuiltinTy {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name())
    }
}

/// A user-defined type.
#[derive(Debug, Clone, PartialEq)]
pub struct UserTy {
    pub name: String,
    pub symbol: TySymbol,
    pub predicate: FunctionSymbol,
    pub functions: Vec<FunctionSymbol>,
}

impl Display for UserTy {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}
