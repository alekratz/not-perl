#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    Any,
    Definite(String),
}
