use syntax::token::Op;
use vm::{Value, Symbol, Condition, CompareOp};

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

    /// Performs a comparison.
    Compare(Condition),

    /// Exit the current function, optionally pushing the returned value on the stack.
    Ret(Option<Value>),

    /// A block of bytecode to execute
    Block(Vec<Bc>),

    /// A block of bytecode that is only executed when the comparison flag is set.
    ConditionBlock(Vec<Bc>),

    /// Jumps to the top of the Nth block above this one.
    ///
    /// If set to 0, this will jump to the top of the current block.
    JumpBlockTop(usize),

    /// Prematurely exits the Nth block above this one.
    ///
    /// If set to 0, this will exit the current block.
    ExitBlock(usize),

    /// Performs a binary operation with the given op and lhs/rhs, and stores it into the given
    /// symbol.
    BinOpStore(Value, Op, Value, Symbol),

    /// Performs a binary operation with the given op and lhs/rhs, and pushes the result.
    ///
    /// The top stack item follows the same pattern as all others: top must be a value ref,
    /// followed by a ref canary. If either are not present, a VM runtime error is thrown.
    BinOpPush(Value, Op, Value),
}

