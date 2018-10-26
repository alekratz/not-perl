use std::{
    collections::{
        BTreeSet,
    },
    ops::{Deref, DerefMut},
};
use crate::common::pos::RangeWrapper;
use crate::compile::{
    Error,
    RegSymbolAlloc,
    Scope,
    State,
    Transform,
    TryTransform,
};
use crate::ir;
use crate::vm::{self, Bc, Ref, Symbolic, Symbol};

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub symbol: vm::RegSymbol,
}

impl Var {
    pub fn new(name: String, symbol: vm::RegSymbol) -> Self {
        Var { name, symbol }
    }
}

impl vm::Symbolic for Var {
    type Symbol = vm::RegSymbol;

    fn name(&self) -> &str {
        &self.name
    }

    fn symbol(&self) -> vm::RegSymbol {
        self.symbol
    }
}

pub (in super) struct ValueContext<'s> {
    /// The type of the value context that we're dealing with.
    kind: ValueContextKind,

    /// A reference to the current state of the compiler.
    state: &'s mut State,
}

impl<'s> ValueContext<'s> {
    pub fn new(kind: ValueContextKind, state: &'s mut State) -> Self {
        ValueContext { kind, state }
    }
}

impl<'r, 's> TryTransform<&'r ir::Value> for ValueContext<'s> {
    type Out = Vec<Bc>;

    fn try_transform(self, value: &'r ir::Value) -> Result<Self::Out, Error> {
        use crate::ir::Value;
        let range = value.range();
        match value {
            // Constant/literal value
            Value::Const(RangeWrapper(_, c)) => {
                let value = vm::Value::Const(c.clone());
                Ok(vec![self.kind.transform(value)])
            }

            // User symbol (function, var, or ty)
            Value::Symbol(RangeWrapper(_, s)) => {
                let ref_value = match s {
                    ir::Symbol::Fun(name) => {
                        let symbol = self.state.fun_scope.get_by_name(name)
                            .ok_or_else(|| Error::unknown_fun(range, name.clone()))?
                            .symbol();
                        Ref::Fun(symbol)
                    }
                    ir::Symbol::Variable(name) => {
                        let symbol = self.state.var_scope.get_by_name(name)
                            .expect("variable does not exist in this scope")
                            .symbol();
                        Ref::Reg(symbol)
                    }
                    ir::Symbol::Ty(name) => {
                        let symbol = self.state.ty_scope.get_by_name(name)
                            .ok_or_else(|| Error::unknown_ty(range, name.clone()))?
                            .symbol();
                        Ref::Ty(symbol)
                    }
                };
                // wrap it in a ref value
                let value = vm::Value::Ref(ref_value);
                Ok(vec![self.kind.transform(value)])
            }

            // Array access
            Value::ArrayAccess(_array, _index) => { unimplemented!("TODO(array) : array access") }

            // Binary expression
            Value::BinaryExpr(lhs, op, rhs) => {
                let op_fun = self.state.fun_scope.get_binary_op(op)
                    .ok_or_else(|| Error::unknown_binary_op(range, op.clone()))?
                    .symbol();
                let lhs_sym = self.state.var_scope.insert_anonymous_var();
                let lhs_code = {
                    let lhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(lhs_sym)), self.state);
                    lhs_ctx.try_transform(lhs)?
                };

                let rhs_sym = self.state.var_scope.insert_anonymous_var();
                let rhs_code = {
                    let rhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(rhs_sym)), self.state);
                    rhs_ctx.try_transform(rhs)?
                };
                
                let mut code: Vec<_> = lhs_code.into_iter()
                    .chain(rhs_code.into_iter())
                    .collect();
                code.push(Bc::Push(vm::Value::Ref(Ref::Reg(lhs_sym))));
                code.push(Bc::Push(vm::Value::Ref(Ref::Reg(rhs_sym))));
                code.push(Bc::Call(op_fun));
                // free the anonymous symbols that were just used
                self.state.var_scope.free_anonymous_var(lhs_sym);
                self.state.var_scope.free_anonymous_var(rhs_sym);
                // allocate storage, pop result into storage, and pass storage along to the value
                // context
                let result_var = self.state.var_scope.insert_anonymous_var();
                code.push(self.kind.transform(vm::Value::Ref(Ref::Reg(result_var))));
                self.state.var_scope.free_anonymous_var(result_var);
                Ok(code)
            }

            // Unary expression
            Value::UnaryExpr(op, value) => {
                let _op_fun = self.state.fun_scope.get_unary_op(op)
                    .ok_or_else(|| Error::unknown_unary_op(range, op.clone()))?
                    .symbol();
                let value_sym = self.state.var_scope.insert_anonymous_var();
                let mut value_code = {
                    let value_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(value_sym)), self.state);
                    value_ctx.try_transform(value)?
                };
                value_code.push(self.kind.transform(vm::Value::Ref(Ref::Reg(value_sym))));
                self.state.var_scope.free_anonymous_var(value_sym);
                Ok(value_code)
            }

            // Fun call
            Value::FunCall(fun, args) => {
                let mut code = Vec::new();
                for arg in args {
                    code.append(&mut ValueContext::new(ValueContextKind::Push, self.state).try_transform(arg)?);
                }
                if let Value::Symbol(RangeWrapper(_, ir::Symbol::Fun(name))) = fun.as_ref() {
                    let fun = self.state
                        .fun_scope
                        .get_by_name_and_params(name, args.len());
                    if let Some(fun) = fun {
                        // compile function call like normal
                        code.push(Bc::Call(fun.symbol()));
                    } else {
                        return Err(Error::unknown_fun(range, name.to_string()));
                    }
                } else {
                    // evaluate LHS and try to call it as a function
                    code.append(&mut ValueContext::new(ValueContextKind::Push, self.state).try_transform(fun)?);
                    code.push(Bc::PopCall);
                }

                match self.kind {
                    // pop return value into the given ref
                    ValueContextKind::Store(r) => code.push(Bc::PopStore(r)),
                    // push already happens as a result of the funcall so nothing needs to be done
                    // here
                    ValueContextKind::Push => { },
                    // the return value is already on top of the stack so a simple exit is all
                    // that's required
                    ValueContextKind::Ret => code.push(Bc::Ret),
                }

                Ok(code)
            }
        }
    }
}

