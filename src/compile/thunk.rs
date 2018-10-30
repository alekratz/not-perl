use crate::compile::{
    State,
    Error,
    ValueContext,
    ValueContextKind,
    transform::*,
};
use crate::ir;
use crate::common::pos::RangeWrapper;
use crate::vm::{
    self,
    Bc,
    Ref,
    Value,
    JumpCond,
};

pub enum Thunk {
    Empty,
    Code(Vec<Bc>),
    Nested(Vec<Thunk>),
    Labeled { entry: vm::BlockSymbol, code: Box<Thunk>, exit: vm::BlockSymbol, }
}

impl Thunk {
    /// Pushes a bytecode instruction to the end of this thunk.
    pub fn push(&mut self, bc: Bc) {
        match self {
            Thunk::Empty => *self = Thunk::Code(vec![bc]),
            Thunk::Code(c) => c.push(bc),
            Thunk::Nested(thunks) => if thunks.is_empty() {
                thunks.push(Thunk::Code(vec![bc]));
            } else {
                thunks.last_mut()
                    .unwrap()
                    .push(bc);
            },
            Thunk::Labeled { entry: _, code, exit: _, } => code.push(bc),

        }
    }
}

pub struct JumpBlock<'s> {
    entry: vm::BlockSymbol,
    exit: vm::BlockSymbol,
    state: &'s mut State,
}

impl<'r, 's> JumpBlock<'s> {
    pub fn new(entry: vm::BlockSymbol, exit: vm::BlockSymbol, state: &'s mut State) -> Self {
        JumpBlock { entry, exit, state, }
    }

    /// Attempts to transform a block of IR actions into bytecode.
    fn try_transform_block(&mut self, block: Vec<ir::Action>) -> Result<Thunk, Error> {
        let thunks = block.into_iter().try_fold(vec![], |mut thunks, action| {
            thunks.push(self.try_transform_mut(action)?);
            Ok(thunks)
        })?;
        Ok(Thunk::Nested(thunks))
    }
}

