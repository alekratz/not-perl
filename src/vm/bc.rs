use vm::{
    Ref,
    Value,
    FunSymbol,
    BlockSymbol,
};

/// A condition for when a jump should be taken.
pub enum JumpCond {
    /// This jump should always be taken.
    Always,

    /// This jump should only be taken when the VM's condition flag has been set.
    CondTrue,

    /// This jump should only be taken when the VM's condition flag has *not* been set.
    CondFalse,
}

pub enum Bc {
    /// Pushes the given value to the value stack.
    Push(Value),

    /// Pops the top N values from the stack, discarding them.
    Pop(usize),

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

    /// Calls the given function symbol.
    Call(FunSymbol),

    /// Pops the top value from the current frame and attempts to convert it to a function
    /// reference. Upon success, the function is called. Otherwise, an error is thrown.
    PopCall,

    /// Exits the current function.
    Ret,

    /// Pushes the given value to the top of the stack and exits the current function.
    PushRet(Value),

    /// Jumps to the given instruction address in the current function.
    JumpAbs(usize, JumpCond),

    /// Jumps to the given block symbol in the current function.
    JumpSymbol(BlockSymbol, JumpCond),

    /// Pops the top value off of the stack and checks if it is true or not.
    PopTest,
}
