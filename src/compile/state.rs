use compile::{
    Error,
    ErrorKind,
    FunScope,
    VarScope,
    TyScope,
    Transform,
    TryTransform,
    TransformMut,
};
use ir;
use syntax::Ranged;
use vm::{self, Bc, Symbol, Symbolic};

pub struct State<'scope> {
    /// Current variable scope.
    pub (in super) var_scope: &'scope mut VarScope,

    /// Current function scope.
    pub (in super) fun_scope: &'scope mut FunScope,

    /// Current type scope.
    pub (in super) ty_scope: &'scope mut TyScope,
}

impl<'n, 'r: 'n, 'scope> TransformMut<&'r ir::Action<'n>> for State<'scope> {
    type Out = Vec<vm::Bc>;

    fn transform_mut(&mut self, action: &'r ir::Action<'n>) -> Self::Out {
        use ir::Action;
        match action {
            // Evaluate an IR value
            Action::Eval(_val) => { unimplemented!() },
            // Assign a value to a place in memory
            Action::Assign(_lhs, _op, _rhs) => { unimplemented!() },
            // Loop over a block
            Action::Loop(_block) => { unimplemented!() },
            // Add a block of actions
            Action::Block(_block) => { unimplemented!() },
            // Execute conditional blocks
            Action::ConditionBlock { if_block: _, elseif_blocks: _, else_block: _ } => { unimplemented!() },
            // Break out of the current block loop
            Action::Break => { unimplemented!() },
            // Continue to the top of this loop
            Action::Continue => { unimplemented!() },
            // Return from the current function
            Action::Return(_value) => { unimplemented!() },
        }
    }
}

struct ValueContext<'s, 'scope: 's> {
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
                let value = match s {
                    ir::Symbol::Fun(name) => {
                        let symbol = self.state.fun_scope.get_by_name(name)
                            .ok_or_else(|| Error::new(range, ErrorKind::UnknownFun(name.clone())))?
                            .symbol();
                        vm::Value::FunRef(symbol)
                    }
                    ir::Symbol::Variable(name) => {
                        let symbol = self.state.var_scope.get_by_name(name)
                            .expect("variable does not exist in this scope")
                            .symbol();
                        vm::Value::Reg(symbol)
                    }
                    ir::Symbol::Ty(name) => {
                        let symbol = self.state.ty_scope.get_by_name(name)
                            .ok_or_else(|| Error::new(range, ErrorKind::UnknownTy(name.clone())))?
                            .symbol();
                        vm::Value::TyRef(symbol)
                    }
                };
                Ok(vec![self.kind.transform(value)])
            }

            // Array access
            Value::ArrayAccess(_array, _index) => { unimplemented!("TODO(array) : array access") }

            // Binary expression
            Value::BinaryExpr(lhs, op, rhs) => {
                let op_fun = self.state.fun_scope.get_op(op)
                    .ok_or_else(|| {
                        Error::new(range, ErrorKind::UnknownOp(op.clone()))
                    })?
                    .symbol();
                let lhs_sym = self.state.var_scope.insert_anonymous_var();
                let lhs_code = {
                    let lhs_ctx = ValueContext::new(ValueContextKind::Store(lhs_sym), self.state);
                    lhs_ctx.try_transform(lhs)?
                };

                let rhs_sym = self.state.var_scope.insert_anonymous_var();
                let rhs_code = {
                    let rhs_ctx = ValueContext::new(ValueContextKind::Store(rhs_sym), self.state);
                    rhs_ctx.try_transform(rhs)?
                };
                
                let mut code: Vec<_> = lhs_code.into_iter()
                    .chain(rhs_code.into_iter())
                    .collect();
                code.push(Bc::PushValue(vm::Value::Reg(lhs_sym)));
                code.push(Bc::PushValue(vm::Value::Reg(rhs_sym)));
                code.push(Bc::Call(op_fun));
                Ok(code)
            }

            // Unary expression
            Value::UnaryExpr(_op, _value) => {
                unimplemented!("TODO : unary expression action")
            }

            // Fun call
            Value::FunCall(_fun, _args) => {
                unimplemented!("TODO : function call action");
            }
        }
    }
}

enum ValueContextKind {
    /// A value that is to be stored in a register.
    Store(vm::RegSymbol),

    /// A value that is to be pushed to the stack.
    Push,

    /// A value that is to be returned.
    Ret,
}

impl Transform<vm::Value> for ValueContextKind {
    type Out = Bc;
    fn transform(self, value: vm::Value) -> Self::Out {
        match self {
            ValueContextKind::Store(sym) => Bc::Store(sym, value),
            ValueContextKind::Push => Bc::PushValue(value),
            ValueContextKind::Ret => Bc::Ret(value),
        }
    }
}