pub (in super) enum ValueContextKind {
    /// A value that is to be stored into the given reference.
    Store(Ref),

    /// A value that is to be pushed to the stack.
    Push,

    /// A value that is to be returned.
    Ret,
}

impl Transform<vm::Value> for ValueContextKind {
    type Out = Bc;
    fn transform(self, value: vm::Value) -> Self::Out {
        match self {
            ValueContextKind::Store(r) => Bc::Store(r, value),
            ValueContextKind::Push => Bc::Push(value),
            ValueContextKind::Ret => Bc::PushRet(value),
        }
    }
}

#[derive(Debug)]
pub struct VarScope {
    scope: Scope<Var, RegSymbolAlloc>,

    /// A stack of all unused anonymous variables.
    unused_anon: Vec<BTreeSet<vm::RegSymbol>>,
}

impl VarScope {
    /// Gets a symbol to a variable with the given name, or inserts it if it doesn't exist.
    ///
    /// This will clone the given name if the inserted variable does not exist.
    pub fn get_or_insert(&mut self, name: &str) -> vm::RegSymbol {
        if let Some(var) = self.scope.get_by_name(name) {
            return var.symbol();
        }

        let sym = self.scope.reserve_symbol();
        self.insert(Var::new(name.to_string(), sym));
        sym
    }

    /// Inserts an anonymous variable.
    pub fn insert_anonymous_var(&mut self) -> vm::RegSymbol {
        self.ensure_unused_anon_size();

        let has_unused = self.unused_anon
            .last()
            .map(|u| !u.is_empty())
            .expect("attempted to reserve anonymous variable from depthless scope");
        if has_unused {
            let active = self.unused_anon
                .last_mut()
                .expect("attempted to free anonymous variable from depthless scope");
            let sym = *active
                .iter()
                .min()
                .unwrap();
            active.remove(&sym);
            sym
        } else {
            let sym = self.scope.reserve_symbol();
            let var = Var::new(format!("anonvalue#{:x}", sym.index()), sym);
            self.insert(var);
            sym
        }
    }

    /// Frees the given anonymous variable.
    ///
    /// Note that this does not check if this is actually an anonymous variable being freed. It is
    /// up to the programmer to determine this themselves.
    pub fn free_anonymous_var(&mut self, sym: vm::RegSymbol) {
        self.ensure_unused_anon_size();

        let active = self.unused_anon
            .last_mut()
            .expect("attempted to free anonymous variable from depthless scope");
        assert!(!active.contains(&sym), "attempted to double-free an anonymous variable");
        active.insert(sym);
    }

