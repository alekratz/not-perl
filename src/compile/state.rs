use compile::{
    Error,
    FunScope,
    VarScope,
    LabelScope,
    TyScope,
    ValueContext,
    ValueContextKind,
    transform::*,
};
use ir;
use syntax::Ranged;
use vm::{
    self,
    Bc,
    Ref,
    Value,
};

pub struct State<'scope> {
    /// Current variable scope.
    pub (in super) var_scope: &'scope mut VarScope,

    /// Current function scope.
    pub (in super) fun_scope: &'scope mut FunScope,

    /// Current type scope.
    pub (in super) ty_scope: &'scope mut TyScope,
}

impl<'scope, 'n, 'r: 'n> State<'scope> {
    /// Attempts to transform a block of IR actions into bytecode.
    fn try_transform_block(&mut self, block: &'r [ir::Action<'n>]) -> Result<Vec<Bc>, Error<'n>> {
        block.iter().try_fold(vec![], |mut code, action| {
            code.append(&mut self.try_transform_mut(&action)?);
            Ok(code)
        })
    }
}

impl<'n, 'r: 'n, 'scope> TryTransformMut<'n, &'r ir::Action<'n>> for State<'scope> {
    type Out = Vec<Bc>;

    fn try_transform_mut(&mut self, action: &'r ir::Action<'n>) -> Result<Vec<Bc>, Error<'n>> {
        use ir::Action;
        match action {
            // Evaluate an IR value
            Action::Eval(val) => {
                let ctx = ValueContext::new(ValueContextKind::Push, self);
                ctx.try_transform(val)
            },
            // Assign a value to a place in memory
            Action::Assign(lhs, _op, rhs) => {
                // TODO : remove assignment ops, desugar assignment ops
                if !lhs.is_assign_candidate() {
                    let range = lhs.range();
                    return Err(Error::invalid_assign_lhs(range, range.text().to_string()));
                }

                let code = match lhs {
                    // unreachable since is_assign_candidate excludes constants
                    ir::Value::Const(_) => unreachable!(),
                    ir::Value::Symbol(Ranged(_, ir::Symbol::Variable(varname))) => {
                        let lhs_store = Ref::Reg(self.var_scope.get_or_insert(varname));
                        ValueContext::new(ValueContextKind::Store(lhs_store), self)
                            .try_transform(rhs)?
                    },
                    // unreachable since is_assign_candidate excludes non-variable symbol
                    ir::Value::Symbol(Ranged(_, _)) => unreachable!(),
                    ir::Value::ArrayAccess(_, _) => unimplemented!("TODO(array) : array assign"),
                    | ir::Value::FunCall(_, _)
                    | ir::Value::UnaryExpr(_, _)
                    | ir::Value::BinaryExpr(_, _, _) => {
                        // Unary/binary expressions and funcalls are all just function calls.
                        //
                        // Function calls may return a reference.
                        //
                        // As a result, anything that ends up being a function call on the LHS of
                        // an assignment should be evaluated.
                        let lhs_store = self.var_scope.insert_anonymous_var();
                        let lhs_code = {
                            let lhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(lhs_store)), self);
                            lhs_ctx.try_transform(lhs)?
                        };
                        let rhs_store = self.var_scope.insert_anonymous_var();
                        let rhs_code = {
                            let rhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(rhs_store)), self);
                            rhs_ctx.try_transform(rhs)?
                        };

                        self.var_scope.free_anonymous_var(lhs_store);
                        self.var_scope.free_anonymous_var(rhs_store);
                        lhs_code.into_iter()
                            .chain(rhs_code.into_iter())
                            // TODO : deref RHS?
                            .chain(vec![Bc::DerefPush(Ref::Reg(lhs_store)), Bc::PopDerefStore(Value::Ref(Ref::Reg(rhs_store)))].into_iter())
                            .collect()
                    }
                };
                Ok(code)
            },
            // Loop over a block
            Action::Loop(block) => {
                // translate block
                let mut code = self.try_transform_block(block)?;
                // relative jump
                let jumpback = -(code.len() as isize);
                code.push(Bc::JmpRel(jumpback));
                Ok(code)
            },
            // Add a block of actions
            Action::Block(block) => self.try_transform_block(block),
            // Execute conditional blocks
            Action::ConditionBlock { if_block: _, elseif_blocks: _, else_block: _ } => { unimplemented!() },
            // Break out of the current block loop
            Action::Break => { unimplemented!() },
            // Continue to the top of this loop
            Action::Continue => { unimplemented!() },
            // Return from the current function
            Action::Return(val) => {
                val.as_ref().map(|val| {
                    let ctx = ValueContext::new(ValueContextKind::Ret, self);
                    ctx.try_transform(val)
                }).unwrap_or_else(|| {
                    let ctx = ValueContextKind::Ret;
                    Ok(vec![ctx.transform(Value::None)])
                })
            },
        }
    }
}
