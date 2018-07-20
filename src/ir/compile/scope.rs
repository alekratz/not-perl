use ir::compile::*;
use vm;

#[derive(Debug, Clone)]
pub struct FunctionScope {
    function_symbols: Vec<vm::Symbol>,
    function_names: Vec<String>,
    scope: Vec<Vec<FunctionStub>>,
}

impl FunctionScope {
    pub fn new() -> Self {
        FunctionScope {
            function_symbols: vec![],
            function_names: vec![],
            scope: vec![],
        }
    }

    /// Creates the next symbol used for a function with the given name.
    pub fn next_symbol(&mut self, name: String) -> vm::Symbol {
        assert!(self.lookup_local_stub_by_name(&name).is_none());
        let num = self.function_symbols.len();
        self.function_names.push(name);
        let sym = vm::Symbol::Function(num);
        self.function_symbols.push(sym);
        sym
    }

    /// Adds a new layer to this function scope.
    pub fn add_scope(&mut self) {
        self.scope.push(vec![]);
    }

    /// Sheds a layer of this function scope.
    pub fn shed_scope(&mut self) {
        self.scope.pop().expect("tried to shed empty function scope");
    }

    /// Inserts a function stub into the current stub scope and all function stubs array, as well
    /// as creating an extra symbol.
    pub fn insert_stub(&mut self, stub: FunctionStub) -> &FunctionStub {
        let stub_scope = self.scope.last_mut()
            .unwrap();
        stub_scope.push(stub);
        stub_scope.last()
            .unwrap()
    }

    /// Looks up a function stub based on its name and parameter count.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    /// * `param_count` - the number of parameters in this function stub.
    ///
    /// # Returns
    /// `Some(symbol)` if the function stub was found - otherwise, `None`.
    pub fn lookup_stub(&self, symbol_name: &str, param_count: usize) -> Option<&FunctionStub> {
        self.scope
            .iter()
            .rev()
            .filter_map(|collection| collection.iter()
                        .find(|function| self.function_names[function.symbol.index()] == symbol_name
                              && function.param_count == param_count))
            .next()
    }

    /// Looks up a function's symbol based on its name and the current function scope.
    pub fn lookup_symbol(&self, name: &str, param_count: usize) -> Option<vm::Symbol> {
        self.lookup_stub(name, param_count)
            .map(|stub| stub.symbol)
    }

    pub fn lookup_name(&self, symbol: vm::Symbol) -> &str {
        assert_matches!(symbol, vm::Symbol::Function(_));
        &self.function_names[symbol.index()]
    }

    /// Looks up a function stub based exclusively on its name.
    ///
    /// Since a parameter count is not supplied, name shadowing is possible and a function
    /// unintended by the user may be selected.
    ///
    /// This type of lookup only happens when a parameter count isn't available; e.g., a reference
    /// to a function and not a function call.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    ///
    /// # Returns
    /// `Some(symbol)` if the function stub was found - otherwise, `None`.
    pub fn lookup_stub_by_name(&self, symbol_name: &str) -> Option<&FunctionStub> {
        self.scope
            .iter()
            .rev()
            .filter_map(|collection| collection.iter()
                        .find(|function| self.function_names[function.symbol.index()] == symbol_name))
            .next()
    }

    /// Looks for the current function in the local scope only (not hopping up the scope if the
    /// function is not found).
    ///
    /// This function is used to determine if a function of the same name has already been defined
    /// in this scope.
    pub fn lookup_local_stub_by_name(&self, symbol_name: &str) -> Option<&FunctionStub> {
        self.scope
            .last()
            .unwrap()
            .iter()
            .filter(|stub| self.function_names[stub.symbol.index()] == symbol_name)
            .next()
    }
}

#[derive(Debug, Clone)]
pub struct VariableScope {
    all_variables: Vec<vm::Symbol>,
    variable_names: Vec<String>,
    scope: Vec<Vec<vm::Symbol>>,
}

impl VariableScope {
    pub fn new() -> Self {
        VariableScope {
            all_variables: vec![],
            variable_names: vec![],
            scope: vec![],
        }
    }

    pub fn add_scope(&mut self) {
        self.scope.push(vec![]);
    }

    pub fn shed_scope(&mut self) -> Vec<vm::Symbol> {
        self.scope.pop()
            .expect("tried to shed empty variable scope")
    }

    fn next_symbol(&mut self, name: String) -> vm::Symbol {
        self.variable_names.push(name);
        let global_idx = self.all_variables.len();
        let local_idx = self.scope.last()
            .unwrap()
            .len();
        let sym = vm::Symbol::Variable(global_idx, local_idx);
        self.all_variables.push(sym);
        let local_variables = self.scope.last_mut()
            .unwrap();
        local_variables.push(sym);
        sym
    }

    /// Inserts a variable symbol into the local symbol table, returning a reference to it.
    ///
    /// If this symbol already exists in the table, the program will panic.
    pub fn insert_local_variable(&mut self, symbol_name: String) -> vm::Symbol {
        assert!(self.lookup_local_variable(&symbol_name).is_none());
        self.next_symbol(symbol_name.clone())
    }

    /// Looks up a variable defined in this scope only.
    ///
    /// This function will not search up the scope stack.
    ///
    /// # Arguments
    ///
    /// * `symbol_name` - the name of the symbol that is being looked up.
    ///
    /// # Returns
    /// `Some(symbol)` if the local symbol was found - otherwise, `None`.
    pub fn lookup_local_variable(&self, symbol_name: &str) -> Option<vm::Symbol> {
        self.scope
            .iter()
            .filter_map(|symbols|
                        symbols.iter().find(|local| {
                            assert_matches!(local, vm::Symbol::Variable(_, _));
                            self.variable_names[local.index()] == symbol_name
                        })
            )
            .map(|s| *s)
            .next()
    }

    /// Looks up a local symbol, or inserts it if necessary.
    pub fn lookup_or_insert_local_variable(&mut self, symbol_name: &str) -> vm::Symbol {
        if let Some(sym) = self.lookup_local_variable(symbol_name) {
            sym
        } else {
            self.insert_local_variable(symbol_name.to_string())
        }
    }

    /// Looks up a variable name based on its VM symbol.
    pub fn lookup_variable_name(&self, symbol: vm::Symbol) -> &str {
        assert_matches!(symbol, vm::Symbol::Variable(_, _));
        &self.variable_names[symbol.index()]
    }

    /// Looks up a local variable name based on its VM symbol.
    ///
    /// This will not traverse the scope stack, only checking the local scope.
    pub fn lookup_local_variable_name(&self, symbol: vm::Symbol) -> Option<&str> {
        assert_matches!(symbol, vm::Symbol::Variable(_, _));
        let name = self.lookup_variable_name(symbol);
        self.lookup_local_variable(name)
            .map(|v| self.lookup_variable_name(v))
    }

    pub fn insert_anonymous_symbol(&mut self) -> vm::Symbol {
        let symbol_name = format!("anonymous symbol #{}", self.all_variables.len());
        self.insert_local_variable(symbol_name)
    }
}
