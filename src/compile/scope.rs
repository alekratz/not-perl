use std::{
    fmt::Debug,
    collections::{
        BTreeMap,
    },
};
use crate::compile::{
    Alloc,
    Ty,
    TySymbolAlloc,
    BlockSymbolAlloc,
};
use crate::vm::{self, Symbolic};

/// A generic scope over a symbolic value.
#[derive(Debug)]
pub struct Scope<T, A>
    where T: Symbolic,
          A: Alloc<T::Symbol>,
{
    pub (in super) scope_stack: Vec<Vec<T::Symbol>>,
    pub (in super) all: BTreeMap<T::Symbol, T>,
    symbol_alloc: A,
}

impl<T, A> Scope<T, A>
    where T: Symbolic + Debug,
          T::Symbol: Debug,
          A: Alloc<T::Symbol> + Default,
{
    /// Reserves a symbol in this scope.
    pub fn reserve_symbol(&mut self) -> T::Symbol {
        self.symbol_alloc.reserve()
    }

    /// Pushes a stack layer to the scope.
    fn push_scope(&mut self, layer: Vec<T>) {
        self.symbol_alloc.on_push_scope();
        self.scope_stack.push(vec![]);
        for value in layer.into_iter() {
            self.insert(value);
        }
    }

    /// Pushes an empty stack layer to the scope.
    ///
    /// This is the equivalent of calling `push_scope(Vec::new())`.
    pub fn push_empty_scope(&mut self) {
        self.push_scope(vec![]);
    }

    /// Pops the top scope layer as a list of symbols.
    ///
    /// Since the actual compile values are still owned by this scope, symbols that point to the
    /// values are popped instead.
    pub fn pop_scope(&mut self) -> Vec<T::Symbol> {
        self.symbol_alloc.on_pop_scope();
        self.scope_stack.pop()
            .expect("attempted to pop depthless scope")
    }

    /// Inserts the given value into this scope.
    pub fn insert(&mut self, value: T) {
        let sym = value.symbol();
        assert!(!self.all.contains_key(&sym), "Symbol already defined in this scope: {:?}", sym);
        self.all.insert(sym, value);
        let top = self.scope_stack
            .last_mut()
            .expect("attempted to push value to depthless scope");
        top.push(sym);
    }

    /// Gets the first scope value that matches this predicate, traversing only the most local
    /// scope.
    pub fn get_local_by<P>(&self, pred: P) -> Option<&T>
        where for <'r> P: Fn(&'r &T) -> bool
    {
        if self.scope_stack.is_empty() {
            return None;
        }
        self.scope_stack
            .last()
            .unwrap()
            .iter()
            .map(|sym| self.all.get(&sym).unwrap())
            .filter(pred)
            .next()
    }

    /// Gets the first scope value that matches the given name, traversing only the most local
    /// scope.
    pub fn get_local_by_name(&self, name: &str) -> Option<&T> {
        self.get_local_by(|v| v.name() == name)
    }

    /// Gets the first scope value that matches the given name, traversing only the most local
    /// scope.
    pub fn get_local_by_symbol(&self, symbol: T::Symbol) -> Option<&T> {
        self.get_local_by(|v| v.symbol() == symbol)
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
        let iter = self.scope_stack
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().map(|sym| *sym))
            .map(move |sym| self.all.get(&sym).unwrap());
        Box::new(iter)
    }

    /// Iterates over all values inserted to this scope.
    pub fn iter_all(&self) -> impl Iterator<Item=&T> {
        let iter = self.all
            .values();
        Box::new(iter)
    }

    /// Consumes this scope, yielding all registered values over the lifetime of this scope.
    pub fn into_all(self) -> Vec<T> {
        self.all
            .into_iter()
            .map(|(_, v)| v)
            .collect()
    }
}

impl<T, A> Default for Scope<T, A>
    where T: Symbolic,
          A: Alloc<T::Symbol> + Default,
{
    fn default() -> Self {
        Scope {
            scope_stack: Vec::new(),
            all: BTreeMap::new(),
            symbol_alloc: A::default(),
        }
    }
}

pub type TyScope = Scope<Ty, TySymbolAlloc>;
pub type LabelScope = Scope<vm::Label, BlockSymbolAlloc>;
