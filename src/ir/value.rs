use syntax::{
    token::{Token, Op},
    tree::Expr,
};
use ir::{Ir, Symbol};

#[derive(Debug, PartialEq, Clone)]
pub enum Const {
    String(String),
    Int(i64),
    // TODO : Bignum
    Float(f64),
    // TODO : user-defined structures
    Bool(bool),
}

impl From<Token> for Const {
    fn from(other: Token) -> Self {
        match other {
            Token::StrLit(s) => Const::String(s),
            Token::IntLit(n, r) => {
                match i64::from_str_radix(n.as_ref(), r as u32) {
                    Ok(v) => Const::Int(v),
                    Err(_) => unimplemented!("bigint")
                }
            },
            Token::FloatLit(f) => Const::Float(str::parse::<f64>(f.as_ref()).expect("invalid float literal")),
            Token::TrueKw => Const::Bool(true),
            Token::FalseKw => Const::Bool(false),
            _ => panic!("invalid constant value: {:?}", other),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Const(Const),
    Symbol(Symbol),
    ArrayAccess(Box<Value>, Box<Value>),
    BinaryExpr(Box<Value>, Op, Box<Value>),
    UnaryExpr(Op, Box<Value>),
    FunCall(Box<Value>, Vec<Value>),
}

impl<'n> Ir<Expr<'n>> for Value {
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
                | Token::Bareword(_) => Value::Symbol(Symbol::from_token(token.token())),
                _ => Value::Const(Const::from(token.token().clone()))
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
