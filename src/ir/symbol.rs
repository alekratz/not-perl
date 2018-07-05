use syntax::{
    token::Token,
    Ranged,
};

/// A symbol which is used to point to a value.
#[derive(Clone, Debug)]
pub enum Symbol {
    /// A function
    Function(String),

    /// A variable.
    Variable(String),

    /// A bareword that hasn't been resolved yet.
    Bareword(String),
}

impl Symbol {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::Variable(ref s) => Symbol::Variable(s.clone()),
            Token::Bareword(ref s) => Symbol::Bareword(s.clone()),
            _ => panic!("invalid conversion from Token {:?} to Symbol", token),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            | Symbol::Function(s)
            | Symbol::Variable(s)
            | Symbol::Bareword(s) => s
        }
    }
}

pub type RangeSymbol<'n> = Ranged<'n, Symbol>;
