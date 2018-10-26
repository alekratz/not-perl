use crate::compile::{
    GatherFunStubs,
    Error,
    FunScope,
    VarScope,
    LabelScope,
    TyScope,
    transform::*,
};
use crate::ir;

pub struct State {
    pub (in super) var_scope: VarScope,
    pub (in super) fun_scope: FunScope,
    pub (in super) ty_scope: TyScope,
    pub (in super) label_scope: LabelScope,
}

impl State {
    pub fn new() -> Self {
        let mut fun_scope = FunScope::default();
        fun_scope.insert_builtin_functions();
        fun_scope.insert_builtin_ops();
        // TODO : insert builtin types
        State {
            var_scope: VarScope::default(),
            fun_scope,
            ty_scope: TyScope::default(),
            label_scope: LabelScope::default(),
        }
    }

    /// Compile a single IR tree, updating this state.
    pub fn update<'r>(&mut self, ir_tree: &'r ir::IrTree) -> Result<(), Error> {
        // Gather function stubs
        GatherFunStubs::new(self)
            .try_transform(ir_tree.functions())?;
        Ok(())
    }
}
