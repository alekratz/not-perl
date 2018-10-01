use std::fmt::{self, Formatter, Debug};
use syntax::token::Op;
use vm::{
    TySymbol,
    FunctionSymbol,
    VariableSymbol,
    Storage,
    Bc,
    Ty,
    BuiltinTy,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Builtin(BuiltinFunction),
    User(UserFunction),
}

impl Function {
    pub fn symbol(&self) -> &FunctionSymbol {
        match self {
            Function::Builtin(b) => &b.symbol,
            Function::User(u) => &u.symbol
        }
    }

    pub fn param_count(&self) -> usize {
        match self {
            Function::Builtin(b) => b.params.len(),
            Function::User(u) => u.params,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub symbol: FunctionSymbol,
    pub name: String,
    pub params: usize,
    pub return_ty: TySymbol,
    pub locals: Vec<VariableSymbol>,
    pub body: Vec<Bc>,
}

impl PartialEq for UserFunction {
    /// Compares two user functions for equality.
    ///
    /// This does *not* check the function bodies, since comparing bytecode is a pain point right
    /// now.
    fn eq(&self, other: &Self) -> bool {
        self.symbol.eq(&other.symbol)
            && self.params.eq(&other.params)
            && self.return_ty.eq(&other.return_ty)
            && self.locals.eq(&other.locals)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub symbol: VariableSymbol,
    pub ty: Ty,
}

#[derive(Clone)]
pub struct BuiltinFunction {
    pub symbol: FunctionSymbol,
    pub name: String,
    pub params: Vec<BuiltinTy>,
    pub return_ty: BuiltinTy,
    pub function: fn(&mut Storage) -> Result<(), String>,
}

impl PartialEq for BuiltinFunction {
    /// Checks equality of two builtin functions.
    ///
    /// This does *not* check function pointers.
    fn eq(&self, other: &Self) -> bool {
        self.symbol.eq(&other.symbol)
            && self.name.eq(&other.name)
            && self.params.eq(&other.params)
            && self.return_ty.eq(&other.return_ty)
    }
}

impl Debug for BuiltinFunction {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("BuiltinFunction")
            .field("symbol", &self.symbol)
            .field("params", &self.params)
            .field("return_ty", &self.return_ty)
            .field("function", &format!("{:#x}", (&self.function as *const _ as usize)))
            .finish()
    }
}

mod functions {
    use vm::{Value, FunctionSymbol, Storage, Result};

    pub fn println(storage: &mut Storage) -> Result<()> {
        let value = storage.value_stack
            .pop()
            .expect("no println stack item");
        let value_string = match value {
            | Value::FunctionRef(FunctionSymbol(f)) => format!("Function #{}", f),
            | Value::Ref(s) => storage.load(s)?
                .display_string(),
            | value => value.display_string(),
        };
        // TODO : use VM's stdout pointer
        println!("{}", value_string);
        Ok(())
    }

    pub fn readln(_: &mut Storage) -> Result<()> {
        // TODO : builtin function readln
        Ok(())
    }

    pub fn is_string(storage: &mut Storage) -> Result<()> {
        // TODO : check against type
        storage.value_stack
            .pop()
            .expect("no is-string stack item");
        storage.value_stack
            .push(Value::Bool(true));
        Ok(())
    }
}

mod operators {
    use vm::{Value, Storage, Result};

    fn apply_arithmetic_pair(lhs: &Value, rhs: &Value, storage: &Storage,
                  apply_ints: impl Fn(i64, i64) -> Value,
                  apply_floats: impl Fn(f64, f64) -> Value) -> Result<Value>
    {
        if let Some(lhs_int) = lhs.cast_to_int_no_float(storage) {
            if let Some(rhs_int) = rhs.cast_to_int_no_float(storage) {
                Ok(apply_ints(lhs_int, rhs_int))
            } else if let Some(rhs_float) = rhs.cast_to_float(storage) {
                Ok(apply_floats(lhs_int as f64, rhs_float))
            } else {
                return Err(format!("cannot cast RHS to an addable value: {}", rhs.display_string()));
            }
        } else if let Some(lhs_float) = lhs.cast_to_float(storage) {
            // we don't need to check if rhs is int because we're going to be doing float addition
            // anyway
            if let Some(rhs_float) = rhs.cast_to_float(storage) {
                Ok(apply_floats(lhs_float, rhs_float))
            } else {
                return Err(format!("cannot cast RHS to an addable value: {}", rhs.display_string()));
            }
        } else {
            return Err(format!("cannot cast LHS to an addable value: {}", lhs.display_string()));
        }
    }

    macro_rules! arithmetic_operator {
        ($name:ident, $apply_ints:expr, $apply_floats:expr) => {
            pub fn $name (storage: &mut Storage) -> Result<()> {
                let lhs_owned = storage.value_stack
                    .pop()
                    .unwrap();
                let rhs_owned = storage.value_stack
                    .pop()
                    .unwrap();
                let result_value = {
                    let lhs = storage.dereference(&lhs_owned)?;
                    let rhs = storage.dereference(&rhs_owned)?;
                    apply_arithmetic_pair(lhs, rhs, storage, $apply_ints, $apply_floats)?
                };
                storage.value_stack.push(result_value);
                Ok(())
            }
        }
    }

    arithmetic_operator!(add, |i, j| Value::Int(i + j), |f, h| Value::Float(f + h));
    arithmetic_operator!(sub, |i, j| Value::Int(i - j), |f, h| Value::Float(f - h));
}

macro_rules! builtin {
    ($function_name:path, $name:ident ($($args:expr),*) -> $return_ty:expr) => {
        BuiltinFunction {
            symbol: FunctionSymbol(0),
            name: stringify!($name).to_string(),
            params: vec![$($args),*],
            return_ty: $return_ty,
            function: $function_name,
        }
    };

    ($function_name:path, $name:expr, ($($args:expr),*) -> $return_ty:expr) => {
        BuiltinFunction {
            symbol: FunctionSymbol(0),
            name: $name.to_string(),
            params: vec![$($args),*],
            return_ty: $return_ty,
            function: $function_name,
        }
    };
}

lazy_static! {
    pub static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunction> = {
        vec![
            // BEGIN BUILTINS //////////////////////////////////////////////////
            builtin!(functions::println, println (BuiltinTy::Any) -> BuiltinTy::None),
            builtin!(functions::readln, readln () -> BuiltinTy::Str),
            builtin!(functions::is_string, "is-string", () -> BuiltinTy::Bool),
            // END BUILTINS ////////////////////////////////////////////////////
        ]
    };

    /// The list of built-in operator functions.
    pub static ref BUILTIN_OPERATORS: Vec<(Op, BuiltinFunction)> = {
        vec![
            // BEGIN BUILTIN OPERATORS /////////////////////////////////////////
            (Op::Plus, builtin!(operators::add, "+", (BuiltinTy::Int, BuiltinTy::Int) -> BuiltinTy::Any)),
            (Op::Minus, builtin!(operators::sub, "-", (BuiltinTy::Int, BuiltinTy::Int) -> BuiltinTy::Any)),
            // END BUILTIN OPERATORS ///////////////////////////////////////////
        ]
    };
}
