use std::path::Path;
use crate::common::{
    prelude::*,
    ProcessError,
};
use crate::compile::{
    Error,
    Fun,
    FunStub,
    State,
    RootBlock,
    transform::*,
};
use crate::ir;
use crate::vm::{self, Symbolic};

pub struct Compile {
    state: State,
}

impl Compile {
    pub fn new() -> Self {
        let mut compile = Compile { state: State::new() };
        compile.state.insert_builtins();
        compile.state.push_empty_scope();
        compile
    }

    pub fn compile_from_path(&mut self, path: impl AsRef<Path>) -> Result<(), ProcessError> {
        self.state.update_from_path(path)?;

        Ok(())
    }
}

/// A function stub gatherer.
///
/// This type implements `TryTransform` over an input of IR function definitions, filling out the
/// state it holds.
struct GatherFunStubs<'s>(&'s mut State);

impl<'r, 's> TryTransform<&'r [ir::Fun]> for GatherFunStubs<'s> {
    type Out = ();

    fn try_transform(self, funs: &'r [ir::Fun]) -> Result<(), Error> {
        for fun in funs {
            let name = fun.name();
            if let Some(f) = self.0.fun_scope.get_local_by_name(name) {
                return Err(Error::duplicate_fun(fun.range(), f.range(), name.to_string()));
            }
            let symbol = self.0.fun_scope.reserve_symbol();
            let stub = FunStub::from_ir_function(symbol, fun);
            self.0.fun_scope.insert(Fun::Stub(stub));
        }
        Ok(())
    }
}

/// Compiles all functions.
pub struct CompileFuns<'s>(pub (in super) &'s mut State);

impl<'s> TryTransform<Vec<ir::Fun>> for CompileFuns<'s> {
    type Out = ();
    fn try_transform(self, funs: Vec<ir::Fun>) -> Result<(), Error> {
        // Get all function stubs
        GatherFunStubs(self.0)
            .try_transform(&funs)?;
        for fun in funs.into_iter() {
            let user_fun = CompileFun(self.0)
                .try_transform(fun)?;
            let vm_fun = vm::Fun::User(user_fun);
            self.0.fun_scope.replace(Fun::Vm(vm_fun));
        }
        Ok(())
    }
}

/// Compiles a single function from its IR to the final VM function.
pub struct CompileFun<'s> (pub (in super) &'s mut State);

impl<'s> TryTransform<ir::Fun> for CompileFun<'s> {
    type Out = vm::UserFun;
    fn try_transform(self, fun: ir::Fun) -> Result<vm::UserFun, Error> {
        let ir::Fun {
            symbol,
            params,
            return_ty: _,
            body,
            inner_functions,
            range,
        } = fun;
        self.0.push_empty_scope();

        let name = if let ir::Symbol::Fun(name) = symbol {
            name
        } else {
            unreachable!();
        };

        // Compile inner functions first
        CompileFuns(self.0).try_transform(inner_functions)?;

        // TODO : Pop function parameters and check their predicates where applicable

        // Compile function body
        let body = RootBlock(self.0)
            .try_transform_block(body)?
            .collapse(self.0);
        self.0.pop_scope();

        let symbol = self.0.fun_scope.get_local_by_name_and_params(&name, params.len())
            .expect(&format!("unregistered function encountered after compilation: {}", name))
            .symbol();
        Ok(vm::UserFun::new(symbol, name, params.len(), body, range))
    }
}
