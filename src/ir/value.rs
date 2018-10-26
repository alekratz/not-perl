use crate::syntax::{
    token::{Token},
    tree::Expr,
};
use crate::common::{
    pos::{
        Range,
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
pub enum Value {
    Const(RangeConst),
    Symbol(RangedSymbol),
    ArrayAccess(Box<Value>, Box<Value>),
    BinaryExpr(Box<Value>, Op, Box<Value>),
    UnaryExpr(Op, Box<Value>),
    FunCall(Box<Value>, Vec<Value>),
}

impl Value {
    pub fn range(&self) -> Range {
        match self {
            | Value::Const(RangeWrapper(r, _))
            | Value::Symbol(RangeWrapper(r, _)) => { r.clone() }
            | Value::ArrayAccess(r1, r2)
            | Value::BinaryExpr(r1, _, r2) => { r1.range().union(&r2.range()) }
            Value::UnaryExpr(_op, value) => { value.range() } // TODO : give ops a range?
            Value::FunCall(fun, args) => {
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
            Value::Const(_) => true,
            Value::BinaryExpr(lhs, _, rhs) => lhs.is_constant() && rhs.is_constant(),
            Value::UnaryExpr(_, expr) => expr.is_constant(),
            | Value::ArrayAccess(_, _)
            | Value::FunCall(_, _)
            | Value::Symbol(_) => false,
        }
    }

    /// Determines whether this value can be treated as an "immediate".
    pub fn is_immediate(&self) -> bool {
        match self {
            // constants and symbols can immediately be accessed
            | Value::Const(_)
            | Value::Symbol(_) => true,
            // arrays, binary exprs, unary exprs, and function calls must be evaluated
            _ => false,
        }
    }

    /// Gets whether this value is allowed to appear on the LHS of an assignment.
    pub fn is_assign_candidate(&self) -> bool {
        // constant expressions cannot be assigned to
        if self.is_constant() { return false; }

        match self {
            Value::Const(_) => false,
            // binary expressions are valid LHS candidates if at least one of its sides is an LHS
            // candidate
            Value::BinaryExpr(l, _, r) => l.is_assign_candidate() || r.is_assign_candidate(),
            // unary expressions pass the value's LHS candidacy through
            Value::UnaryExpr(_, u) => u.is_assign_candidate(),
            // symbols, array accesses, and function calls are always valid LHS candidates
            | Value::Symbol(RangeWrapper(_, Symbol::Variable(_)))
            | Value::ArrayAccess(_, _)
            | Value::FunCall(_, _) => true,
            _ => false,
        }
    }
}

impl Ir<Expr> for Value {
    fn from_syntax(expr: &Expr) -> Self {
        match expr {
            // TODO(range) ir::Value range
            Expr::FunCall { function, args, range } => {
                let function = Value::from_syntax(function);
                let mut fun_args = vec![];
                for arg in args.iter() {
                    fun_args.push(Value::from_syntax(arg));
                }
                Value::FunCall(Box::new(function), fun_args)
            }
            // TODO(range) ir::Value range
            Expr::ArrayAccess { array, index, range } => {
                let array = Value::from_syntax(array);
                let index = Value::from_syntax(index);
                Value::ArrayAccess(Box::new(array), Box::new(index))
            }
            Expr::Atom(ref token) => match token.token() {
                | Token::Variable(_)
                | Token::Bareword(_) => Value::Symbol(token.map(Symbol::from_token)),
                _ => Value::Const(token.map(Const::from_token))
            },
            Expr::Binary(ref lhs, ref op, ref rhs) => {
                let lhs = Value::from_syntax(lhs);
                let rhs = Value::from_syntax(rhs);
                Value::BinaryExpr(Box::new(lhs), op.clone(), Box::new(rhs))
            }
            Expr::Unary(ref op, ref expr) => {
                let expr = Value::from_syntax(expr);
                Value::UnaryExpr(op.clone(), Box::new(expr))
            }
        }
    }
}
