use std::{
    fmt::Debug,
    path::Path,
};
use crate::common::{
    lang::Op,
    pos::{Range, RangeWrapper, Ranged},
    error::*,
    FromPath,
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
where
    T: Ast + Clone + Debug + Ranged,
{
    fn token_is_lookahead(token: &Token) -> bool {
        T::token_is_lookahead(token)
    }
    fn name() -> &'static str {
        T::name()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub funs: Vec<Fun>,
    pub tys: Vec<UserTy>,
    pub stmts: Vec<Stmt>,
    pub range: Range,
}

impl_ranged!(Block::range);

impl Block {
    pub fn new(funs: Vec<Fun>, tys: Vec<UserTy>, stmts: Vec<Stmt>, range: Range) -> Self {
        Block {
            funs,
            tys,
            stmts,
            range,
        }
    }
}

impl FromPath for Block {
    type Err = Error;
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        use crate::{
            util,
            syntax::{Lexer, Parser},
        };
        let path = path.as_ref();
        let contents = util::read_file(path)?;
        let lexer = Lexer::new(path.display(), &contents);
        let parser = Parser::from_lexer(lexer);
        Ok(parser.into_parse_tree()?)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
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
        Expr::token_is_lookahead(token)
            || token_is_lookahead!(token, Token::FunKw, Token::ReturnKw, Token::IfKw)
    }

    fn name() -> &'static str {
        "statement"
    }
}

impl Ranged for Stmt {
    fn range(&self) -> Range {
        match self {
            Stmt::Expr(e) => e.range(),
            Stmt::Assign(lhs, _, rhs) => lhs.range().union(&rhs.range()),
            Stmt::While(c) => c.range(),
            Stmt::Loop(b) => b.range(),
            Stmt::If {
                if_block,
                elseif_blocks,
                else_block,
            } => {
                if let Some(else_block) = else_block {
                    if_block.range().union(&else_block.range())
                } else if let Some(elseif_block) = elseif_blocks.last() {
                    if_block.range().union(&elseif_block.range())
                } else {
                    if_block.range()
                }
            }
            Stmt::Continue(r) | Stmt::Break(r) | Stmt::Return(_, r) => r.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Fun(Fun),
    UserTy(UserTy),
    Stmt(Stmt),
}

impl Ranged for Item {
    fn range(&self) -> Range {
        match self {
            Item::Fun(f) => f.range(),
            Item::UserTy(t) => t.range(),
            Item::Stmt(s) => s.range(),
        }
    }
}

impl Ast for Item {
    fn token_is_lookahead(token: &Token) -> bool {
        Fun::token_is_lookahead(token) || UserTy::token_is_lookahead(token) || Stmt::token_is_lookahead(token)
    }

    fn name() -> &'static str {
        "item"
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

    fn name() -> &'static str {
        "type definition"
    }
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

    fn name() -> &'static str {
        "function definition"
    }
}

impl_ranged!(Fun::range);

#[derive(Debug, Clone, PartialEq)]
pub struct FunParam {
    pub name: String,
    pub ty: Option<String>,
    pub default: Option<Expr>,
    pub range: Range,
}

impl FunParam {
    pub fn new(name: String, ty: Option<String>, default: Option<Expr>, range: Range) -> Self {
        FunParam {
            name,
            ty,
            default,
            range,
        }
    }
}

impl Ast for FunParam {
    fn token_is_lookahead(token: &Token) -> bool {
        matches!(token, Token::Variable(_))
    }

    fn name() -> &'static str {
        "function parameter"
    }
}

impl_ranged!(FunParam::range);

/// A generic block that comes with a (presumably) conditional expression.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionBlock {
    pub condition: Expr,
    pub block: Block,
}

impl ConditionBlock {
    pub fn new(condition: Expr, block: Block) -> Self {
        ConditionBlock { condition, block }
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
            Expr::Binary(lhs, op, rhs) => {
                format!("({} {} {})", lhs.canonicalize(), op, rhs.canonicalize())
            }
            Expr::Atom(e) => format!("{}", e.token()),
            _ => unreachable!(),
        }
    }

    pub fn token_is_atom_lookahead(token: &Token) -> bool {
        token_is_lookahead!(
            token,
            Token::StrLit(_),
            Token::IntLit(_, _),
            Token::FloatLit(_),
            Token::Variable(_),
            Token::Bareword(_)
        )
    }
}

impl Ast for Expr {
    fn token_is_lookahead(token: &Token) -> bool {
        token_is_lookahead!(
            token,
            Token::StrLit(_),
            Token::IntLit(_, _),
            Token::FloatLit(_),
            Token::TrueKw,
            Token::FalseKw,
            Token::Variable(_),
            Token::Bareword(_),
            Token::Op(Op::Plus),
            Token::Op(Op::Minus),
            Token::Op(Op::Bang),
            Token::LParen
        )
    }

    fn name() -> &'static str {
        "expression"
    }
}

impl Ranged for Expr {
    fn range(&self) -> Range {
        match self {
            Expr::FunCall {
                function: _,
                args: _,
                range,
            }
            | Expr::ArrayAccess {
                array: _,
                index: _,
                range,
            } => range.clone(),
            Expr::Atom(t) => t.range(),
            Expr::Unary(_, e) => e.range(),
            Expr::Binary(l, _, r) => l.range().union(&r.range()),
        }
    }
}

