use vm::Value;
use syntax::token::Op;

/// A condition that must be met, and can be checked.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// A condition that is always met.
    Always,

    /// A condition that is never met.
    Never,

    /// A condition based upon a comparison of two values
    Compare(Value, CompareOp, Value),

    /// A condition that checks a value's "truthiness".
    ///
    /// This is equivalent to doing a fuzzy match with "true", i.e.,
    ///
    /// `value ~~ true`
    Truthy(Value),
}

/// A comparison for a `Condition`.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum CompareOp {
    Or,
    And,
    Equals,
    NotEquals,
    FuzzyEquals,
    Less,
    Greater,
    LessEquals,
    GreaterEquals,
}

impl CompareOp {
    /// Converts the supplied `syntax::token::Op` to a `CompareOp`.
    pub fn from_syntax(op: &Op) -> Option<Self> {
        match op {
            Op::Or => Some(CompareOp::Or),
            Op::And => Some(CompareOp::And),
            Op::DoubleEquals => Some(CompareOp::Equals),
            Op::DoublePercent => unimplemented!("VM: double percent comparison op"),
            Op::DoubleTilde => Some(CompareOp::FuzzyEquals),
            Op::NotEquals => Some(CompareOp::NotEquals),
            Op::LessEquals => Some(CompareOp::LessEquals),
            Op::GreaterEquals => Some(CompareOp::GreaterEquals),
            Op::Less => Some(CompareOp::Less),
            Op::Greater => Some(CompareOp::Greater),
            _ => panic!("cannot convert IR op {:?} to VM comparison op"),
        }
    }
}
