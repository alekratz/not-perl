use std::fmt::{self, Formatter, Debug};
use vm::{
    Symbol,
    Storage,
    Bc,
    Ty,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Builtin(BuiltinFunction),
    User(UserFunction),
}

impl Function {
    pub fn symbol(&self) -> &Symbol {
        match self {
            Function::Builtin(b) => &b.symbol,
            Function::User(u) => &u.symbol
        }
    }

    pub fn param_count(&self) -> usize {
        match self {
            Function::Builtin(b) => b.params.len(),
            Function::User(u) => u.params.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub symbol: Symbol,
    pub params: Vec<FunctionParam>,
    pub return_ty: Ty,
    pub locals: Vec<Symbol>,
    pub body: Vec<Bc>,
}

impl UserFunction {
    pub fn new(symbol: Symbol, params: Vec<FunctionParam>, return_ty: Ty, locals: Vec<Symbol>, body: Vec<Bc>) -> Self {
        UserFunction {
            symbol,
            params,
            return_ty,
            locals,
            body,
        }
    }
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
    pub symbol: Symbol,
    pub ty: Ty,
}

#[derive(Clone)]
pub struct BuiltinFunction {
    pub symbol: Symbol,
    pub name: String,
    pub params: Vec<Ty>,
    pub return_ty: Ty,
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
    use vm::{Value, Storage, Result};

    pub fn println(storage: &mut Storage) -> Result<()> {
        let value = storage.value_stack
            .pop()
            .expect("no println stack item");
        let value_string = match value {
            | Value::FunctionRef(s) 
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
}

macro_rules! builtin {
    ($function_name:path, $name:ident ($head:expr $(, $tail:expr)*) -> $return_ty:expr) => {
        BuiltinFunction {
            symbol: Symbol::Function(0),
            name: stringify!($name).to_string(),
            params: vec![$head $(,$tail)*],
            return_ty: $return_ty,
            function: $function_name,
        }
    };

    ($function_name:path, $name:ident () -> $return_ty:expr) => {
        BuiltinFunction {
            symbol: Symbol::Function(0),
            name: stringify!($name).to_string(),
            params: vec![],
            return_ty: $return_ty,
            function: $function_name,
        }
    };
}

lazy_static! {
    pub static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunction> = {
        vec![
            // BEGIN BUILTINS //////////////////////////////////////////////////
            builtin!(functions::println, println (Ty::Any) -> Ty::None),
            builtin!(functions::readln, readln () -> Ty::Str),
            // END BUILTINS ////////////////////////////////////////////////////
        ]
    };
}
