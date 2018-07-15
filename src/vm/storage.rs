use vm::{Scope, Value, Symbol, Function, Result, Error};

/// The storage state of the VM, which can be passed around if necessary.
#[derive(Debug)]
pub struct Storage {
    /// A stack of local variables for each scope we are inside.
    ///
    /// This usually will have a height of 1.
    pub scope_stack: Vec<Scope>,

    /// Main program stack.
    pub value_stack: Vec<Value>,

    /// An array of functions, indexed by the function "number".
    pub code: Vec<Function>,

    /// A list of read-only constants.
    pub constants: Vec<Value>,
}

impl Storage {
    pub fn new(code: Vec<Function>, constants: Vec<Value>) -> Self {
        Storage {
            scope_stack: vec![],
            value_stack: vec![],
            code,
            constants,
        }
    }

    /// Gets the function at the given index.
    ///
    /// You can alternatively use `storage.code[idx]`, but this is more symbolic.
    #[inline]
    pub fn load_function(&self, idx: usize) -> &Function {
        &self.code[idx]
    }

    pub fn load(&self, symbol: &Symbol) -> Result<Value> {
        match symbol {
            Symbol::Variable(_, ref name) => {
                let follow_ref = |value| {
                    // weird construction due to &value borrow
                    // NLL should fix this
                    {
                        if let Value::Ref(ref sym) = &value {
                            // follow references
                            return self.load(sym);
                        }
                    }
                    Ok(value)
                };

                if let Some(value) = self.current_scope().try_get(symbol) {
                    follow_ref(value)
                } else {
                    for scope in &self.scope_stack {
                        if let Some(value) = scope.try_get(symbol) {
                            return follow_ref(value);
                        }
                    }
                    Err(self.err(format!("could not resolve symbol: {}", name)))
                }
            }
            Symbol::Constant(idx, _) => {
                Ok(self.constants[*idx].clone())
            }
            Symbol::Function(idx, _) => {
                let function = &self.code[*idx];
                Ok(Value::FunctionRef(function.symbol().clone()))
            }
        }
    }

    pub fn store(&mut self, symbol: &Symbol, value: Value) -> Result<()> {
        if self.current_scope_mut().try_set(symbol, value.clone()) {
            Ok(())
        } else {
            Err(self.err(format!("could not set symbol: {:?} to value: {:?}", symbol, value)))
        }
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scope_stack
            .last_mut()
            .expect("no current scope")
    }

    pub fn current_scope(&self) -> &Scope {
        self.scope_stack
            .last()
            .expect("no current scope")
    }

    fn err(&self, message: String) -> Error {
        message
    }
}
