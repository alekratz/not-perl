use syntax::token::{Op, AssignOp};
use ir::*;
use vm::{
    self, Bc, Condition, CompareOp,
};

mod function;
mod scope;
pub use self::function::*;
pub use self::scope::*;

/// A compilation error.
pub type Error = String;

/// A compilation result.
pub type Result<T> = ::std::result::Result<T, Error>;


/// IR to bytecode compiler, complete with state.
#[derive(Debug)]
pub struct Compile {
    compiled_functions: Vec<vm::Function>,
    function_scope: FunctionScope,

    all_variables: Vec<vm::Symbol>,
    local_variables: Vec<Vec<vm::Symbol>>,
    variable_names: Vec<String>,

    local_variable_count: usize,
}

impl Compile {
    pub fn new() -> Self {
        let mut function_scope = FunctionScope::new();
        function_scope.add_scope();
        let compiled_functions: Vec<_> = vm::BUILTIN_FUNCTIONS.clone()
            .into_iter()
            .map(|mut function| {
                function.symbol = function_scope.next_function_symbol(function.name.clone());
                vm::Function::Builtin(function)
            })
            .collect();
        Compile {
            compiled_functions,
            function_scope,

            all_variables: vec![],
            local_variables: vec![],
            variable_names: vec![],

            local_variable_count: 0,
        }
    }

