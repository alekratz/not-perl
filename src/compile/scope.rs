use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};
use compile::{
    ReserveSymbol,
    TySymbolAlloc,
    FunctionSymbolAlloc,
    FunctionStub,
    Variable,
    VariableSymbolAlloc,
};
use ir;
use vm::{
    Symbol,
    Symbolic,
    self,
};

#[derive(Clone, Debug)]
pub struct Scope<T, A>
    where T: Symbolic,
          A: ReserveSymbol<Symbol=T::Symbol>
{
    symbol_alloc: A,
    scope: Vec<Vec<Rc<T>>>,
    all: Vec<Rc<T>>,
}

impl<T, A> Scope<T, A>
    where T: Symbolic,
          A: ReserveSymbol<Symbol=T::Symbol>
{
    /// Creates an empty scope with the given symbol allocator.
    pub fn empty(symbol_alloc: A) -> Self {
        Scope {
            symbol_alloc,
            scope: vec![],
            all: vec![],
        }
    }

    /// Pushes the given scope to the value scope stack.
    pub fn push_scope(&mut self, mut scope_layer: Vec<Rc<T>>) {
        self.symbol_alloc.enter_scope();
        self.scope.push(scope_layer.clone());
        self.all.append(&mut scope_layer);
    }

    /// Shortcut method for `self.push_scope(vec![])`.
    pub fn push_empty_scope(&mut self) {
        self.push_scope(vec![]);
    }

    /// Pops the current scope from the value scope stack.
    pub fn pop_scope(&mut self) -> Option<Vec<Rc<T>>> {
        self.symbol_alloc.exit_scope();
        self.scope.pop()
    }

    /// Pushes the given value to the current scope.
    ///
    /// This function will panic if there is no current scope.
    pub fn push_value(&mut self, value: T) {
        // TODO : check that scope symbol value doesn't already exist
        let last = self.scope.last_mut()
            .expect("attempted to push value to empty scope");
        let rc = Rc::new(value);
        last.push(Rc::clone(&rc));
        self.all.push(rc);
    }
    
    /// Pushes all values in the given array to the current scope.
    ///
    /// This function will panic if there is no current scope.
    pub fn push_all_values(&mut self, values: Vec<T>) {
        for value in values {
            self.push_value(value);
        }
    }

    /// Removes a value using the given predicate.
    ///
    /// This will remove the first value to return true from the given predicate.
    pub fn remove_value_by<P>(&mut self, mut predicate: P) -> Option<Rc<T>>
        where for<'r> P: FnMut(&'r &T) -> bool
    {
        let position = self.iter()
            .position(|t| (predicate)(&t))?;
        let last = self.scope
            .last_mut()
            .unwrap();
        Some(last.remove(position))
    }

    pub fn remove_value_by_symbol(&mut self, symbol: T::Symbol) -> Option<Rc<T>> {
        self.remove_value_by(|value| value.symbol() == symbol)
    }

    pub fn remove_value_by_name(&mut self, name: &str) -> Option<Rc<T>> {
        self.remove_value_by(|value| value.name() == name)
    }

    /// Looks up a value using the given predicate.
    ///
    /// Iteration order is described in `iter` documentation.
    ///
    /// # Returns
    /// The first value to match the given predicate is returned. If no value matches the
    /// predicate, `None` is returned.
    pub fn get_value_by<P>(&self, mut predicate: P) -> Option<&T>
        where for<'r> P: FnMut(&'r &T) -> bool
    {
        self.iter()
            .filter(|t| (predicate)(&t))
            .next()
    }

    /// Looks up a local value using the given predicate.
    ///
    /// A local value lookup only looks up a value in the topmost scope, not traversing up the
    /// stack.
    pub fn get_local_value_by<P>(&self, mut predicate: P) -> Option<&T>
        where for<'r> P: FnMut(&'r &T) -> bool
    {
        self.iter_local()
            .filter(|t| (predicate)(&t))
            .next()
    }

    /// Looks up a value by its symbol.
    pub fn get_value_by_symbol(&self, symbol: T::Symbol) -> Option<&T> {
        self.get_value_by(|value| value.symbol() == symbol)
    }

    /// Looks up a local value by its symbol.
    pub fn get_local_value_by_symbol(&self, symbol: T::Symbol) -> Option<&T> {
        self.get_local_value_by(|value| value.symbol() == symbol)
    }

    /// Looks up a value by its name.
    pub fn get_value_by_name(&self, name: &str) -> Option<&T> {
        self.get_value_by(|value| value.name() == name)
    }

    /// Looks up a local value by its name.
    pub fn get_local_value_by_name(&self, name: &str) -> Option<&T> {
        self.get_local_value_by(|value| value.name() == name)
    }

    /// Creates an iterator across all values in the scope.
    ///
    /// The scope is iterated from the the bottom-most (local) scope to the top-most (global)
    /// scope.
    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.scope
            .iter()
            .rev()
            .flatten()
            .map(Rc::as_ref)
    }

    /// Creates an iterator across all values in the local scope.
    ///
    /// This only iterates "local" scope values. The ordering of values is arbitrary and not bound
    /// to any standard. This method panics if no scope is present.
    pub fn iter_local(&self) -> impl Iterator<Item=&T> {
        self.scope
            .last()
            .unwrap()
            .iter()
            .map(Rc::as_ref)
    }

    /// Gets all scopes in order of most-global to most-local, i.e. "topmost" to "bottom-most".
    pub fn scopes(&self) -> impl Iterator<Item=impl Iterator<Item=&T>> {
        self.scope
            .iter()
            .map(|s| s.iter().map(Rc::as_ref))
    }

    /// Gets the array of all values held in and created by this scope.
    pub fn all(&self) -> impl Iterator<Item=&T> {
        self.all
            .iter()
            .map(Rc::as_ref)
    }
}

