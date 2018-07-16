#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TyExpr {
    Any,
    Definite(String),
    None,
}
