use crate::vm::{
    Ref,
    Value,
    FunSymbol,
    BlockSymbol,
    TySymbol,
};

/// A condition for when a jump should be taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpCond {
    /// This jump should always be taken.
    Always,

    /// This jump should only be taken when the VM's condition flag has been set.
    CondTrue,

    /// This jump should only be taken when the VM's condition flag has *not* been set.
    CondFalse,
}

#[derive(Debug, Clone)]
pub enum Bc {
    /// Pushes the given value to the value stack.
    Push(Value),

    // Pops the top N values from the stack, discarding them.
    //Pop(usize),

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

    // Jumps to the given instruction address in the current function, checking the jump condition
    // against the VM's condition flag.
    //JumpAbs(usize, JumpCond),

    /// Jumps to the given block symbol in the current function, checking the jump condition
    /// against the VM's condition flag.
    JumpSymbol(BlockSymbol, JumpCond),

    /// Pops the top value off of the stack and checks if it is true or not, setting the VM
    /// condition flag appropriately.
    PopTest,

    // Checks the given value against a given type predicate, setting the VM condition flag
    // appropriately.
    //Check(Value, TySymbol),

    // Peeks at the top value of the stack, and checks it against the given type predicate,
    // setting the VM condition flag appropriately.
    //PeekCheck(TySymbol),
}
