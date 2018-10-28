use crate::common::pos::{Ranged, RangeWrapper};
use crate::ir::{Ir, Value};
use crate::syntax::{
    self,
    tree::{Stmt, ConditionBlock},
    token::AssignOp,
};

/// An executable action.
///
/// This is something that changes the state of the VM (e.g. assign a value, evaluate an
/// expression, conditionally execute).
#[derive(Debug, Clone)]
pub enum ActionKind {
    Eval(Value),
    Assign(Value, AssignOp, Value),
    Loop(Block),
    Block(Block),
    ConditionBlock {
        if_block: Box<ConditionAction>,
        elseif_blocks: Vec<ConditionAction>,
        else_block: Option<Box<Action>>,
    },
    Break,
    Continue,
    Return(Option<Value>),
}

pub type Action = RangeWrapper<ActionKind>;

impl Action {
    pub fn from_syntax_block(block: &syntax::tree::Block) -> Self {
        let kind = ActionKind::Block(block.iter()
            .map(Action::from_syntax)
            .collect());
        Action::new(block.range(), kind)
    }
}

impl Ir<Stmt> for Action {
    fn from_syntax(stmt: &Stmt) -> Self {
        let action_kind = match stmt {
            Stmt::UserTy(_) => unreachable!(), // user types are covered as non-action types
            Stmt::Fun(_) => unreachable!(), // functions are covered as non-action types
            Stmt::Expr(expr) => ActionKind::Eval(Value::from_syntax(expr)),
            Stmt::Assign(lhs, op, rhs) => {
                let lhs = Value::from_syntax(lhs);
                let rhs = Value::from_syntax(rhs);
                ActionKind::Assign(lhs, *op, rhs)
            }
            Stmt::If { if_block, elseif_blocks, else_block } => {
                let if_cond_action = ConditionAction::from_condition_block(if_block);
                let elseif_action_blocks = elseif_blocks.iter()
                    .map(ConditionAction::from_condition_block)
                    .collect();
                let else_action_block = else_block.as_ref()
                    .map(Action::from_syntax_block)
                    .map(Box::new);
                ActionKind::ConditionBlock {
                    if_block: Box::new(if_cond_action),
                    elseif_blocks: elseif_action_blocks,
                    else_block: else_action_block,
                }
            }
            Stmt::While(ConditionBlock { condition, block }) => {
                let mut loop_block: Vec<_> = block.iter().map(Action::from_syntax).collect();
                let range = condition.range();
                let condition = ActionKind::ConditionBlock {
                    if_block: Box::new(ConditionAction {
                        condition: Value::from_syntax(condition),
                        action: Action::new(condition.range(), ActionKind::Block(vec![])),
                    }),
                    elseif_blocks: vec![],
                    else_block: Some(Box::new(Action::new(condition.range(), ActionKind::Break))),
                };
                loop_block.push(Action::new(range, condition));
                ActionKind::Loop(loop_block)
            }
            Stmt::Loop(block) => ActionKind::Loop(block.iter().map(Action::from_syntax).collect()),
            // TODO(range) : ActionKind::Break range
            Stmt::Return(expr, range) => ActionKind::Return(expr.as_ref().map(Value::from_syntax)),
            // TODO(range) : ActionKind::Break range
            Stmt::Break(range) => ActionKind::Break,
            // TODO(range) : ActionKind::Continue range
            Stmt::Continue(range) => ActionKind::Continue,
        };
        Action::new(stmt.range(), action_kind)
    }
}

/// A block of actions.
pub type Block = Vec<Action>;

/// An action that executes as a result of the given condition value.
#[derive(Debug, Clone)]
pub struct ConditionAction {
    pub condition: Value,
    pub action: Action,
}

impl ConditionAction {
    pub fn from_condition_block(cond_block: &ConditionBlock) -> Self {
        ConditionAction {
            condition: Value::from_syntax(&cond_block.condition),
            action: Action::from_syntax_block(&cond_block.block),
        }
    }
}
