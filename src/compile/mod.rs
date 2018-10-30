use std::path::Path;
use crate::common::ProcessError;

mod alloc;
mod error;
mod function;
mod state;
mod unit;
mod value;
mod scope;
mod transform;
mod thunk;
mod ty;

pub use self::alloc::*;
pub use self::error::*;
pub use self::function::*;
pub use self::state::*;
pub use self::unit::*;
pub (in self) use self::value::*;
pub use self::scope::*;
pub (in self) use self::transform::*;
pub use self::thunk::*;
pub use self::ty::*;

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
