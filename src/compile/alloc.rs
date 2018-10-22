use std::{
    collections::BTreeSet,
    mem,
};
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
pub type TySymbolAlloc = SymbolAlloc<vm::TySymbol>;

#[derive(Debug)]
pub struct RegSymbolAlloc {
    next: vm::RegSymbol,
    unused: BTreeSet<vm::RegSymbol>,
}

impl RegSymbolAlloc {
    pub fn free(&mut self, sym: vm::RegSymbol) {
        assert!(!self.unused.contains(&sym), "double free of reg symbol");
        self.unused.insert(sym);
    }
}

impl Alloc<vm::RegSymbol> for RegSymbolAlloc {
    fn reserve(&mut self) -> vm::RegSymbol {
        if self.unused.is_empty() {
            let next = self.next.next();
            mem::replace(&mut self.next, next)
        } else {
            let min = *self.unused.iter().min()
                .unwrap();
            assert!(self.unused.remove(&min));
            min
        }
    }
}

impl Default for RegSymbolAlloc {
    fn default() -> Self {
        RegSymbolAlloc {
            next: vm::RegSymbol::default(),
            unused: BTreeSet::new(),
        }
    }
}
