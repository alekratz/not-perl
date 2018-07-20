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

    /// All function names in this program.
    pub function_names: Vec<String>,

    /// All variable names in this program.
    pub variable_names: Vec<String>,
}

impl Storage {
    pub fn new(code: Vec<Function>, constants: Vec<Value>, function_names: Vec<String>,
               variable_names: Vec<String>) -> Self {
        Storage {
            scope_stack: vec![],
            value_stack: vec![],
            code,
            constants,
            function_names,
            variable_names,
        }
    }

    /// Gets the function at the given index.
    ///
    /// You can alternatively use `storage.code[idx]`, but this is more symbolic.
    #[inline]
    pub fn load_function(&self, idx: usize) -> &Function {
        &self.code[idx]
    }

    pub fn load<'v>(&'v self, symbol: Symbol) -> Result<&'v Value> {
        match symbol {
            Symbol::Variable(global, _) => {
                if let Some(value) = self.current_scope().try_get(symbol) {
                    self.dereference(value)
                } else {
                    for scope in &self.scope_stack {
                        if let Some(value) = scope.try_get(symbol) {
                            return self.dereference(value);
                        }
                    }
                    // TODO : String table
                    Err(self.err(format!("could not resolve symbol: {}", global)))
                }
            }
            Symbol::Constant(idx) => {
                Ok(&self.constants[idx])
            }
            Symbol::Function(idx) => panic!("tried to load the value of a function symbol (sym {})", idx),
        }
    }

    pub fn dereference<'v>(&'v self, value: &'v Value) -> Result<&'v Value> {
        match value {
            Value::Ref(sym) => {
                let value = self.load(*sym)?;
                self.dereference(&value)
            }
            _ => Ok(value),
        }
    }

    pub fn store(&mut self, symbol: Symbol, value: Value) -> Result<()> {
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

    pub fn function_name(&self, symbol: Symbol) -> &str {
        if let Symbol::Function(sym) = symbol {
            &self.function_names[sym]
        } else {
            panic!("not a function symbol: {:?}", symbol)
        }
    }

    pub fn variable_name(&self, symbol: Symbol) -> &str {
        if let Symbol::Variable(sym, _) = symbol {
            &self.variable_names[sym]
        } else {
            panic!("not a variable symbol: {:?}", symbol)
        }
    }

    fn err(&self, message: String) -> Error {
        message
    }
}
