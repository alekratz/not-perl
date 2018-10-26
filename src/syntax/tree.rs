use std::fmt::Debug;
use crate::common::{
    pos::{Range, Ranged, RangeWrapper},
    lang::Op,
};
use crate::syntax::token::*;

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

pub trait Ast: Ranged {
    fn token_is_lookahead(token: &Token) -> bool;
    fn name() -> &'static str;
}

impl<T> Ast for RangeWrapper<T>
    where T: Ast + Clone + Debug + Ranged
{
    fn token_is_lookahead(token: &Token) -> bool { T::token_is_lookahead(token) }
    fn name() -> &'static str { T::name() }
}

#[derive(Debug, Clone)]
pub struct SyntaxTree {
    pub stmts: Vec<Stmt>,
    pub range: Range,
}

impl SyntaxTree {
    pub fn new(stmts: Vec<Stmt>, range: Range) -> Self {
        SyntaxTree {
            stmts,
            range,
        }
    }
}

impl Ast for SyntaxTree {
    fn token_is_lookahead(token: &Token) -> bool {
        Stmt::token_is_lookahead(token)
    }

    fn name() -> &'static str { "syntax tree" }
}

impl_ranged!(SyntaxTree::range);

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Fun(Fun),
    UserTy(UserTy),
    Expr(Expr),
    Assign(Expr, AssignOp, Expr),
    While(ConditionBlock),
    Loop(Block),
    If {
        if_block: ConditionBlock,
        elseif_blocks: Vec<ConditionBlock>,
        else_block: Option<Block>,
    },
    Continue(Range),
    Break(Range),
    Return(Option<Expr>, Range),
}

impl Ast for Stmt {
    fn token_is_lookahead(token: &Token) -> bool {
        Expr::token_is_lookahead(token) || token_is_lookahead!(token, Token::FunKw, Token::ReturnKw, Token::IfKw)
    }

    fn name() -> &'static str { "statement" }
}

impl Ranged for Stmt {
    fn range(&self) -> Range {
        match self {
            Stmt::Fun(f) => f.range(),
            Stmt::UserTy(u) => u.range(),
            Stmt::Expr(e) => e.range(),
            Stmt::Assign(lhs, _, rhs) => lhs.range().union(&rhs.range()),
            Stmt::While(c) => c.range(),
            Stmt::Loop(b) => b.range(),
            Stmt::If { if_block, elseif_blocks, else_block } => {
                unimplemented!()
            }
            | Stmt::Continue(r)
            | Stmt::Break(r)
            | Stmt::Return(_, r) => r.clone(),
        }
    }
}

pub type Block = RangeWrapper<Vec<Stmt>>;

impl AsRef<[Stmt]> for Block {
    fn as_ref(&self) -> &[Stmt] {
        &self.1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserTy {
    pub name: String,
    pub parents: Vec<String>,
    pub functions: Vec<Fun>,
    pub range: Range,
}

impl Ast for UserTy {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(token, Token::TypeKw)
    }

    fn name() -> &'static str { "type definition" }
}

impl_ranged!(UserTy::range);

#[derive(Debug, Clone, PartialEq)]
pub struct Fun {
    pub name: String,
    pub params: Vec<FunParam>,
    pub return_ty: Option<String>,
    pub body: Block,
    pub range: Range,
}

impl Ast for Fun {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(token, Token::FunKw)
    }

    fn name() -> &'static str { "function definition" }
}

impl_ranged!(Fun::range);

#[derive(Debug, Clone, PartialEq)]
pub enum FunParam {
    SelfKw(Range),
    Variable {
        name: String,
        ty: Option<String>,
        default: Option<Expr>,
        range: Range,
    },
}

impl Ast for FunParam {
    fn token_is_lookahead(token: &Token) -> bool {
        matches!(token, Token::Variable(_)) || token == &Token::SelfKw
    }

    fn name() -> &'static str { "function parameter" }
}

impl Ranged for FunParam {
    fn range(&self) -> Range {
        match self {
            FunParam::SelfKw(r) => r.clone(),
            FunParam::Variable { name: _, ty: _, default: _, range } => range.clone(),
        }
    }
}

/// A generic block that comes with a (presumably) conditional expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionBlock {
    pub condition: Expr,
    pub block: Block,
}

impl ConditionBlock {
    pub fn new(condition: Expr, block: Block) -> Self {
        ConditionBlock { condition, block, }
    }
}

impl Ranged for ConditionBlock {
    fn range(&self) -> Range {
        self.condition.range().union(&self.block.range())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    FunCall {
        function: Box<Expr>,
        args: Vec<Expr>,
        range: Range,
    },
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
        range: Range,
    },
    Atom(RangedToken),
    Unary(Op, Box<Expr>),
    Binary(Box<Expr>, Op, Box<Expr>),
}

impl Expr {
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
            Token::Variable(_), Token::Bareword(_), Token::SelfKw
        )
    }
}

impl Ast for Expr {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(
            token,
            Token::StrLit(_), Token::IntLit(_, _), Token::FloatLit(_), Token::TrueKw, Token::FalseKw,
            Token::Variable(_), Token::Bareword(_),
            Token::Op(Op::Plus),
            Token::Op(Op::Minus),
            Token::Op(Op::Bang),
            Token::LParen, Token::SelfKw
        )
    }

    fn name() -> &'static str { "expression" }
}


impl Ranged for Expr {
    fn range(&self) -> Range {
        match self {
            | Expr::FunCall { function: _, args: _, range }
            | Expr::ArrayAccess { array: _, index: _, range } => range.clone(),
            Expr::Atom(t) => t.range(),
            Expr::Unary(_, e) => e.range(),
            Expr::Binary(l, _, r) => l.range().union(&r.range()),
        }
    }
}
