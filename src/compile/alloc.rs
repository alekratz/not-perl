use std::mem;
use vm::{self, Symbol};

/// A symbol allocator for a symbolic VM symbol type.
pub trait Alloc<T>
    where T: vm::Symbol
{
    /// Reserves the next symbol available.
    fn reserve(&mut self) -> T;
}

#[derive(Debug)]
pub struct SymbolAlloc<T: vm::Symbol> {
    next: T,
}

impl<T: vm::Symbol + Default> Default for SymbolAlloc<T> {
    fn default() -> Self {
        SymbolAlloc {
            next: T::default(),
        }
    }
}

impl<T: vm::Symbol> Alloc<T> for SymbolAlloc<T> {
    fn reserve(&mut self) -> T {
        let next = self.next.next();
        mem::replace(&mut self.next, next)
    }
}

pub type FunSymbolAlloc = SymbolAlloc<vm::FunSymbol>;
pub type RegSymbolAlloc = SymbolAlloc<vm::RegSymbol>;
pub type TySymbolAlloc = SymbolAlloc<vm::TySymbol>;
