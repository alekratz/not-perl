use crate::{
    common::prelude::*,
    compile::{AllocScope, TySymbolAlloc},
    vm::{self, Symbolic},
};

/// A compile-time type.
///
/// This may either be a fully compiled VM type, or a discovered compile-time stub.
#[derive(Debug)]
pub enum Ty {
    Stub(TyStub),
    Vm(vm::Ty),
}

impl Symbolic for Ty {
    type Symbol = vm::TySymbol;

    fn name(&self) -> &str {
        match self {
            Ty::Stub(s) => &s.name,
            Ty::Vm(v) => v.name(),
        }
    }

    fn symbol(&self) -> vm::TySymbol {
        match self {
            Ty::Stub(s) => s.symbol,
            Ty::Vm(v) => v.symbol(),
        }
    }
}

impl Ranged for Ty {
    fn range(&self) -> Range {
        match self {
            Ty::Stub(s) => s.range(),
            Ty::Vm(v) => v.range(),
        }
    }
}

#[derive(Debug)]
pub struct TyStub {
    pub name: String,
    pub symbol: vm::TySymbol,
    pub range: Range,
}

impl Ranged for TyStub {
    fn range(&self) -> Range {
        self.range.clone()
    }
}

pub type TyScope = AllocScope<Ty, TySymbolAlloc>;