    pub fn compile_ir_tree<'n>(mut self, ir_tree: &IrTree<'n>) -> Result<CompileUnit> {
        self.function_scope.add_scope();
        self.local_variables.push(vec![]);

        // gather all function stubs
        for function in ir_tree.functions() {
            if self.function_scope.lookup_local_function_stub_name(function.name()).is_some() {
                return Err(self.err(format!("function `{}` defined twice in the same scope", function.name())));
            }
            let stub = FunctionStub {
                symbol: self.next_function_symbol(function.name().to_string()),
                param_count: function.params.len(),
            };
            self.function_scope.insert_function_stub(stub);
        }

        // compile user types and their functions
        // TODO

        // compile functions
        for function in ir_tree.functions() {
            let function = self.compile_function(function)?;
            self.insert_vm_function(vm::Function::User(function));
        }

        let code = self.compile_action_list(ir_tree.actions())?;
        let globals = self.local_variables.pop()
            .unwrap();
        let main_function_symbol = self.next_function_symbol("<main>".to_string());
        let functions = self.compiled_functions;
        // TODO : make sure all stubs have their counterpart?
        let main_function = vm::Function::User(vm::UserFunction {
            symbol: main_function_symbol,
            params: vec![],
            return_ty: vm::Ty::None,
            locals: globals,
            body: code,
        });

        Ok(CompileUnit {
            name: String::new(),
            main_function,
            functions,
        })
    }

    /// Converts a sequence of IR actions to a sequence of bytecode.
    fn compile_action_list<'n>(&mut self, actions: &[Action<'n>]) -> Result<Vec<Bc>> {
        let mut body = vec![];
        for action in actions {
            body.append(&mut self.compile_action(action)?);
        }
        Ok(body)
    }

    /// Compiles an IR action into a sequence of bytecode.
    fn compile_action<'n>(&mut self, action: &Action<'n>) -> Result<Vec<Bc>> {
        let thunk = match action {
            Action::Eval(value) => self.compile_value(value, ValueContext::Push)?,
            Action::Assign(lhs, op, rhs) => self.compile_action_assign(lhs, *op, rhs)?,
            Action::Loop(block) => {
                let mut loop_body = self.compile_action_list(block)?;
                loop_body.push(Bc::Compare(Condition::Always));
                loop_body.push(Bc::JumpBlockTop(0));
                loop_body
            },
            Action::Block(block) => self.compile_action_list(block)?,
            Action::ConditionBlock { if_block, elseif_blocks, else_block } => {
                let mut bc = vec![];

                // if block
                {
                    bc.append(&mut self.compile_comparison(&if_block.condition)?);
                    bc.append(&mut self.compile_action(&if_block.action)?);
                }

                // elseif blocks
                for block in elseif_blocks {
                    bc.append(&mut self.compile_comparison(&block.condition)?);
                    bc.append(&mut self.compile_action(&block.action)?);
                }

                // else block
                if let Some(block) = else_block {
                    bc.append(&mut self.compile_action(block)?);
                }

                vec![Bc::Block(bc)]
            }
            Action::Return(None) => vec![Bc::Ret(None)],
            Action::Return(Some(ref s)) => self.compile_value(s, ValueContext::Ret)?,
            Action::Break => vec![Bc::Compare(Condition::Always), Bc::ExitBlock(0)],
            Action::Continue => vec![Bc::Compare(Condition::Always), Bc::JumpBlockTop(0)],
        };
        Ok(thunk)
    }

    /// Compiles an IR function into a VM function.
    pub fn compile_function<'n>(&mut self, function: &Function<'n>) -> Result<vm::UserFunction> {
        self.local_variables.push(vec![]);
        self.function_scope.add_scope();

        let symbol = match &function.symbol {
            Symbol::Function(name) => {
                self.next_function_symbol(name.to_string())
            },
            sym => panic!("got non-function symbol name from IR::Function: {:?}", sym),
        };

        let mut params: Vec<vm::FunctionParam> = vec![];
        for param in &function.params {
            {
                let unique = params.iter()
                    .find(|p| self.variable_names[p.symbol.index()] == param.name());
                if let Some(dupe) = unique {
                    let pname = &self.variable_names[dupe.symbol.index()];
                    return Err(self.err(format!("duplicate parameter `{}` in function `{}`", pname, function.name())));
                }
            }
            params.push(self.compile_function_param(param)?);
        }

        let return_ty = function.return_ty.clone().into();
        let body = self.compile_action_list(&function.body)?;
        let locals = self.local_variables.pop().unwrap();
        self.function_scope.shed_scope();
        Ok(vm::UserFunction { symbol, params, return_ty, locals, body })
    }

    fn compile_function_param<'n>(&mut self, FunctionParam { symbol, ty, default: _ }: &FunctionParam<'n>)
        -> Result<vm::FunctionParam>
    {
        let symbol = self.insert_local_variable(symbol.name().to_string())
            .clone();
        let ty = ty.clone().into();
        Ok(vm::FunctionParam { symbol, ty })
    }

    /// Compiles an assignment (lhs, operator, and rhs) into a thunk.
    fn compile_action_assign(&mut self, lhs: &Value, op: AssignOp, rhs: &Value) -> Result<Vec<Bc>> {
        let lhs_context = match &lhs {
            // if there's only a Symbol::Variable on the LHS, then we can do a direct store into
            // this value
            Value::Symbol(range_sym) => {
                // get the known LHS symbol
                let vm_symbol = match range_sym.as_inner() {
                    Symbol::Function(f) =>
                        return Err(self.err(format!("found function `{}` on the lhs of an assignment, which is not valid", f))),
                    Symbol::Bareword(b) =>
                        return Err(self.err(format!("found bareword `{}` on the lhs of an assignment, which is not valid", b))),
                    Symbol::Variable(s) => self.lookup_or_insert_local_variable(s).clone(),
                };
                ValueContext::StoreInto(vm_symbol)
            }
            // other, more "complex" values on the LHS mean that we need to do a push and then pop
            // off a symbol ref
            _ => ValueContext::Push,
        };
        let mut assign_body = vec![];
        

        let vm_op = match op {
            AssignOp::PlusEquals => Some(Op::Plus),
            AssignOp::MinusEquals => Some(Op::Minus),
            AssignOp::SplatEquals => Some(Op::Splat),
            AssignOp::FSlashEquals => Some(Op::FSlash),
            AssignOp::TildeEquals => Some(Op::Tilde),
            AssignOp::Equals => None,
        };

        if let Some(op) = vm_op {
            let lhs = Box::new(lhs.clone());
            let rhs = Box::new(rhs.clone());
            assign_body.append(&mut self.compile_value(&Value::BinaryExpr(lhs, op, rhs), lhs_context)?)
        } else {
            if lhs_context == ValueContext::Push {
                // evaluate LHS, evaluate RHS, pop RHS into LHS ref
                assign_body.push(Bc::PushValue(vm::Value::RefCanary));
                assign_body.append(&mut self.compile_value(lhs, ValueContext::Push)?);
                assign_body.append(&mut self.compile_value(rhs, ValueContext::Push)?);
                assign_body.push(Bc::PopRefAndStore);
            } else {
                // boring 'ol store
                assign_body.append(&mut self.compile_value(rhs, lhs_context)?);
            }
        }

        Ok(assign_body)
    }

    /// Converts a value known to be an immediate into a VM value.
    fn convert_immediate_value(&mut self, value: &Value) -> vm::Value {
        match value {
            Value::Const(value) => {
                let value = value.as_inner()
                    .clone();
                vm::Value::from(value)
            },
            Value::Symbol(value) => {
                let sym = self.lookup_or_insert_local_variable(value.as_inner().name())
                    .clone();
                vm::Value::Ref(sym)
            },
            _ => panic!("{:?} is not an immediate value", value),
        }
    }

    /// Compiles the given value (with usage context) into a thunk.
    fn compile_value(&mut self, value: &Value, context: ValueContext) -> Result<Vec<Bc>> {
        match value {
            Value::Const(value) => Ok(context.with_value_to_bytecode(value.as_inner().clone().into())),
            Value::Symbol(sym) => {
                match sym.as_inner() {
                    Symbol::Function(s) => {
                        let function = self.function_scope.lookup_function_stub_name(s)
                            .ok_or(format!("unknown function `{}`", s))?
                            .clone();
                        Ok(context.with_value_to_bytecode(vm::Value::FunctionRef(function.symbol)))
                    }
                    Symbol::Bareword(_) => unimplemented!("compiling IR to bytecode => constant value lookup"),
                    Symbol::Variable(s) => {
                        let symbol = self.lookup_or_insert_local_variable(s).clone();
                        Ok(context.with_symbol_to_bytecode(symbol))
                    }
                }
            }
            Value::ArrayAccess(_, _) => unimplemented!("compiling IR to bytecode => array access"),
            Value::BinaryExpr(lhs, op, rhs) => {
                let mut expr_body = vec![];
                // TODO : compound this into a lambda or something

                // LHS
                let lhs_value = if lhs.is_immediate() {
                    self.convert_immediate_value(lhs)
                } else {
                    let lhs_sym = self.insert_anonymous_symbol();
                    expr_body.append(&mut self.compile_value(lhs, ValueContext::StoreInto(lhs_sym.clone()))?);
                    vm::Value::Ref(lhs_sym)
                };
                // RHS
                let rhs_value = if rhs.is_immediate() {
                    self.convert_immediate_value(rhs)
                } else {
                    let rhs_sym = self.insert_anonymous_symbol();
                    expr_body.append(&mut self.compile_value(rhs, ValueContext::StoreInto(rhs_sym.clone()))?);
                    vm::Value::Ref(rhs_sym)
                };
                // make this mutable so we can update it in the event that it's a Ret context
                let mut context = context;
                if context == ValueContext::Ret {
                    let result_sym = self.insert_anonymous_symbol();
                    context = ValueContext::StoreInto(result_sym);
                }
                expr_body.append(&mut context.with_binop_to_bytecode(lhs_value, op.clone(), rhs_value));
                Ok(expr_body)
            }
            Value::UnaryExpr(_, _) => unimplemented!("compiling IR to bytecode => unary operation"), 
            Value::FunCall(expr, args) => {
                let mut funcall_body = vec![];
                for arg in args {
                    funcall_body.append(&mut self.compile_value(arg, ValueContext::Push)?);
                }

                let function_name = match expr.as_ref() {
                    // if we're dealing with a function call name, we can use that directly
                    Value::Symbol(range_sym) => if let Symbol::Function(function_name) = range_sym.as_inner() {
                        Some(function_name)
                    } else { None },
                    // otherwise, we have to evaluate into a function ref
                    _ => None,
                    
                };

                if let Some(function_name) = function_name {
                    if let Some(symbol) = self.function_scope.lookup_function_symbol(function_name, args.len()) {
                        funcall_body.push(Bc::Call(symbol));
                    }
                } else {
                    funcall_body.push(Bc::PushValue(vm::Value::FunctionRefCanary));
                    funcall_body.append(&mut self.compile_value(expr, ValueContext::Push)?);
                    funcall_body.push(Bc::PopFunctionRefAndCall);
                }
                Ok(funcall_body)
            }
        }
    }

    fn compile_comparison(&mut self, value: &Value) -> Result<Vec<Bc>> {
        let comparison = match value {
            Value::BinaryExpr(lhs, op, rhs) => {
                match op {
                    | Op::Or
                    | Op::And
                    | Op::DoubleEquals
                    | Op::DoublePercent
                    | Op::DoubleTilde
                    | Op::NotEquals
                    | Op::LessEquals
                    | Op::GreaterEquals
                    | Op::Less
                    | Op::Greater => {
                        let lhs_sym = self.insert_anonymous_symbol();
                        let rhs_sym = self.insert_anonymous_symbol();
                        // TODO : short-circuiting?
                        let mut body = self.compile_value(lhs, ValueContext::StoreInto(lhs_sym.clone()))?;
                        body.append(&mut self.compile_value(rhs, ValueContext::StoreInto(rhs_sym.clone()))?);
                        let lhs_sym = vm::Value::Ref(lhs_sym);
                        let rhs_sym = vm::Value::Ref(rhs_sym);
                        vec![Bc::Compare(Condition::Compare(lhs_sym, CompareOp::from_syntax(&op).unwrap(), rhs_sym))]
                    }
                    _ => {
                        let result_sym = self.insert_anonymous_symbol();
                        let mut value_body = self.compile_value(value, ValueContext::StoreInto(result_sym.clone()))?;
                        let result_sym = vm::Value::Ref(result_sym);
                        value_body.push(Bc::Compare(Condition::Truthy(result_sym)));
                        value_body
                    }
                }
            }
            _ => {
                let result_sym = self.insert_anonymous_symbol();
                let mut value_body = self.compile_value(value, ValueContext::StoreInto(result_sym.clone()))?;
                let result_sym = vm::Value::Ref(result_sym);
                value_body.push(Bc::Compare(Condition::Truthy(result_sym)));
                value_body
            }
        };
        Ok(comparison)
    }

    /// Looks up a local symbol, or inserts it if necessary.
    fn lookup_or_insert_local_variable(&mut self, symbol_name: &str) -> vm::Symbol {
        if let Some(sym) = self.lookup_local_variable(symbol_name) {
            sym
        } else {
            self.insert_local_variable(symbol_name.to_string())
        }
    }

    /// Looks up a symbol in the `local_variables` table.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    ///
    /// # Returns
    /// `Some(symbol)` if the local symbol was found - otherwise, `None`.
    fn lookup_local_variable(&self, symbol_name: &str) -> Option<vm::Symbol> {
        self.local_variables
            .iter()
            .filter_map(|symbols|
                        symbols.iter().find(|local| {
                            assert_matches!(local, vm::Symbol::Variable(_));
                            self.variable_names[local.index()] == symbol_name
                        })
            )
            .map(|s| *s)
            .next()
    }

    /// Inserts a variable symbol into the local symbol table, returning a reference to it.
    ///
    /// If this symbol already exists in the table, the program will panic.
    fn insert_local_variable(&mut self, symbol_name: String) -> vm::Symbol {
        assert!(self.lookup_local_variable(&symbol_name).is_none());
        {
            let new_sym = self.next_variable_symbol(symbol_name.clone());
            let local_variables = self.local_variables.last_mut()
                .unwrap();
            local_variables.push(new_sym);
        }
        self.local_variable_count += 1;
        *self.local_variables
            .last().unwrap()
            .last().unwrap()
    }

    /// Creates an anonymous, compiler-generated symbol that cannot be referred to in code.
    fn insert_anonymous_symbol(&mut self) -> vm::Symbol {
        // TODO : Figure out how to re-use these when they're done (mite be difficult)
        let symbol_name = {
            let local_variables = self.local_variables.last().unwrap();
            format!("anonymous symbol #{}", local_variables.len())
        };
        self.insert_local_variable(symbol_name).clone()
    }

    /// Inserts a function symbol into the function symbol table, returning a reference to it.
    fn insert_vm_function(&mut self, function: vm::Function) -> &vm::Function {
        self.compiled_functions.push(function);
        self.compiled_functions.last()
            .unwrap()
    }

    /// Creates the next symbol used for a function with the given name.
    fn next_function_symbol(&mut self, name: String) -> vm::Symbol {
        self.function_scope.next_function_symbol(name)
    }

    /// Creates the next symbol used for a variable with the given name.
    fn next_variable_symbol(&mut self, name: String) -> vm::Symbol {
        self.variable_names.push(name);
        let num = self.all_variables.len();
        let sym = vm::Symbol::Variable(num);
        self.all_variables.push(sym);
        sym
    }

    fn err(&self, message: String) -> Error {
        message
    }
}

