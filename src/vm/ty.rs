use vm::{Symbol, Function};
use ir;

#[derive(Debug, Clone, PartialEq)]
#[repr(usize)]
pub enum Ty {
    Float,
    Bool,
    Int,
    Array,
    Str,
    Any,
    User(String),
    None,
}

impl From<ir::TyExpr> for Ty {
    fn from(other: ir::TyExpr) -> Self {
        match other {
            ir::TyExpr::Definite(def) => match def.as_str() {
                "Int" => Ty::Int,
                "Float" => Ty::Float,
                "Bool" => Ty::Bool,
                "Array" => Ty::Array,
                "Str" => Ty::Str,
                "Any" => Ty::Any,
                u => Ty::User(u.to_string()),
            }
            ir::TyExpr::Any => Ty::Any,
            ir::TyExpr::None => Ty::None,
        }
    }
}

/// A user-defined type.
#[derive(Debug, Clone, PartialEq)]
pub struct UserTy {
    pub symbol: Symbol,
    pub predicate: Function,
    pub functions: Vec<Function>,
}
