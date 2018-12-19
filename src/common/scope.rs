use std::{collections::BTreeMap, fmt::Debug};

/// A generic scope over a symbolic value.
#[derive(Debug)]
pub struct ReadOnlyScope<T>
where
    T: Symbolic,
{
    pub(in crate) scope_stack: Vec<Vec<T::Symbol>>,
    pub(in crate) all: BTreeMap<T::Symbol, T>,
}

impl<T> ReadOnlyScope<T>
where
    T: Symbolic + Debug,
    T::Symbol: Debug,
{
    /// Inserts the given value into this scope.
    pub fn insert(&mut self, value: T) {
        let sym = value.symbol();
        assert!(
            !self.all.contains_key(&sym),
            "Symbol already defined in this scope: {:?}",
            sym
        );
        self.all.insert(sym, value);
        let top = self
            .scope_stack
            .last_mut()
            .expect("attempted to push value to depthless scope");
        top.push(sym);
    }

    /// Gets the first scope value that matches this predicate, traversing only the most local
    /// scope.
    pub fn get_local_by<P>(&self, pred: P) -> Option<&T>
    where
        for<'r> P: Fn(&'r &T) -> bool,
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
    /// `ReadOnlyScope::iter`.
    pub fn get_by<P>(&self, pred: P) -> Option<&T>
    where
        for<'r> P: Fn(&'r &T) -> bool,
    {
        self.iter().filter(pred).next()
    }

    /// Gets an item by its name, using the same traversal order as `ReadOnlyScope::iter`.
    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.get_by(|t| t.name() == name)
    }

    /// Gets an item by its symbol, using the same traversal order as `ReadOnlyScope::iter`.
    pub fn get_by_symbol(&self, symbol: T::Symbol) -> Option<&T> {
        self.get_by(|t| t.symbol() == symbol)
    }

    /// Iterates over values that are visible in the current scope, starting at the values defined
    /// most locally to the values defined most globally (i.e., in reverse).
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let iter = self
            .scope_stack
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().map(|sym| *sym))
            .map(move |sym| self.all.get(&sym).unwrap());
        Box::new(iter)
    }

    /// Iterates over all values inserted to this scope.
    pub fn iter_all(&self) -> impl Iterator<Item = &T> {
        let iter = self.all.values();
        Box::new(iter)
    }

    /// Consumes this scope, yielding all registered values over the lifetime of this scope.
    pub fn into_all(self) -> Vec<T> {
        self.all.into_iter().map(|(_, v)| v).collect()
    }

    /// Replaces the first item to match the predicate.
    ///
    /// # Preconditions
    /// The item to replace must be registered. It does not necessarily need to be visible in the
    /// current scope.
    pub fn replace(&mut self, value: T) -> T {
        assert!(
            self.all.contains_key(&value.symbol()),
            "tried to replace unregistered function, symbol: {:?} name: {:?}",
            value.symbol(),
            value.name()
        );
        self.all.insert(value.symbol(), value).unwrap()
    }
}

impl<T> Default for ReadOnlyScope<T>
where
    T: Symbolic,
{
    fn default() -> Self {
        ReadOnlyScope {
            scope_stack: Vec::new(),
            all: BTreeMap::new(),
        }
    }
}
