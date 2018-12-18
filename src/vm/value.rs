use crate::vm::symbol::*;

pub use crate::common::value::Const;

#[derive(Debug, Clone)]
pub enum Value {
    Const(Const),
    Ref(Ref),
    None,
}

/// A reference to a value stored someplace.
#[derive(Debug, Clone)]
pub enum Ref {
    /// A register local to this function.
    Var(VarSymbol),

    /// A reference to a function.
    Fun(FunSymbol),

    /// A reference to a type.
    Ty(TySymbol),
}
