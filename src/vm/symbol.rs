use std::fmt::{self, Formatter, Display};

pub type SymbolIndex = usize;

pub trait Symbol: Copy + Clone + PartialEq + PartialOrd + Ord {
    fn index(&self) -> SymbolIndex;

    fn next(&self) -> Self;
}

pub trait Symbolic {
    type Symbol: Symbol;

    fn symbol(&self) -> Self::Symbol;

    fn name(&self) -> &str;
}

macro_rules! symbol_def {
    ($name:ident) => {
        #[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
        pub struct $name (pub SymbolIndex);

        impl Symbol for $name {
            fn index(&self) -> SymbolIndex {
                self.0
            }

            fn next(&self) -> Self {
                $name(self.0 + 1)
            }
        }

        impl Display for $name {
            fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, concat!(stringify!($name), "-{}"), self.index())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name ( 0 )
            }
        }
    }
}

symbol_def!(FunSymbol);
symbol_def!(RegSymbol);
symbol_def!(TySymbol);
