use vm::{
    Ref,
    Value,
    FunSymbol,
    BlockSymbol,
};

pub enum Bc {
    /// Pushes the given value to the value stack.
    Push(Value),

    /// Pops the top value of the value from the stack, discarding it.
    Pop,

    /// Pops the top value of the value from the stack and stores it into the given reference.
    PopStore(Ref),

    /// Stores the given value into the given reference.
    Store(Ref, Value),

    /// Dereferences the given ref and pushes the value to the stack.
    DerefPush(Ref),

    /// Pops a reference off of the stack, and stores the given value into it.
    ///
    /// If the top value of the stack is not a reference, or the reference value is incompatible
    /// with the given value, an error is raised.
    PopDerefStore(Value),

    /// Pushes a new stack frame.
    PushFrame,

    /// Pops a stack frame.
    PopFrame,

    /// Calls the given function symbol.
    Call(FunSymbol),

    /// Pops the top value from the current frame and attempts to convert it to a function
    /// reference. Upon success, the function is called. Otherwise, an error is thrown.
    PopCall,

    /// Pushes the given value to the top of the stack and exits the current function.
    Ret(Value),

    /// Jumps forwards or backwards by the given number of instructions, starting at the first
    /// instruction after this one.
    JmpRelative(isize),

    /// Jumps to the given instruction address in the current function.
    JmpAbsolute(usize),

    /// Jumps to the given block symbol in the current block.
    JmpSymbol(BlockSymbol),
}
