use vm::{self, Symbolic};

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

#[derive(Debug)]
pub struct TyStub {
    pub name: String,
    pub symbol: vm::TySymbol,
}
