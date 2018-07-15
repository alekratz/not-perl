use std::fmt::{self, Formatter, Debug};
use vm::{
    Symbol,
    Storage,
    Bc,
    ty::{Ty, self},
};

#[derive(Debug, Clone)]
pub enum Function {
    Builtin(BuiltinFunction),
    User(UserFunction),
}

impl Function {
    pub fn name(&self) -> &str {
        self.symbol().name()
    }

    pub fn symbol(&self) -> &Symbol {
        match self {
            Function::Builtin(b) => &b.symbol,
            Function::User(u) => &u.symbol
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
    pub fn name(&self) -> &str {
        &self.symbol.name()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub symbol: Symbol,
    pub ty: Ty,
}

impl FunctionParam {

    pub fn name(&self) -> &str {
        self.symbol.name()
    }
}

#[derive(Clone)]
pub struct BuiltinFunction {
    pub symbol: Symbol,
    pub params: Vec<Ty>,
    pub return_ty: Ty,
    pub function: fn(&mut Storage) -> Result<(), String>,
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
            | Value::FunctionRef(ref s) 
            | Value::Ref(ref s) => storage.load(s)?
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
            symbol: Symbol::Function(0, stringify!($name).to_string()),
            params: vec![$head $(,$tail)*],
            return_ty: $return_ty,
            function: $function_name,
        }
    };

    ($function_name:path, $name:ident () -> $return_ty:expr) => {
        BuiltinFunction {
            symbol: Symbol::Function(0, stringify!($name).to_string()),
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
            builtin!(functions::readln, readln () -> Ty::Definite(ty::STR_DEFINITE.to_string())),
            // END BUILTINS ////////////////////////////////////////////////////
        ].into_iter()
            .enumerate()
            .map(|(num, BuiltinFunction { symbol, params, return_ty, function })| {
                 let symbol = if let Symbol::Function(_, name) = symbol {
                     Symbol::Function(num, name)
                 } else { unreachable!() };
                 BuiltinFunction { symbol, params, return_ty, function }
            })
            .collect()
    };
}
