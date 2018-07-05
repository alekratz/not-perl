use vm::{Value, Symbol};

#[derive(Debug, Clone)]
pub enum Bc {
    /// Pushes a value read from a symbol onto the stack.
    PushSymbolValue(Symbol),

    /// Pushes a value onto the stack.
    PushValue(Value),

    /// Pops a value off the top of the stack, followed by a(n expected) symbol ref, and finally
    /// the symbol ref canary, storing the value in the symbol ref.
    ///
    /// If the penultimate item popped off the stack is not a symbol ref, or if the canary is not
    /// present, a runtime VM error is thrown.
    PopRefAndStore,

    /// Pops a value from the stack into this symbol.
    Pop(Symbol),

    /// Stores a value into a variable slot.
    Store(Symbol, Value),

    /// Calls a function in the given slot with the given arguments.
    Call(Symbol),

    /// Pops off a function ref, and calls it.
    PopFunctionRefAndCall,
}
