use std::path::Path;
use crate::{
    common::{
        prelude::*,
        FromPath,
        error::*,
    },
    syntax::tree::self,
    ir::{
        Action,
        Fun,
        Ty,
    },
};

#[derive(Debug, Clone)]
pub struct Block {
    pub funs: Vec<Fun>,
    pub tys: Vec<Ty>,
    pub actions: Vec<Action>,
    pub range: Range,
}

impl FromPath for Block {
    type Err = Error;
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let syntax_tree = tree::Block::from_path(path)?;
        Ok(Block::from(syntax_tree))
    }
}

impl From<tree::Block> for Block {
    fn from(tree::Block { funs, tys, stmts, range, }: tree::Block) -> Self {
        Block {
            funs: funs.into_iter().map(From::from).collect(),
            tys: tys.into_iter().map(From::from).collect(),
            actions: stmts.into_iter().map(From::from).collect(),
            range,
        }
    }
}

