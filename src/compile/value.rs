use compile::{
    Error,
    State,
    Transform,
    TryTransform,
};
use ir;
use syntax::Ranged;
use vm::{self, Bc, Ref, Symbolic};

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub symbol: vm::RegSymbol,
}

impl Var {
    pub fn new(name: String, symbol: vm::RegSymbol) -> Self {
        Var { name, symbol }
    }
}

impl vm::Symbolic for Var {
    type Symbol = vm::RegSymbol;

    fn name(&self) -> &str {
        &self.name
    }

    fn symbol(&self) -> vm::RegSymbol {
        self.symbol
    }
}

pub (in super) struct ValueContext<'s, 'scope: 's> {
    /// The type of the value context that we're dealing with.
    kind: ValueContextKind,

    /// A reference to the current state of the compiler.
    state: &'s mut State<'scope>,
}

impl<'s, 'scope: 's> ValueContext<'s, 'scope> {
    pub fn new(kind: ValueContextKind, state: &'s mut State<'scope>) -> Self {
        ValueContext { kind, state }
    }
}

impl<'n, 'r: 'n, 's, 'scope: 's> TryTransform<'n, &'r ir::Value<'n>> for ValueContext<'s, 'scope> {
    type Out = Vec<Bc>;

    fn try_transform(self, value: &'r ir::Value<'n>) -> Result<Self::Out, Error<'n>> {
        use ir::Value;
        let range = value.range();
        match value {
            // Constant/literal value
            Value::Const(Ranged(_, c)) => {
                let value = vm::Value::Const(c.clone());
                Ok(vec![self.kind.transform(value)])
            }

            // User symbol (function, var, or ty)
            Value::Symbol(Ranged(_, s)) => {
                let ref_value = match s {
                    ir::Symbol::Fun(name) => {
                        let symbol = self.state.fun_scope.get_by_name(name)
                            .ok_or_else(|| Error::unknown_fun(range, name.clone()))?
                            .symbol();
                        Ref::Fun(symbol)
                    }
                    ir::Symbol::Variable(name) => {
                        let symbol = self.state.var_scope.get_by_name(name)
                            .expect("variable does not exist in this scope")
                            .symbol();
                        Ref::Reg(symbol)
                    }
                    ir::Symbol::Ty(name) => {
                        let symbol = self.state.ty_scope.get_by_name(name)
                            .ok_or_else(|| Error::unknown_ty(range, name.clone()))?
                            .symbol();
                        Ref::Ty(symbol)
                    }
                };
                // wrap it in a ref value
                let value = vm::Value::Ref(ref_value);
                Ok(vec![self.kind.transform(value)])
            }

            // Array access
            Value::ArrayAccess(_array, _index) => { unimplemented!("TODO(array) : array access") }

            // Binary expression
            Value::BinaryExpr(lhs, op, rhs) => {
                let op_fun = self.state.fun_scope.get_binary_op(op)
                    .ok_or_else(|| Error::unknown_binary_op(range, op.clone()))?
                    .symbol();
                let lhs_sym = self.state.var_scope.insert_anonymous_var();
                let lhs_code = {
                    let lhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(lhs_sym)), self.state);
                    lhs_ctx.try_transform(lhs)?
                };

                let rhs_sym = self.state.var_scope.insert_anonymous_var();
                let rhs_code = {
                    let rhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(rhs_sym)), self.state);
                    rhs_ctx.try_transform(rhs)?
                };
                
                let mut code: Vec<_> = lhs_code.into_iter()
                    .chain(rhs_code.into_iter())
                    .collect();
                code.push(Bc::Push(vm::Value::Ref(Ref::Reg(lhs_sym))));
                code.push(Bc::Push(vm::Value::Ref(Ref::Reg(rhs_sym))));
                code.push(Bc::Call(op_fun));
                // free the anonymous symbols
                self.state.var_scope.free_symbol(lhs_sym);
                self.state.var_scope.free_symbol(rhs_sym);
                Ok(code)
            }

            // Unary expression
            Value::UnaryExpr(op, value) => {
                let op_fun = self.state.fun_scope.get_unary_op(op)
                    .ok_or_else(|| Error::unknown_unary_op(range, op.clone()))?
                    .symbol();
                let value_sym = self.state.var_scope.insert_anonymous_var();
                let value_code = {
                    let value_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(value_sym)), self.state);
                    value_ctx.try_transform(value)?
                };
                self.state.var_scope.free_symbol(value_sym);
                Ok(value_code)
            }

            // Fun call
            Value::FunCall(_fun, _args) => {
                unimplemented!("TODO : function call action");
            }
        }
    }
}

pub (in super) enum ValueContextKind {
    /// A value that is to be stored into the given reference.
    Store(Ref),

    /// A value that is to be pushed to the stack.
    Push,

    /// A value that is to be returned.
    Ret,
}

impl Transform<vm::Value> for ValueContextKind {
    type Out = Bc;
    fn transform(self, value: vm::Value) -> Self::Out {
        match self {
            ValueContextKind::Store(r) => Bc::Store(r, value),
            ValueContextKind::Push => Bc::Push(value),
            ValueContextKind::Ret => Bc::Ret(value),
        }
    }
}
