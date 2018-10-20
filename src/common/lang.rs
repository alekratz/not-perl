use std::{
    fmt::{self, Display, Formatter},
};

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Bang,
    Plus,
    Minus,
    Splat,
    FSlash,
    Tilde,
    Or,
    And,
    DoublePercent,
    DoubleEquals,
    NotEquals,
    DoubleTilde,
    LessEquals,
    GreaterEquals,
    Less,
    Greater,
    Custom(String),
}

impl<S> From<S> for Op
    where S: Into<String>,
          String: From<S>,
{
    fn from(other: S) -> Self {
        let other = String::from(other);
        match other.as_str() {
            "!" => Op::Bang,
            "+" => Op::Plus,
            "-" => Op::Minus,
            "*" => Op::Splat,
            "/" => Op::FSlash,
            "~" => Op::Tilde,
            "||" => Op::Or,
            "&&" => Op::And,
            "%%" => Op::DoublePercent,
            "==" => Op::DoubleEquals,
            "~~" => Op::DoubleTilde,
            "!=" => Op::NotEquals,
            "<=" => Op::LessEquals,
            ">=" => Op::GreaterEquals,
            "<" => Op::Less,
            ">" => Op::Greater,
            _ => Op::Custom(other),
        }
    }
}

impl Display for Op {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Op::Bang => write!(fmt, "!"),
            Op::Plus => write!(fmt, "+"),
            Op::Minus => write!(fmt, "-"),
            Op::Splat => write!(fmt, "*"),
            Op::FSlash => write!(fmt, "/"),
            Op::Tilde => write!(fmt, "~"),
            Op::Or => write!(fmt, "||"),
            Op::And => write!(fmt, "&&"),
            Op::DoubleEquals => write!(fmt, "=="),
            Op::DoublePercent => write!(fmt, "%%"),
            Op::DoubleTilde => write!(fmt, "~~"),
            Op::NotEquals => write!(fmt, "!="),
            Op::LessEquals => write!(fmt, "<="),
            Op::GreaterEquals => write!(fmt, ">="),
            Op::Less => write!(fmt, "<"),
            Op::Greater => write!(fmt, ">"),
            Op::Custom(o) => write!(fmt, "{}", o),
        }
    }
}
