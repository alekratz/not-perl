use std::{
    fmt::Debug,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    mem,
};
use compile::{
    Alloc,
    Fun,
    Var,
    RegSymbolAlloc,
    FunSymbolAlloc,
    TySymbolAlloc,
    Op,
};
use vm::{self, Symbol, Symbolic};

#[derive(Debug)]
pub struct Scope<T, A>
    where T: Symbolic,
          A: Alloc<T::Symbol>,
{
    scope_stack: Vec<Vec<T::Symbol>>,
    all: BTreeMap<T::Symbol, T>,
    symbol_alloc: A,
}

impl<T, A> Scope<T, A>
    where T: Symbolic + Debug,
          T::Symbol: Debug,
          A: Alloc<T::Symbol> + Default,
{
    pub fn empty() -> Self {
        Scope {
            scope_stack: vec![],
            all: BTreeMap::new(),
            symbol_alloc: A::default(),
        }
    }

    pub fn reserve_symbol(&mut self) -> T::Symbol {
        self.symbol_alloc.reserve()
    }

    /// Pushes the given layer to this scope.
    pub fn push_scope(&mut self, layer: Vec<T>) {
        // push a new layer on, and insert each value one-by-one
        self.scope_stack.push(vec![]);
        for value in layer.into_iter() {
            self.insert(value);
        }
    }

    /// Convenience function which is equivalent to `Scope::push_scope(Vec::new())`.
    pub fn push_empty_scope(&mut self) {
        self.push_scope(Vec::new())
    }

    /// Pops the top scope layer as a list of symbols.
    ///
    /// Since the actual compile values are still owned by this scope, symbols that point to the
    /// values are popped instead.
    pub fn pop_scope(&mut self) -> Vec<T::Symbol> {
        self.scope_stack.pop()
            .expect("attempted to pop depthless scope")
    }

    /// Inserts the given value into this scope.
    pub fn insert(&mut self, value: T) {
        let sym = value.symbol();
        assert!(self.all.contains_key(&sym), "Symbol already defined in this scope: {:?}", sym);
        self.all.insert(sym, value);
        let top = self.scope_stack
            .last_mut()
            .expect("attempted to push value to depthless scope");
        top.push(sym);
    }

    /// Gets the first scope value that matches this predicate, using the same traversal order as
    /// `Scope::iter`.
    pub fn get_by<P>(&self, pred: P) -> Option<&T>
        where for <'r> P: Fn(&'r &T) -> bool
    {
        self.iter()
            .filter(pred)
            .next()
    }

    /// Gets an item by its name, using the same traversal order as `Scope::iter`.
    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.get_by(|t| t.name() == name)
    }

    /// Gets an item by its symbol, using the same traversal order as `Scope::iter`.
    pub fn get_by_symbol(&self, symbol: T::Symbol) -> Option<&T> {
        self.get_by(|t| t.symbol() == symbol)
    }

    /// Iterates over values that are visible in the current scope, starting at the values defined
    /// most locally to the values defined most globally (i.e., in reverse).
    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.scope_stack
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().map(|sym| *sym))
            .map(move |sym| self.all.get(&sym).unwrap())
    }

    /// Iterates over all values inserted to this scope.
    pub fn iter_all(&self) -> impl Iterator<Item=&T> {
        self.all
            .values()
    }

    /// Consumes this scope, yielding all registered values over the lifetime of this scope.
    pub fn into_all(self) -> Vec<T> {
        self.all
            .into_iter()
            .map(|(_, v)| v)
            .collect()
    }
}

pub type TyScope = Scope<vm::Ty, TySymbolAlloc>;

#[derive(Debug)]
pub struct VarScope {
    scope: Scope<Var, RegSymbolAlloc>,
}

impl VarScope {
    pub fn insert_anonymous_var(&mut self) -> vm::RegSymbol {
        let sym = self.scope.reserve_symbol();
        let var = Var::new(format!("anonvalue#{:x}", sym.index()), sym);
        self.insert(var);
        sym
    }
}

impl From<Scope<Var, RegSymbolAlloc>> for VarScope {
    fn from(scope: Scope<Var, RegSymbolAlloc>) -> Self { VarScope { scope } }
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

#[derive(Debug)]
pub struct FunScope {
    scope: Scope<Fun, FunSymbolAlloc>,
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

    /// Replaces the first function to match this predicate.
    ///
    /// # Preconditions
    /// The function to replace must be registered. It does not necessarily need to be visible in
    /// the current scope.
    pub fn replace<P>(&mut self, value: Fun) -> Fun {
        assert!(self.all.contains_key(&value.symbol()),
            format!("tried to replace unregistered function, symbol: {:?} name: {:?}", value.symbol(), value.name()));
        self.all.insert(value.symbol(), value)
            .unwrap()
    }

    /// Gets a function based on its name and parameter count.
    pub fn get_by_name_and_params(&self, name: &str, params: usize) -> Option<&Fun> {
        self.get_by(|f| f.name() == name && f.params() == params)
    }

    /// Gets a builtin function by its name.
    pub fn get_builtin(&self, name: &str) -> Option<&Fun> {
        self.get_by(|f| matches!(f, Fun::Vm(vm::Fun::Builtin(_, _))) && f.name() == name)
    }

    /// Gets a builtin function by its name.
    pub fn get_op(&self, op: &Op) -> Option<&Fun> {
        self.get_by(|f| if let Fun::Op(o, _) = f { op == o } else { false })
    }
}

impl From<Scope<Fun, FunSymbolAlloc>> for FunScope {
    fn from(scope: Scope<Fun, FunSymbolAlloc>) -> Self { FunScope { scope } }
}

impl From<FunScope> for Scope<Fun, FunSymbolAlloc> {
    fn from(scope: FunScope) -> Self { scope.scope }
}

impl Deref for FunScope {
    type Target = Scope<Fun, FunSymbolAlloc>;

    fn deref(&self) -> &Scope<Fun, FunSymbolAlloc> { &self.scope }
}

impl DerefMut for FunScope {
    fn deref_mut(&mut self) -> &mut Scope<Fun, FunSymbolAlloc> { &mut self.scope }
}
