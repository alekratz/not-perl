use std::{
    ops::Deref,
    cmp::{PartialOrd, Ordering},
};
use vm::ValueIndex;

macro_rules! symbol {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name (pub ValueIndex);

        symbol_impl!($name, 0);
    }
}

/// Base impls for symbols.
macro_rules! symbol_impl {
    ($name:ident, $($accessor:tt)+) => {
        impl $name {
            pub fn index(&self) -> ValueIndex {
                self.$($accessor)+
            }
        }

        impl Deref for $name {
            type Target = ValueIndex;
            fn deref(&self) -> &Self::Target {
                &self.$($accessor)+
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.$($accessor)+.partial_cmp(&other.$($accessor)+)
            }
        }

        impl PartialEq<ValueIndex> for $name {
            fn eq(&self, other: &ValueIndex) -> bool {
                self.$($accessor)+.eq(other)
            }
        }

        impl PartialOrd<ValueIndex> for $name {
            fn partial_cmp(&self, other: &ValueIndex) -> Option<Ordering> {
                self.$($accessor)+.partial_cmp(other)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VariableSymbol {
    pub global: ValueIndex,
    pub local: ValueIndex,
}

symbol!(FunctionSymbol);
symbol!(TySymbol);
symbol_impl!(VariableSymbol, global);
//symbol!(ConstantSymbol);
