use crate::{
    syntax::{tree::{self, Stmt, ConditionBlock}, token::AssignOp},
    common::prelude::*,
    ir::{Block, Value},
};

/// A kind of action that can be taken by the language.
///
/// This roughly translates to a statement in the syntax tree.
#[derive(Debug, Clone)]
pub enum ActionKind {
    Eval(Value),
    Assign(Value, Value),

    /// An "augmented assign", for when an augmented assignment operator (e.g. `+=` or `*=`) is
    /// used.
    AugAssign(Value, Op, Value),

    Loop(Box<Action>),
    Block(Block),
    ConditionBlock {
        condition: Value,
        success: Box<Action>,
        failure: Box<Action>,
    },
    Continue,
    Break,
    Return(Option<Value>),
    Nop,
}

pub type Action = RangeWrapper<ActionKind>;

impl From<Stmt> for Action {
    fn from(stmt: Stmt) -> Self {
        let range = stmt.range();
        RangeWrapper(range, stmt.into())
    }
}

impl From<Stmt> for ActionKind {
    fn from(stmt: Stmt) -> Self {
        match stmt {
            Stmt::Expr(e) => ActionKind::Eval(e.into()),
            Stmt::Assign(lhs, AssignOp::Equals, rhs) => ActionKind::Assign(lhs.into(), rhs.into()),
            Stmt::Assign(lhs, op, rhs) => ActionKind::AugAssign(
                lhs.into(), op.into_op().expect("could not convert AssignOp into appropriate Op"), rhs.into()),
            Stmt::While(condition_block) => {
                let full_range = condition_block.range();
                let ConditionBlock { condition, block } = condition_block;
                let cond_range = condition.range();
                let condition_block = ActionKind::ConditionBlock {
                    condition: condition.into(),
                    success: Box::new(block.into()),
                    failure: Box::new(RangeWrapper(cond_range, ActionKind::Break)),
                };
                ActionKind::Loop(Box::new(RangeWrapper(full_range, condition_block)))
            }
            Stmt::Loop(block) => ActionKind::Loop(Box::new(block.into())),
            Stmt::If { if_block, elseif_blocks, else_block } => {
                let mut tail_range = else_block.as_ref()
                    .map(|b| b.range())
                    .unwrap_or_else(|| elseif_blocks.last()
                                    .map(|b| b.range())
                                    .unwrap_or_else(|| if_block.range()));
                let else_block = else_block
                    .map(ActionKind::from)
                    .unwrap_or(ActionKind::Nop);

                let mut tail = else_block;
                for elseif_block in elseif_blocks.into_iter().rev() {
                    let new_tail_range = elseif_block.range();
                    let ConditionBlock { condition, block } = elseif_block;
                    tail = ActionKind::ConditionBlock {
                        condition: condition.into(),
                        success: Box::new(block.into()),
                        failure: Box::new(RangeWrapper(tail_range, tail)),
                    };
                    tail_range = new_tail_range;
                }

                let ConditionBlock { condition: if_cond, block: if_block } = if_block;
                ActionKind::ConditionBlock {
                    condition: if_cond.into(),
                    success: Box::new(if_block.into()),
                    failure: Box::new(RangeWrapper(tail_range, tail)),
                }
            }
            Stmt::Continue(_) => ActionKind::Continue,
            Stmt::Break(_) => ActionKind::Break,
            Stmt::Return(expr, _) => ActionKind::Return(expr.map(From::from)),
        }
    }
}

impl From<tree::Block> for Action {
    fn from(block: tree::Block) -> Self {
        let range = block.range();
        RangeWrapper(range, ActionKind::from(block))
    }
}

impl From<tree::Block> for ActionKind {
    fn from(block: tree::Block) -> Self {
        ActionKind::Block(block.into())
    }
}
