use vm::{
    Value,
    RegSymbol,
    FunSymbol,
};

pub enum Bc {
    /// Pushes the given value to the value stack.
    PushValue(Value),

    /// Pops the top value of the value from the stack.
    PopValue,

    /// Pops the top value of the value from the stack and stores it in the given register.
    PopValueInto(RegSymbol),

    /// Stores the given value into the given register.
    Store(RegSymbol, Value),

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
}
