use crate::{
    vm::{
        Pool, VmString, Symbol, Symbolic,
        mem::HeapRef,
    },
};

#[derive(Debug, Clone)]
pub enum Value {
    Str(VmString),
    Int(i64),
    Float(f64),
    HeapRef(HeapRef),
}

/// A pool of string constants used by the VM.
pub struct StringPool {
    strings: Vec<VmString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringSymbol(usize);

impl Symbol for StringSymbol {
    fn index(&self) -> usize { self.0 }
}
