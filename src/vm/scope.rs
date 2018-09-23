use vm::{Symbol, Value};

#[derive(Debug, Clone)]
pub struct Scope {
    /// The list of `Symbol`s that this scope defines.
    symbols: Vec<Symbol>,

    /// The list of values defined by this scope.
    values: Vec<Value>,
}

impl Scope {
    pub fn new(symbols: Vec<Symbol>, values: Vec<Value>) -> Self {
        assert_eq!(symbols.len(), values.len());
        Scope {
            symbols,
            values,
        }
    }

    pub fn try_get(&self, sym: Symbol) -> Option<&Value> {
        match sym {
            Symbol::Function(sym) => panic!("attempted to load scope value from function symbol {}", sym),
            Symbol::Constant(sym) => panic!("attempted to load scope value from constant symbol {}", sym),
            Symbol::Variable(_, idx) => {
                if self.values.len() <= idx || self.symbols[idx] != sym {
                    None
                } else {
                    Some(&self.values[idx])
                }
            }
            Symbol::Ty(sym) => panic!("attempted to load scope value from type symbol {}", sym),
        }
    }

    pub fn get(&self, sym: Symbol) -> &Value {
        if let Some(value) = self.try_get(sym) {
            value
        } else {
            panic!("invalid symbol lookup: {:?}", sym)
        }
    }

    pub fn try_set(&mut self, sym: Symbol, val: Value) -> bool {
        match sym {
            Symbol::Function(sym) => panic!("attempted to set function symbol to a value {}", sym),
            Symbol::Constant(sym) => panic!("attempted to set constant symbol to a value {}", sym),
            Symbol::Variable(_, idx) => {
                if self.symbols.len() <= idx || self.symbols[idx] != sym {
                    false
                } else {
                    self.values[idx] = val;
                    true
                }
            }
            Symbol::Ty(sym) => panic!("attempted to set type symbol to a value {}", sym),
        }
    }

    pub fn set(&mut self, sym: Symbol, val: Value) {
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
