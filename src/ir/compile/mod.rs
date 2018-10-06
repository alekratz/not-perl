use std::collections::{HashMap, HashSet};
use syntax::token::{Op, AssignOp};
use compile::{
    Variable,
    FunctionStub,
    FunctionScope,
    TyScope,
    VariableScope,
    ReserveSymbol,
};
use ir::*;
use vm::{
    self, Bc, Condition, CompareOp, Symbolic,
};

/// A compilation error.
pub type Error = String;

/// A compilation result.
pub type Result<T> = ::std::result::Result<T, Error>;


/// IR to bytecode compiler, complete with state.
#[derive(Debug, Clone)]
pub struct CompileState {
    operators: HashMap<Op, vm::FunctionSymbol>,
    body: Vec<Bc>,
    ty_scope: TyScope,
    function_scope: FunctionScope,
    variable_scope: VariableScope,
    repl: bool,
}

impl CompileState {
    pub fn new() -> Self {
        let builtin_functions = vm::BUILTIN_FUNCTIONS.iter()
            .cloned()
            .chain(vm::BUILTIN_OPERATORS.iter().map(|(_, f)| f).cloned())
            .collect();
        let function_scope = FunctionScope::new()
            .with_builtins(builtin_functions);
        let mut operators = HashMap::new();
        for (ref op, ref function) in vm::BUILTIN_OPERATORS.iter() {
            let sym = function_scope.get_stub_by_params(&function.name, function.params.len())
                .unwrap()
                .symbol();
            operators.insert(op.clone(), sym);
        }
        let ty_scope = TyScope::new().with_builtins();
        CompileState {
            operators,
            ty_scope,
            function_scope,
            variable_scope: VariableScope::new(),
            body: vec![],
            repl: false,
        }
    }

    pub fn repl() -> Self {
        let mut compile_state = CompileState::new();
        compile_state.repl = true;
        compile_state.begin();
        compile_state
    }

    pub fn begin(&mut self) {
        self.function_scope.push_empty_scope();
        self.variable_scope.push_empty_scope();
        self.ty_scope.push_empty_scope();
    }

    pub fn into_compile_unit(mut self) -> CompileUnit {
        let main_function_symbol = self.function_scope.reserve_symbol();

        let CompileState {
            // drop operators; they just keep track of the operators that the functions point at
            operators: _,
            body,
            ty_scope,
            function_scope,
            variable_scope,
            repl: _repl,
        } = self;

        let globals = variable_scope.all()
            .map(Variable::symbol)
            .collect();

        let main_function = vm::Function::User(vm::UserFunction {
            symbol: main_function_symbol,
            name: String::from("#main#"),
            params: 0,
            return_ty: ty_scope.get_builtin(vm::BuiltinTy::None).symbol(),
            locals: globals,
            body,
        });

        let mut tys = ty_scope.into_all();
        let mut functions = function_scope.into_vm_functions();
        // unstable sort is OK because there (hypothetically) are not duplicates
        functions.sort_unstable_by(|a, b| a.symbol().cmp(&b.symbol()));
        tys.sort_unstable_by(|a, b| a.symbol().cmp(&b.symbol()));

        CompileUnit {
            name: String::new(),
            main_function,
            functions,
            tys,
            function_names: vec![],
            variable_names: vec![],
            ty_names: vec![],
        }
    }

    pub fn to_compile_unit(&self) -> CompileUnit {
        self.clone().into_compile_unit()
    }

    pub fn feed_str(&mut self, filename: &str, contents: &str) -> Result<()> {
        use syntax::{Lexer, Parser};
        use ir::IrTree;

        let lexer = Lexer::new(contents.chars(), &filename);
        let parser = Parser::from_lexer(lexer);
        let tree = match parser.into_parse_tree() {
            Ok(t) => t,
            Err(e) => {
                return Err(format!("parse error: {}", e));
            },
        };

        let ir_tree = IrTree::from_syntax(&tree);
        
        self.feed(&ir_tree)
    }

