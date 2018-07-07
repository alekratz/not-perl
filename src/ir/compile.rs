use syntax::token::AssignOp;
use ir::*;
use vm::{
    self, Bc, Label
};

/// A compilation error.
pub type Error = String;

/// A compilation result.
pub type Result<T> = ::std::result::Result<T, Error>;

/// IR to bytecode compiler, complete with state.
pub struct Compile {
    vm_functions: Vec<Vec<vm::Function>>,
    //constants: Vec<
    local_symbols: Vec<Vec<vm::Symbol>>,
    labels: Vec<Vec<Label>>,

    function_count: usize,
    local_symbol_count: usize,
    label_count: usize,
    block_label: Option<Label>,
}

impl Compile {
    pub fn new() -> Self {
        Compile {
            vm_functions: vec![],
            local_symbols: vec![],
            labels: vec![],

            function_count: 0,
            local_symbol_count: 0,
            label_count: 0,
            block_label: None,
        }
    }

    pub fn compile_ir_tree<'n>(&mut self, ir_tree: &IrTree<'n>) -> Result<()> {
        // TODO : can compiler state be re-used after compiling one ir tree?
        self.vm_functions.push(vec![]);
        self.local_symbols.push(vec![]);
        self.labels.push(vec![]);

        // compile functions
        for function in ir_tree.functions() {
            let function = self.compile_function(function)?;
            if self.lookup_vm_function(function.name()).is_some() {
                return Err(self.err(format!("function `{}` defined twice", function.name())));
            }
            self.insert_vm_function(function);
        }

        self.compile_action_list(ir_tree.actions())?;

        // TODO : compiled unit
        Ok(())
    }

    /// Converts a sequence of IR actions to a sequence of bytecode.
    fn compile_action_list<'n>(&mut self, actions: &[Action<'n>]) -> Result<Vec<Bc>> {
        let mut body = vec![];
        for action in actions {
            let mut thunk = match action {
                Action::Eval(value) => self.compile_value(value, ValueContext::Push)?,
                Action::Assign(lhs, op, rhs) => self.compile_action_assign(lhs, *op, rhs)?,
                Action::Loop(_block) => unimplemented!("IR compile Action::Loop"),
                Action::Block(_block) => unimplemented!("IR compile Action::Block"),
                Action::ConditionBlock { if_block, elseif_blocks, else_block } => unimplemented!("IR compile Action::CondtionBlock"),
                Action::Return(_) => unimplemented!("IR compile Action::Return"),
                Action::Break => unimplemented!("IR compile Action::Break"),
                Action::Continue => unimplemented!("IR compile Action::Continue"),
            };
            body.append(&mut thunk);
        }
        Ok(body)
    }

    /// Compiles an IR function into a VM function.
    pub fn compile_function<'n>(&mut self, function: &Function<'n>) -> Result<vm::Function>
    {
        self.local_symbols.push(vec![]);
        if let Some(function) = self.lookup_vm_function(function.name()) {
            return Err(self.err(format!("function with name `{}` already defined", function.name())))
        }
        let symbol = match &function.symbol {
            Symbol::Function(name) => {
                let function_num = self.vm_functions
                    .last()
                    .unwrap()
                    .len();
                vm::Symbol::Function(function_num, name.to_string())
            },
            sym => panic!("got non-function symbol name from IR::Function: {:?}", sym),
        };

        let mut params = vec![];
        for param in &function.params {
            let param = self.compile_function_param(param)?;
            let defined = params.iter()
                .any(|l: &vm::FunctionParam| l.name() == param.name());
            if defined {
                return Err(self.err(format!("duplicate parameter `{}` in function `{}`",
                                            param.name(), function.name())));
            } else {
                params.push(param);
            }
        }

        let return_ty = match &function.return_ty {
            Ty::Any => vm::Ty::Any,
            Ty::Definite(name) => vm::Ty::Definite(name.to_string()),
        };

        // create a new label context since we're inside of a function
        let old_label_count = self.label_count;
        self.label_count = 0;
        self.labels.push(vec![]);

        let body = self.compile_action_list(&function.body)?;

        self.label_count = old_label_count;
        let labels = self.labels.pop().unwrap();
        let locals = self.local_symbols.pop().unwrap();
        Ok(vm::Function { symbol, params, return_ty, locals, body, labels })
    }

    fn compile_function_param<'n>(&mut self, FunctionParam { name, ty, default: _ }: &FunctionParam<'n>)
        -> Result<vm::FunctionParam>
    {
        let symbol = self.insert_local_symbol(name.name().to_string())
            .clone();
        let ty = match ty {
            Ty::Any => vm::Ty::Any,
            Ty::Definite(name) => vm::Ty::Definite(name.to_string()),
        };
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
                    Symbol::Function(s) => unimplemented!("IR function lookup (return an error because it's on the LHS)"),
                    Symbol::Bareword(s) => unimplemented!("IR constant lookup (return an error because it's on the LHS)"),
                    Symbol::Variable(s) => self.lookup_or_insert_local_symbol(s).clone(),
                };
                ValueContext::StoreInto(vm_symbol)
            }
            // other, more "complex" values on the LHS mean that we need to do a push and then pop
            // off a symbol ref
            _ => ValueContext::Push,
        };
        let mut assign_body = vec![];

        if op != AssignOp::Equals {
            unimplemented!("IR compound assignment operators such as += or -=")
        }

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

        Ok(assign_body)
    }

    /// Compiles the given value (with usage context) into a thunk.
    fn compile_value(&mut self, value: &Value, context: ValueContext) -> Result<Vec<Bc>> {
        match value {
            Value::Const(value) => context.with_value_to_bytecode(value.as_inner().clone().into()),
            Value::Symbol(sym) => {
                match sym.as_inner() {
                    Symbol::Function(s) => {
                        let function = self.lookup_vm_function(s)
                            .expect(&format!("Attempted to look up unregistered function symbol (name: {})", s))
                            .clone();
                        context.with_value_to_bytecode(vm::Value::FunctionRef(function.symbol.clone()))
                    }
                    Symbol::Bareword(s) => unimplemented!("compiling IR to bytecode => constant value lookup"),
                    Symbol::Variable(s) => {
                        let symbol = self.lookup_or_insert_local_symbol(s).clone();
                        context.with_symbol_to_bytecode(symbol)
                    }
                }
            }
            Value::ArrayAccess(_, _) => unimplemented!("compiling IR to bytecode => array access"),
            Value::BinaryExpr(_, _, _) => unimplemented!("compiling IR to bytecode => binary operation"), 
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
                    let function = self.lookup_vm_function(function_name)
                        .expect(&format!("Attempted to look up unregistered function symbol (name: {})", function_name))
                        .clone();
                    funcall_body.push(Bc::Call(function.symbol.clone()));
                } else {
                    funcall_body.push(Bc::PushValue(vm::Value::FunctionRefCanary));
                    funcall_body.append(&mut self.compile_value(expr, ValueContext::Push)?);
                    funcall_body.push(Bc::PopFunctionRefAndCall);
                }
                Ok(funcall_body)
            }
        }
    }

    /// Looks up a local symbol, or inserts it if necessary.
    fn lookup_or_insert_local_symbol(&mut self, symbol_name: &str) -> &vm::Symbol {
        // safe because we're only ever returning our borrowed value if it exists
        unsafe {
            let local = self.lookup_local_symbol(symbol_name)
                .map(|s| s as *const _);
            if let Some(sym) = local {
                &*sym
            } else {
                self.insert_local_symbol(symbol_name.to_string())
            }
        }
    }

    /// Inserts a variable symbol into the local symbol table, returning a reference to it.
    ///
    /// If this symbol already exists in the table, the program will panic.
    fn insert_local_symbol(&mut self, symbol_name: String) -> &vm::Symbol {
        assert!(self.lookup_local_symbol(&symbol_name).is_none());
        {
            let local_symbols = self.local_symbols.last_mut()
                .unwrap();
            let index = self.local_symbol_count;
            let new_sym = vm::Symbol::Variable(index, symbol_name.clone());
            local_symbols.push(new_sym);
        }
        self.local_symbol_count += 1;
        self.local_symbols
            .last().unwrap()
            .last().unwrap()
    }

    /// Looks up a symbol in the `local_symbols` table.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    ///
    /// # Returns
    /// `Some(symbol)` if the local symbol was found - otherwise, `None`.
    fn lookup_local_symbol(&self, symbol_name: &str) -> Option<&vm::Symbol> {
        self.local_symbols
            .iter()
            .filter_map(|symbols|
                        symbols.iter().find(|local| {
                            assert_matches!(local, vm::Symbol::Variable(_, _));
                            local.name() == symbol_name
                        })
            )
            .next()
    }

    /// Inserts a function symbol into the function symbol table, returning a reference to it.
    ///
    /// If this symbol already exists in the table, the program will panic.
    fn insert_vm_function(&mut self, function: vm::Function) -> &vm::Function {
        assert!(self.lookup_vm_function(function.name()).is_none());
        {
            let functions = self.vm_functions
                .last_mut()
                .unwrap();
            functions.push(function);
        }
        self.vm_functions
            .last().unwrap()
            .last().unwrap()
    }

    /// Looks up a symbol in the `functions` table.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    ///
    /// # Returns
    /// `Some(symbol)` if the function symbol was found - otherwise, `None`.
    fn lookup_vm_function(&self, symbol_name: &str) -> Option<&vm::Function> {
        self.vm_functions
            .last()
            .unwrap()
            .iter()
            .find(|function| function.name() == symbol_name)
    }

    /// Creates a new label, incrementing the label sequence in this context.
    fn next_label(&mut self) -> Label {
        let label = Label(self.label_count);
        self.label_count += 1;
        label
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

}

impl ValueContext {
    fn with_value_to_bytecode(self, value: vm::Value) -> Result<Vec<Bc>> {
        match self {
            ValueContext::Push => Ok(vec![Bc::PushValue(value)]),
            ValueContext::StoreInto(sym_store) => Ok(vec![Bc::Store(sym_store, value)]),
        }
    }

    fn with_symbol_to_bytecode(self, sym: vm::Symbol) -> Result<Vec<Bc>> {
        match self {
            ValueContext::Push => Ok(vec![Bc::PushSymbolValue(sym)]),
            ValueContext::StoreInto(sym_store) => Ok(vec![Bc::Store(sym_store, vm::Value::Ref(sym))]),
        }
    }
}
