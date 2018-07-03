use syntax::tree::{Ast, SyntaxTree};

mod action;
mod cycle;
mod symbol;
mod value;

pub use self::action::*;
pub use self::cycle::*;
pub use self::symbol::*;
pub use self::value::*;

pub type Error = String;
//pub type Result<T> = ::std::result::Result<T, String>;

pub trait Ir<A>
    where A: Ast + Sized,
          Self: Sized,
{
    fn from_syntax(ast: &A) -> Self;
}

#[derive(Debug)]
pub struct IrTree {
    actions: Vec<Action>,
}

impl<'n> Ir<SyntaxTree<'n>> for IrTree {
    fn from_syntax(ast: &SyntaxTree<'n>) -> Self {
        let mut actions = vec![];

        for stmt in ast.stmts.iter() {
            actions.push(Action::from_syntax(stmt));
        }

        IrTree {
            actions,
        }
    }
}

