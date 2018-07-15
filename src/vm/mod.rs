use ir::CompileUnit;

mod value;
mod symbol;
mod scope;
mod function;
mod ty;
mod bc;
mod condition;
mod storage;
mod error;

pub use self::value::*;
pub use self::symbol::*;
pub use self::scope::*;
pub use self::function::*;
pub use self::ty::*;
pub use self::bc::*;
pub use self::condition::*;
pub use self::storage::*;
pub use self::error::*;

pub type StackIndex = usize;

#[derive(Debug)]
pub struct Vm {
    /// The storage of this VM.
    storage: Storage,


    /// A stack of function indices.
    call_stack: Vec<StackIndex>,

    /// Comparison flag.
    ///
    /// This is set to "true" when a `Bc::Condition` instruction evaluates to "true".
    ///
    /// On VM startup, this is set to false.
    compare_flag: bool,

    /// How many blocks we need to jump to the end of.
    block_jump_depth: usize,

    /// The direction to jump when jumping in a block.
    block_jump_top: bool,
}

impl Vm {
    pub fn from_compile_unit( CompileUnit { name: _name, main_function, mut functions }: CompileUnit) -> Self {
        functions.push(main_function);
        Vm {
            storage: Storage::new(functions, vec![/* TODO: constants */]),
            call_stack: vec![],
            compare_flag: false,
            block_jump_depth: 0,
            block_jump_top: false,
        }
    }

    /// Starts this VM a-runnin'.
    pub fn launch(&mut self) -> Result<()> {
        assert!(self.storage.code.len() >= 1);
        self.call_stack.push(self.storage.code.len() - 1);
        self.run_function()
    }

    /// Runs the function on top of the call stack.
    fn run_function(&mut self) -> Result<()> {
        let function = self.current_function()
            .clone();
        match function {
            Function::User(function) => {
                self.storage
                    .scope_stack
                    .push(Scope::new(function.locals.clone()));
                self.run_block(&function.body)?;
                self.storage.scope_stack.pop()
                    .expect("uneven scope stack");
                Ok(())
            }
            Function::Builtin(function) => {
                (function.function)(&mut self.storage)
            }
        }
    }