/// A definition of where and how a value is being used.
#[derive(Debug, PartialEq, Eq, Clone)]
enum ValueContext {
    /// This value is to be pushed to the stack for later use.
    Push,

    /// This value appears on the right hand side of an assignment and can be directly stored into
    /// a symbol.
    StoreInto(vm::Symbol),

    /// This value is going to be returned.
    Ret,
}

impl ValueContext {
    fn with_value_to_bytecode(self, value: vm::Value) -> Vec<Bc> {
        match self {
            ValueContext::Push => vec![Bc::PushValue(value)],
            ValueContext::StoreInto(sym_store) => vec![Bc::Store(sym_store, value)],
            ValueContext::Ret => vec![Bc::Ret(Some(value))],
        }
    }

    fn with_symbol_to_bytecode(self, sym: vm::Symbol) -> Vec<Bc> {
        self.with_value_to_bytecode(vm::Value::Ref(sym))
    }

    fn with_binop_to_bytecode(self, lhs: vm::Value, op: Op, rhs: vm::Value) -> Vec<Bc> {
        match self {
            ValueContext::Push => vec![Bc::BinOpPush(lhs, op, rhs)],
            ValueContext::StoreInto(sym_store) => vec![Bc::BinOpStore(lhs, op, rhs, sym_store)],
            ValueContext::Ret => panic!("can't compile ValueContext::Ret with binary operations, this should have been caught"),
        }
    }
}

#[derive(Debug)]
pub struct CompileUnit {
    pub name: String,
    pub main_function: vm::Function,
    pub functions: Vec<vm::Function>,
}
