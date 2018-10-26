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

pub trait Ast<'n>: Ranged<'n> {
    fn token_is_lookahead(token: &Token) -> bool;
    fn name() -> &'static str;
}

impl<'n, T> Ast<'n> for RangeWrapper<'n, T>
    where T: Ast<'n> + Clone + Debug + Ranged<'n>
{
    fn token_is_lookahead(token: &Token) -> bool { T::token_is_lookahead(token) }
    fn name() -> &'static str { T::name() }
}

#[derive(Debug, Clone)]
pub struct SyntaxTree<'n> {
    pub stmts: Vec<Stmt<'n>>,
    pub range: Range<'n>,
}

impl<'n> SyntaxTree<'n> {
    pub fn new(stmts: Vec<Stmt<'n>>, range: Range<'n>) -> Self {
        SyntaxTree {
            stmts,
            range,
        }
    }
}

impl<'n> Ast<'n> for SyntaxTree<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        Stmt::token_is_lookahead(token)
    }

    fn name() -> &'static str { "syntax tree" }
}

impl_ranged!(SyntaxTree::range);

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'n> {
    Fun(Fun<'n>),
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
    Continue(Range<'n>),
    Break(Range<'n>),
    Return(Option<Expr<'n>>, Range<'n>),
}

impl<'n> Ast<'n> for Stmt<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        Expr::token_is_lookahead(token) || token_is_lookahead!(token, Token::FunKw, Token::ReturnKw, Token::IfKw)
    }

    fn name() -> &'static str { "statement" }
}

impl<'n> Ranged<'n> for Stmt<'n> {
    fn range(&self) -> Range<'n> {
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
            | Stmt::Return(_, r) => *r
        }
    }
}

pub type Block<'n> = RangeWrapper<'n, Vec<Stmt<'n>>>;

impl<'n> AsRef<[Stmt<'n>]> for Block<'n> {
    fn as_ref(&self) -> &[Stmt<'n>] {
        &self.1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserTy<'n> {
    pub name: String,
    pub parents: Vec<String>,
    pub functions: Vec<Fun<'n>>,
    pub range: Range<'n>,
}

impl<'n> Ast<'n> for UserTy<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(token, Token::TypeKw)
    }

    fn name() -> &'static str { "type definition" }
}

impl_ranged!(UserTy::range);

#[derive(Debug, Clone, PartialEq)]
pub struct Fun<'n> {
    pub name: String,
    pub params: Vec<FunParam<'n>>,
    pub return_ty: Option<String>,
    pub body: Block<'n>,
    pub range: Range<'n>,
}

impl<'n> Ast<'n> for Fun<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(token, Token::FunKw)
    }

    fn name() -> &'static str { "function definition" }
}

impl_ranged!(Fun::range);

#[derive(Debug, Clone, PartialEq)]
pub enum FunParam<'n> {
    SelfKw(Range<'n>),
    Variable {
        name: String,
        ty: Option<String>,
        default: Option<Expr<'n>>,
        range: Range<'n>,
    },
}

impl<'n> Ast<'n> for FunParam<'n> {
    fn token_is_lookahead(token: &Token) -> bool {
        matches!(token, Token::Variable(_)) || token == &Token::SelfKw
    }

    fn name() -> &'static str { "function parameter" }
}

impl<'n> Ranged<'n> for FunParam<'n> {
    fn range(&self) -> Range<'n> {
        match self {
            FunParam::SelfKw(r) => *r,
            FunParam::Variable { name: _, ty: _, default: _, range } => *range,
        }
    }
}

/// A generic block that comes with a (presumably) conditional expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionBlock<'n> {
    pub condition: Expr<'n>,
    pub block: Block<'n>,
}

impl<'n> ConditionBlock<'n> {
    pub fn new(condition: Expr<'n>, block: Block<'n>) -> Self {
        ConditionBlock { condition, block, }
    }
}

impl<'n> Ranged<'n> for ConditionBlock<'n> {
    fn range(&self) -> Range<'n> {
        self.condition.range().union(&self.block.range())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'n> {
    FunCall {
        function: Box<Expr<'n>>,
        args: Vec<Expr<'n>>,
        range: Range<'n>,
    },
    ArrayAccess {
        array: Box<Expr<'n>>,
        index: Box<Expr<'n>>,
        range: Range<'n>,
    },
    Atom(RangedToken<'n>),
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
            Token::Variable(_), Token::Bareword(_), Token::SelfKw
        )
    }
}

impl<'n> Ast<'n> for Expr<'n> {
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


impl<'n> Ranged<'n> for Expr<'n> {
    fn range(&self) -> Range<'n> {
        match self {
            Expr::FunCall { function: _, args: _, range } => *range,
            Expr::ArrayAccess { array: _, index: _, range } => *range,
            Expr::Atom(t) => t.range(),
            Expr::Unary(_, e) => e.range(),
            Expr::Binary(l, _, r) => l.range().union(&r.range()),
        }
    }
}