    pub fn feed<'n>(&mut self, ir_tree: &IrTree<'n>) -> Result<()> {
        let known_good = self.clone();
        let result = {
            let mut feed = || {
                if self.repl {
                    // repls get a new body each time
                    self.body.clear();
                }
                // gather all function stubs
                let stubs = self.compile_function_stubs(ir_tree.functions())?;
                self.function_scope.push_all_values(stubs);

                for user_type in ir_tree.user_types() {
                    let ty = vm::Ty::User(self.compile_user_type(user_type)?);
                    self.ty_scope.push_value(ty);
                }

                // compile functions
                for function in ir_tree.functions() {
                    let function = self.compile_function(function)?;
                    self.function_scope.push_vm_function(vm::Function::User(function));
                }

                let mut body = self.compile_action_list(ir_tree.actions())?;
                self.body.append(&mut body);
                Ok(())
            };
            feed()
        };

        // undo any changes if an error occurred
        if result.is_err() {
            *self = known_good;
        }
        result
    }

    fn compile_function_stubs<'n>(&mut self, functions: &[Function<'n>]) -> Result<Vec<FunctionStub>> {
        // gather all function stubs
        let mut stubs = vec![];
        for function in functions {
            if self.function_scope.get_value_by_name(function.name()).is_some() {
                return Err(self.err(format!("function `{}` defined twice in the same scope", function.name())));
            }
            let stub = FunctionStub {
                name: function.name().to_string(),
                symbol: self.function_scope.reserve_symbol(),
                params: function.params.len(),
                return_ty: function.return_ty.clone(),
            };
            stubs.push(stub);
        }
        Ok(stubs)
    }

    fn compile_user_type<'n>(&mut self, udt: &UserTy<'n>) -> Result<vm::UserTy> {
        // TODO(predicate) : order-agnostic user defined types
        self.function_scope.push_empty_scope();
        if !udt.parents.is_empty() {
            // TODO(predicate) : deal with udt parents
            unimplemented!("TODO : compile IR user-defined type with parents");
        }

        // check if this type is already defined
        if self.ty_scope.get_local_value_by_name(&udt.name).is_some() {
            return Err(self.err(format!("type `{}` has already been defined in this scope", udt.name)));
        }
        
        // collect function stubs
        let stubs = self.compile_function_stubs(&udt.functions)?;
        self.function_scope.push_all_values(stubs);

        // TODO(predicate) order agnostic user types

        // compile functions
        let mut udt_functions = vec![];
        for ir_function in &udt.functions {
            let function = self.compile_function(ir_function)?;
            // XXX(predicate) : reference functions instead of cloning them
            udt_functions.push(function.symbol);
            self.function_scope.push_vm_function(vm::Function::User(function));
        }

        let udt_scope = self.function_scope.pop_scope()
            .unwrap();

        // get predicate function
        let user_predicate = udt_scope.iter()
            .filter(|f| f.name == "is?")
            .next();

        let predicate = if let Some(p) = user_predicate {
            if p.params == 1 {
                p.symbol()
            } else {
                return Err(self.err(format!("predicate function in type `{}` must have exactly one param", udt.name)));
            }
        } else {
            // everything's a string!!!!!
            *self.function_scope
                .get_builtin("is-string")
                .unwrap()
                .symbol()
        };
        let user_ty_symbol = self.ty_scope.reserve_symbol();

        Ok(vm::UserTy{
            name: udt.name.clone(),
            symbol: user_ty_symbol,
            predicate,
            functions: udt_functions,
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
        self.function_scope.push_empty_scope();
        self.variable_scope.push_empty_scope();

        let symbol = match &function.symbol {
            Symbol::Function(name) => {
                self.function_scope.get_stub_by_params(name, function.params.len())
                    .expect(&format!("symbol for function {} (param count {}) was expected to exist, but does not",
                                     name, function.params.len()))
                    .symbol()
            },
            sym => panic!("got non-function symbol name from IR::Function: {:?}", sym),
        };

        let mut param_names = HashSet::new();
        let mut body: Vec<Bc> = vec![];
        for param in &function.params {
            let param_name = param.name()
                .to_string();
            if param_names.contains(&param_name) {
                return Err(self.err(format!("duplicate function parameter `{}` in function definition `{}`",
                                            param_name,
                                            function.symbol.name())));
            }

            match param {
                FunctionParam::Variable { symbol: _, ty, default } => {
                    // TyExpr::None and TyExpr::All are simply not checked
                    let local_symbol = self.variable_scope.reserve_symbol();
                    self.variable_scope.push_value(Variable(param_name.clone(), local_symbol));
                    if let TyExpr::Definite(ty_name) = ty {
                        if let Some(ty) = self.ty_scope.get_value_by_name(ty_name) {
                            // insert the predicate check here
                            body.push(Bc::CheckSymbolTy { symbol: local_symbol, ty: ty.symbol(), });
                        } else {
                            return Err(self.err(format!("unknown type name for parameter `{}` in function definition `{}`: `{}`",
                                                        param_name, function.symbol.name(), ty_name)));
                        }
                    }

                    if let Some(_default) = default {
                        // TODO: default param values
                    }

                }
                FunctionParam::SelfKw => { unimplemented!("Self keyword param in ir::compile_function") }
            }

            param_names.insert(param_name);
        }


        // gather all function stubs
        let stubs = self.compile_function_stubs(&function.inner_functions)?;
        self.function_scope.push_all_values(stubs);

        // TODO: compile user types and their functions

        // compile functions
        for inner in &function.inner_functions {
            let inner = self.compile_function(inner)?;
            self.function_scope.push_vm_function(vm::Function::User(inner));
        }

        let return_ty = self.ty_scope.get_value_by_expr(&function.return_ty)
            .ok_or(format!("undefined type: {}", function.return_ty))?
            .symbol();
        body.append(&mut self.compile_action_list(&function.body)?);
        let locals = self.variable_scope.all()
            .map(Variable::symbol)
            .collect();
        self.function_scope.pop_scope();
        Ok(vm::UserFunction {
            symbol,
            name: function.name().to_string(),
            params: function.params.len(),
            return_ty,
            locals,
            body
        })
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
        
        // TODO : move AssignOp stuff into the IR
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
                        let function = self.function_scope.get_value_by_name(s)
                            .ok_or(format!("unknown function `{}`", s))?
                            .clone();
                        Ok(context.with_value_to_bytecode(vm::Value::FunctionRef(function.symbol)))
                    }
                    Symbol::Bareword(b) => {
                        if let Some(stub) = self.function_scope.get_value_by_name(b) {
                            Ok(context.with_value_to_bytecode(vm::Value::FunctionRef(stub.symbol)))
                        } else {
                            unimplemented!("compiling IR to bytecode => bareword lookup (name: {})", b)
                        }
                    }
                    Symbol::Variable(s) => {
                        let symbol = self.lookup_or_insert_local_variable(s).clone();
                        Ok(context.with_symbol_to_bytecode(symbol))
                    }
                }
            }
            Value::ArrayAccess(_, _) => unimplemented!("compiling IR to bytecode => array access"),
            Value::BinaryExpr(lhs, op, rhs) => {
                let op_function_symbol = if let Some(sym) = self.operators.get(op) {
                    *sym
                } else {
                    return Err(self.err(format!("`{}` is not a legal binary operator", op)));
                };
                let mut expr_body = vec![];
                // TODO : compound this into a lambda or something

                // LHS
                let lhs_value = if lhs.is_immediate() {
                    self.convert_immediate_value(lhs)
                } else {
                    let lhs_sym = self.variable_scope.push_anonymous_symbol()
                        .symbol();
                    expr_body.append(&mut self.compile_value(lhs, ValueContext::StoreInto(lhs_sym.clone()))?);
                    vm::Value::Ref(lhs_sym)
                };
                // RHS
                let rhs_value = if rhs.is_immediate() {
                    self.convert_immediate_value(rhs)
                } else {
                    let rhs_sym = self.variable_scope.push_anonymous_symbol()
                        .symbol();
                    expr_body.append(&mut self.compile_value(rhs, ValueContext::StoreInto(rhs_sym.clone()))?);
                    vm::Value::Ref(rhs_sym)
                };
                // TODO : operators which don't return a value
                expr_body.push(Bc::PushValue(rhs_value));
                expr_body.push(Bc::PushValue(lhs_value));
                expr_body.push(Bc::Call(op_function_symbol));
                match context {
                    ValueContext::Push => {}
                    ValueContext::StoreInto(sym) => expr_body.push(Bc::Pop(sym)),
                    ValueContext::Ret => expr_body.push(Bc::Ret(None)),
                }
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
                    if let Some(stub) = self.function_scope.get_stub_by_params(function_name, args.len()) {
                        funcall_body.push(Bc::Call(stub.symbol));
                        if context != ValueContext::Push {
                            if stub.return_ty == TyExpr::None {
                                return Err(self.err(format!("function `{}` doesn't return a value", stub.name)));
                            }
                            match context {
                                ValueContext::StoreInto(sym) => funcall_body.push(Bc::Pop(sym)),
                                // ValueContext::Ret means that we're just returning the returned value
                                ValueContext::Ret => funcall_body.push(Bc::Ret(None)),
                                ValueContext::Push => unreachable!(),
                            }
                        }
                    } else {
                        return Err(self.err(format!("no such function `{}`", function_name)));
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
                        let lhs_sym = self.variable_scope.push_anonymous_symbol()
                            .symbol();
                        let rhs_sym = self.variable_scope.push_anonymous_symbol()
                            .symbol();
                        // TODO : short-circuiting?
                        let mut body = self.compile_value(lhs, ValueContext::StoreInto(lhs_sym.clone()))?;
                        body.append(&mut self.compile_value(rhs, ValueContext::StoreInto(rhs_sym.clone()))?);
                        let lhs_sym = vm::Value::Ref(lhs_sym);
                        let rhs_sym = vm::Value::Ref(rhs_sym);
                        vec![Bc::Compare(Condition::Compare(lhs_sym, CompareOp::from_syntax(&op).unwrap(), rhs_sym))]
                    }
                    _ => {
                        let result_sym = self.variable_scope.push_anonymous_symbol()
                            .symbol();
                        let mut value_body = self.compile_value(value, ValueContext::StoreInto(result_sym.clone()))?;
                        let result_sym = vm::Value::Ref(result_sym);
                        value_body.push(Bc::Compare(Condition::Truthy(result_sym)));
                        value_body
                    }
                }
            }
            _ => {
                let result_sym = self.variable_scope.push_anonymous_symbol()
                    .symbol();
                let mut value_body = self.compile_value(value, ValueContext::StoreInto(result_sym.clone()))?;
                let result_sym = vm::Value::Ref(result_sym);
                value_body.push(Bc::Compare(Condition::Truthy(result_sym)));
                value_body
            }
        };
        Ok(comparison)
    }

    /// Looks up a local symbol, or inserts it if necessary.
    fn lookup_or_insert_local_variable(&mut self, symbol_name: &str) -> vm::VariableSymbol {
        if let Some(sym) = self.variable_scope.get_value_by_name(symbol_name).map(Variable::symbol) {
            sym
        } else {
            let sym = self.variable_scope.reserve_symbol();
            let var = Variable(symbol_name.to_string(), sym);
            self.variable_scope.push_value(var);
            sym
        }
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
    StoreInto(vm::VariableSymbol),

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

    fn with_symbol_to_bytecode(self, sym: vm::VariableSymbol) -> Vec<Bc> {
        self.with_value_to_bytecode(vm::Value::Ref(sym))
    }
}

#[derive(Debug)]
pub struct CompileUnit {
    pub name: String,
    pub main_function: vm::Function,
    pub functions: Vec<vm::Function>,
    pub tys: Vec<vm::Ty>,
    pub function_names: Vec<String>,
    pub variable_names: Vec<String>,
    pub ty_names: Vec<String>,
}
