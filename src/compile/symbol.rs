use std::mem;
use vm::{
    Symbol,
    Symbolic,
    TySymbol,
    VariableSymbol,
    ValueIndex,
    self,
};

/// Behavior definition for symbols that will give the next ordinal symbol.
pub trait NextSymbol {
    type Symbol: Symbol;
    /// Gets the next symbol after this one.
    fn next_symbol(&self) -> Self::Symbol;
}

// Impl for all symbolic types

impl<T> NextSymbol for T
    where T: Symbolic,
          T::Symbol: NextSymbol<Symbol=T::Symbol>
{
    type Symbol = T::Symbol;
    fn next_symbol(&self) -> Self::Symbol {
        self.symbol().next_symbol()
    }
}

// Impl for individual symbols

impl NextSymbol for TySymbol {
    type Symbol = Self;

    fn next_symbol(&self) -> Self {
        let TySymbol(sym) = self;
        TySymbol(sym + 1)
    }
}

impl NextSymbol for VariableSymbol {
    type Symbol = Self;

    fn next_symbol(&self) -> Self {
        let VariableSymbol { global, local } = self;
        VariableSymbol {
            global: global + 1,
            local: local + 1,
        }
    }
}

/// Behavior for reserving symbols in the compiler.
pub trait ReserveSymbol {
    type Symbol: Symbol;

    /// Reserves a symbol for the scope.
    fn reserve_symbol(&mut self) -> Self::Symbol;
    
    /// This method gets called on the allocator by the parent scope right before a new scope layer
    /// is added.
    fn enter_scope(&mut self) { }

    /// This method gets called on the allocator by the parent scope right after a new scope layer
    /// is removed.
    ///
    /// This is useful for symbols whose values depend on where they exist in the scope stack.
    fn exit_scope(&mut self) { }
}

/// A generic symbol allocator that will naiively reserve symbols in order.
///
/// This implementation ignores the `enter_scope` and `exit_scope` methods of the `ReserveSymbol`
/// trait, leaving them as default no-ops.
#[derive(Debug, Clone)]
pub struct SymbolAlloc<T>
    where T: NextSymbol,
          T::Symbol: NextSymbol<Symbol=T::Symbol>
{
    reserve_next: T::Symbol,
}

impl<T> NextSymbol for SymbolAlloc<T>
    where T: NextSymbol,
          T::Symbol: NextSymbol<Symbol=T::Symbol>
{
    type Symbol = T::Symbol;
    fn next_symbol(&self) -> T::Symbol {
        self.reserve_next.next_symbol()
    }
}

impl<T> ReserveSymbol for SymbolAlloc<T>
    where T: NextSymbol,
          T::Symbol: NextSymbol<Symbol=T::Symbol>
{
    type Symbol = T::Symbol;
    fn reserve_symbol(&mut self) -> T::Symbol {
        let next = self.next_symbol();
        mem::replace(&mut self.reserve_next, next)
    }
}

impl<T> SymbolAlloc<T>
    where T: NextSymbol,
          T::Symbol: NextSymbol<Symbol=T::Symbol>
{
    /// Creates an empty symbol allocator.
    pub fn new() -> Self {
        SymbolAlloc { reserve_next: T::Symbol::default() }
    }
}

/// A symbol allocator for VM variables.
#[derive(Debug, Clone)]
pub struct VariableSymbolAlloc {
    reserve_next: VariableSymbol,
    locals: Vec<ValueIndex>,
}

impl NextSymbol for VariableSymbolAlloc {
    type Symbol = VariableSymbol;

    fn next_symbol(&self) -> Self::Symbol { self.reserve_next.next_symbol() }
}

impl ReserveSymbol for VariableSymbolAlloc {
    type Symbol = VariableSymbol;

    fn reserve_symbol(&mut self) -> Self::Symbol {
        let next = self.next_symbol();
        mem::replace(&mut self.reserve_next, next)
    }

    fn enter_scope(&mut self) {
        // save the state of the most recent local variable
        self.locals.push(self.reserve_next.local);
        self.reserve_next.local = 0;
    }

    fn exit_scope(&mut self) {
        // restore state of the most recent local variable
        let local = self.locals
            .pop()
            .expect("attempted to exit a non-existent scope");
        self.reserve_next.local = local;
    }
}

/// A symbol allocator for VM types.
pub type TySymbolAlloc = SymbolAlloc<vm::Ty>;

