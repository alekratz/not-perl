use std::path::Path;
use crate::common::{
    ProcessError,
    pos::{Ranged, Range},
};
use crate::syntax::{
    self,
    tree::{Ast, SyntaxTree, Stmt}
};
use crate::util;

mod ty;
mod function;
mod action;
mod symbol;
mod value;

pub use self::ty::*;
pub use self::function::*;
pub use self::action::*;
pub use self::symbol::*;
pub use self::value::*;

pub trait Ir<A>: Sized + Ranged
    where A: Ast + Sized,
{
    fn from_syntax(ast: &A) -> Self;
}

#[derive(Debug)]
pub struct IrTree {
    pub body: Fun,
    pub range: Range,
}

impl IrTree {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ProcessError> {
        let path = path.as_ref();
        let contents = match util::read_file(path) {
            Ok(c) => c,
            Err(e) => return Err(ProcessError::Io(e)),
        };

        let lexer = syntax::Lexer::new(path.display(), &contents);
        let parse_tree = syntax::Parser::from_lexer(lexer)
            .into_parse_tree()?;

        Ok(IrTree::from_syntax(&parse_tree))
    }
}

impl Ir<SyntaxTree> for IrTree {
    fn from_syntax(ast: &SyntaxTree) -> Self {
        let mut actions = vec![];
        let mut functions = vec![];
        let mut user_types = vec![];

        for stmt in ast.stmts.iter() {
            match stmt {
                Stmt::Fun(function) => functions.push(Fun::from_syntax(function)),
                Stmt::UserTy(user_ty) => user_types.push(UserTy::from_syntax(user_ty)),
                _ => actions.push(Action::from_syntax(stmt)),
            }
        }
        let main = Fun {
            symbol: Symbol::Fun(crate::common::strings::MAIN_FUN_NAME.to_string()),
            params: Vec::new(),
            return_ty: TyExpr::None,
            body: actions,
            inner_types: user_types,
            inner_functions: functions,
            range: ast.range(),
        };

        IrTree {
            body: main,
            range: ast.range(),
        }
    }
}

impl Ranged for IrTree {
    fn range(&self) -> Range {
        self.body.range()
    }
}