impl<'r, 's> TryTransformMut<ir::Action> for JumpBlock<'s> {
    type Out = Thunk;

    fn try_transform_mut(&mut self, action: ir::Action) -> Result<Thunk, Error> {
        use crate::ir::ActionKind;
        let RangeWrapper(range, action) = action;
        match action {
            // Evaluate an IR value
            ActionKind::Eval(val) => {
                let ctx = ValueContext::new(ValueContextKind::Push, self.state);
                ctx.try_transform(val).map(Thunk::Code)
            },
            // Assign a value to a place in memory
            ActionKind::Assign(lhs, _op, rhs) => {
                // TODO : remove assignment ops, desugar assignment ops
                if !lhs.is_assign_candidate() {
                    let range = lhs.range();
                    return Err(Error::invalid_assign_lhs(range.clone(), range.source_text().to_string()));
                }

                let RangeWrapper(_, ref lhs_value) = lhs;
                let code = match lhs_value {
                    // unreachable since is_assign_candidate excludes constants
                    ir::ValueKind::Const(_) => unreachable!(),
                    ir::ValueKind::Symbol(RangeWrapper(_, ir::Symbol::Variable(varname))) => {
                        let lhs_store = Ref::Reg(self.state.var_scope.get_or_insert(&varname));
                        ValueContext::new(ValueContextKind::Store(lhs_store), self.state)
                            .try_transform(rhs)?
                    },
                    // unreachable since is_assign_candidate excludes non-variable symbol
                    ir::ValueKind::Symbol(RangeWrapper(_, _)) => unreachable!(),
                    ir::ValueKind::ArrayAccess(_, _) => unimplemented!("TODO(array) : array assign"),
                    | ir::ValueKind::FunCall(_, _)
                    | ir::ValueKind::UnaryExpr(_, _)
                    | ir::ValueKind::BinaryExpr(_, _, _) => {
                        // Unary/binary expressions and funcalls are all just function calls.
                        //
                        // Function calls may return a reference.
                        //
                        // As a result, anything that ends up being a function call on the LHS of
                        // an assignment should be evaluated.
                        let lhs_store = self.state.var_scope.insert_anonymous_var();
                        let lhs_code = {
                            let lhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(lhs_store)), self.state);
                            lhs_ctx.try_transform(lhs)?
                        };
                        let rhs_store = self.state.var_scope.insert_anonymous_var();
                        let rhs_code = {
                            let rhs_ctx = ValueContext::new(ValueContextKind::Store(Ref::Reg(rhs_store)), self.state);
                            rhs_ctx.try_transform(rhs)?
                        };

                        self.state.var_scope.free_anonymous_var(lhs_store);
                        self.state.var_scope.free_anonymous_var(rhs_store);
                        lhs_code.into_iter()
                            .chain(rhs_code.into_iter())
                            // TODO : deref RHS?
                            .chain(vec![Bc::DerefPush(Ref::Reg(lhs_store)), Bc::PopDerefStore(Value::Ref(Ref::Reg(rhs_store)))].into_iter())
                            .collect()
                    }
                };
                Ok(Thunk::Code(code))
            },
            // Loop over a block
            ActionKind::Loop(block) => {
                let entry = self.state.label_scope.reserve_symbol();
                let exit = self.state.label_scope.reserve_symbol();
                // translate block
                let mut jump_block = JumpBlock::new(entry, exit, self.state);
                let mut code = jump_block.try_transform_block(block)?;
                code.push(Bc::JumpSymbol(entry, JumpCond::Always));
                Ok(Thunk::Labeled { entry, code: Box::new(code), exit })
            },
            // Add a block of actions
            ActionKind::Block(block) => self.try_transform_block(block),
            // Execute conditional blocks
            ActionKind::ConditionBlock { if_block, elseif_blocks, else_block, } => {
                // entry and exit symbols for the entire statement
                let cond_entry = self.state.label_scope.reserve_symbol();
                let cond_exit = self.state.label_scope.reserve_symbol();

                // If entry/exit definition
                let if_entry = cond_entry;
                let if_exit = self.state.label_scope.reserve_symbol();

                // Else entry/exit definition
                let else_entry;
                let else_exit = cond_exit;

                // if block
                let if_thunk = {
                    let mut cond_code = ValueContext::new(ValueContextKind::Push, self.state)
                        .try_transform(if_block.condition)?;
                    cond_code.push(Bc::PopTest);
                    cond_code.push(Bc::JumpSymbol(if_exit, JumpCond::CondFalse));
                    let mut block_code = self.try_transform_mut(if_block.action)?;
                    block_code.push(Bc::JumpSymbol(cond_exit, JumpCond::Always));
                    Thunk::Labeled { entry: if_entry, code: Box::new(block_code), exit: if_exit }
                };

                let elif_thunk = if elseif_blocks.is_empty() {
                    else_entry = if_exit;
                    Thunk::Empty
                } else {
                    let mut elif_entry = if_exit;
                    let mut elif_exit = self.state.label_scope.reserve_symbol();
                    let mut thunks = Vec::new();
                    let last_index = elseif_blocks.len() - 1;
                    for (idx, elif) in elseif_blocks.into_iter().enumerate() {
                        let mut cond_code = ValueContext::new(ValueContextKind::Push, self.state)
                            .try_transform(elif.condition)?;
                        cond_code.push(Bc::PopTest);
                        cond_code.push(Bc::JumpSymbol(elif_exit, JumpCond::CondFalse));

                        let mut block_code = self.try_transform_mut(elif.action)?;
                        block_code.push(Bc::JumpSymbol(cond_exit, JumpCond::Always));
                        thunks.push(Thunk::Labeled { entry: elif_entry, code: Box::new(block_code), exit: elif_exit });

                        // update entry and exit symbols if we're not on the last element
                        if idx != last_index {
                            elif_entry = elif_exit;
                            elif_exit = self.state.label_scope.reserve_symbol();
                        }
                    }
                    else_entry = elif_exit;
                    // use if_exit for the elif_entry since elif_entry has changed in the for loop
                    Thunk::Labeled { entry: if_exit, code: Box::new(Thunk::Nested(thunks)), exit: elif_exit }
                };

                let else_thunk = if let Some(else_block) = else_block {
                    let block_code = self.try_transform_mut(*else_block)?;
                    Thunk::Labeled { entry: else_entry, code: Box::new(block_code), exit: else_exit }
                } else {
                    Thunk::Empty
                };

                let condition_thunk = Thunk::Nested(vec![if_thunk, elif_thunk, else_thunk]);
                // Gather our children together
                Ok(Thunk::Labeled { entry: cond_entry, code: Box::new(condition_thunk), exit: cond_exit })
            },
            // Break out of the current block loop
            ActionKind::Break => Ok(Thunk::Code(vec![Bc::JumpSymbol(self.exit, JumpCond::Always)])),
            // Continue to the top of this loop
            ActionKind::Continue => Ok(Thunk::Code(vec![Bc::JumpSymbol(self.entry, JumpCond::Always)])),
            // Return from the current function
            ActionKind::Return(val) => {
                val.map(|val| {
                    let ctx = ValueContext::new(ValueContextKind::Ret, self.state);
                    ctx.try_transform(val)
                        .map(Thunk::Code)
                }).unwrap_or_else(|| {
                    let ctx = ValueContextKind::Ret;
                    Ok(Thunk::Code(vec![ctx.transform(Value::None)]))
                })
            },
        }
    }
}
