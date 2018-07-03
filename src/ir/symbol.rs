use syntax::token::Token;

#[derive(Clone, Debug)]
pub enum Symbol {
    /// A function
    Function(String),

    /// A variable.
    Variable(String),

    /// A bareword that hasn't been resolved yet.
    Bareword(String),

    /// An anonymous value.
    AnonVal(usize),

    /// An anonymous function.
    AnonFun(usize),
}

impl Symbol {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::Variable(ref s) => Symbol::Variable(s.clone()),
            Token::Bareword(ref s) => Symbol::Bareword(s.clone()),
            _ => panic!("invalid conversion from Token {:?} to Symbol", token),
        }
    }
}
