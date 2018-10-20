use std::{
    fmt::{self, Debug, Formatter},
};
use syntax::token::Op;
use vm::{
    symbol::*,
    storage::*,
    ty::BuiltinTy,
};

/// A function in the VM.
#[derive(Debug, Clone)]
pub enum Fun {
    User(UserFun),
    Builtin(&'static BuiltinFun, FunSymbol),
}

impl Fun {
    /// Gets the number of parameters expected for this function.
    pub fn params(&self) -> usize {
        match self {
            Fun::User(u) => u.params,
            Fun::Builtin(b, _) => b.params,
        }
    }
}

impl Symbolic for Fun {
    type Symbol = FunSymbol;

    fn symbol(&self) -> FunSymbol {
        match self {
            Fun::User(u) => u.symbol,
            Fun::Builtin(_, s) => *s,
        }
    }

    fn name(&self) -> &str {
        match self {
            Fun::User(u) => &u.name,
            Fun::Builtin(b, _) => &b.name,
        }
    }
}

/// A user-defined function.
#[derive(Debug, Clone)]
pub struct UserFun {
    pub symbol: FunSymbol,
    pub name: String,
    pub params: usize,
}

/// A builtin function.
#[derive(Clone)]
pub struct BuiltinFun {
    pub name: String,
    pub params: usize,
    pub return_ty: BuiltinTy,
    pub builtin: fn(&mut Storage),
}

pub struct BuiltinOp(pub Op, pub BuiltinFun);

impl Debug for BuiltinFun {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("BuiltinFun")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("return_ty", &self.return_ty)
            .field("builtin", &(&self.builtin as *const _))
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Builtin functions and operators
////////////////////////////////////////////////////////////////////////////////

macro_rules! builtin_fun {
    ($fun_name:ident = $builtin_name:ident ( $count:tt ) -> $($retval:tt)+) => {{
        use self::builtins;
        BuiltinFun {
            name: String::from(stringify!($builtin_name)),
            params: $count,
            return_ty: $($retval)+,
            builtin: builtins::$fun_name,
        }
    }}
}

macro_rules! builtin_op {
    ($fun_name:ident = $op:ident ( $count:tt ) -> $($retval:tt)+) => {{
        use self::builtins;
        use syntax::token::Op;
        BuiltinOp(Op::$op, BuiltinFun {
            name: Op::$op.to_string(),
            params: $count,
            return_ty: $($retval)+,
            builtin: builtins::$fun_name,
        })
    }};
}

mod builtins {
    use vm::Storage;

    /// Writes string value to a file descriptor.
    ///
    /// # Preconditions
    /// * Argument count: 2
    /// * Expected stack:
    ///     * `TOP`
    ///     * `str` - String - value to write to the file handle
    ///     * `descriptor` - Int - the file descriptor to write the string to.
    /// 
    /// # Postconditions
    /// Leaves an integer on the top of the stack containing the number of bytes written.
    pub fn writef(_storage: &mut Storage) {
        // TODO(builtin) : write to a file descriptor
    }

    /// Reads a string value from the given file descriptor.
    ///
    /// # Preconditions
    /// * Argument count: 1
    /// * Expected stack:
    ///     * `TOP`
    ///     * `descriptor` - Int - the file descriptor to read the string from.
    /// 
    /// # Postconditions
    /// Leaves a string on top of the stack, with the contents of the file.
    pub fn readf(_storage: &mut Storage) {
        // TODO(builtin) : read from a file descriptor
    }

    pub fn plus_binop(_storage: &mut Storage) {
        // TODO(builtin) : + operator
    }

    pub fn minus_binop(_storage: &mut Storage) {
        // TODO(builtin) : - operator
    }

    pub fn splat_binop(_storage: &mut Storage) {
        // TODO(builtin) : * operator
    }
    
    pub fn fslash_binop(_storage: &mut Storage) {
        // TODO(builtin) : / operator
    }

    pub fn tilde_binop(_storage: &mut Storage) {
        // TODO(builtin) : ~ operator
    }
}

lazy_static! {
    pub static ref builtin_functions: Vec<BuiltinFun> = vec![
        builtin_fun!(writef = writef ( 2 ) -> BuiltinTy::Int),
        builtin_fun!(readf = readf ( 1 ) -> BuiltinTy::Str),
    ];
    
    pub static ref builtin_ops: Vec<BuiltinOp> = vec![
        builtin_op!(plus_binop = Plus ( 2 ) -> BuiltinTy::Float),
        builtin_op!(minus_binop = Minus ( 2 ) -> BuiltinTy::Float),
        builtin_op!(splat_binop = Splat ( 2 ) -> BuiltinTy::Float),
        builtin_op!(fslash_binop = FSlash ( 2 ) -> BuiltinTy::Float),
        builtin_op!(tilde_binop = Tilde ( 2 ) -> BuiltinTy::Str),
    ];
}
