use crate::ir::{Ir, Value};
use crate::syntax::{
    tree::{Stmt, ConditionBlock},
    token::AssignOp,
};

/// An executable action.
///
/// This is something that changes the state of the VM (e.g. assign a value, evaluate an
/// expression, conditionally execute).
#[derive(Debug)]
pub enum Action {
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

impl Action {
    pub fn from_syntax_block(block: impl AsRef<[Stmt]>) -> Self {
        Action::Block(block.as_ref().iter()
            .map(Action::from_syntax)
            .collect())
    }
}

impl Ir<Stmt> for Action {
    fn from_syntax(stmt: &Stmt) -> Self {
        match stmt {
            Stmt::UserTy(_) => unreachable!(), // user types are covered as non-action types
            Stmt::Fun(_) => unreachable!(), // functions are covered as non-action types
            Stmt::Expr(expr) => Action::Eval(Value::from_syntax(expr)),
            Stmt::Assign(lhs, op, rhs) => {
                let lhs = Value::from_syntax(lhs);
                let rhs = Value::from_syntax(rhs);
                Action::Assign(lhs, *op, rhs)
            }
            Stmt::If { ref if_block, ref elseif_blocks, ref else_block } => {
                let if_cond_action = ConditionAction::from_condition_block(if_block);
                let elseif_action_blocks = elseif_blocks.iter()
                    .map(ConditionAction::from_condition_block)
                    .collect();
                let else_action_block = else_block.as_ref()
                    .map(Action::from_syntax_block)
                    .map(Box::new);
                Action::ConditionBlock {
                    if_block: Box::new(if_cond_action),
                    elseif_blocks: elseif_action_blocks,
                    else_block: else_action_block,
                }
            }
            Stmt::While(ConditionBlock { ref condition, ref block }) => {
                let mut loop_block: Vec<_> = block.iter().map(Action::from_syntax).collect();
                let condition = Action::ConditionBlock {
                    if_block: Box::new(ConditionAction {
                        condition: Value::from_syntax(condition),
                        action: Action::Block(vec![]),
                    }),
                    elseif_blocks: vec![],
                    else_block: Some(Box::new(Action::Break)),
                };
                loop_block.push(condition);
                Action::Loop(loop_block)
            }
            Stmt::Loop(block) => Action::Loop(block.iter().map(Action::from_syntax).collect()),
            // TODO(range) : Action::Break range
            Stmt::Return(expr, range) => Action::Return(expr.as_ref().map(Value::from_syntax)),
            // TODO(range) : Action::Break range
            Stmt::Break(range) => Action::Break,
            // TODO(range) : Action::Continue range
            Stmt::Continue(range) => Action::Continue,
        }
    }
}

/// A block of actions.
pub type Block = Vec<Action>;

/// An action that executes as a result of the given condition value.
#[derive(Debug)]
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
