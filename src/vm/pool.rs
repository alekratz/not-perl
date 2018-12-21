use std::ops::Index;
use crate::vm::symbol::*;

/// A constant pool of program data.
///
/// Pools are generally assumed to be read-only.
pub trait Pool {
    type Symbolic: Symbolic;

    fn get(&self, symbol: <<Self as Pool>::Symbolic as Symbolic>::Symbol) -> &Self::Symbolic;
}

impl<S> Index<S::Symbol> for Pool<Symbolic=S>
where S: Symbolic
{
    type Output = S;

    fn index(&self, symbol: S::Symbol) -> &Self::Output {
        self.get(symbol)
    }
}
