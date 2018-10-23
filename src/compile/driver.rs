use compile::{
    Unit,
    State,
    Error,
    FunScope,
    VarScope,
    LabelScope,
    TyScope,
};
use ir;

/// Compiler driver.
///
/// This is what directly converts IR into a final compile unit.
pub struct Driver {
    var_scope: VarScope,
    fun_scope: FunScope,
    ty_scope: TyScope,
    label_scope: LabelScope,
}

impl Driver {
    pub fn new() -> Self {
        let mut fun_scope = FunScope::default();
        fun_scope.insert_builtin_functions();
        fun_scope.insert_builtin_ops();
        // TODO : insert builtin types
        Driver {
            var_scope: VarScope::default(),
            fun_scope,
            ty_scope: TyScope::default(),
            label_scope: LabelScope::default(),
        }
    }

    /// Compile a single IR tree, updating this driver's current state.
    pub fn update(&mut self, ir_tree: &ir::IrTree) -> Result<(), Error> {
        let mut state = State {
            var_scope: &mut self.var_scope,
            fun_scope: &mut self.fun_scope,
            ty_scope: &mut self.ty_scope,
            label_scope: &mut self.label_scope,
        };

        unimplemented!()
    }
}

impl From<Driver> for Unit {
    fn from(other: Driver) -> Self {
        unimplemented!("TODO : Driver -> compile unit")
    }
}
