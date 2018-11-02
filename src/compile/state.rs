use crate::compile::{
    FunScope,
    VarScope,
    LabelScope,
    TyScope,
};

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
}
