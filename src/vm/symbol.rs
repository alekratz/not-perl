/// A symbol that is used to index something at runtime in the VM.
pub trait Symbol {
    fn index(&self) -> usize;
}

/// A thing that has a symbol and also a name.
pub trait Symbolic {
    type Symbol: Symbol;

    fn symbol(&self) -> &Self::Symbol;
}
