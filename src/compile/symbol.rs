use crate::vm::{self, Symbol, SymbolIndex};
use std::{collections::VecDeque, mem};

/// A symbol allocator for a symbolic VM symbol type.
pub trait Alloc<T>
where
    T: vm::Symbol,
{
    /// Reserves the next symbol available.
    fn reserve(&mut self) -> T;

    /// An optional callback for when a scope is pushed.
    ///
    /// The default behavior is a no-op.
    fn on_push_scope(&mut self) {}

    /// An optional callback for when a scope is popped.
    ///
    /// The default behavior is a no-op.
    fn on_pop_scope(&mut self) {}
}

#[derive(Debug)]
pub struct SymbolAlloc<T: vm::Symbol> {
    next: T,
}

impl<T: vm::Symbol + Default> Default for SymbolAlloc<T> {
    fn default() -> Self {
        SymbolAlloc { next: T::default() }
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
pub type BlockSymbolAlloc = SymbolAlloc<vm::BlockSymbol>;

/// A register symbol layer allocator.
///
/// Since registers are function-local, this represents symbols being allocated for a single scope
/// layer. This also implements `Alloc` for convenience.
#[derive(Debug)]
struct VarSymbolLayer {
    next: vm::VarSymbol,
}

impl VarSymbolLayer {
    pub fn new(global: SymbolIndex) -> Self {
        VarSymbolLayer {
            next: vm::VarSymbol { global, local: 0 },
        }
    }
}

impl Alloc<vm::VarSymbol> for VarSymbolLayer {
    fn reserve(&mut self) -> vm::VarSymbol {
        let next = self.next.next();
        mem::replace(&mut self.next, next)
    }
}

/// A register symbol allocator.
///
/// This wraps the logic defined in VarSymbolLayer, except defining a stack of these layers.
#[derive(Debug)]
pub struct VarSymbolAlloc {
    scope_stack: VecDeque<VarSymbolLayer>,
}

impl VarSymbolAlloc {
    /// Gets the topmost reg symbol layer defined.
    fn active_mut(&mut self) -> &mut VarSymbolLayer {
        self.scope_stack
            .back_mut()
            // oddly specific error messages are the best error messages
            .expect("tried to get topmost register symbol allocator from depthless VarSymbolAlloc stack")
    }
}

impl Alloc<vm::VarSymbol> for VarSymbolAlloc {
    fn reserve(&mut self) -> vm::VarSymbol {
        self.active_mut().reserve()
    }

    fn on_push_scope(&mut self) {
        let global = self.scope_stack.len();
        self.scope_stack.push_back(VarSymbolLayer::new(global));
    }

    fn on_pop_scope(&mut self) {
        // this doesn't actually "pop" a value - it moves the top scope value to the front of the
        // list. Thus, new scopes get unique global values, but they are never accessed again (but
        // parent scopes are still available).
        let back = self
            .scope_stack
            .pop_back()
            .expect("tried to pop top value from depthless VarSymbolAlloc");
        self.scope_stack.push_front(back);
    }
}

impl Default for VarSymbolAlloc {
    fn default() -> Self {
        VarSymbolAlloc {
            scope_stack: VecDeque::new(),
        }
    }
}
