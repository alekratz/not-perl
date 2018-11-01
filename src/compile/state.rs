use std::path::Path;
use crate::common::ProcessError;
use crate::compile::{
    self,
    CompileFuns,
    FunScope,
    VarScope,
    LabelScope,
    TyScope,
    transform::*,
};
use crate::ir;

#[derive(Debug)]
pub struct State {
    pub (in super) var_scope: VarScope,
    pub (in super) fun_scope: FunScope,
    pub (in super) ty_scope: TyScope,
    pub (in super) label_scope: LabelScope,
}

impl State {
    pub fn new() -> Self {
        State {
            var_scope: VarScope::default(),
            fun_scope: FunScope::default(),
            ty_scope: TyScope::default(),
            label_scope: LabelScope::default(),
        }
    }

    /// Pops a layer off of all compile scopes.
    pub fn pop_scope(&mut self) {
        self.ty_scope.pop_scope();
        self.var_scope.pop_scope();
        self.fun_scope.pop_scope();
        self.label_scope.pop_scope();
    }

    /// Pushes an empty layer onto all compile scopes.
    pub fn push_empty_scope(&mut self) {
        self.ty_scope.push_empty_scope();
        self.var_scope.push_empty_scope();
        self.fun_scope.push_empty_scope();
        self.label_scope.push_empty_scope();
    }

    /// Inserts builtin types, functions, and operators.
    ///
    /// An empty function scope layer and type scope layer are pushed before inserting builtins.
    pub fn insert_builtins(&mut self) {
        self.fun_scope.push_empty_scope();
        self.fun_scope.insert_builtin_functions();
        self.fun_scope.insert_builtin_ops();
    }

    /// Compile a single IR tree, updating this state.
    pub fn update<'r>(&mut self, ir_tree: ir::IrTree) -> Result<(), compile::Error> {
        let ir::IrTree {
            actions,
            functions,
            user_types,
            range,
        } = ir_tree;

        // Compile all functions
        CompileFuns(self).try_transform(functions)?;

        // Create main function

        Ok(())
    }

    pub fn update_from_path(&mut self, path: impl AsRef<Path>) -> Result<(), ProcessError> {
        let ir_tree = ir::IrTree::from_path(path)?;
        self.update(ir_tree)
            .map_err(|e| e.into())
    }
}
