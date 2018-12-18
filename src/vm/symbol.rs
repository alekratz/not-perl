use std::fmt::{self, Display, Formatter};

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
        pub struct $name(pub SymbolIndex);

        impl Default for $name {
            fn default() -> Self {
                $name(0)
            }
        }

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
    };
}

symbol_def!(FunSymbol);
symbol_def!(TySymbol);
symbol_def!(BlockSymbol);

#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct VarSymbol {
    pub global: SymbolIndex,
    pub local: SymbolIndex,
}

impl Default for VarSymbol {
    fn default() -> Self {
        VarSymbol {
            global: 0,
            local: 0,
        }
    }
}

impl Symbol for VarSymbol {
    fn index(&self) -> SymbolIndex {
        self.local
    }

    fn next(&self) -> Self {
        VarSymbol {
            global: self.global,
            local: self.local + 1,
        }
    }
}

impl Display for VarSymbol {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "VarSymbol-{}-{}", self.global, self.local)
    }
}
