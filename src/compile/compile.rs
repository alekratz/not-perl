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
    TyStub,
    Ty,
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

    /// Compile a single IR tree, updating this state.
    pub fn update<'r>(&mut self, ir_tree: ir::IrTree) -> Result<(), Error> {
        let ir::IrTree {
            actions: _actions,
            functions,
            user_types,
            range: _range,
        } = ir_tree;

        // Gather and compile all types
        GatherCompile(&mut self.state).try_transform((user_types, functions))?;

        // Create main function

        Ok(())
    }

    pub fn update_from_path(&mut self, path: impl AsRef<Path>) -> Result<(), ProcessError> {
        let ir_tree = ir::IrTree::from_path(path)?;
        self.update(ir_tree)
            .map_err(|e| e.into())
    }
}

struct GatherCompile<'s>(&'s mut State);

impl<'s> TryTransform<(Vec<ir::UserTy>, Vec<ir::Fun>)> for GatherCompile<'s> {
    type Out = ();

    fn try_transform(self, (tys, funs): (Vec<ir::UserTy>, Vec<ir::Fun>)) -> Result<(), Error> {
        // Gather all types
        GatherTyStubs(self.0).try_transform(&tys)
        // Gather all functions
            .and_then(|_| GatherFunStubs(self.0).try_transform(&funs))
        // Compile all types
            .and_then(|_| CompileTys(self.0).try_transform(tys))
        // Compile all functions
            .and_then(|_| CompileFuns(self.0).try_transform(funs))
    }
}

/// Compiles all user types.
struct CompileTys<'s>(&'s mut State);

impl<'s> TryTransform<Vec<ir::UserTy>> for CompileTys<'s> {
    type Out = ();
    fn try_transform(mut self, tys: Vec<ir::UserTy>) -> Result<(), Error> {
        for ty in tys.into_iter() {
            let user_ty = self.try_transform_mut(ty)?;
            let vm_ty = vm::Ty::User(user_ty);
            self.0.ty_scope.replace(Ty::Vm(vm_ty));
        }
        Ok(())
    }
}

impl<'s> TryTransformMut<ir::UserTy> for CompileTys<'s> {
    type Out = vm::UserTy;
    fn try_transform_mut(&mut self, ty: ir::UserTy) -> Result<vm::UserTy, Error> {
        let ir::UserTy {
            name,
            parents, // TODO: implement type parent behavior
            functions,
            range,
        } = ty;
        self.0.push_empty_scope();

        GatherFunStubs(self.0).try_transform(&functions)
            .and_then(|_| CompileFuns(self.0).try_transform(functions))?;
        // register this type
        self.0.pop_scope();
        let symbol = self.0.ty_scope.get_local_by_name(&name)
            .expect(&format!("unregistered type encountered after compilation: {}", name))
            .symbol();
        Ok(vm::UserTy { name, symbol, range })

    }
}

/// A type stub gatherer.
struct GatherTyStubs<'s>(&'s mut State);

impl<'r, 's> TryTransform<&'r [ir::UserTy]> for GatherTyStubs<'s> {
    type Out = ();

    fn try_transform(self, tys: &'r [ir::UserTy]) -> Result<(), Error> {
        for ty in tys {
            let name = &ty.name;
            if let Some(t) = self.0.ty_scope.get_local_by_name(name) {
                return Err(Error::duplicate_ty(ty.range(), t.range(), name.to_string()));
            }
            let symbol = self.0.ty_scope.reserve_symbol();
            let range = ty.range();
            let stub = TyStub {
                name: name.clone(),
                symbol,
                range,
            };
            self.0.ty_scope.insert(Ty::Stub(stub));
        }
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
struct CompileFuns<'s>(&'s mut State);

impl<'s> TryTransform<Vec<ir::Fun>> for CompileFuns<'s> {
    type Out = ();
    fn try_transform(mut self, funs: Vec<ir::Fun>) -> Result<(), Error> {
        for fun in funs.into_iter() {
            let user_fun = self.try_transform_mut(fun)?;
            let vm_fun = vm::Fun::User(user_fun);
            self.0.fun_scope.replace(Fun::Vm(vm_fun));
        }
        Ok(())
    }
}

impl<'s> TryTransformMut<ir::Fun> for CompileFuns<'s> {
    type Out = vm::UserFun;
    fn try_transform_mut(&mut self, fun: ir::Fun) -> Result<vm::UserFun, Error> {
        let ir::Fun {
            symbol,
            params,
            return_ty, // TODO: implement return type checking
            body,
            inner_types,
            inner_functions,
            range,
        } = fun;
        self.0.push_empty_scope();

        let name = if let ir::Symbol::Fun(name) = symbol {
            name
        } else {
            unreachable!();
        };

        GatherCompile(self.0).try_transform((inner_types, inner_functions))?;

        // Compile function body
        let thunk_list = RootBlock(self.0)
            .try_transform_block(body)?;

        // TODO : Pop function parameters and check their predicates where applicable
        // TODO : Or, let the VM handle it? Don't worry about inserting instructions here, maybe?

        let body = thunk_list.collapse(self.0);
        self.0.pop_scope();

        let symbol = self.0.fun_scope.get_local_by_name_and_params(&name, params.len())
            .expect(&format!("unregistered function encountered after compilation: {}", name))
            .symbol();
        // TODO : Check return value
        Ok(vm::UserFun::new(symbol, name, params.len(), body, range))
    }
}
