use std::{
    ops::Deref,
    cmp::{PartialOrd, Ordering},
};
use vm::{
    ValueIndex,
    Function,
    Ty,
};

macro_rules! symbol {
    ($name:ident) => {
        #[derive(Clone, Copy, Eq, Ord, Debug)]
        pub struct $name (pub ValueIndex);

        impl Default for $name {
            fn default() -> Self { $name(0) }
        }

        symbol_impl!($name, 0);
    }
}

/// Base impls for symbols.
macro_rules! symbol_impl {
    ($name:ident, $($accessor:tt)+) => {
        impl Deref for $name {
            type Target = ValueIndex;
            fn deref(&self) -> &Self::Target { &self.$($accessor)+ }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.index().eq(&other.index())
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.index().partial_cmp(&other.index())
            }
        }

        impl PartialEq<ValueIndex> for $name {
            fn eq(&self, other: &ValueIndex) -> bool {
                self.index().eq(other)
            }
        }

        impl PartialOrd<ValueIndex> for $name {
            fn partial_cmp(&self, other: &ValueIndex) -> Option<Ordering> {
                self.index().partial_cmp(other)
            }
        }

        impl Symbol for $name {
            fn index(&self) -> ValueIndex {
                self.$($accessor)+
            }
        }
    }
}

#[derive(Clone, Copy, Eq, Ord, Debug)]
pub struct VariableSymbol {
    pub global: ValueIndex,
    pub local: ValueIndex,
}

impl Default for VariableSymbol {
    fn default() -> Self {
        VariableSymbol { global: 0, local: 0 }
    }
}

pub trait Symbol: Clone + Copy + PartialOrd + PartialEq + Ord + Default {
    fn index(&self) -> ValueIndex;
}

symbol!(FunctionSymbol);
symbol!(TySymbol);
symbol_impl!(VariableSymbol, global);
//symbol!(ConstantSymbol);

/// A type that has a name and a symbol.
pub trait Symbolic {
    type Symbol: Symbol;
    fn symbol(&self) -> Self::Symbol;
    fn name(&self) -> &str;
}

impl Symbolic for Ty {
    type Symbol = TySymbol;

    fn symbol(&self) -> Self::Symbol {
        match self {
            | Ty::Builtin(_, sym) => *sym,
            | Ty::User(u) => u.symbol,
        }
    }

    fn name(&self) -> &str {
        match self {
            Ty::Builtin(b, _) => b.name(),
            Ty::User(u) => &u.name,
        }
    }
}

impl Symbolic for Function {
    type Symbol = FunctionSymbol;

    fn symbol(&self) -> Self::Symbol {
        match self {
            | Function::Builtin(b) => b.symbol,
            | Function::User(u) => u.symbol,
        }
    }

    fn name(&self) -> &str {
        match self {
            | Function::Builtin(b) => &b.name,
            | Function::User(u) => &u.name,
        }
    }
}
