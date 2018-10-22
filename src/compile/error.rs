use std::{
    fmt::{self, Formatter, Display},
};
use failure::{Context, Fail, Backtrace};
use common::lang::Op;
use syntax::Range;

macro_rules! error_kind_def {
    (fn $builder_name:ident ( $($argname:ident : $argty:ty ),+ )
     -> $error_kind:ident
     => ($($display_args:expr),+)
        $body:block
     $($tail:tt)*) => {
        impl<'n> Error<'n> {
            #[allow(dead_code)]
            pub fn $builder_name (range: Range, $($argname: $argty),*) -> Error {
                Error::new(range, $body)
            }
        }

        error_kind_def! {
            $($tail)*
            @DISPLAY_ARGS $error_kind ($($argname:$argty),+) ($($display_args),+)
        }
    };

    (fn $builder_name:ident ( $($argname:ident : $argty:ty),+ )
     -> $error_kind:ident
     => ($($display_args:expr),+)
     $($tail:tt)*
     ) => {
        error_kind_def! {
            fn $builder_name ( $($argname: $argty),+ ) -> $error_kind => ($($display_args),+) {
                ErrorKind::$error_kind($($argname),*)
            }

            $($tail)*
        }
    };

    ($(@DISPLAY_ARGS $kind:ident ($($argname:ident:$argty:ty),+) ($($display_args:expr),+) )+) => {
        impl Display for ErrorKind {
            fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
                use self::ErrorKind::*;
                match self {
                    $(
                        $kind ( $($argname),+ ) => write!(fmt, $($display_args),+),
                    )+
                }
            }
        }

        #[derive(Debug)]
        pub enum ErrorKind {
            $(
                $kind ( $($argty),+)
            ),+
        }
    };

    () => {};
}

error_kind_def! {
    fn unknown_unary_op(op: Op)        -> UnknownUnaryOp   => ("unknown binary operator {}", op)
    fn unknown_binary_op(op: Op)       -> UnknownBinaryOp  => ("unknown unary operator {}", op)
    fn unknown_fun(name: String)       -> UnknownFun       => ("unknown function `{}`", name)
    fn unknown_ty(name: String)        -> UnknownTy        => ("unknown type `{}`", name)
    fn invalid_assign_lhs(lhs: String) -> InvalidAssignLhs => ("invalid left-hand side of assignment: {}", lhs)
}

#[derive(Debug)]
pub struct Error<'n> {
    range: Range<'n>,
    kind: Context<ErrorKind>,
}

impl<'n> Error<'n> {
    pub fn new(range: Range<'n>, kind: ErrorKind) -> Self {
        Error { range, kind: Context::new(kind) }
    }

    pub fn range(&self) -> Range<'n> {
        self.range
    }

    pub fn kind(&self) -> &ErrorKind {
        self.kind.get_context()
    }
}

impl<'n> Fail for Error<'n>
    where Self: 'static
{
    fn cause(&self) -> Option<&Fail> {
        self.kind.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.kind.backtrace()
    }
}

impl<'n> Display for Error<'n> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.kind, fmt)
    }
}

//pub type Result<T> = ::std::result::Result<T, Error<'_>>;
