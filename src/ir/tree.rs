use crate::ir::{
    Action,
    Fun,
    Ty,
};

#[derive(Debug)]
pub struct Tree {
    pub actions: Vec<Action>,
    pub funs: Vec<Fun>,
    pub tys: Vec<Ty>,
}
