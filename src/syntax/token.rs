use std::fmt::{self, Formatter, Display};
use crate::common::{
    lang::Op,
    pos::RangeWrapper,
};
use crate::syntax::{
    tree::Ast,
};

// HOW TO ADD A NEW TOKEN:
//
// 1. Add the token to `enum Token`
// 2. Add the canonicalization in `fn canonicalize`
// 3. Add the human-readable format in `fn fmt`
// 4. If it's a keyword, add it to `fn next_bareword` in the lexer.
// 5. If it's a lookahead (e.g. for an expression), add it as a lookahead to the appropriate AST
//    items. Also add it to the parser as a lookahead.


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Equals,
    PlusEquals,
    MinusEquals,
    SplatEquals,
    FSlashEquals,
    TildeEquals,
}

impl AssignOp {
    pub fn from_str(other: impl AsRef<str>) -> Option<Self> {
        match other.as_ref() {
            "=" => Some(AssignOp::Equals),
            "+=" => Some(AssignOp::PlusEquals),
            "-=" => Some(AssignOp::MinusEquals),
            "*=" => Some(AssignOp::SplatEquals),
            "/=" => Some(AssignOp::FSlashEquals),
            "~=" => Some(AssignOp::TildeEquals),
            _ => None,
        }
    }
}

impl Display for AssignOp {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", match self {
            AssignOp::Equals => "=",
            AssignOp::PlusEquals => "+=",
            AssignOp::MinusEquals => "-=",
            AssignOp::SplatEquals => "*=",
            AssignOp::FSlashEquals => "/=",
            AssignOp::TildeEquals => "~=",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {

    //
    // Language literals
    //

    StrLit(String),
    IntLit(String, usize),
    FloatLit(String),

    //
    // User-defined names n stuff
    //

    Comment,
    Variable(String),
    Bareword(String),

    //
    // Keywords
    //
    IfKw,
    ElseKw,
    WhileKw,
    LoopKw,
    ContinueKw,
    BreakKw,
    ReturnKw,
    TrueKw,
    FalseKw,
    FunKw,
    TypeKw,
    SelfKw,

    //
    // Symbols
    //

    AssignOp(AssignOp),
    Op(Op),
    Comma,
    Colon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    //
    // Control tokens
    //
    LineEnd,
    NewLine,
}

impl Token {
    /// Gets whether this token is a lookahead to the given AST type.
    pub fn is_lookahead<'n, A: Ast<'n>>(&self) -> bool {
        A::token_is_lookahead(self)
    }

    pub fn into_op(self) -> Op {
        if let Token::Op(op) = self {
            op
        } else {
            panic!("attempted to convert non-Token::Op to Op (got {:?})", self)
        }
    }

    pub fn is_op(&self) -> bool {
        if let &Token::Op(_) = self {
            true
        } else {
            false
        }
    }

    pub fn into_assign_op(self) -> AssignOp {
        if let Token::AssignOp(op) = self {
            op
        } else {
            panic!("attempted to convert non-Token::AssignOp to AssignOp (got {:?})", self)
        }
    }

    pub fn is_assign_op(&self) -> bool {
        if let &Token::AssignOp(_) = self {
            true
        } else {
            false
        }
    }


    pub fn canonicalize(&self) -> String {
        use self::Token::*;
        match self {
            StrLit(ref s) => format!("{:?}", s),
            IntLit(i, r) => match r {
                2  => format!("0b{}", i),
                8  => format!("0o{}", i),
                10 => format!("{}", i),
                16 => format!("0x{}", i),
                _ => unreachable!(),
            },
            FloatLit(f) => f.to_string(),
            Comment => "#".to_string(),
            Variable(ref s) => s.to_string(),
            Bareword(ref s) => s.to_string(),
            IfKw => "if".to_string(),
            ElseKw => "else".to_string(),
            WhileKw => "while".to_string(),
            LoopKw => "loop".to_string(),
            ContinueKw => "continue".to_string(),
            BreakKw => "break".to_string(),
            ReturnKw => "return".to_string(),
            TrueKw => "true".to_string(),
            FalseKw => "false".to_string(),
            FunKw => "fun".to_string(),
            TypeKw => "type".to_string(),
            SelfKw => "self".to_string(),
            Op(s) => s.to_string(),
            AssignOp(s) => s.to_string(),
            Comma => ",".to_string(),
            Colon => ":".to_string(),
            LParen => "(".to_string(),
            RParen => ")".to_string(),
            LBrace => "{".to_string(),
            RBrace => "}".to_string(),
            LBracket => "[".to_string(),
            RBracket => "]".to_string(),
            LineEnd => ";".to_string(),
            NewLine => "\n".to_string(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        use self::Token::*;
        match self {
            StrLit(_) => write!(fmt, "string literal"),
            IntLit(_, _) => write!(fmt, "int literal"),
            FloatLit(_) => write!(fmt, "float literal"),
            Comment => write!(fmt, "comment"),
            Variable(ref s) => write!(fmt, "variable ${}", s),
            Bareword(ref s) => write!(fmt, "bareword {}", s),
            IfKw => write!(fmt, "if keyword"),
            ElseKw => write!(fmt, "else keyword"),
            WhileKw => write!(fmt, "while keyword"),
            LoopKw => write!(fmt, "loop keyword"),
            ContinueKw => write!(fmt, "continue keyword"),
            BreakKw => write!(fmt, "break keyword"),
            ReturnKw => write!(fmt, "return keyword"),
            TrueKw => write!(fmt, "true keyword"),
            FalseKw => write!(fmt, "false keyword"),
            FunKw => write!(fmt, "fun keyword"),
            TypeKw => write!(fmt, "type keyword"),
            SelfKw => write!(fmt, "self keyword"),
            Op(s) =>  write!(fmt, "operator {}", s),
            AssignOp(s) =>  write!(fmt, "assignment operator {}", s),
            Comma => write!(fmt, "comma"),
            Colon => write!(fmt, "colon"),
            LParen => write!(fmt, "left paren"),
            RParen => write!(fmt, "right paren"),
            LBrace => write!(fmt, "left brace"),
            RBrace => write!(fmt, "right brace"),
            LBracket => write!(fmt, "left bracket"),
            RBracket => write!(fmt, "right bracket"),
            NewLine | LineEnd => write!(fmt, "end-of-line"),
        }
    }
}

impl<'n> From<RangedToken<'n>> for Token {
    fn from(other: RangedToken<'n>) -> Self {
        other.1
    }
}

impl<'n> Display for RangedToken<'n> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        (self.1).fmt(fmt)
    }
}

pub type RangedToken<'n> = RangeWrapper<'n, Token>;

impl<'n> RangedToken<'n> {
    pub fn token(&self) -> &Token {
        &self.1
    }
}
