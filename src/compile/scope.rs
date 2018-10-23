use std::{
    fmt::Debug,
    collections::{
        BTreeMap,
        BTreeSet,
    },
    ops::{Deref, DerefMut},
};
use common::lang::Op;
use compile::{
    Alloc,
    Fun,
    Var,
    Ty,
    RegSymbolAlloc,
    FunSymbolAlloc,
    TySymbolAlloc,
    BlockSymbolAlloc,
};
use vm::{self, Symbol, Symbolic};

#[derive(Debug)]
pub struct Scope<T, A>
    where T: Symbolic,
          A: Alloc<T::Symbol>,
{
    scope_stack: Vec<Vec<T::Symbol>>,
    all: BTreeMap<T::Symbol, T>,
    symbol_alloc: A,
}

impl<T, A> Scope<T, A>
    where T: Symbolic + Debug,
          T::Symbol: Debug,
          A: Alloc<T::Symbol> + Default,
{
    /// Reserves a symbol in this scope.
    pub fn reserve_symbol(&mut self) -> T::Symbol {
        self.symbol_alloc.reserve()
    }

    /// Pushes a stack layer to the scope.
    fn push_scope(&mut self, layer: Vec<T>) {
        self.symbol_alloc.on_push_scope();
        self.scope_stack.push(vec![]);
        for value in layer.into_iter() {
            self.insert(value);
        }
    }

    /// Pushes an empty stack layer to the scope.
    ///
    /// This is the equivalent of calling `push_scope(Vec::new())`.
    pub fn push_empty_scope(&mut self) {
        self.push_scope(vec![]);
    }

    /// Pops the top scope layer as a list of symbols.
    ///
    /// Since the actual compile values are still owned by this scope, symbols that point to the
    /// values are popped instead.
    pub fn pop_scope(&mut self) -> Vec<T::Symbol> {
        self.symbol_alloc.on_pop_scope();
        self.scope_stack.pop()
            .expect("attempted to pop depthless scope")
    }

    /// Inserts the given value into this scope.
    pub fn insert(&mut self, value: T) {
        let sym = value.symbol();
        assert!(!self.all.contains_key(&sym), "Symbol already defined in this scope: {:?}", sym);
        self.all.insert(sym, value);
        let top = self.scope_stack
            .last_mut()
            .expect("attempted to push value to depthless scope");
        top.push(sym);
    }

    /// Gets the first scope value that matches this predicate, traversing only the most local
    /// scope.
    pub fn get_local_by<P>(&self, pred: P) -> Option<&T>
        where for <'r> P: Fn(&'r &T) -> bool
    {
        if self.scope_stack.is_empty() {
            return None;
        }
        self.scope_stack
            .last()
            .unwrap()
            .iter()
            .map(|sym| self.all.get(&sym).unwrap())
            .filter(pred)
            .next()
    }

    /// Gets the first scope value that matches the given name, traversing only the most local
    /// scope.
    pub fn get_local_by_name(&self, name: &str) -> Option<&T> {
        self.get_local_by(|v| v.name() == name)
    }

    /// Gets the first scope value that matches the given name, traversing only the most local
    /// scope.
    pub fn get_local_by_symbol(&self, symbol: T::Symbol) -> Option<&T> {
        self.get_local_by(|v| v.symbol() == symbol)
    }

    /// Gets the first scope value that matches this predicate, using the same traversal order as
    /// `Scope::iter`.
    pub fn get_by<P>(&self, pred: P) -> Option<&T>
        where for <'r> P: Fn(&'r &T) -> bool
    {
        self.iter()
            .filter(pred)
            .next()
    }

    /// Gets an item by its name, using the same traversal order as `Scope::iter`.
    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.get_by(|t| t.name() == name)
    }

    /// Gets an item by its symbol, using the same traversal order as `Scope::iter`.
    pub fn get_by_symbol(&self, symbol: T::Symbol) -> Option<&T> {
        self.get_by(|t| t.symbol() == symbol)
    }

    /// Iterates over values that are visible in the current scope, starting at the values defined
    /// most locally to the values defined most globally (i.e., in reverse).
    pub fn iter(&self) -> impl Iterator<Item=&T> {
        let iter = self.scope_stack
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().map(|sym| *sym))
            .map(move |sym| self.all.get(&sym).unwrap());
        Box::new(iter)
    }

    /// Iterates over all values inserted to this scope.
    pub fn iter_all(&self) -> impl Iterator<Item=&T> {
        let iter = self.all
            .values();
        Box::new(iter)
    }

    /// Consumes this scope, yielding all registered values over the lifetime of this scope.
    pub fn into_all(self) -> Vec<T> {
        self.all
            .into_iter()
            .map(|(_, v)| v)
            .collect()
    }
}