    /// Runs a block of bytecode.
    ///
    /// This is the primary execution loop.
    fn run_block(&mut self, block: &[Bc]) -> Result<()> {
        let mut pc = 0;
        while pc < block.len() {
            let ref bc = block[pc];
            pc += 1;
            // jump out of the current block if requested
            if self.block_jump_depth > 0 {
                self.block_jump_depth -= 1;
                break;
            } else if self.block_jump_top {
                self.block_jump_top = false;
                pc = 0;
            }
            // TODO : investigate optimization:
            // * Have a "is_conditional()" check at the very start, and jump if condition is not
            //   matched?
            // * this skips a lot of code, not sure if it would be more efficient in assembly
            // * something to test
            match bc {
                Bc::PushSymbolValue(symbol) => {
                    let value = self.load(symbol)?;
                    self.push_stack(value);
                }
                Bc::PushValue(value) => self.push_stack(value.clone()),
                Bc::PopRefAndStore => {
                    let value = self.pop_stack();
                    let sym_value = self.pop_stack();
                    let sym = if let Value::Ref(sym) = sym_value {
                        sym
                    } else { panic!("non-ref sym on top of the stack: {:?}", sym_value) };
                    assert_matches!(sym, Symbol::Variable(_, _));
                    let canary = self.pop_stack();
                    assert_eq!(canary, Value::RefCanary, "ref canary error; got {:?} instead", canary);
                    self.store(&sym, value)?;
                }
                Bc::Pop(ref symbol) => {
                    let value = self.pop_stack();
                    self.store(symbol, value)?;
                }
                Bc::Store(sym, val) => self.store(sym, val.clone())?,
                Bc::Call(sym) => {
                    // store current state
                    let index = sym.index();
                    let start_depth = self.call_stack.len();
                    let block_depth = self.block_jump_depth;
                    let jump_top = self.block_jump_top;

                    self.call_stack.push(index);
                    self.run_function()?;
                    let popped = self.call_stack.pop().expect("empty call stack at end of function call");

                    assert!(self.block_jump_depth == 0, "block jump depth from called function was > 0");
                    self.block_jump_depth = block_depth;
                    self.block_jump_top = jump_top;
                    let end_depth = self.call_stack.len();
                    debug_assert_eq!(index, popped,
                               "mismatched call stack: pushed {}, popped {}. start depth: {}, end depth: {}.",
                               index, popped, start_depth, end_depth);
                    debug_assert_eq!(start_depth, end_depth,
                               "mismatched call stack: pushed {}, popped {}. start depth: {}, end depth: {}.",
                               index, popped, start_depth, end_depth);
                },
                Bc::PopFunctionRefAndCall => {
                    let function_ref = self.pop_stack();
                    let sym = if let Value::FunctionRef(sym) = function_ref {
                        sym
                    } else { panic!("non-function ref on top of the stack: {:?}", function_ref) };
                    let canary = self.pop_stack();
                    assert_eq!(canary, Value::FunctionRefCanary, "function ref canary errror; got {:?} instead", canary);
                    self.call(&sym)?;
                }
                Bc::Compare(Condition::Always) => { self.compare_flag = true; },
                Bc::Compare(Condition::Never) => { self.compare_flag = false; },
                Bc::Compare(Condition::Truthy(_value)) => { unimplemented!("truthy compare bc op") }
                Bc::Compare(Condition::Compare(_lhs, _op, _rhs)) => { unimplemented!("comparison bc op") }
                Bc::Ret(r) => {
                    if let Some(v) = r {
                        self.push_stack(v.clone());
                    }
                    self.call_stack.pop();
                    return Ok(());
                },
                Bc::ConditionBlock(b) => {
                    if !self.compare_flag {
                        continue;
                    }
                    // FIXME: reduce repetition without multiple if-statements
                    let start_depth = self.call_stack.len();
                    self.run_block(b)?;
                    let end_depth = self.call_stack.len();
                    if start_depth != end_depth {
                        return Ok(());
                    }
                }
                Bc::Block(b) => {
                    let start_depth = self.call_stack.len();
                    self.run_block(b)?;
                    let end_depth = self.call_stack.len();
                    // if the start and end call stack depth are not equal, then the inner block
                    // jumped out and returned
                    if start_depth != end_depth {
                        return Ok(());
                    }
                }
                Bc::JumpBlockTop(n) => {
                    self.block_jump_top = true;
                    self.block_jump_depth += n;
                }
                Bc::ExitBlock(n) => {
                    self.block_jump_top = false;
                    self.block_jump_depth += n;
                }
                Bc::BinOpStore(_lhs, _op, _rhs, _sym) => {
                }
                Bc::BinOpPush(_lhs, _op, _rhs) => {
                }
            }
        }
        Ok(())
    }

    /// Pops a value off of the value stack.
    fn pop_stack(&mut self) -> Value {
        self.storage
            .value_stack
            .pop()
            .expect("tried to pop empty stack")
    }

    /// Pushes a value to the value stack.
    fn push_stack(&mut self, value: Value) {
        self.storage
            .value_stack
            .push(value);
    }

    fn call(&mut self, sym: &Symbol) -> Result<()> {
        let idx = match sym {
            Symbol::Function(idx, _) => *idx,
            // TODO : Clean this up
            | Symbol::Variable(_, name) 
            | Symbol::Constant(_, name) => {
                let function_sym = self.load(sym)?;
                if let Value::FunctionRef(Symbol::Function(idx, _)) = &function_sym {
                    *idx
                } else {
                    if matches!(sym, Symbol::Variable(_, _)) {
                        return Err(self.err(format!("local variable ${} is not a function ref", name)));
                    } else {
                        return Err(self.err(format!("named constant {} is not a function ref", name)));
                    }
                }
            }
        };
        self.call_stack.push(idx);
        self.run_function()?;
        self.call_stack.pop();
        Ok(())
    }

    /*
    fn current_scope_mut(&mut self) -> &mut Scope {
        self.storage.current_scope_mut()
    }

    fn current_scope(&self) -> &Scope {
        self.storage.current_scope()
    }
    */

    /// Gets the currently executing function; i.e., the function on top of the call stack.
    fn current_function(&self) -> &Function {
        let function_idx = *self.call_stack.last().unwrap();
        &self.storage.load_function(function_idx)
    }

    fn store(&mut self, symbol: &Symbol, value: Value) -> Result<()> {
        self.storage.store(symbol, value)
    }

    fn load(&self, symbol: &Symbol) -> Result<Value> {
        self.storage.load(symbol)
    }

    fn err(&self, message: String) -> Error {
        message
    }
}
