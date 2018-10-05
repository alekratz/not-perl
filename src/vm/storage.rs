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

    /// The main function, if any, that should get executed.
    pub main_function: Option<Function>,

    /// A list of read-only constants.
    pub constants: Vec<Value>,

    /// All types in this VM.
    pub tys: Vec<Ty>,

    /// All function names in this program.
    pub function_names: Vec<String>,

    /// All variable names in this program.
    pub variable_names: Vec<String>,

    /// All type names in this program.
    pub ty_names: Vec<String>,
}

impl From<CompileUnit> for Storage {
    fn from(CompileUnit { name: _name, main_function, functions, tys, function_names, variable_names, ty_names, }: CompileUnit) -> Self {
        Storage {
            scope_stack: vec![],
            value_stack: vec![],
            functions: functions,
            main_function: Some(main_function),
            constants: vec![/* TODO: constants */],
            tys,
            function_names,
            variable_names,
            ty_names,
        }
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            scope_stack: vec![],
            value_stack: vec![],
            functions: vec![],
            main_function: None,
            constants: vec![],
            tys: vec![],
            function_names: vec![],
            variable_names: vec![],
            ty_names: vec![],
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
        &self.function_names[sym]
    }

    pub fn variable_name(&self, sym: VariableSymbol) -> &str {
        &self.variable_names[sym.global]
    }

    pub fn ty_name(&self, symbol: TySymbol) -> &str {
        self.get_ty(symbol).name()
    }

    fn err(&self, message: String) -> Error {
        message
    }
}
