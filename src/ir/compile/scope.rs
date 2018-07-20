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
    pub fn next_function_symbol(&mut self, name: String) -> vm::Symbol {
        self.function_names.push(name);
        let num = self.function_symbols.len();
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
    pub fn insert_function_stub(&mut self, stub: FunctionStub) -> &FunctionStub {
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
    pub fn lookup_function_stub(&self, symbol_name: &str, param_count: usize) -> Option<&FunctionStub> {
        self.scope
            .iter()
            .rev()
            .filter_map(|collection| collection.iter()
                        .find(|function| self.function_names[function.symbol.index()] == symbol_name
                              && function.param_count == param_count))
            .next()
    }

    /// Looks up a function's symbol based on its name and the current function scope.
    pub fn lookup_function_symbol(&self, name: &str, param_count: usize) -> Option<vm::Symbol> {
        self.lookup_function_stub(name, param_count)
            .map(|stub| stub.symbol)
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
    pub fn lookup_function_stub_name(&self, symbol_name: &str) -> Option<&FunctionStub> {
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
    pub fn lookup_local_function_stub_name(&self, symbol_name: &str) -> Option<&FunctionStub> {
        self.scope
            .last()
            .unwrap()
            .iter()
            .filter(|stub| self.function_names[stub.symbol.index()] == symbol_name)
            .next()
    }
}

