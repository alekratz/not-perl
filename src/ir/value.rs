use crate::syntax::{
    token::{Token},
    tree::Expr,
};
use crate::common::{
    pos::{
        Range,
        Ranged,
        RangeWrapper,
    },
    lang::Op,
};
use crate::ir::{Ir, Symbol, RangedSymbol};

pub use crate::common::value::Const;

pub type RangeConst = RangeWrapper<Const>;

impl Const {
    pub fn from_token(other: &Token) -> Self {
        match other {
            Token::StrLit(s) => Const::Str(s.clone()),
            Token::IntLit(n, r) => {
                match i64::from_str_radix(n.as_str(), *r as u32) {
                    Ok(v) => Const::Int(v),
                    Err(_) => unimplemented!("bigint")
                }
            },
            Token::FloatLit(ref f) => Const::Float(str::parse::<f64>(f.as_str()).expect("invalid float literal")),
            Token::TrueKw => Const::Bool(true),
            Token::FalseKw => Const::Bool(false),
            _ => panic!("invalid constant value: {:?}", other),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ValueKind {
    Const(RangeConst),
    Symbol(RangedSymbol),
    ArrayAccess(Box<Value>, Box<Value>),
    BinaryExpr(Box<Value>, Op, Box<Value>),
    UnaryExpr(Op, Box<Value>),
    FunCall(Box<Value>, Vec<Value>),
}

impl ValueKind {
    pub fn range(&self) -> Range {
        match self {
            | ValueKind::Const(RangeWrapper(r, _))
            | ValueKind::Symbol(RangeWrapper(r, _)) => { r.clone() }
            | ValueKind::ArrayAccess(r1, r2)
            | ValueKind::BinaryExpr(r1, _, r2) => { r1.range().union(&r2.range()) }
            ValueKind::UnaryExpr(_op, value) => { value.range() } // TODO : give ops a range?
            ValueKind::FunCall(fun, args) => {
                if let Some(last) = args.last() {
                    fun.range().union(&last.range())
                } else {
                    fun.range()
                }
            }
        }
    }

    /// Gets whether this value consists of only constant values.
    pub fn is_constant(&self) -> bool {
        match self {
            ValueKind::Const(_) => true,
            ValueKind::BinaryExpr(lhs, _, rhs) => lhs.is_constant() && rhs.is_constant(),
            ValueKind::UnaryExpr(_, expr) => expr.is_constant(),
            | ValueKind::ArrayAccess(_, _)
            | ValueKind::FunCall(_, _)
            | ValueKind::Symbol(_) => false,
        }
    }

    /// Determines whether this value can be treated as an "immediate".
    pub fn is_immediate(&self) -> bool {
        match self {
            // constants and symbols can immediately be accessed
            | ValueKind::Const(_)
            | ValueKind::Symbol(_) => true,
            // arrays, binary exprs, unary exprs, and function calls must be evaluated
            _ => false,
        }
    }

    /// Gets whether this value is allowed to appear on the LHS of an assignment.
    pub fn is_assign_candidate(&self) -> bool {
        // constant expressions cannot be assigned to
        if self.is_constant() { return false; }

        match self {
            ValueKind::Const(_) => false,
            // binary expressions are valid LHS candidates if at least one of its sides is an LHS
            // candidate
            ValueKind::BinaryExpr(l, _, r) => l.is_assign_candidate() || r.is_assign_candidate(),
            // unary expressions pass the value's LHS candidacy through
            ValueKind::UnaryExpr(_, u) => u.is_assign_candidate(),
            // symbols, array accesses, and function calls are always valid LHS candidates
            | ValueKind::Symbol(RangeWrapper(_, Symbol::Variable(_)))
            | ValueKind::ArrayAccess(_, _)
            | ValueKind::FunCall(_, _) => true,
            _ => false,
        }
    }
}

impl Ir<Expr> for Value {
    fn from_syntax(expr: &Expr) -> Self {
        let kind = match expr {
            // TODO(range) ir::ValueKind range
            Expr::FunCall { function, args, range } => {
                let function = Value::from_syntax(function);
                let mut fun_args = vec![];
                for arg in args.iter() {
                    fun_args.push(Value::from_syntax(arg));
                }
                ValueKind::FunCall(Box::new(function), fun_args)
            }
            // TODO(range) ir::ValueKind range
            Expr::ArrayAccess { array, index, range } => {
                let array = Value::from_syntax(array);
                let index = Value::from_syntax(index);
                ValueKind::ArrayAccess(Box::new(array), Box::new(index))
            }
            Expr::Atom(ref token) => match token.token() {
                | Token::Variable(_)
                | Token::Bareword(_) => ValueKind::Symbol(token.map(Symbol::from_token)),
                _ => ValueKind::Const(token.map(Const::from_token))
            },
            Expr::Binary(ref lhs, ref op, ref rhs) => {
                let lhs = Value::from_syntax(lhs);
                let rhs = Value::from_syntax(rhs);
                ValueKind::BinaryExpr(Box::new(lhs), op.clone(), Box::new(rhs))
            }
            Expr::Unary(ref op, ref expr) => {
                let expr = Value::from_syntax(expr);
                ValueKind::UnaryExpr(op.clone(), Box::new(expr))
            }
        };
        Value::new(expr.range(), kind)
    }
}

pub type Value = RangeWrapper<ValueKind>;
