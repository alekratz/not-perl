/*
 * This I Hate About This Module:
 *
 * * insert_symbol - Symbol types can't be made by the Scope<T> type because it's variant over T.
 *                   Thus, insert_symbol requires both a 'name' and 'symbol' argument, which makes
 *                   things really weird when you consider that insert_value doesn't require a
 *                   symbol at all. I can see things easily getting mis-matched because a symbol
 *                   that should have been inserted actually wasn't inserted at all.
 * * into_names - used only for Scope<T> and VariableScope (of which VariableScope is
 *                Deref<Target=Scope<String>>. Prevents unnecessary copies, but it's incredibly
 *                inconsistent ._.
 */

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use syntax::token::Op;
use ir::compile::*;
use vm;

/// A generic scope that keeps track of multiple layers of some given type.
#[derive(Debug, Clone)]
pub struct Scope<T>
    where T: Debug + Clone
{
    symbols: Vec<vm::Symbol>,
    names: Vec<String>,
    // TODO(scope) : add "all" group, like this:
    // all: Vec<Rc<T>>
    // and just refcount the scope members
    scope: Vec<Vec<T>>,
}

impl<T> Scope<T>
    where T: Debug + Clone
{
    pub fn new() -> Self {
        Scope {
            symbols: vec![],
            names: vec![],
            scope: vec![],
        }
    }

    pub fn names(&self) -> &[String] {
        self.names.as_ref()
    }

    pub fn into_names(self) -> Vec<String> {
        self.names
    }

    pub fn top(&self) -> &[T] {
        self.scope.last().unwrap()
    }

    pub fn symbols(&self) -> &[vm::Symbol] {
        &self.symbols
    }

    /// Adds a new layer to this function scope.
    pub fn add_scope(&mut self) {
        self.scope.push(vec![]);
    }

    /// Sheds a layer of this function scope.
    pub fn shed_scope(&mut self) -> Vec<T> {
        self.scope.pop().expect("tried to shed empty function scope")
    }

    /// Looks up a name of an item based on the given symbol.
    pub fn lookup_name(&self, symbol: vm::Symbol) -> &str {
        &self.names[symbol.index()]
    }

    /// Looks up an item based on the given predicate.
    ///
    /// This method will start at the end of the scope, and work its way towards the beginning. The
    /// first item to match the predicate is returned.
    pub fn lookup<P>(&self, mut predicate: P) -> Option<&T>
        where for<'r> P: FnMut(&'r &T) -> bool
    {
        self.scope
            .iter()
            .rev()
            .filter_map(|collection| collection.iter().find(|t| (predicate)(t)))
            .next()
    }

    /// Looks up an item based on the given predicate.
    ///
    /// This method *only* checks the last scope.
    pub fn lookup_one<P>(&self, predicate: P) -> Option<&T>
        where for<'r> P: FnMut(&'r &T) -> bool
    {
        self.scope
            .last()
            .expect("attempt to search empty scope")
            .iter()
            .filter(predicate)
            .next()
    }

    pub fn insert_symbol(&mut self, symbol: vm::Symbol, name: String) {
        self.names.push(name);
        self.symbols.push(symbol);
    }

    pub fn insert_value(&mut self, value: T) -> &T {
        let current = self.scope.last_mut()
            .unwrap();
        current.push(value);
        current.last()
               .unwrap()
    }
    
    pub fn insert_values(&mut self, mut values: Vec<T>) {
        let current = self.scope.last_mut()
            .unwrap();
        current.append(&mut values);
    }
}

#[derive(Debug, Clone)]
pub struct FunctionScope {
    pub(in super) scope: Scope<FunctionStub>,
    pub(in super) compiled_functions: Vec<vm::Function>,
}

impl FunctionScope {
    pub fn new() -> Self {
        FunctionScope {
            scope: Scope::new(),
            compiled_functions: vec![],
        }
    }

    pub fn with_operators(mut self, operators: Vec<(Op, vm::BuiltinFunction)>) -> Self {
        self.add_scope();
        for (_op, mut function) in operators {
            function.symbol = self.next_symbol(function.name.clone());
            let stub = FunctionStub {
                name: function.name.clone(),
                symbol: function.symbol,
                param_count: function.params.len(),
                return_ty: TyExpr::from_builtin_ty(function.return_ty),
            };
            self.insert_value(stub);
            self.insert_vm_function(vm::Function::Builtin(function));
        }

        self
    }

