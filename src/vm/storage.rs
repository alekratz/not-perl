use crate::vm::Value;

/// Storage for the VM.
///
/// This includes the heap, the stack, and all functions.
pub struct Storage {
    stack: Vec<Value>,
    // TODO : vm heap
    // TODO : move compile scope to common and let the VM use it as well
}

impl Storage {
    pub fn push_stack(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop_stack(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn peek_stack(&mut self) -> Option<&Value> {
        self.stack.last()
    }
}
