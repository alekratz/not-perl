use crate::{
    common::scope::ReadOnlyScope,
    compile,
    vm::{
        Fun,
        Storage,
        Symbolic,
        Symbol,
        Result,
        Error,
    },
};

pub struct State {
    funs: Vec<Fun>,
    storage: Storage,
    cond_flag: bool,
}

impl State {
    pub fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

impl From<compile::State> for State {
    fn from(compile::State { var_scope, fun_scope, ty_scope, label_scope }: compile::State) -> Self {
        let mut funs: Vec<_> = ReadOnlyScope::from(fun_scope)
            .into_all()
            .into_iter()
            .map(|fun| match fun {
                | compile::Fun::Vm(fun)
                | compile::Fun::Op(_, fun) => fun,
                _ => panic!("uncompiled function: {}", fun.name())
            })
            .collect();
        funs.sort_unstable_by(|a, b| a.symbol().cmp(&b.symbol()));

        for (i, fun) in funs.iter().enumerate() {
            assert_eq!(i, fun.symbol().index(), "function symbol and index mismatch: {} (symbol: {:?}, index: {})",
                fun.name(), fun.symbol(), i);
        }

        State {
            funs,
            storage: Storage::new(),
            cond_flag: false,
        }
    }
}