    pub fn with_builtins(mut self, builtins: Vec<vm::BuiltinFunction>) -> Self {
        self.add_scope();
        for mut function in builtins {
            function.symbol = self.next_symbol(function.name.clone());
            let stub = FunctionStub {
                name: function.name.clone(),
                symbol: function.symbol,
                param_count: function.params.len(),
                return_ty: TyExpr::from_builtin_ty(function.return_ty.into()),
            };
            self.insert_value(stub);
            self.insert_vm_function(vm::Function::Builtin(function));
        }

        self 
    }

    /// Creates the next symbol used for a function with the given name.
    pub fn next_symbol(&mut self, name: String) -> vm::Symbol {
        assert!(self.lookup_local_stub_by_name(&name).is_none());
        let num = self.symbols.len();
        let sym = vm::Symbol::Function(num);
        self.insert_symbol(sym, name);
        sym
    }

    pub fn insert_vm_function(&mut self, function: vm::Function) -> &vm::Function {
        assert!(function.symbol().index() < self.names.len(), "Function symbol number lies outside of name list");
        self.compiled_functions.push(function);
        self.compiled_functions.last()
            .unwrap()
    }

    /// Looks up a function stub based on its name and parameter count.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    /// * `param_count` - the number of parameters in this function stub.
    ///
    /// # Returns
    /// `Some(symbol)` if the function stub was found - otherwise, `None`.
    pub fn lookup_stub(&self, symbol_name: &str, param_count: usize) -> Option<&FunctionStub> {
        self.lookup(|function| self.lookup_name(function.symbol) == symbol_name && function.param_count == param_count)
    }

    /// Looks up a function's symbol based on its name and the current function scope.
    pub fn lookup_symbol(&self, name: &str, param_count: usize) -> Option<vm::Symbol> {
        self.lookup_stub(name, param_count)
            .map(|stub| stub.symbol)
    }

    /// Looks up a function stub based exclusively on its name.
    ///
    /// Since a parameter count is not supplied, name shadowing is possible and a function
    /// unintended by the user may be selected.
    ///
    /// This type of lookup only happens when a parameter count isn't available; e.g., a reference
    /// to a function and not a function call.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    ///
    /// # Returns
    /// `Some(symbol)` if the function stub was found - otherwise, `None`.
    pub fn lookup_stub_by_name(&self, symbol_name: &str) -> Option<&FunctionStub> {
        self.lookup(|function| self.lookup_name(function.symbol) == symbol_name)
    }

    /// Looks for the current function in the local scope only (not hopping up the scope if the
    /// function is not found).
    ///
    /// This function is used to determine if a function of the same name has already been defined
    /// in this scope.
    pub fn lookup_local_stub_by_name(&self, symbol_name: &str) -> Option<&FunctionStub> {
        self.lookup_one(|stub| self.lookup_name(stub.symbol) == symbol_name)
    }

    pub fn lookup_builtin(&self, name: &str) -> Option<vm::Symbol> {
        self.compiled_functions.iter()
            .filter(|f| if let vm::Function::Builtin(f) = f { f.name == name } else { false })
            .map(|f| *f.symbol())
            .next()
    }
}

impl Deref for FunctionScope {
    type Target = Scope<FunctionStub>;

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
    scope: Scope<vm::Symbol>,
}

impl VariableScope {
    pub fn new() -> Self {
        VariableScope {
            scope: Scope::new(),
        }
    }
    
    pub fn into_names(self) -> Vec<String> {
        self.scope.names
    }

    fn next_symbol(&mut self, name: String) -> vm::Symbol {
        let global_idx = self.symbols().len();
        let local_idx = self.scope
            .scope
            .last()
            .unwrap()
            .len();
        let sym = vm::Symbol::Variable(global_idx, local_idx);
        self.insert_symbol(sym, name);
        self.insert_value(sym);
        sym
    }

    /// Inserts a variable symbol into the local symbol table, returning a reference to it.
    ///
    /// If this symbol already exists in the table, the program will panic.
    pub fn insert_local_variable(&mut self, symbol_name: String) -> vm::Symbol {
        assert!(self.lookup_local(&symbol_name).is_none());
        self.next_symbol(symbol_name.clone())
    }

    /// Looks up a variable defined in this scope only.
    ///
    /// This function will not search up the scope stack.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    ///
    /// # Returns
    /// `Some(symbol)` if the local symbol was found - otherwise, `None`.
    pub fn lookup_local(&self, symbol_name: &str) -> Option<vm::Symbol> {
        self.scope
            .lookup_one(|local|{
                assert_matches!(local, vm::Symbol::Variable(_, _));
                self.lookup_name(**local) == symbol_name
            })
            .map(|s| *s)
    }

