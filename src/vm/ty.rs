use std::fmt::{self, Display, Formatter};
use vm::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Builtin(BuiltinTy),
    User(Symbol),
}

impl Ty {
    pub fn symbol(&self) -> Symbol {
        match self {
            | Ty::Builtin(_) => panic!("Builtin types do not have symbols"),
            | Ty::User(sym) => *sym,
        }
    }

    /// Gets whether this is a user-defined type.
    pub fn is_user(&self) -> bool {
        match self {
            Ty::User(_) => true,
            _ => false,
        }
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
        match self {
            BuiltinTy::Float => write!(fmt, "Float"),
            BuiltinTy::Bool => write!(fmt, "Bool"),
            BuiltinTy::Int => write!(fmt, "Int"),
            BuiltinTy::Array => write!(fmt, "Array"),
            BuiltinTy::Str => write!(fmt, "Str"),
            BuiltinTy::Any => write!(fmt, "Any"),
            BuiltinTy::None => write!(fmt, "None"),
        }
    }
}

/// A user-defined type.
#[derive(Debug, Clone, PartialEq)]
pub struct UserTy {
    pub name: String,
    pub symbol: Symbol,
    pub predicate: Symbol,
    pub functions: Vec<Symbol>,
}
