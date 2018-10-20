use compile::{
    FunScope,
    VarScope,
    TyScope,
    TransformMut,
};
use ir;
use vm::Bc;

pub struct State<'scope> {
    /// Current variable scope.
    pub (in super) var_scope: &'scope mut VarScope,

    /// Current function scope.
    pub (in super) fun_scope: &'scope mut FunScope,

    /// Current type scope.
    pub (in super) ty_scope: &'scope mut TyScope,
}

impl<'n, 'r: 'n, 'scope> TransformMut<&'r ir::Action<'n>> for State<'scope> {
    type Out = Vec<Bc>;

    fn transform_mut(&mut self, action: &'r ir::Action<'n>) -> Self::Out {
        use ir::Action;
        match action {
            // Evaluate an IR value
            Action::Eval(_val) => { unimplemented!() },
            // Assign a value to a place in memory
            Action::Assign(_lhs, _op, _rhs) => { unimplemented!() },
            // Loop over a block
            Action::Loop(_block) => { unimplemented!() },
            // Add a block of actions
            Action::Block(_block) => { unimplemented!() },
            // Execute conditional blocks
            Action::ConditionBlock { if_block: _, elseif_blocks: _, else_block: _ } => { unimplemented!() },
            // Break out of the current block loop
            Action::Break => { unimplemented!() },
            // Continue to the top of this loop
            Action::Continue => { unimplemented!() },
            // Return from the current function
            Action::Return(_value) => { unimplemented!() },
        }
    }
}
