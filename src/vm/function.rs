use vm::{
    Symbol,
    Bc,
    Ty,
    ty,
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

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub symbol: Symbol,
    pub params: Vec<Ty>,
    pub return_ty: Ty,
    // TODO: builtin function type
}

/// Current philosophy:
///
/// Builtin functions should encompass the most *important* and *common* functions. This list may become
/// unwieldy. *This is okay*.
///
/// Performance intensive stuff implemented in the language itself is likely to be very, very slow.
/// Things like:
///     * regex engine
///     * grep
///     * strings
///     * string find, string replace
///     * yaml/json parsing, maybe?
///     * language primitives
///     * the language itself
///     * the language's package manager, maybe?
/// should **not** be implemented in this language, and instead offloaded onto other languages.
/// 
/// This language *uses* these features. It is not designed to be big and powerful enough to *provide* them.

macro_rules! builtin {
    ($name:ident ($head:expr $(, $tail:expr)*) -> $return_ty:expr) => {
        BuiltinFunction {
            symbol: Symbol::Function(0, stringify!($name).to_string()),
            params: vec![$head $(,$tail)*],
            return_ty: $return_ty,
        }
    };

    ($name:ident () -> $return_ty:expr) => {
        BuiltinFunction {
            symbol: Symbol::Function(0, stringify!($name).to_string()),
            params: vec![],
            return_ty: $return_ty,
        }
    };
}

lazy_static! {
    pub static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunction> = {
        vec![
            builtin!(print (Ty::Any) -> Ty::None),
            builtin!(readln () -> Ty::Definite(ty::STR_DEFINITE.to_string())),
        ].into_iter()
            .enumerate()
            .map(|(num, BuiltinFunction { symbol, params, return_ty })| {
                 let symbol = if let Symbol::Function(_, name) = symbol {
                     Symbol::Function(num, name)
                 } else { unreachable!() };
                 BuiltinFunction { symbol, params, return_ty }
            })
            .collect()
    };
}
