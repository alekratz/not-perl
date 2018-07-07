use syntax::{
    token::{Token, Op},
    tree::Expr,
    Ranged,
};
use vm::{
    self,
    Bc,
};
use ir::{Ir, Symbol, RangeSymbol};

// NOTE: not Eq because f64 is not Eq
#[derive(Debug, PartialEq, Clone)]
pub enum Const {
    Str(String),
    Int(i64),
    // TODO : Bignum
    Float(f64),
    // TODO : user-defined structures
    Bool(bool),
}

pub type RangeConst<'n> = Ranged<'n, Const>;

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
pub enum Value<'n> {
    Const(RangeConst<'n>),
    Symbol(RangeSymbol<'n>),
    ArrayAccess(Box<Value<'n>>, Box<Value<'n>>),
    BinaryExpr(Box<Value<'n>>, Op, Box<Value<'n>>),
    UnaryExpr(Op, Box<Value<'n>>),
    FunCall(Box<Value<'n>>, Vec<Value<'n>>),
}

impl<'n> Ir<Expr<'n>> for Value<'n> {
    fn from_syntax(expr: &Expr<'n>) -> Self {
        match expr {
            Expr::FunCall { ref function, ref args } => {
                let function = Value::from_syntax(function);
                let mut fun_args = vec![];
                for arg in args.iter() {
                    fun_args.push(Value::from_syntax(arg));
                }
                Value::FunCall(Box::new(function), fun_args)
            }
            Expr::ArrayAccess { ref array, ref index } => {
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
