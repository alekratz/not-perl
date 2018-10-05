use vm::{
    Result,
    Symbol,
    VariableSymbol,
    FunctionSymbol,
    Storage,
    Ty,
    BuiltinTy,
};
use ir::Const;

/// The index type for a value.
///
/// Numerically indexed values are the primary method of storing and loading values.
pub type ValueIndex = usize;

#[derive(EnumIsA, EnumAsGetters, Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),

    /// A canary placed before an expected symbol ref.
    ///
    /// This is used when the LHS of an expression is not necessarily a variable (e.g. an array
    /// index, or a function returning a ref). When the VM is ready to pop off the top value
    /// (expecting a reference), it makes sure that the next value is the ref canary to ensure that
    /// the expected reference *was* actually pushed.
    ///
    /// This prevents a mismatched stack and erroneous variable assignments.
    RefCanary,

    /// A reference to something.
    Ref(VariableSymbol),

    /// A canary placed before an expected function ref.
    ///
    /// This is very similar to the `RefCanary`, except that it expects a function ref on top of the
    /// stack instead of a regular symbol ref.
    FunctionRefCanary,

    FunctionRef(FunctionSymbol),

    /// An unset value.
    ///
    /// Attempting to access a value that is unset will result in a run-time exception.
    Unset,
}

impl Value {
    pub fn display_string(&self) -> String {
        match self {
            Value::Int(i) => format!("{}", i),
            Value::Float(f) => format!("{}", f),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => format!("{}", b),
            Value::Array(_) => format!("TODO : vm::Value array display string"),
            Value::RefCanary => "<Ref Canary>".to_string(),
            Value::Ref(s) => format!("<Reference to symbol {:#x}>", s.index()),
            Value::FunctionRefCanary => "<Function Ref Canary>".to_string(),
            Value::FunctionRef(c) => format!("<Reference to Function {:#x}>", c.index()),
            Value::Unset => "<Unset Value>".to_string(),
        }
    }

    pub fn is_immediate(&self) -> bool {
        match self {
            | Value::Int(_) 
            | Value::Float(_) 
            | Value::Str(_) 
            | Value::Bool(_) 
            | Value::Array(_) 
            | Value::RefCanary 
            | Value::FunctionRefCanary 
            | Value::Unset => true,
            | Value::Ref(_)
            | Value::FunctionRef(_) => false,
        }
    }

    /// Attempts to cast this value to a value of the supplied type.
    ///
    /// # Returns
    /// If this value can be cast to the supplied type, 
    pub fn cast(&self, ty: Ty, storage: &Storage) -> CastResult {
        match ty {
            Ty::Builtin(builtin, _) => self.cast_to_builtin(builtin, storage),
            Ty::User(_udt) => unimplemented!("TODO(predicate) : UDT value casting"),
        }
    }

    pub fn cast_to_builtin(&self, builtin: BuiltinTy, storage: &Storage) -> CastResult {
        match storage.dereference(self).unwrap() {
            Value::Int(i) => match builtin {
                BuiltinTy::Int => CastResult::SelfValid,
                BuiltinTy::Float => CastResult::Value(Value::Float((*i) as f64)),
                BuiltinTy::Str => CastResult::Value(Value::Str(i.to_string())),
                BuiltinTy::Bool => CastResult::Value(Value::Bool(*i != 0)),
                BuiltinTy::Array => unimplemented!("TODO : cast int to array value"),
                BuiltinTy::Any => unimplemented!("TODO : cast int to Any value"),
                BuiltinTy::None => CastResult::Invalid,
            },
            Value::Float(f) => match builtin {
                BuiltinTy::Int => CastResult::Value(Value::Int(f.trunc() as i64)),
                BuiltinTy::Float => CastResult::SelfValid,
                BuiltinTy::Str => CastResult::Value(Value::Str(f.to_string())),
                BuiltinTy::Bool => CastResult::Value(Value::Bool(*f != 0.0)),
                BuiltinTy::Array => unimplemented!("TODO : cast float to array value"),
                BuiltinTy::Any => unimplemented!("TODO : cast float to Any value"),
                BuiltinTy::None => CastResult::Invalid,
            },
            Value::Str(s) => match builtin {
                BuiltinTy::Int => if let Ok(i) = s.parse::<i64>() {
                    CastResult::Value(Value::Int(i))
                } else {
                    CastResult::Invalid
                },
                BuiltinTy::Float => if let Ok(f) = s.parse::<f64>() {
                    CastResult::Value(Value::Float(f))
                } else {
                    CastResult::Invalid
                },
                BuiltinTy::Str => CastResult::SelfValid,
                BuiltinTy::Bool => CastResult::Value(Value::Bool(!s.is_empty())),
                BuiltinTy::Array => unimplemented!("TODO : cast str to array value"),
                BuiltinTy::Any => unimplemented!("TODO : cast str to Any value"),
                BuiltinTy::None => CastResult::Invalid,
            },
            Value::Unset => CastResult::Invalid,
            Value::Ref(_) => panic!("Reference gotten even though self was dereferenced (self: {:?})", self),
            Value::FunctionRef(_) => CastResult::Invalid,
            r => panic!("Attempted to cast invalid value {:?} to {:?}", r, builtin),
        }
    }

    pub fn is_truthy(&self, storage: &Storage) -> Result<bool> {
        match self {
            Value::Int(i) => Ok(*i != 0),
            Value::Float(f) => Ok(*f != 0.0),
            Value::Str(s) => Ok(!s.is_empty()),
            Value::Bool(b) => Ok(*b),
            Value::Array(_) => unimplemented!("TODO(array) : is_truthy"),
            Value::Ref(sym) => storage.load(*sym)?.is_truthy(storage),
            Value::FunctionRef(_) => Ok(true),
            Value::RefCanary | Value::FunctionRefCanary | Value::Unset =>
                panic!("invalid truthy value checked on value {:?}", self),
        }
    }

    pub fn cast_to_int_no_float(&self, storage: &Storage) -> Option<i64> {
        let base_value = storage.dereference(self).ok()?;
        if base_value.is_float() {
            return None;
        } else {
            base_value.cast_to_int(storage)
        }
    }

    pub fn cast_to_int(&self, storage: &Storage) -> Option<i64> {
        match self.cast_to_builtin(BuiltinTy::Int, storage) {
            CastResult::SelfValid => Some(*self.as_int()),
            CastResult::Value(Value::Int(i)) => Some(i),
            CastResult::Invalid => None,
            _ => unreachable!(),
        }
    }
    
    pub fn cast_to_float(&self, storage: &Storage) -> Option<f64> {
        match self.cast_to_builtin(BuiltinTy::Float, storage) {
            CastResult::SelfValid => Some(*self.as_float()),
            CastResult::Value(Value::Float(f)) => Some(f),
            CastResult::Invalid => None,
            _ => unreachable!(),
        }
    }
}

impl<'n> From<Const> for Value {
    fn from(other: Const) -> Self {
        match other {
            Const::Str(s) => Value::Str(s),
            Const::Int(i) => Value::Int(i),
            Const::Float(f) => Value::Float(f),
            Const::Bool(b) => Value::Bool(b),
        }
    }
}

pub enum CastResult {
    SelfValid,
    Value(Value),
    Invalid,
}

impl CastResult {
    pub fn is_valid(&self) -> bool {
        match self {
            | CastResult::SelfValid
            | CastResult::Value(_) => true,
            CastResult::Invalid => false,
        }
    }
}
