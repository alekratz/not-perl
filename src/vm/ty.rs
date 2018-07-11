#[derive(Debug, Clone)]
pub enum Ty {
    Definite(String),
    Any,
    None,
}

// TODO : other known types

pub const STR_DEFINITE: &str = "Str";
