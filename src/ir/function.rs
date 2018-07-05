use syntax::tree;
use ir::{
    Ir,
    Action, Symbol, Ty, Value, Block,
};

#[derive(Debug)]
pub struct Function<'n> {
    name: Symbol,
    params: Vec<FunctionParam<'n>>,
    return_ty: Ty,
    body: Block<'n>,
}

impl<'n> Function<'n> {
    pub fn new(name: Symbol, params: Vec<FunctionParam<'n>>, return_ty: Ty, body: Block<'n>) -> Self {
        Function { name, params, return_ty, body }
    }

    pub fn name(&self) -> &str { &self.name.name() }

    pub fn symbol(&self) -> &Symbol { &self.name }

    pub fn params(&self) -> &[FunctionParam<'n>] { &self.params }
    
    pub fn return_ty(&self) -> &Ty { &self.return_ty }

    pub fn body(&self) -> &[Action<'n>] { &self.body }
}

impl<'n> Ir<tree::Stmt<'n>> for Function<'n> {
    fn from_syntax(stmt: &tree::Stmt<'n>) -> Self {
        if let tree::Stmt::Function { name, params, return_ty, body } = stmt {
            let name = Symbol::Function(name.clone());
            let params = params.iter()
                .map(FunctionParam::from_syntax)
                .collect();
            let return_ty = if let Some(return_ty) = return_ty {
                Ty::Definite(return_ty.to_string())
            } else {
                Ty::Any
            };
            let body = body.iter()
                .map(Action::from_syntax)
                .collect();
            Function { name, params, return_ty, body }
        } else {
            panic!("Attempted to convert non-Function Stmt to IR Function")
        }
    }
}

#[derive(Debug)]
pub struct FunctionParam<'n> {
    name: Symbol,
    ty: Ty,
    default: Option<Value<'n>>,
}

impl<'n> FunctionParam<'n> {
    pub fn new(name: Symbol, ty: Ty, default: Option<Value<'n>>) -> Self {
        FunctionParam { name, ty, default, }
    }
}

impl<'n> Ir<tree::FunctionParam<'n>> for FunctionParam<'n> {
    fn from_syntax(tree::FunctionParam { name, ty, default }: &tree::FunctionParam<'n>) -> Self {
        let name = Symbol::Variable(name.to_string());
        let ty = if let Some(ty) = ty {
            Ty::Definite(ty.to_string())
        } else {
            // variables, by default, have a type of "any"
            Ty::Any
        };
        let default = default.as_ref().map(Value::from_syntax);
        FunctionParam::new(name, ty, default)
    }
}
