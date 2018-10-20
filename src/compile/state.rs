use compile::{
    Error,
    FunScope,
    VarScope,
    TyScope,
    ValueContext,
    ValueContextKind,
    transform::*,
};
use ir;
use vm::{self, Bc};

pub struct State<'scope> {
    /// Current variable scope.
    pub (in super) var_scope: &'scope mut VarScope,

    /// Current function scope.
    pub (in super) fun_scope: &'scope mut FunScope,

    /// Current type scope.
    pub (in super) ty_scope: &'scope mut TyScope,
}

impl<'n, 'r: 'n, 'scope> TryTransformMut<'n, &'r ir::Action<'n>> for State<'scope> {
    type Out = Vec<Bc>;

    fn try_transform_mut(&mut self, action: &'r ir::Action<'n>) -> Result<Self::Out, Error<'n>> {
        use ir::Action;
        match action {
            // Evaluate an IR value
            Action::Eval(val) => {
                let ctx = ValueContext::new(ValueContextKind::Push, self);
                ctx.try_transform(val)
            },
            // Assign a value to a place in memory
            Action::Assign(lhs, _op, rhs) => {
                if !lhs.is_assign_candidate() {
                    return Err(Error::invalid_assign_lhs(lhs.range(), unimplemented!("TODO: output LHS verbatim from code parsed")));
                }
                
                if lhs.is_immediate() {
                    assert!(matches!(lhs, ir::Value::Symbol(_)));
                } else {
                }
                unimplemented!()
            },
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
            Action::Return(val) => {
                val.as_ref().map(|val| {
                    let ctx = ValueContext::new(ValueContextKind::Ret, self);
                    ctx.try_transform(val)
                }).unwrap_or_else(|| {
                    let ctx = ValueContextKind::Ret;
                    Ok(vec![ctx.transform(vm::Value::None)])
                })
            },
        }
    }
}
