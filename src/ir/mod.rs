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
    actions: Vec<Action>,
    functions: Vec<Fun>,
    user_types: Vec<UserTy>,
    range: Range,
}

impl IrTree {
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn functions(&self) -> &[Fun] {
        &self.functions
    }

    pub fn user_types(&self) -> &[UserTy] {
        &self.user_types
    }

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

        IrTree {
            actions,
            functions,
            user_types,
            range: ast.range(),
        }
    }
}

impl_ranged!(IrTree::range);
