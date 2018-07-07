use vm::{Value, Symbol};

/// A label in code for the VM.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Label(pub usize);

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

    /// Compares two values, setting the comparison flag.
    Cmp(Value, Value),

    /// Jumps to a label, if the comparison flag is 0.
    JmpEq(Label),

    /// Jumps to a label, if the comparison flag is not 0.
    JmpNeq(Label),
    
    /// Jumps to a label, if the comparison flag is <0.
    JmpLt(Label),

    /// Jumps to a label, if the comparison flag is <=0.
    JmpLe(Label),

    /// Jumps to a label, if the comparison flag is >0.
    JmpGt(Label),

    /// Jumps to a label, if the comparison flag is >=0.
    JmpGe(Label),
}

