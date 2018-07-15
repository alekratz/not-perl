use vm::{Scope, Value};

/// The state of the VM, which can be passed around if necessary.
#[derive(Debug)]
pub struct State {
    /// A stack of local variables for each scope we are inside.
    ///
    /// This usually will have a height of 1.
    pub scope_stack: Vec<Scope>,

    /// Main program stack.
    pub value_stack: Vec<Value>,

}

impl State {
    pub fn new() -> Self {
        State {
            scope_stack: vec![],
            value_stack: vec![],
        }
    }
}