impl<T, A> Default for Scope<T, A>
    where T: Symbolic,
          A: Alloc<T::Symbol> + Default,
{
    fn default() -> Self {
        Scope {
            scope_stack: Vec::new(),
            all: BTreeMap::new(),
            symbol_alloc: A::default(),
        }
    }
}

pub type TyScope = Scope<Ty, TySymbolAlloc>;
pub type LabelScope = Scope<vm::Label, BlockSymbolAlloc>;

#[derive(Debug)]
pub struct VarScope {
    scope: Scope<Var, RegSymbolAlloc>,

    /// A stack of all unused anonymous variables.
    unused_anon: Vec<BTreeSet<vm::RegSymbol>>,
}

impl VarScope {
    /// Gets a symbol to a variable with the given name, or inserts it if it doesn't exist.
    ///
    /// This will clone the given name if the inserted variable does not exist.
    pub fn get_or_insert(&mut self, name: &str) -> vm::RegSymbol {
        if let Some(var) = self.scope.get_by_name(name) {
            return var.symbol();
        }

        let sym = self.scope.reserve_symbol();
        self.insert(Var::new(name.to_string(), sym));
        sym
    }

    /// Inserts an anonymous variable.
    pub fn insert_anonymous_var(&mut self) -> vm::RegSymbol {
        self.ensure_unused_anon_size();

        let has_unused = self.unused_anon
            .last()
            .map(|u| !u.is_empty())
            .expect("attempted to reserve anonymous variable from depthless scope");
        if has_unused {
            let active = self.unused_anon
                .last_mut()
                .expect("attempted to free anonymous variable from depthless scope");
            let sym = *active
                .iter()
                .min()
                .unwrap();
            active.remove(&sym);
            sym
        } else {
            let sym = self.scope.reserve_symbol();
            let var = Var::new(format!("anonvalue#{:x}", sym.index()), sym);
            self.insert(var);
            sym
        }
    }

    /// Frees the given anonymous variable.
    ///
    /// Note that this does not check if this is actually an anonymous variable being freed. It is
    /// up to the programmer to determine this themselves.
    pub fn free_anonymous_var(&mut self, sym: vm::RegSymbol) {
        self.ensure_unused_anon_size();

        let active = self.unused_anon
            .last_mut()
            .expect("attempted to free anonymous variable from depthless scope");
        assert!(!active.contains(&sym), "attempted to double-free an anonymous variable");
        active.insert(sym);
    }

    /// Pushes or pops an appropriate number of values to the the `unused_anon` stack so that it
    /// matches the current scope stack size.
    fn ensure_unused_anon_size(&mut self) {
        let size_diff: isize = self.unused_anon.len() as isize - self.scope.scope_stack.len() as isize;
        if size_diff < 0 {
            self.unused_anon.append(&mut vec!(BTreeSet::new(); (-size_diff) as usize));
        } else if size_diff > 0 {
            self.unused_anon.truncate(size_diff as usize);
        }
    }
}

impl From<Scope<Var, RegSymbolAlloc>> for VarScope {
    fn from(scope: Scope<Var, RegSymbolAlloc>) -> Self {
        let depth = scope.scope_stack.len();
        VarScope {
            scope, unused_anon: vec!(BTreeSet::new(); depth)
        }
    }
}

impl From<VarScope> for Scope<Var, RegSymbolAlloc> {
    fn from(scope: VarScope) -> Self { scope.scope }
}

impl Deref for VarScope {
    type Target = Scope<Var, RegSymbolAlloc>;

    fn deref(&self) -> &Self::Target { &self.scope }
}

impl DerefMut for VarScope {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.scope }
}

