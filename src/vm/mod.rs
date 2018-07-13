mod value;
mod symbol;
mod scope;
mod function;
mod ty;
mod bc;
mod condition;

pub use self::value::*;
pub use self::symbol::*;
pub use self::scope::*;
pub use self::function::*;
pub use self::ty::*;
pub use self::bc::*;
pub use self::condition::*;

pub type Error = String;
pub type Result<T> = ::std::result::Result<T, Error>;

pub type StackIndex = usize;

#[derive(Debug)]
pub struct Vm {
    /// A stack of local variables for each scope we are inside.
    ///
    /// This usually will have a height of 1.
    scope_stack: Vec<Scope>,

    /// Stack of function call stacks.
    stack_stack: Vec<Vec<Value>>,

    /// An array of functions, indexed by the function "number".
    code: Vec<Function>,

    /// A stack of function indices.
    call_stack: Vec<StackIndex>,

    /// A list of read-only constants.
    constants: Vec<Value>,

    /// Comparison flag.
    ///
    /// This is the result of the most recent Bc::Cmp comparison.
    cmp_flag: isize,

    /// A stack of program counter values, per call frame.
    program_counter: Vec<usize>,
}

impl Vm {

    /// Starts this VM a-runnin'.
    pub fn launch(&mut self) -> Result<()> {
        // first code index is the global scope
        assert!(self.code.len() >= 1);
        self.call_stack.push(0);
        Ok(())
    }

    /// Runs the function on top of the call stack.
    fn run_function(&mut self) -> Result<()> {
        let function = self.current_function()
            .clone();
        if let Function::User(function) = function {
            let function_body = function.body.clone();
            let mut local_stack = vec![];
            let mut local_scope = Scope::new(function.locals.clone());

            for bc in function_body {
                match bc {
                    Bc::PushSymbolValue(ref symbol) => {
                        let value = self.load(symbol, &local_scope)?;
                        local_stack.push(value);
                    }
                    Bc::PushValue(ref value) => local_stack.push(value.clone()),
                    Bc::PopRefAndStore => {
                        unimplemented!("Bc::PopRefAndStore")
                    }
                    Bc::Pop(ref symbol) => {
                        let value = local_stack.pop()
                            .expect("attempted to pop from empty stack in Bc::Pop");
                        local_scope.set(symbol, value);
                    }
                    Bc::Store(ref _sym, ref _val) => {
                        unimplemented!("Bc::store")
                    }
                    Bc::Call(ref _sym) => {
                        unimplemented!("Bc::call")
                    }
                    Bc::PopFunctionRefAndCall => {
                        unimplemented!("Bc::PopFunctionRefAndCall")
                    }
                    _ => unimplemented!(),
                }
            }
            Ok(())
        } else {
            unimplemented!("VM: Builtin function call")
        }
    }

    /// Gets the currently executing function; i.e., the function on top of the call stack.
    fn current_function(&self) -> &Function {
        let function_idx = *self.call_stack.last().unwrap();
        &self.code[function_idx]
    }

    fn load(&self, symbol: &Symbol, local_scope: &Scope) -> Result<Value> {
        match symbol {
            Symbol::Variable(_, ref name) => {
                let follow_ref = |value| {
                    // weird construction due to &value borrow
                    // NLL should fix this
                    {
                        if let Value::Ref(ref sym) = &value {
                            // follow references
                            return self.load(sym, local_scope);
                        }
                    }
                    Ok(value)
                };

                if let Some(value) = local_scope.try_get(symbol) {
                    follow_ref(value)
                } else {
                    for scope in &self.scope_stack {
                        if let Some(value) = scope.try_get(symbol) {
                            return follow_ref(value);
                        }
                    }
                    Err(self.err(format!("could not resolve symbol: {}", name)))
                }
            }
            Symbol::Constant(idx, _) => {
                Ok(self.constants[*idx].clone())
            }
            Symbol::Function(idx, _) => {
                let function = &self.code[*idx];
                Ok(Value::FunctionRef(function.symbol().clone()))
            }
        }
    }

    fn err(&self, message: String) -> Error {
        message
    }
}
