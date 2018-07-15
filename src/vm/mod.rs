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
}

impl Vm {
    pub fn from_compile_unit( CompileUnit { name: _name, main_function, mut functions }: CompileUnit) -> Self {
        functions.push(main_function);
        Vm {
            storage: Storage::new(functions, vec![/* TODO: constants */]),
            call_stack: vec![],
            compare_flag: false,
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
                let function_body = function.body.clone();
                let mut local_stack = vec![];
                self.storage
                    .scope_stack
                    .push(Scope::new(function.locals.clone()));
                let mut current_block = function_body;

                for bc in current_block {
                    match bc {
                        Bc::PushSymbolValue(ref symbol) => {
                            let value = self.load(symbol)?;
                            local_stack.push(value);
                        }
                        Bc::PushValue(value) => self.push_stack(value),
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
                            let value = local_stack.pop()
                                .expect("attempted to pop from empty stack in Bc::Pop");
                            self.store(symbol, value)?;
                        }
                        Bc::Store(sym, val) => self.store(&sym, val)?,
                        Bc::Call(ref sym) => self.call(sym)?,
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
                        Bc::Compare(Condition::Truthy(_value)) => {
                        }
                        Bc::Compare(Condition::Compare(_lhs, _op, _rhs)) => {
                        }
                        _ => unimplemented!(),
                    }
                }
                // pop scope stack
                self.storage.scope_stack.pop()
                    .expect("uneven scope stack");
                Ok(())
            }
            Function::Builtin(function) => {
                (function.function)(&mut self.storage)
            }
        }
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
