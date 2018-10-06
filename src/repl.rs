use compile::CompileState;
use vm::{self, Vm, Value};

const REPL_NAME: &'static str = "<stdin>";

pub struct Repl {
    state: CompileState,
    vm: Vm,
}

impl Repl {
    pub fn new() -> Self {
        let mut state = CompileState::repl();
        state.begin();
        Repl {
            state,
            vm: Vm::new(),
        }
    }

    pub fn execute_line(&mut self, line: &str) -> vm::Result<Option<Value>> {
        self.state.feed_str(REPL_NAME, line)?;
        let compile_unit = self.state.to_compile_unit();
        self.vm.repl_launch(compile_unit)
    }
}
