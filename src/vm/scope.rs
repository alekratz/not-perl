use vm::{VariableSymbol, Value};

#[derive(Debug, Clone)]
pub struct Scope {
    /// The list of `VariableSymbol`s that this scope defines.
    symbols: Vec<VariableSymbol>,

    /// The list of values defined by this scope.
    values: Vec<Value>,
}

impl Scope {
    pub fn new(symbols: Vec<VariableSymbol>, values: Vec<Value>) -> Self {
        assert_eq!(symbols.len(), values.len());
        Scope {
            symbols,
            values,
        }
    }

    pub fn try_get(&self, sym: VariableSymbol) -> Option<&Value> {
        let idx = sym.local;
        if self.values.len() <= idx || self.symbols[idx] != sym {
            None
        } else {
            Some(&self.values[idx])
        }
    }

    pub fn get(&self, sym: VariableSymbol) -> &Value {
        if let Some(value) = self.try_get(sym) {
            value
        } else {
            panic!("invalid symbol lookup: {:?}", sym)
        }
    }

    pub fn try_set(&mut self, sym: VariableSymbol, val: Value) -> bool {
        let idx = sym.local;
        if self.symbols.len() <= idx || self.symbols[idx] != sym {
            false
        } else {
            self.values[idx] = val;
            true
        }
    }

    pub fn set(&mut self, sym: VariableSymbol, val: Value) {
        if !self.try_set(sym, val) {
            panic!("invalid symbol store: {:?}", sym);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Updates this scope's values with values from another, possibly overwriting values.
    pub fn update(&mut self, other: Self) {
        let items = other.symbols.into_iter().zip(other.values.into_iter());
        for (sym, value) in items {
            self.set(sym, value);
        }
    }
}
