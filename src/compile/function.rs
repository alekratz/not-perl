use ir::TyExpr;
use vm;

/// An intermediate function stub collected during compile-time.
///
/// This is used so that function declaration is order-agnostic.
#[derive(Debug, Clone)]
pub struct FunctionStub {
    /// Name of this function.
    pub symbol: vm::FunctionSymbol,

    pub name: String,

    /// Number of parameters for this function. Types are not yet enforced at this point.
    pub params: usize,

    pub return_ty: TyExpr,
}

impl vm::Symbolic for FunctionStub {
    type Symbol = vm::FunctionSymbol;

    fn symbol(&self) -> Self::Symbol {
        self.symbol
    }

    fn name(&self) -> &str {
        &self.name
    }
}
