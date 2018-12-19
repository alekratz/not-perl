use crate::{
    syntax::tree::Stmt,
    common::prelude::*,
    ir::Value,
};

/// A kind of action that can be taken by the language.
///
/// This roughly translates to a statement in the syntax tree.
#[derive(Debug, Clone)]
pub enum ActionKind {
    Eval(Value),
    Assign(Value, Value),
    Loop(Value, Block),
    Block(Block),
    Continue,
    Break,
    Return(Option<Value>),
}

pub type Action = RangeWrapper<ActionKind>;

/// A list of actions.
pub type Block = Vec<Action>; 

impl From<Stmt> for Action {
    fn from(stmt: Stmt) -> Self {
        let range = stmt.range();
        RangeWrapper(range, stmt.into())
    }
}

impl From<Stmt> for ActionKind {
    fn from(stmt: Stmt) -> Self {
        unimplemented!()
    }
}
