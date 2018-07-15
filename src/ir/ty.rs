#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ty {
    Any,
    Definite(String),
    None,
}