    /// Looks up a local symbol, or inserts it if necessary.
    pub fn lookup_or_insert_local(&mut self, symbol_name: &str) -> vm::Symbol {
        if let Some(sym) = self.lookup_local(symbol_name) {
            sym
        } else {
            self.insert_local_variable(symbol_name.to_string())
        }
    }

    /// Looks up a local variable name based on its VM symbol.
    ///
    /// This will not traverse the scope stack, only checking the local scope.
    pub fn lookup_local_name(&self, symbol: vm::Symbol) -> Option<&str> {
        assert_matches!(symbol, vm::Symbol::Variable(_, _));
        let name = self.lookup_name(symbol);
        self.lookup_local(name)
            .map(|v| self.lookup_name(v))
    }

    pub fn insert_anonymous_symbol(&mut self) -> vm::Symbol {
        let symbol_name = format!("anonymous symbol #{}", self.symbols.len());
        self.insert_local_variable(symbol_name)
    }
}

impl Deref for VariableScope {
    type Target = Scope<vm::Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for VariableScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

#[derive(Debug, Clone)]
pub struct TyScope {
    scope: Scope<vm::Ty>,
    function_scope: FunctionScope,
}

impl TyScope {
    /// Creates a new user-defined type scope.
    pub fn new() -> Self {
        TyScope {
            scope: Scope::new(),
            function_scope: FunctionScope::new(),
        }
    }

    pub fn function_scope(&self) -> &FunctionScope {
        &self.function_scope
    }

    pub fn function_scope_mut(&mut self) -> &mut FunctionScope {
        &mut self.function_scope
    }

    /// Creates a new user-defined type scope, pre-populated with all of the known builtin types.
    pub fn with_builtins() -> Self {
        const BUILTINS: &'static [(vm::BuiltinTy, &'static str)] = &[
            (vm::BuiltinTy::Float, "Float"),
            (vm::BuiltinTy::Bool, "Bool"),
            (vm::BuiltinTy::Int, "Int"),
            (vm::BuiltinTy::Array, "Array"),
            (vm::BuiltinTy::Str, "Str"),
            (vm::BuiltinTy::Any, "Any"),
            (vm::BuiltinTy::None, "None"),
        ];
        let mut scope = TyScope::new();
        scope.add_scope();
        for (ty, name) in BUILTINS {
            let sym = scope.next_symbol(name.to_string());
            scope.insert_value(vm::Ty::Builtin(*ty, sym));
        }
        scope
    }

    /// Looks for the given type in the local scope only (not hopping up the scope if the type is
    /// not found).
    ///
    /// This function is used to determine if a type of the same name has already been defined in
    /// this scope.
    pub fn lookup_local_ty_by_name(&self, symbol_name: &str) -> Option<&vm::Ty> {
        self.lookup_one(|ty| match ty {
            vm::Ty::User(u) => ty.symbol() == u.symbol,
            vm::Ty::Builtin(b, _) => b.name() == symbol_name,
        })
    }

    /// Looks for the given type in the entire scope, from bottom-to-top.
    pub fn lookup_ty_by_name(&self, symbol_name: &str) -> Option<&vm::Ty> {
        self.lookup(|ty| match ty {
            vm::Ty::User(u) => ty.symbol() == u.symbol,
            vm::Ty::Builtin(b, _) => b.name() == symbol_name,
        })
    }

    pub fn lookup_by_expr(&self, ty_expr: &TyExpr) -> Option<&vm::Ty> {
        match ty_expr {
            TyExpr::Any => Some(self.lookup_builtin(vm::BuiltinTy::Any)),
            TyExpr::Definite(s) => self.lookup_ty_by_name(s),
            TyExpr::None => Some(self.lookup_builtin(vm::BuiltinTy::None)),
        }
    }

    pub fn lookup_builtin(&self, builtin: vm::BuiltinTy) -> &vm::Ty {
        self.lookup(|ty| match ty {
            vm::Ty::User(_) => false,
            vm::Ty::Builtin(b, _) => *b == builtin,
        }).unwrap()
    }

    /// Creates the next symbol used for a type with the given name.
    pub fn next_symbol(&mut self, name: String) -> vm::Symbol {
        assert!(self.lookup_local_ty_by_name(&name).is_none());
        let num = self.symbols.len();
        let sym = vm::Symbol::Ty(num);
        self.insert_symbol(sym, name);
        sym
    }
}

impl Deref for TyScope {
    type Target = Scope<vm::Ty>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for TyScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}
