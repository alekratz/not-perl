use crate::{
    common::prelude::*,
    syntax::{tree::Expr, token::{Token, RangedToken}},
};

#[derive(Debug, Clone)]
pub enum ValueKind {
    FunCall(Box<Value>, Vec<Value>),
    BinaryExpr(Box<Value>, Op, Box<Value>),
    UnaryExpr(Op, Box<Value>),
    Immediate(Immediate),
}

pub type Value = RangeWrapper<ValueKind>;

impl From<Expr> for Value {
    fn from(expr: Expr) -> Self {
        let range = expr.range();
        RangeWrapper(range, expr.into())
    }
}

impl From<Expr> for ValueKind {
    fn from(expr: Expr) -> Self {
        match expr {
            Expr::FunCall { function, args, .. } => ValueKind::FunCall(
                Box::new((*function).into()), args.into_iter().map(From::from).collect()),
            Expr::ArrayAccess { .. } => { unimplemented!("TODO(array) array access From<Expr> for ValueKind") }
            Expr::Atom(token) => ValueKind::Immediate(token.into()),
            Expr::Unary(op, expr) => ValueKind::UnaryExpr(op, Box::new((*expr).into())),
            Expr::Binary(lhs, op, rhs) => ValueKind::BinaryExpr(
                Box::new((*lhs).into()), op, Box::new((*rhs).into())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Immediate {
    Var(String),
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<RangedToken> for Immediate {
    fn from(RangeWrapper(r, token): RangedToken) -> Self {
        match token {
            Token::Variable(v) => Immediate::Var(v),
            Token::StrLit(s) => Immediate::Str(s),
            Token::IntLit(i, base) => Immediate::Int(i64::from_str_radix(&i, base as u32)
                                                     .expect("invalid parsed int - this is a compiler bug")),
            Token::FloatLit(f) => Immediate::Float(f.parse().expect("invalid parsed float - this is a compiler bug")),
            // this could be done with token == ... but I wanted an excuse to use @ binding syntax
            t @ Token::TrueKw | t @ Token::FalseKw => Immediate::Bool(t == Token::TrueKw),
            _ => panic!("invalid literal value from token at {}: {:?}", r, token),
        }
    }
}
