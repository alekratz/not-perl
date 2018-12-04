use std::{
    ops::{Deref, DerefMut},
};
use crate::{
    common::prelude::*,
    compile::{
        AllocScope,
        FunSymbolAlloc,
    },
    ir,
    vm::{self, Symbolic},
};

#[derive(Debug, Clone)]
pub enum Fun {
    /// A known function stub.
    Stub(FunStub),

    /// A compiled or built-in VM function.
    Vm(vm::Fun),

    /// A compiled or built-in VM function for an operator.
    Op(Op, vm::Fun),
}

impl Fun {
    pub fn params(&self) -> usize {
        match self {
            Fun::Stub(s) => s.params,
            | Fun::Vm(b)
            | Fun::Op(_, b) => b.params(),
        }
    }
}

impl Symbolic for Fun {
    type Symbol = vm::FunSymbol;
    fn symbol(&self) -> vm::FunSymbol {
        match self {
            Fun::Stub(s) => s.symbol,
            Fun::Vm(b) | Fun::Op(_, b) => b.symbol(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Fun::Stub(s) => &s.name,
            Fun::Vm(b) | Fun::Op(_, b) => b.name(),
        }
    }
}

impl Ranged for Fun {
    fn range(&self) -> Range {
        match self {
            Fun::Stub(s) => s.range.clone(),
            Fun::Vm(v) => v.range(),
            Fun::Op(_, o) => o.range(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunStub {
    pub name: String,
    pub symbol: vm::FunSymbol,
    pub params: usize,
    pub return_ty: ir::TyExpr,
    pub range: Range,
}

impl FunStub {
    pub fn from_ir_function(symbol: vm::FunSymbol, fun: &ir::Fun) -> Self {
        let name = fun.name().to_string();
        let params = fun.params.len();
        let return_ty = fun.return_ty.clone();
        let range = fun.range();
        FunStub {
            name,
            symbol,
            params,
            return_ty,
            range,
        }
    }
}

#[derive(Debug)]
pub struct FunScope {
    scope: AllocScope<Fun, FunSymbolAlloc>,
}

impl FunScope {
    /// Inserts builtin functions to this scope.
    ///
    /// # Preconditions
    /// A scope layer must exist before builtins are inserted.
    pub fn insert_builtin_functions(&mut self) {
        for builtin in vm::builtin_functions.iter() {
            let sym = self.reserve_symbol();
            self.insert(Fun::Vm(vm::Fun::Builtin(builtin, sym)));
        }
    }

    /// Inserts builtin functions to this scope.
    ///
    /// # Preconditions
    /// A scope layer must exist before builtins are inserted.
    pub fn insert_builtin_ops(&mut self) {
        for vm::BuiltinOp(op, builtin) in vm::builtin_ops.iter() {
            let sym = self.reserve_symbol();
            self.insert(Fun::Op(op.clone(), vm::Fun::Builtin(builtin, sym)));
        }
    }

    /// Gets a function based on its name and parameter count.
    pub fn get_by_name_and_params(&self, name: &str, params: usize) -> Option<&Fun> {
        self.get_by(|f| f.name() == name && f.params() == params)
    }

    /// Gets a function based on its name and parameter count.
    pub fn get_local_by_name_and_params(&self, name: &str, params: usize) -> Option<&Fun> {
        self.get_local_by(|f| f.name() == name && f.params() == params)
    }

    /// Gets a builtin function by its name.
    pub fn get_builtin(&self, name: &str) -> Option<&Fun> {
        self.get_by(|f| matches!(f, Fun::Vm(vm::Fun::Builtin(_, _))) && f.name() == name)
    }

    /// Gets a builtin function by its name.
    pub fn get_binary_op(&self, op: &Op) -> Option<&Fun> {
        self.get_by(|f| if let Fun::Op(o, f) = f { op == o && f.params() == 2 } else { false })
    }

    /// Gets a builtin function by its name.
    pub fn get_unary_op(&self, op: &Op) -> Option<&Fun> {
        self.get_by(|f| if let Fun::Op(o, f) = f { op == o && f.params() == 1 } else { false })
    }
}

impl From<AllocScope<Fun, FunSymbolAlloc>> for FunScope {
    fn from(scope: AllocScope<Fun, FunSymbolAlloc>) -> Self { FunScope { scope } }
}

impl From<FunScope> for AllocScope<Fun, FunSymbolAlloc> {
    fn from(scope: FunScope) -> Self { scope.scope }
}

impl Deref for FunScope {
    type Target = AllocScope<Fun, FunSymbolAlloc>;

    fn deref(&self) -> &AllocScope<Fun, FunSymbolAlloc> { &self.scope }
}

impl DerefMut for FunScope {
    fn deref_mut(&mut self) -> &mut AllocScope<Fun, FunSymbolAlloc> { &mut self.scope }
}

impl Default for FunScope {
    fn default() -> Self {
        FunScope {
            scope: AllocScope::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ir;
    use crate::compile::{FunStub, self};
    use crate::vm::*;
    use super::*;

    #[test]
    fn test_fun_scope() {
        let mut fun_scope = FunScope::default();
        fun_scope.push_empty_scope();
        fun_scope.insert_builtin_functions();
        fun_scope.insert_builtin_ops();

        // Check that builtin functions are added (use both get_by_name_and_params and get_builtin)
        for builtin in builtin_functions.iter() {
            let found = fun_scope.get_by_name_and_params(&builtin.name, builtin.params)
                .expect("Failed to get registered builtin");
            assert_eq!(fun_scope.get_builtin(&builtin.name).unwrap().symbol(), found.symbol());
        }

        // Check that builtin operators are added
        for BuiltinOp(op, builtin) in builtin_ops.iter() {
            if builtin.params == 2 {
                assert!(fun_scope.get_binary_op(op).is_some());
            } else if builtin.params == 1 {
                assert!(fun_scope.get_unary_op(op).is_some());
            }
        }

        // Check that insertion works
        fun_scope.push_empty_scope();
        let stub_a_sym = fun_scope.reserve_symbol();
        let stub_a = compile::Fun::Stub(FunStub {
            name: "a".to_string(),
            symbol: stub_a_sym,
            params: 2,
            range: Range::Builtin,
            return_ty: ir::TyExpr::None,
        });

        fun_scope.insert(stub_a);

        assert!(fun_scope.get_by_name_and_params("a", 2).unwrap().symbol() == stub_a_sym);

        // Check that adding a sub-scope with the same function name and params will yield the more
        // local function
        fun_scope.push_empty_scope();
        let new_stub_a_sym = fun_scope.reserve_symbol();
        let stub_a = compile::Fun::Stub(FunStub {
            name: "a".to_string(),
            symbol: new_stub_a_sym,
            params: 2,
            range: Range::Builtin,
            return_ty: ir::TyExpr::None,
        });
        fun_scope.insert(stub_a);

        {
            let stub_a_lookup = fun_scope.get_by_name_and_params("a", 2)
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), new_stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), stub_a_sym);
        }
        fun_scope.pop_scope();

        // Check that functions with the same name and different args are resolved correctly
        fun_scope.push_empty_scope();
        let params_stub_a_sym = fun_scope.reserve_symbol();
        let stub_a = compile::Fun::Stub(FunStub {
            name: "a".to_string(),
            symbol: params_stub_a_sym,
            params: 3,
            range: Range::Builtin,
            return_ty: ir::TyExpr::None,
        });
        fun_scope.insert(stub_a);

        {
            // Check that we get a(arg, arg, arg) correctly
            let stub_a_lookup = fun_scope.get_by_name_and_params("a", 3)
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), params_stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), stub_a_sym);
            // Check that we get a(arg, arg, arg) correctly with a simple name lookup
            let stub_a_lookup = fun_scope.get_by_name("a")
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), params_stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), stub_a_sym);
            // Check that we get the global a(arg, arg) function
            let stub_a_lookup = fun_scope.get_by_name_and_params("a", 2)
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), params_stub_a_sym);
        }
        fun_scope.pop_scope();

        // Check that functions can be replaced correctly
        let stub_b = compile::Fun::Stub(FunStub {
            name: "b".to_string(),
            symbol: stub_a_sym,
            params: 2,
            range: Range::Builtin,
            return_ty: ir::TyExpr::None,
        });
        let stub_a = fun_scope.replace(stub_b);
        assert_eq!(stub_a.symbol(), stub_a_sym);
        {
            let stub_b_lookup = fun_scope.get_by_name("b")
                .expect("Failed to get replaced function");
            assert_eq!(stub_b_lookup.symbol(), stub_a.symbol());
        }
    }
}
