use common::lang::Op;
use compile::{
    State,
    TransformMut,
};
use ir;
use vm::{self, Symbolic};

#[derive(Debug, Clone)]
pub enum Fun {
    /// A known function stub.
    Stub(FunStub),

    /// A compiled or built-in VM function.
    Vm(vm::Fun),

    /// A compiled or built-in VM function for an operator.
    Op(Op, vm::Fun),
}

impl Fun {
    pub fn params(&self) -> usize {
        match self {
            Fun::Stub(s) => s.params,
            | Fun::Vm(b)
            | Fun::Op(_, b) => b.params(),
                
        }
    }
}

impl Symbolic for Fun {
    type Symbol = vm::FunSymbol;
    fn symbol(&self) -> vm::FunSymbol {
        match self {
            Fun::Stub(s) => s.symbol,
            Fun::Vm(b) | Fun::Op(_, b) => b.symbol(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Fun::Stub(s) => &s.name,
            Fun::Vm(b) | Fun::Op(_, b) => b.name(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunStub {
    pub name: String,
    pub symbol: vm::FunSymbol,
    pub params: usize,
    pub return_ty: ir::TyExpr,
}

impl FunStub {
    pub fn from_ir_function<'n>(symbol: vm::FunSymbol, fun: &ir::Fun<'n>) -> Self {
        let name = fun.name().to_string();
        let params = fun.params.len();
        let return_ty = fun.return_ty.clone();
        FunStub {
            name,
            symbol,
            params,
            return_ty,
        }
    }
}

impl<'n, 'r: 'n, 'scope> TransformMut<&'r ir::Fun<'n>> for State<'scope> {
    type Out = FunStub;

    fn transform_mut(&mut self, value: &'r ir::Fun<'n>) -> Self::Out {
        let symbol = self.fun_scope.reserve_symbol();
        FunStub::from_ir_function(symbol, value)
    }
}