impl Default for VarScope {
    fn default() -> Self {
        VarScope {
            scope: Scope::default(),
            unused_anon: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunScope {
    scope: Scope<Fun, FunSymbolAlloc>,
}

impl FunScope {
    /// Inserts builtin functions to this scope.
    ///
    /// # Preconditions
    /// A scope layer must exist before builtins are inserted.
    pub fn insert_builtin_functions(&mut self) {
        for builtin in vm::builtin_functions.iter() {
            let sym = self.reserve_symbol();
            self.insert(Fun::Vm(vm::Fun::Builtin(builtin, sym)));
        }
    }

    /// Inserts builtin functions to this scope.
    ///
    /// # Preconditions
    /// A scope layer must exist before builtins are inserted.
    pub fn insert_builtin_ops(&mut self) {
        for vm::BuiltinOp(op, builtin) in vm::builtin_ops.iter() {
            let sym = self.reserve_symbol();
            self.insert(Fun::Op(op.clone(), vm::Fun::Builtin(builtin, sym)));
        }
    }

    /// Replaces the first function to match this predicate.
    ///
    /// # Preconditions
    /// The function to replace must be registered. It does not necessarily need to be visible in
    /// the current scope.
    pub fn replace(&mut self, value: Fun) -> Fun {
        assert!(self.all.contains_key(&value.symbol()),
            format!("tried to replace unregistered function, symbol: {:?} name: {:?}", value.symbol(), value.name()));
        self.all.insert(value.symbol(), value)
            .unwrap()
    }

    /// Gets a function based on its name and parameter count.
    pub fn get_by_name_and_params(&self, name: &str, params: usize) -> Option<&Fun> {
        self.get_by(|f| f.name() == name && f.params() == params)
    }

    /// Gets a builtin function by its name.
    pub fn get_builtin(&self, name: &str) -> Option<&Fun> {
        self.get_by(|f| matches!(f, Fun::Vm(vm::Fun::Builtin(_, _))) && f.name() == name)
    }

    /// Gets a builtin function by its name.
    pub fn get_binary_op(&self, op: &Op) -> Option<&Fun> {
        self.get_by(|f| if let Fun::Op(o, f) = f { op == o && f.params() == 2 } else { false })
    }

    /// Gets a builtin function by its name.
    pub fn get_unary_op(&self, op: &Op) -> Option<&Fun> {
        self.get_by(|f| if let Fun::Op(o, f) = f { op == o && f.params() == 1 } else { false })
    }
}

impl From<Scope<Fun, FunSymbolAlloc>> for FunScope {
    fn from(scope: Scope<Fun, FunSymbolAlloc>) -> Self { FunScope { scope } }
}

impl From<FunScope> for Scope<Fun, FunSymbolAlloc> {
    fn from(scope: FunScope) -> Self { scope.scope }
}

impl Deref for FunScope {
    type Target = Scope<Fun, FunSymbolAlloc>;

    fn deref(&self) -> &Scope<Fun, FunSymbolAlloc> { &self.scope }
}

impl DerefMut for FunScope {
    fn deref_mut(&mut self) -> &mut Scope<Fun, FunSymbolAlloc> { &mut self.scope }
}

impl Default for FunScope {
    fn default() -> Self {
        FunScope {
            scope: Scope::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ir;
    use compile::{FunStub, self};
    use vm::*;
    use super::*;

    #[test]
    fn test_reg_scope() {
        // Check that values are inserted correctly
        let mut reg_scope = VarScope::default();
        reg_scope.push_empty_scope();
        let a_sym = reg_scope.reserve_symbol();
        assert_eq!(a_sym, RegSymbol { global: 0, local: 0 });
        let a = Var::new("a".to_string(), a_sym);
        reg_scope.insert(a);
        let b_sym = reg_scope.reserve_symbol();
        assert_eq!(b_sym, RegSymbol { global: 0, local: 1 });
        let b = Var::new("b".to_string(), b_sym);
        reg_scope.insert(b);

        // Check that local layers can be added while still having access to parent layers
        reg_scope.push_empty_scope();
        let c_sym = reg_scope.reserve_symbol();
        assert_eq!(c_sym, RegSymbol { global: 1, local: 0 });
        let c = Var::new("c".to_string(), c_sym);
        reg_scope.insert(c);
        assert_eq!(reg_scope.get_by_name("a").unwrap().symbol(), a_sym);
        assert_eq!(reg_scope.get_by_name("c").unwrap().symbol(), c_sym);

        // Check that scope layers that have been shed don't yield old values
        reg_scope.pop_scope();
        assert_eq!(reg_scope.get_by_name("b").unwrap().symbol(), b_sym);
        assert!(reg_scope.get_by_name("c").is_none());

        // Check that using the same name in two sibling scopes yields the correct register
        reg_scope.push_empty_scope();
        assert!(reg_scope.get_by_name("c").is_none());
        let c_sym = reg_scope.reserve_symbol();
        assert_eq!(c_sym, RegSymbol { global: 2, local: 0 });
        let c = Var::new("c".to_string(), c_sym);
        reg_scope.insert(c);
        assert_eq!(reg_scope.get_by_name("c").unwrap().symbol(), c_sym);

        // Check that overriding values in the parent scope yields the correct register
        let new_a_sym = reg_scope.reserve_symbol();
        assert_eq!(new_a_sym, RegSymbol { global: 2, local: 1 });
        let a = Var::new("a".to_string(), new_a_sym);
        reg_scope.insert(a);
        assert_eq!(reg_scope.get_by_name("a").unwrap().symbol(), new_a_sym);

        // Check that anonymous symbols are inserted and freed correctly
        let anon_sym1 = reg_scope.insert_anonymous_var();
        reg_scope.free_anonymous_var(anon_sym1);
        let anon_sym2 = reg_scope.insert_anonymous_var();
        assert_eq!(anon_sym1, anon_sym2);

        // Check that overriden values are restored after the layer is shed
        reg_scope.pop_scope();
        assert_eq!(reg_scope.get_by_name("a").unwrap().symbol(), a_sym);

        // Check that anonymous symbols are not allocated to inappropriate scopes
        let anon_sym3 = reg_scope.insert_anonymous_var();
        assert_ne!(anon_sym1, anon_sym3);

        // Check that values are inserted or removed appropriately
        let old_a_sym = reg_scope.get_or_insert("a");
        assert_eq!(a_sym, old_a_sym);
        let new_d_sym = reg_scope.get_or_insert("d");
        assert_eq!(new_d_sym, RegSymbol { global: 0, local: 3 });
    }

    #[test]
    fn test_fun_scope() {
        let mut fun_scope = FunScope::default();
        fun_scope.push_empty_scope();
        fun_scope.insert_builtin_functions();
        fun_scope.insert_builtin_ops();

        // Check that builtin functions are added (use both get_by_name_and_params and get_builtin)
        for builtin in builtin_functions.iter() {
            let found = fun_scope.get_by_name_and_params(&builtin.name, builtin.params)
                .expect("Failed to get registered builtin");
            assert_eq!(fun_scope.get_builtin(&builtin.name).unwrap().symbol(), found.symbol());
        }

        // Check that builtin operators are added
        for BuiltinOp(op, builtin) in builtin_ops.iter() {
            if builtin.params == 2 {
                assert!(fun_scope.get_binary_op(op).is_some());
            } else if builtin.params == 1 {
                assert!(fun_scope.get_unary_op(op).is_some());
            }
        }

        // Check that insertion works
        fun_scope.push_empty_scope();
        let stub_a_sym = fun_scope.reserve_symbol();
        let stub_a = compile::Fun::Stub(FunStub {
            name: "a".to_string(),
            symbol: stub_a_sym,
            params: 2,
            return_ty: ir::TyExpr::None,
        });

        fun_scope.insert(stub_a);

        assert!(fun_scope.get_by_name_and_params("a", 2).unwrap().symbol() == stub_a_sym);

        // Check that adding a sub-scope with the same function name and params will yield the more
        // local function
        fun_scope.push_empty_scope();
        let new_stub_a_sym = fun_scope.reserve_symbol();
        let stub_a = compile::Fun::Stub(FunStub {
            name: "a".to_string(),
            symbol: new_stub_a_sym,
            params: 2,
            return_ty: ir::TyExpr::None,
        });
        fun_scope.insert(stub_a);

        {
            let stub_a_lookup = fun_scope.get_by_name_and_params("a", 2)
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), new_stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), stub_a_sym);
        }
        fun_scope.pop_scope();

        // Check that functions with the same name and different args are resolved correctly
        fun_scope.push_empty_scope();
        let params_stub_a_sym = fun_scope.reserve_symbol();
        let stub_a = compile::Fun::Stub(FunStub {
            name: "a".to_string(),
            symbol: params_stub_a_sym,
            params: 3,
            return_ty: ir::TyExpr::None,
        });
        fun_scope.insert(stub_a);

        {
            // Check that we get a(arg, arg, arg) correctly
            let stub_a_lookup = fun_scope.get_by_name_and_params("a", 3)
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), params_stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), stub_a_sym);
            // Check that we get a(arg, arg, arg) correctly with a simple name lookup
            let stub_a_lookup = fun_scope.get_by_name("a")
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), params_stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), stub_a_sym);
            // Check that we get the global a(arg, arg) function
            let stub_a_lookup = fun_scope.get_by_name_and_params("a", 2)
                .unwrap();
            assert_eq!(stub_a_lookup.symbol(), stub_a_sym);
            assert_ne!(stub_a_lookup.symbol(), params_stub_a_sym);
        }
        fun_scope.pop_scope();

        // Check that functions can be replaced correctly
        let stub_b = compile::Fun::Stub(FunStub {
            name: "b".to_string(),
            symbol: stub_a_sym,
            params: 2,
            return_ty: ir::TyExpr::None,
        });
        let stub_a = fun_scope.replace(stub_b);
        assert_eq!(stub_a.symbol(), stub_a_sym);
        {
            let stub_b_lookup = fun_scope.get_by_name("b")
                .expect("Failed to get replaced function");
            assert_eq!(stub_b_lookup.symbol(), stub_a.symbol());
        }
    }
}
