use syntax::token::*;

macro_rules! token_is_lookahead {
    ($token:expr, $head:pat $(, $tail:pat)*) => {{
        match $token {
            | $head
            $(
                | $tail
            )* => true,
            _ => false,
        }
    }};
}

pub trait Ast {
    fn token_is_lookahead(token: &Token) -> bool;
    fn name() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct SyntaxTree<'n> {
    pub stmts: Vec<Stmt<'n>>,
}

impl<'n> Ast for SyntaxTree<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        Stmt::token_is_lookahead(token)
    }

    fn name() -> &'static str { "syntax tree" }
}

impl<'n> Default for SyntaxTree<'n> {
    fn default() -> Self {
        SyntaxTree { stmts: vec![] }
    }    
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'n> {
    Function(Function<'n>),
    UserTy(UserTy<'n>),
    Expr(Expr<'n>),
    Assign(Expr<'n>, AssignOp, Expr<'n>),
    While(ConditionBlock<'n>),
    Loop(Block<'n>),
    If {
        if_block: ConditionBlock<'n>,
        elseif_blocks: Vec<ConditionBlock<'n>>,
        else_block: Option<Block<'n>>,
    },
    Continue,
    Break,
    Return(Option<Expr<'n>>),
}

impl<'n> Ast for Stmt<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        Expr::token_is_lookahead(token) || token_is_lookahead!(token, Token::FunKw, Token::ReturnKw, Token::IfKw)
    }

    fn name() -> &'static str { "statement" }
}

pub type Block<'n> = Vec<Stmt<'n>>;

#[derive(Debug, Clone, PartialEq)]
pub struct UserTy<'n> {
    pub name: String,
    pub parents: Vec<String>,
    pub functions: Vec<Function<'n>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function<'n> {
    pub name: String,
    pub params: Vec<FunctionParam<'n>>,
    pub return_ty: Option<String>,
    pub body: Block<'n>,
}

impl<'n> Ast for Function<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(token, Token::FunKw)
    }

    fn name() -> &'static str { "function definition" }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam<'n> {
    pub name: String,
    pub ty: Option<String>,
    pub default: Option<Expr<'n>>,
}

impl<'n> FunctionParam<'n> {
    pub fn new(name: String, ty: Option<String>, default: Option<Expr<'n>>) -> Self {
        FunctionParam { name, ty, default }
    }
}

impl<'n> Ast for FunctionParam<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        matches!(token, Token::Variable(_))
    }

    fn name() -> &'static str { "function parameter" }
}

/// A generic block that comes with a (presumably) conditional expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionBlock<'n> {
    pub condition: Expr<'n>,
    pub block: Vec<Stmt<'n>>,
}

impl<'n> ConditionBlock<'n> {
    pub fn new(condition: Expr<'n>, block: Vec<Stmt<'n>>) -> Self {
        ConditionBlock { condition, block, }
    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'n> {
    FunCall {
        function: Box<Expr<'n>>,
        args: Vec<Expr<'n>>,
    },
    ArrayAccess {
        array: Box<Expr<'n>>,
        index: Box<Expr<'n>>,
    },
    Atom(RangeToken<'n>),
    Unary(Op, Box<Expr<'n>>),
    Binary(Box<Expr<'n>>, Op, Box<Expr<'n>>),
}

impl<'n> Expr<'n> {
    pub fn canonicalize(&self) -> String {
        match self {
            Expr::Binary(lhs, op, rhs) => format!("({} {} {})", lhs.canonicalize(), op, rhs.canonicalize()),
            Expr::Atom(e) => format!("{}", e.token()),
            _ => unreachable!()
        }
    }

    pub fn token_is_atom_lookahead(token: &Token) -> bool {
        token_is_lookahead!(
            token,
            Token::StrLit(_), Token::IntLit(_, _), Token::FloatLit(_),
            Token::Variable(_), Token::Bareword(_)
        )
    }
}

impl<'n> Ast for Expr<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(
            token,
            Token::StrLit(_), Token::IntLit(_, _), Token::FloatLit(_), Token::TrueKw, Token::FalseKw,
            Token::Variable(_), Token::Bareword(_),
            Token::Op(Op::Plus),
            Token::Op(Op::Minus),
            Token::Op(Op::Bang),
            Token::LParen
        )
    }

    fn name() -> &'static str { "expression" }
}
