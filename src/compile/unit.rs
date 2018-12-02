use crate::{
    compile::State,
    vm::Fun,
};

/// A final or in-progress compile-unit.
pub struct Unit {
    main_function: Fun,
    functions: Vec<Fun>,
}

impl Unit {
    /// Absorbs the given state into this compilation unit.
    ///
    /// The main function will be overwritten and discarded.
    pub fn update(&mut self, state: State) {
        
    }
}
