use vm::{
    symbol::*,
    StackIndex,
};

pub use common::value::Const;

#[derive(Debug, Clone)]
pub enum Value {
    Const(Const),
    Reg(RegSymbol),
    FunRef(FunSymbol),
    TyRef(TySymbol),
    StackRef(StackIndex),
    None,
}
