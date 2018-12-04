use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use crate::{
    common::scope::*,
    compile::{
        Alloc,
        BlockSymbolAlloc,
    },
    vm::{Label, Symbolic},
};

/// A scope that is paired with a symbol allocator.
///
/// This is useful for compiler functions that create new symbols.
#[derive(Debug)]
pub struct AllocScope<T, A>
    where T: Symbolic,
          T::Symbol: Debug,
          A: Alloc<T::Symbol>,
{
    scope: ReadOnlyScope<T>,
    alloc: A,
}

impl<T, A> AllocScope<T, A>
    where T: Symbolic + Debug,
          T::Symbol: Debug,
          A: Alloc<T::Symbol> + Debug,
{

    /// Reserves a symbol in this scope.
    pub fn reserve_symbol(&mut self) -> T::Symbol {
        self.alloc.reserve()
    }

    /// Pushes a stack layer to the scope.
    fn push_scope(&mut self, layer: Vec<T>) {
        self.alloc.on_push_scope();
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
        self.alloc.on_pop_scope();
        self.scope_stack.pop()
            .expect("attempted to pop depthless scope")
    }
}

impl<T, A> Default for AllocScope<T, A>
    where T: Symbolic,
          T::Symbol: Debug,
          A: Alloc<T::Symbol> + Default
{
    fn default() -> Self {
        AllocScope {
            scope: ReadOnlyScope::default(),
            alloc: A::default(),
        }
    }
}

impl<T, A> Deref for AllocScope<T, A>
    where T: Symbolic,
          T::Symbol: Debug,
          A: Alloc<T::Symbol>,
{
    type Target = ReadOnlyScope<T>;

    fn deref(&self) -> &Self::Target { &self.scope }
}

impl<T, A> DerefMut for AllocScope<T, A>
    where T: Symbolic,
          T::Symbol: Debug,
          A: Alloc<T::Symbol>,
{
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.scope }
}

pub type LabelScope = AllocScope<Label, BlockSymbolAlloc>;
