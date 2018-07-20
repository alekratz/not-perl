use vm;

/// An intermediate function stub collected during compile-time.
///
/// This is used so that function declaration is order-agnostic.
#[derive(Debug, Clone)]
pub struct FunctionStub {
    /// Name of this function.
    pub symbol: vm::Symbol,

    /// Number of parameters for this function. Types are not yet enforced at this point.
    pub param_count: usize,
}
