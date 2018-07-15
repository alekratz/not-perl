use vm::Symbol;
use ir::Const;

/// The index type for a value.
///
/// Numerically indexed values are the primary method of storing and loading values.
pub type ValueIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    //Bignum(
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
    Ref(Symbol),

    /// A canary placed before an expected function ref.
    ///
    /// This is very similar to the `RefCanary`, except that it expects a function ref on top of the
    /// stack instead of a regular symbol ref.
    FunctionRefCanary,

    FunctionRef(Symbol),

    //ConstantRef(Symbol),

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
            Value::Array(_) => unimplemented!("vm::Value array display string"),
            Value::RefCanary => "<Ref Canary, enjoy your crash>".to_string(),
            Value::Ref(s) => format!("<Reference to {}>", s.name()),
            Value::FunctionRefCanary => "<Function Ref Canary, enjoy your crash>".to_string(),
            Value::FunctionRef(c) => format!("<Reference to Function {}>", c.name()),
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
