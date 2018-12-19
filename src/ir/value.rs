use crate::{
    common::prelude::*,
};

#[derive(Debug, Clone)]
pub enum ValueKind {
    BinaryExpr(Box<Value>, Op, Box<Value>),
    UnaryExpr(Op, Box<Value>),
    Literal(LiteralValue),
    Variable(String),
}

pub type Value = RangeWrapper<ValueKind>;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Str(String),
    Int(i64),
    Float(f64),
}
