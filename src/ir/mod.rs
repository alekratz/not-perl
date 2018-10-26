use crate::syntax::tree::{Ast, SyntaxTree, Stmt};

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

pub trait Ir<A>
    where A: Ast + Sized,
          Self: Sized,
{
    fn from_syntax(ast: &A) -> Self;
}

#[derive(Debug)]
pub struct IrTree {
    actions: Vec<Action>,
    functions: Vec<Fun>,
    user_types: Vec<UserTy>,
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
        }
    }
}

