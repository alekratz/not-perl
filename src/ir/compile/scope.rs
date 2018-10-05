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
use vm::{
    self,
    Symbol,
};

/// A generic scope that keeps track of multiple layers of some given type.
#[derive(Debug, Clone)]
pub struct Scope<T, SymbolT>
    where T: Debug + Clone,
          SymbolT: Debug + Clone + Symbol
{
    symbols: Vec<SymbolT>,
    names: Vec<String>,
    // TODO(scope) : add "all" group, like this:
    // all: Vec<Rc<T>>
    // and just refcount the scope members
    scope: Vec<Vec<T>>,
}

impl<T, SymbolT> Scope<T, SymbolT>
    where T: Debug + Clone,
          SymbolT: Debug + Clone + Symbol
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

    pub fn symbols(&self) -> &[SymbolT] {
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
    pub fn lookup_name(&self, symbol: SymbolT) -> &str {
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

    pub fn insert_symbol(&mut self, symbol: SymbolT, name: String) {
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
pub struct VariableScope {
    scope: Scope<vm::VariableSymbol, vm::VariableSymbol>,
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

    fn next_symbol(&mut self, name: String) -> vm::VariableSymbol {
        let global = self.symbols().len();
        let local = self.scope
            .scope
            .last()
            .unwrap()
            .len();
        let sym = vm::VariableSymbol {
            global,
            local,
        };
        self.insert_symbol(sym, name);
        self.insert_value(sym);
        sym
    }

    /// Inserts a variable symbol into the local symbol table, returning a reference to it.
    ///
    /// If this symbol already exists in the table, the program will panic.
    pub fn insert_local_variable(&mut self, symbol_name: String) -> vm::VariableSymbol {
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
    pub fn lookup_local(&self, symbol_name: &str) -> Option<vm::VariableSymbol> {
        self.scope
            .lookup_one(|local| self.lookup_name(**local) == symbol_name)
            .map(|s| *s)
    }

    /// Looks up a local symbol, or inserts it if necessary.
    pub fn lookup_or_insert_local(&mut self, symbol_name: &str) -> vm::VariableSymbol {
        if let Some(sym) = self.lookup_local(symbol_name) {
            sym
        } else {
            self.insert_local_variable(symbol_name.to_string())
        }
    }

    /// Looks up a local variable name based on its VM symbol.
    ///
    /// This will not traverse the scope stack, only checking the local scope.
    pub fn lookup_local_name(&self, symbol: vm::VariableSymbol) -> Option<&str> {
        let name = self.lookup_name(symbol);
        self.lookup_local(name)
            .map(|v| self.lookup_name(v))
    }

    pub fn insert_anonymous_symbol(&mut self) -> vm::VariableSymbol {
        let symbol_name = format!("anonymous symbol #{}", self.symbols.len());
        self.insert_local_variable(symbol_name)
    }
}

impl Deref for VariableScope {
    type Target = Scope<vm::VariableSymbol, vm::VariableSymbol>;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl DerefMut for VariableScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}

