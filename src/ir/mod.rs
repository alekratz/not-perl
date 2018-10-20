use syntax::tree::{Ast, SyntaxTree, Stmt};

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
pub struct IrTree<'n> {
    actions: Vec<Action<'n>>,
    functions: Vec<Fun<'n>>,
    user_types: Vec<UserTy<'n>>,
}

impl<'n> IrTree<'n> {
    pub fn actions(&self) -> &[Action<'n>] {
        &self.actions
    }

    pub fn functions(&self) -> &[Fun<'n>] {
        &self.functions
    }

    pub fn user_types(&self) -> &[UserTy<'n>] {
        &self.user_types
    }
}

impl<'n> Ir<SyntaxTree<'n>> for IrTree<'n> {
    fn from_syntax(ast: &SyntaxTree<'n>) -> Self {
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

