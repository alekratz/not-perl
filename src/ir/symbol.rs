use syntax::{
    token::Token,
    Ranged,
};

/// A symbol which is used to point to a value.
#[derive(Clone, Debug)]
pub enum Symbol {
    /// A function.
    Fun(String),

    /// A variable.
    Variable(String),

    /// A type.
    Ty(String),
}

impl Symbol {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::Variable(ref s) => Symbol::Variable(s.clone()),
            Token::Bareword(ref s) => {
                // upper-case barewords are types
                if s.starts_with("ABCDEFGHIJKLMNOPQRSTUVWXYZ") {
                    Symbol::Ty(s.clone())
                } else {
                    Symbol::Fun(s.clone())
                }
            },
            _ => panic!("invalid conversion from Token {:?} to Symbol", token),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            | Symbol::Fun(s)
            | Symbol::Variable(s)
            | Symbol::Ty(s) => s
        }
    }
}

pub type RangeSymbol<'n> = Ranged<'n, Symbol>;
