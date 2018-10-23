use compile::{
    FunScope,
    VarScope,
    LabelScope,
    TyScope,
};

pub struct State<'scope> {
    /// Current variable scope.
    pub (in super) var_scope: &'scope mut VarScope,

    /// Current function scope.
    pub (in super) fun_scope: &'scope mut FunScope,

    /// Current type scope.
    pub (in super) ty_scope: &'scope mut TyScope,

    /// Current label scope.
    pub (in super) label_scope: &'scope mut LabelScope,
}