    /// Pushes or pops an appropriate number of values to the the `unused_anon` stack so that it
    /// matches the current scope stack size.
    fn ensure_unused_anon_size(&mut self) {
        let size_diff: isize = self.unused_anon.len() as isize - self.scope.scope_stack.len() as isize;
        if size_diff < 0 {
            self.unused_anon.append(&mut vec!(BTreeSet::new(); (-size_diff) as usize));
        } else if size_diff > 0 {
            self.unused_anon.truncate(size_diff as usize);
        }
    }
}

impl From<Scope<Var, RegSymbolAlloc>> for VarScope {
    fn from(scope: Scope<Var, RegSymbolAlloc>) -> Self {
        let depth = scope.scope_stack.len();
        VarScope {
            scope, unused_anon: vec!(BTreeSet::new(); depth)
        }
    }
}

impl From<VarScope> for Scope<Var, RegSymbolAlloc> {
    fn from(scope: VarScope) -> Self { scope.scope }
}

impl Deref for VarScope {
    type Target = Scope<Var, RegSymbolAlloc>;

    fn deref(&self) -> &Self::Target { &self.scope }
}

impl DerefMut for VarScope {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.scope }
}

impl Default for VarScope {
    fn default() -> Self {
        VarScope {
            scope: Scope::default(),
            unused_anon: Vec::new(),
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
    fn test_reg_scope() {
        // Check that values are inserted correctly
        let mut reg_scope = VarScope::default();
        reg_scope.push_empty_scope();
        let a_sym = reg_scope.reserve_symbol();
        assert_eq!(a_sym, RegSymbol { global: 0, local: 0 });
        let a = Var::new("a".to_string(), a_sym);
        reg_scope.insert(a);
        let b_sym = reg_scope.reserve_symbol();
        assert_eq!(b_sym, RegSymbol { global: 0, local: 1 });
        let b = Var::new("b".to_string(), b_sym);
        reg_scope.insert(b);

        // Check that local layers can be added while still having access to parent layers
        reg_scope.push_empty_scope();
        let c_sym = reg_scope.reserve_symbol();
        assert_eq!(c_sym, RegSymbol { global: 1, local: 0 });
        let c = Var::new("c".to_string(), c_sym);
        reg_scope.insert(c);
        assert_eq!(reg_scope.get_by_name("a").unwrap().symbol(), a_sym);
        assert_eq!(reg_scope.get_by_name("c").unwrap().symbol(), c_sym);

        // Check that scope layers that have been shed don't yield old values
        reg_scope.pop_scope();
        assert_eq!(reg_scope.get_by_name("b").unwrap().symbol(), b_sym);
        assert!(reg_scope.get_by_name("c").is_none());

        // Check that using the same name in two sibling scopes yields the correct register
        reg_scope.push_empty_scope();
        assert!(reg_scope.get_by_name("c").is_none());
        let c_sym = reg_scope.reserve_symbol();
        assert_eq!(c_sym, RegSymbol { global: 2, local: 0 });
        let c = Var::new("c".to_string(), c_sym);
        reg_scope.insert(c);
        assert_eq!(reg_scope.get_by_name("c").unwrap().symbol(), c_sym);

        // Check that overriding values in the parent scope yields the correct register
        let new_a_sym = reg_scope.reserve_symbol();
        assert_eq!(new_a_sym, RegSymbol { global: 2, local: 1 });
        let a = Var::new("a".to_string(), new_a_sym);
        reg_scope.insert(a);
        assert_eq!(reg_scope.get_by_name("a").unwrap().symbol(), new_a_sym);

        // Check that anonymous symbols are inserted and freed correctly
        let anon_sym1 = reg_scope.insert_anonymous_var();
        reg_scope.free_anonymous_var(anon_sym1);
        let anon_sym2 = reg_scope.insert_anonymous_var();
        assert_eq!(anon_sym1, anon_sym2);

        // Check that overriden values are restored after the layer is shed
        reg_scope.pop_scope();
        assert_eq!(reg_scope.get_by_name("a").unwrap().symbol(), a_sym);

        // Check that anonymous symbols are not allocated to inappropriate scopes
        let anon_sym3 = reg_scope.insert_anonymous_var();
        assert_ne!(anon_sym1, anon_sym3);

        // Check that values are inserted or removed appropriately
        let old_a_sym = reg_scope.get_or_insert("a");
        assert_eq!(a_sym, old_a_sym);
        let new_d_sym = reg_scope.get_or_insert("d");
        assert_eq!(new_d_sym, RegSymbol { global: 0, local: 3 });
    }
}
