use std::{
    fmt::{self, Formatter, Display},
};
use failure::{Context, Fail, Backtrace};
use crate::common::{
    lang::Op,
    pos::*,
};

macro_rules! error_kind_def {
    (fn $builder_name:ident ( $($argname:ident : $argty:ty ),* )
     -> $error_kind:ident
     => ($($display_args:expr),+)
        $body:block
     $($tail:tt)*) => {
        impl Error {
            #[allow(dead_code)]
            pub fn $builder_name (range: Range, $($argname: $argty),*) -> Error {
                Error::new(range, $body)
            }
        }

        error_kind_def! {
            $($tail)*
            @DISPLAY_ARGS $error_kind ($($argname:$argty),*) ($($display_args),+)
        }
    };

    (fn $builder_name:ident ( $($argname:ident : $argty:ty),* )
     -> $error_kind:ident
     => ($($display_args:expr),+)
     $($tail:tt)*
     ) => {
        error_kind_def! {
            fn $builder_name ( $($argname: $argty),* ) -> $error_kind => ($($display_args),+) {
                ErrorKind::$error_kind { $($argname),* }
            }

            $($tail)*
        }
    };

    ($(@DISPLAY_ARGS $kind:ident ($($argname:ident:$argty:ty),*) ($($display_args:expr),+) )+) => {
        impl Display for ErrorKind {
            fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
                match self {
                    $(
                        ErrorKind::$kind { $($argname),* } => write!(fmt, $($display_args),+),
                    )+
                }
            }
        }

        #[derive(Debug)]
        pub enum ErrorKind {
            $(
                $kind { $($argname: $argty),* }
            ),+
        }
    };

    () => {};
}

error_kind_def! {
    fn unknown_unary_op(op: Op)         -> UnknownUnaryOp   => ("unknown unary operator {}", op)
    fn unknown_binary_op(op: Op)        -> UnknownBinaryOp  => ("unknown binary operator {}", op)
    fn unknown_fun(name: String)        -> UnknownFun       => ("unknown function `{}`", name)
    fn unknown_ty(name: String)         -> UnknownTy        => ("unknown type `{}`", name)
    fn invalid_assign_lhs(lhs: String)  -> InvalidAssignLhs => ("invalid left-hand side of assignment: {}", lhs)
    fn duplicate_fun(first_def: Range, name: String)
                                        -> DuplicateFun     => ("duplicate function definition: {} (first definition here: {})", name, first_def)
    fn duplicate_ty(first_def: Range, name: String)
                                        -> DuplicateTy      => ("duplicate type definition: {} (first definition here: {})", name, first_def)
    fn break_outside_of_loop()          -> BreakOutsideOfLoop
                                                            => ("break statement defined outside of loop")
    fn continue_outside_of_loop()       -> ContinueOutsideOfLoop
                                                            => ("continue statement used outside of loop")
}

#[derive(Debug)]
pub struct Error
    where ErrorKind: 'static
{
    range: Range,
    kind: Context<ErrorKind>,
}

impl Error {
    pub fn new(range: Range, kind: ErrorKind) -> Self {
        Error { range, kind: Context::new(kind) }
    }

    pub fn range(&self) -> Range {
        self.range.clone()
    }

    pub fn kind(&self) -> &ErrorKind {
        self.kind.get_context()
    }
}

impl Fail for Error
    where Self: 'static
{
    fn cause(&self) -> Option<&Fail> {
        self.kind.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.kind.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.kind, fmt)
    }
}

//pub type Result<T> = ::std::result::Result<T, Error<'_>>;
