use vm::*;

/// The storage state of the VM, which can be passed around if necessary.
#[derive(Debug, Clone)]
pub struct Storage {
    /// A stack of local variables for each scope we are inside.
    ///
    /// This usually will have a height of 1.
    pub scope_stack: Vec<Scope>,

    /// Main program stack.
    pub value_stack: Vec<Value>,

    /// An array of functions, indexed by the function "number".
    pub functions: Vec<Function>,

    /// The script body.
    pub body: Vec<Bc>,

    /// A list of read-only constants.
    pub constants: Vec<Value>,

    /// All types in this VM.
    pub tys: Vec<Ty>,

    pub variables: Vec<Variable>,
}

impl From<CompileUnit> for Storage {
    fn from(CompileUnit { body, functions, tys, variables, globals, }: CompileUnit) -> Self {
        let unset_globals = vec!(Value::Unset; globals.len());
        Storage {
            scope_stack: vec![Scope::new(globals, unset_globals)],
            value_stack: vec![],
            functions: functions,
            body,
            constants: vec![/* TODO: constants */],
            tys,
            variables,
        }
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            scope_stack: vec![],
            value_stack: vec![],
            functions: vec![],
            body: vec![],
            constants: vec![],
            tys: vec![],
            variables: vec![],
        }
    }

    /// Gets the function at the given index.
    ///
    /// You can alternatively use `storage.functions[idx]`, but this is more symbolic.
    #[inline]
    pub fn get_function(&self, FunctionSymbol(idx): FunctionSymbol) -> &Function {
        &self.functions[idx]
    }

    pub fn load<'v>(&'v self, symbol: VariableSymbol) -> Result<&'v Value> {
        if let Some(value) = self.current_scope().try_get(symbol) {
            self.dereference(value)
        } else {
            for scope in &self.scope_stack {
                if let Some(value) = scope.try_get(symbol) {
                    return self.dereference(value);
                }
            }
            Err(self.err(format!("could not resolve symbol: {}", self.variable_name(symbol))))
        }
    }

    pub fn dereference<'v>(&'v self, value: &'v Value) -> Result<&'v Value> {
        // TODO : This doesn't need to return a Result
        match value {
            Value::Ref(sym) => {
                let value = self.load(*sym)?;
                self.dereference(&value)
            }
            _ => Ok(value),
        }
    }

    pub fn store(&mut self, symbol: VariableSymbol, value: Value) -> Result<()> {
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

    pub fn get_ty(&self, TySymbol(sym): TySymbol) -> &Ty {
        &self.tys[sym]
    }

    pub fn function_name(&self, FunctionSymbol(sym): FunctionSymbol) -> &str {
        self.functions[sym].name()
    }

    pub fn variable_name(&self, sym: VariableSymbol) -> &str {
        &self.variables[sym.global].name()
    }

    pub fn ty_name(&self, TySymbol(sym): TySymbol) -> &str {
        self.tys[sym].name()
    }

    fn err(&self, message: String) -> Error {
        message
    }
}