impl<T, A> ReserveSymbol for Scope<T, A>
    where T: Symbolic,
          A: ReserveSymbol<Symbol=T::Symbol>
{
    type Symbol = T::Symbol;

    fn reserve_symbol(&mut self) -> Self::Symbol {
        self.symbol_alloc.reserve_symbol()
    }

    fn enter_scope(&mut self) { panic!("enter_scope should not be called on Scope types"); }
    fn exit_scope(&mut self) { panic!("enter_scope should not be called on Scope types"); }
}

/// A specialized implementation of a scope for VM types.
#[derive(Clone, Debug)]
pub struct TyScope {
    scope: Scope<vm::Ty, TySymbolAlloc>,
}

impl TyScope {
    pub fn new() -> Self {
        TyScope {
            scope: Scope::empty(TySymbolAlloc::new()),
        }
    }

    /// Appends builtin types to this scope, adding a new scope layer as well.
    pub fn with_builtins(mut self) -> Self {
        const BUILTINS: &'static [vm::BuiltinTy] = &[
            vm::BuiltinTy::Float,
            vm::BuiltinTy::Bool,
            vm::BuiltinTy::Int,
            vm::BuiltinTy::Array,
            vm::BuiltinTy::Str,
            vm::BuiltinTy::Any,
            vm::BuiltinTy::None,
        ];
        self.scope.push_empty_scope();
        for ty in BUILTINS {
            let sym = self.reserve_symbol();
            self.scope.push_value(vm::Ty::Builtin(*ty, sym));
        }
        self
    }

    /// Gets the defined type instance for the given builtin.
    pub fn get_builtin(&self, builtin: vm::BuiltinTy) -> &vm::Ty {
        self.get_value_by(|ty| match ty {
            vm::Ty::Builtin(b, _) => *b == builtin,
            _ => false,
        }).expect("could not find builtin")
    }

    /// Looks up a type by the given type expression.
    pub fn get_value_by_expr(&self, ty_expr: &ir::TyExpr) -> Option<&vm::Ty> {
        match ty_expr {
            ir::TyExpr::Any => Some(self.get_builtin(vm::BuiltinTy::Any)),
            ir::TyExpr::Definite(s) => self.get_value_by_name(s),
            ir::TyExpr::None => Some(self.get_builtin(vm::BuiltinTy::None)),
        }
    }

    pub fn into_all(mut self) -> Vec<vm::Ty> {
        // clear the current scope because it holds Rc<T> in the 'all' array
        self.scope.scope.clear();
        self.scope
            .all
            .into_iter()
            .map(Rc::try_unwrap)
            .map(Result::unwrap)
            .collect()
    }
}

impl Deref for TyScope {
    type Target = Scope<vm::Ty, TySymbolAlloc>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for TyScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

#[derive(Debug, Clone)]
pub struct FunctionScope {
    scope: Scope<FunctionStub, FunctionSymbolAlloc>,
    vm_functions: Vec<vm::Function>,
}

impl FunctionScope {
    pub fn new() -> Self {
        FunctionScope {
            scope: Scope::empty(FunctionSymbolAlloc::new()),
            vm_functions: vec![],
        }
    }

    pub fn with_builtins(mut self, builtins: Vec<vm::BuiltinFunction>) -> Self {
        self.push_empty_scope();
        for mut function in builtins {
            function.symbol = self.reserve_symbol();
            let stub = FunctionStub {
                name: function.name.clone(),
                symbol: function.symbol,
                params: function.params.len(),
                return_ty: ir::TyExpr::from_builtin_ty(function.return_ty.into()),
            };
            self.push_value(stub);
            self.push_vm_function(vm::Function::Builtin(function));
        }

        self 
    }

    pub fn push_vm_function(&mut self, function: vm::Function) {
        self.vm_functions.push(function);
    }

    pub fn get_stub_by_params(&self, name: &str, params: usize) -> Option<&FunctionStub> {
        self.get_value_by(|function| function.params == params && function.name() == name)
    }

    pub fn get_builtin(&self, name: &str) -> Option<&vm::Function> {
        self.vm_functions.iter()
            .filter(|function| function.is_builtin() && function.name() == name)
            .next()
    }

    pub fn into_vm_functions(self) -> Vec<vm::Function> {
        self.vm_functions
    }
}

impl Deref for FunctionScope {
    type Target = Scope<FunctionStub, FunctionSymbolAlloc>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for FunctionScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

#[derive(Debug, Clone)]
pub struct VariableScope {
    scope: Scope<Variable, VariableSymbolAlloc>,
}

impl VariableScope {
    pub fn new() -> Self {
        VariableScope {
            scope: Scope::empty(VariableSymbolAlloc::new()),
        }
    }
    
    pub fn push_anonymous_symbol(&mut self) -> &Variable {
        let sym = self.reserve_symbol();
        let var = Variable(format!("##anonymous var${:x}##", sym.index()), sym);
        self.push_value(var);
        self.all()
            .last()
            .unwrap()
    }
}

impl Deref for VariableScope {
    type Target = Scope<Variable, VariableSymbolAlloc>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for VariableScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

/*

mod test {
    use galvanic_test::*;

    test_suite! {
        name test_compile_scope;

        test test_get_value
    }
}
 */
