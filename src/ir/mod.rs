use syntax::tree::{Ast, SyntaxTree, Stmt};
use vm;

mod ty;
mod function;
mod action;
mod symbol;
mod value;
mod compile;

pub use self::ty::*;
pub use self::function::*;
pub use self::action::*;
pub use self::symbol::*;
pub use self::value::*;
pub use self::compile::*;

pub(self) struct BcContext {
    sym_count: usize,
}

impl BcContext {
    pub fn new() -> Self { BcContext { sym_count: 0 } }

    pub fn next_sym(&mut self) -> usize {
        let s = self.sym_count;
        self.sym_count += 1;
        s
    }
}

pub trait Ir<A>
    where A: Ast + Sized,
          Self: Sized,
{
    fn from_syntax(ast: &A) -> Self;
}

#[derive(Debug)]
pub struct IrTree<'n> {
    actions: Vec<Action<'n>>,
    functions: Vec<Function<'n>>,
}

impl<'n> IrTree<'n> {
    pub fn actions(&self) -> &[Action<'n>] {
        &self.actions
    }
}

impl<'n> Ir<SyntaxTree<'n>> for IrTree<'n> {
    fn from_syntax(ast: &SyntaxTree<'n>) -> Self {
        let mut actions = vec![];
        let mut functions = vec![];

        for stmt in ast.stmts.iter() {
            if matches!(stmt, Stmt::Function { name: _, params: _, return_ty: _, body: _ }) {
                functions.push(Function::from_syntax(stmt));
            } else {
                actions.push(Action::from_syntax(stmt));
            }
        }

        IrTree {
            actions,
            functions,
        }
    }
}

