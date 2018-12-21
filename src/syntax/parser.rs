use std::{collections::VecDeque, fmt::Display, mem};
use crate::common::prelude::*;
use crate::syntax::{token::*, tree::*, Error, ErrorKind, Lexer, Result};

macro_rules! ranged {
    ( $lexer:expr, $block:block ) => {{
        let begin = ($lexer).pos();
        let value = $block;
        let end = ($lexer).pos();
        let range = Range::Src(SrcRange::new(begin, end));
        (range, value)
    }};
}

pub struct Parser<'c> {
    lexer: Lexer<'c>,
    curr: Option<RangedToken>,
    next: Option<RangedToken>,
    stmt_level: usize,
}

impl<'c> Parser<'c> {
    pub fn new(source_name: impl ToString, source_text: &'c str) -> Self {
        let lexer = Lexer::new(source_name, source_text);
        Parser::from_lexer(lexer)
    }

    pub fn from_lexer(lexer: Lexer<'c>) -> Self {
        Parser {
            lexer,
            curr: None,
            next: None,
            stmt_level: 0,
        }
    }

    pub fn into_parse_tree(mut self) -> Result<Block> {
        self.init()?;
        self.next_block(&[])
    }

    /// Readies this parser by filling in the first two tokens.
    fn init(&mut self) -> Result<()> {
        assert!(self.curr.is_none());
        assert!(self.next.is_none());
        // Option<Result<Token>> -> Option<Token>
        self.curr = if let Some(result) = self.lexer.next() {
            Some(result?)
        } else {
            None
        };

        self.next = if let Some(result) = self.lexer.next() {
            Some(result?)
        } else {
            None
        };
        self.skip_whitespace()?;
        Ok(())
    }

    fn skip_whitespace(&mut self) -> Result<()> {
        while self.is_token_match(&Token::LineEnd) || self.is_token_match(&Token::Comment) {
            self.next_token()?;
        }
        Ok(())
    }

    fn next_item(&mut self) -> Result<Item> {
        assert_eq!(self.stmt_level, 0);
        self.skip_whitespace()?;

        let curr = if let Some(curr) = self.curr.clone() {
            Token::from(curr)
        } else {
            return Err(self.err_expected_got_eof(Stmt::name()));
        };
        let item = match curr {
            Token::FunKw => Item::Fun(self.next_function()?),
            Token::TypeKw => Item::UserTy(self.next_user_type()?),
            _ => Item::Stmt(self.next_stmt()?),
        };
        let is_newline_needed = matches!(item, Item::UserTy(_)); 

        if is_newline_needed {
            self.next_eol_or_eof()?;
        }
        Ok(item)
    }

    fn next_stmt(&mut self) -> Result<Stmt> {
        assert_eq!(self.stmt_level, 0);
        self.skip_whitespace()?;

        let curr = if let Some(curr) = self.curr.clone() {
            Token::from(curr)
        } else {
            return Err(self.err_expected_got_eof(Stmt::name()));
        };
        let stmt = match curr {
            Token::ReturnKw => {
                let (range, stmt) = ranged!(self.lexer, {
                    self.next_token_or_newline()?;
                    if self.is_lookahead::<Expr>() {
                        Some(self.next_expr()?)
                    } else {
                        None
                    }
                });
                Stmt::Return(stmt, range)
            }
            Token::ContinueKw => {
                let token = self.next_token_or_newline()?.unwrap();
                Stmt::Continue(token.range())
            }
            Token::BreakKw => {
                let token = self.next_token_or_newline()?.unwrap();
                Stmt::Break(token.range())
            }
            Token::WhileKw => {
                self.next_token()?;
                let condblock = self.next_condition_block()?;
                Stmt::While(condblock)
            }
            Token::LoopKw => {
                self.next_token()?;
                let block = self.next_body()?;
                Stmt::Loop(block)
            }
            Token::IfKw => {
                self.next_token()?;
                let if_block = self.next_condition_block()?;
                let mut else_block = None;
                let mut elseif_blocks = vec![];
                while self.is_token_match(&Token::ElseKw) {
                    self.next_token()?;
                    if self.is_token_match(&Token::IfKw) {
                        // else-if block
                        self.next_token()?;
                        elseif_blocks.push(self.next_condition_block()?);
                    } else {
                        // else block
                        else_block = Some(self.next_body()?);
                        break;
                    }
                }
                Stmt::If {
                    if_block,
                    elseif_blocks,
                    else_block,
                }
            }
            ref t if t.is_lookahead::<Expr>() => {
                // expr, assignment
                let lhs = self.next_expr()?;
                if self.is_curr_assign_op() {
                    let op = self.next_assign_op()?;
                    let rhs = self.next_expr()?;
                    Stmt::Assign(lhs, op, rhs)
                } else {
                    Stmt::Expr(lhs)
                }
            }
            _ => return Err(self.err_expected_got("statement", self.curr.as_ref())),
        };
        let is_newline_needed = match stmt {
            Stmt::While(_) => false,
            Stmt::If {
                if_block: _,
                elseif_blocks: _,
                else_block: _,
            } => false,
            Stmt::Loop(_) => false,
            _ => true,
        };

        if is_newline_needed {
            self.next_eol_or_eof()?;
        }
        Ok(stmt)
    }

    fn next_eol_or_eof(&mut self) -> Result<()> {
        if self.is_token_match(&Token::LineEnd) || self.is_token_match(&Token::Comment) {
            self.next_token().map(|_| ())
        } else if self.curr.is_none() {
            // EOF
            Ok(())
        } else {
            Err(self.err_expected_got("end-of-line (`;`) or EOF", self.curr.as_ref()))
        }
    }

    fn next_condition_block(&mut self) -> Result<ConditionBlock> {
        let condition = self.next_expr()?;
        let block = self.next_body()?;
        Ok(ConditionBlock::new(condition, block))
    }

    fn next_body(&mut self) -> Result<Block> {
        self.match_token(Token::LBrace)?;
        let block = self.next_block(&[Token::RBrace])?;
        self.match_token(Token::RBrace)?;
        Ok(block)
    }

    fn next_block(&mut self, end_tokens: &[Token]) -> Result<Block> {
        let (range, (funs, tys, stmts)) = ranged!(self.lexer, {
            let mut funs = Vec::new();
            let mut tys = Vec::new();
            let mut stmts = Vec::new();
            while !self.is_any_token_match(end_tokens) {
                match self.next_item()? {
                    Item::Stmt(stmt) => stmts.push(stmt),
                    Item::UserTy(ty) => tys.push(ty),
                    Item::Fun(fun) => funs.push(fun),
                }
            }
            (funs, tys, stmts)
        });
        Ok(Block::new(funs, tys, stmts, range))
    }

    fn next_expr(&mut self) -> Result<Expr> {
        let op_queue = VecDeque::from(vec![
            vec![
                Op::DoublePercent,
                Op::DoubleEquals,
                Op::DoubleTilde,
                Op::NotEquals,
                Op::LessEquals,
                Op::GreaterEquals,
                Op::Less,
                Op::Greater,
            ],
            vec![Op::Or],
            vec![Op::And],
            vec![Op::Tilde],
            vec![Op::Plus, Op::Minus],
            vec![Op::Splat, Op::FSlash],
        ]);
        self.next_binary_expr(op_queue)
    }

    fn next_binary_expr(&mut self, mut op_queue: VecDeque<Vec<Op>>) -> Result<Expr> {
        if let Some(top) = op_queue.pop_front() {
            let lhs = self.next_binary_expr(op_queue.clone())?;
            let op_matches = self
                .curr
                .as_ref()
                .map(|t| {
                    if let &Token::Op(ref op) = t.token() {
                        top.contains(op)
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if op_matches {
                let op = self.next_op()?;
                op_queue.push_front(top);
                let rhs = self.next_binary_expr(op_queue)?;
                Ok(Expr::Binary(Box::new(lhs), op, Box::new(rhs)))
            } else {
                Ok(lhs)
            }
        } else {
            self.next_unary_expr()
        }
    }

    fn next_unary_expr(&mut self) -> Result<Expr> {
        if self.is_curr_op() {
            let token = self.next_token()?.unwrap();
            if token.is_lookahead::<Expr>() {
                let op = Token::from(token).into_op();
                let expr = self.next_unary_expr()?;
                Ok(Expr::Unary(op, Box::new(expr)))
            } else {
                Err(self.err_expected_got("unary operator", Some(&token)))
            }
        } else {
            self.next_atom_expr()
        }
    }

    fn next_atom_expr(&mut self) -> Result<Expr> {
        let begin = self.lexer.pos();
        let curr = if let Some(curr) = self.curr.clone() {
            Token::from(curr)
        } else {
            return Err(self.err_expected_got_eof(Expr::name()));
        };

        if !curr.is_lookahead::<Expr>() {
            return Err(self.err_expected_got(Expr::name(), self.curr.as_ref()));
        }

        let mut expr = match curr {
            Token::LParen => {
                self.next_token()?;
                self.stmt_level += 1;
                let inner = self.next_expr()?;
                self.stmt_level -= 1;
                // stmt_level is set to 0 at the start of each stmt rule, so stmts that end in
                // expressions are *required* to have a newline at the end
                if self.stmt_level == 0 {
                    self.match_token_preserve_newline(Token::RParen)?;
                } else {
                    self.match_token(Token::RParen)?;
                }
                inner
            }
            _ => {
                if self.stmt_level == 0 {
                    Expr::Atom(self.next_token_or_newline()?.unwrap())
                } else {
                    Expr::Atom(self.next_token()?.unwrap())
                }
            }
        };

        if self.is_token_match(&Token::LParen) {
            let args = self.next_funcall_args()?;
            let end = self.lexer.pos();
            let range = Range::Src(SrcRange::new(begin.clone(), end));
            expr = Expr::FunCall {
                function: Box::new(expr),
                args,
                range,
            }
        }

        if self.is_token_match(&Token::LBracket) {
            self.next_token()?;
            let index = self.next_expr()?;
            self.match_token_preserve_newline(Token::RBracket)?;
            let end = self.lexer.pos();
            let range = Range::Src(SrcRange::new(begin.clone(), end));
            Ok(Expr::ArrayAccess {
                array: Box::new(expr),
                index: Box::new(index),
                range,
            })
        } else {
            Ok(expr)
        }
    }

    fn next_function(&mut self) -> Result<Fun> {
        let begin = self.lexer.pos();
        self.match_token(Token::FunKw)?;
        let name = self.next_bareword()?;
        let mut params = vec![];
        let mut return_ty = None;
        let mut defaults = false;
        self.match_token(Token::LParen)?;
        while !self.is_token_match(&Token::RParen) {
            let begin = self.lexer.pos();
            let param_name = self.next_variable()?;
            let mut ty = None;
            let mut default = None;
            if self.is_token_match(&Token::Colon) {
                self.match_token(Token::Colon)?;
                ty = Some(self.next_bareword()?);
            }

            if defaults || self.is_token_match(&Token::AssignOp(AssignOp::Equals)) {
                defaults = true;
                self.match_token(Token::AssignOp(AssignOp::Equals))?;
                default = Some(self.next_expr()?);
            }
            let end = self.lexer.pos();
            let range = Range::Src(SrcRange::new(begin, end));
            params.push(FunParam::new(param_name, ty, default, range));

            if !self.is_token_match(&Token::RParen) {
                self.match_token(Token::Comma)?;
            }
        }
        self.match_token(Token::RParen)?;
        if self.is_token_match(&Token::Colon) {
            self.next_token()?;
            return_ty = Some(self.next_bareword()?);
        }
        let body = self.next_body()?;
        let end = self.lexer.pos();
        let range = Range::Src(SrcRange::new(begin, end));
        Ok(Fun {
            name,
            params,
            return_ty,
            body,
            range,
        })
    }

    fn next_user_type(&mut self) -> Result<UserTy> {
        let begin = self.lexer.pos();

        self.match_token(Token::TypeKw)?;
        let name = self.next_bareword()?;

        let mut parents = Vec::new();
        if self.is_token_match(&Token::Colon) {
            self.next_token()?;
            // get comma separated list of "parent" types
            let parent = self.next_bareword()?;
            parents.push(parent);
            while self.is_token_match(&Token::Comma) {
                self.next_token()?;
                let parent = self.next_bareword()?;
                parents.push(parent);
            }
        }

        self.match_token(Token::LBrace)?;
        let mut functions = Vec::new();
        while self.is_lookahead::<Fun>() {
            let function = self.next_function()?;
            functions.push(function);

            while self.is_token_match(&Token::Comment) {
                self.next_token()?;
            }
        }
        self.match_token_preserve_newline(Token::RBrace)?;

        let end = self.lexer.pos();
        let range = Range::Src(SrcRange::new(begin, end));
        Ok(UserTy {
            name,
            parents,
            functions,
            range,
        })
    }

    fn next_funcall_args(&mut self) -> Result<Vec<Expr>> {
        self.match_token(Token::LParen)?;
        let mut args = vec![];
        if !self.is_token_match(&Token::RParen) {
            args.push(self.next_expr()?);
            while self.is_token_match(&Token::Comma) {
                self.next_token()?;
                args.push(self.next_expr()?);
            }
        }
        if self.stmt_level == 0 {
            self.match_token_preserve_newline(Token::RParen)?;
        } else {
            self.match_token(Token::RParen)?;
        }
        Ok(args)
    }

    fn next_variable(&mut self) -> Result<String> {
        if let Some(token) = self.next_token()? {
            match token.as_inner() {
                Token::Variable(var) => Ok(var.clone()),
                _ => Err(self.err_expected_got("variable", Some(&token))),
            }
        } else {
            Err(self.err_expected_got_eof("variable"))
        }
    }

    fn next_bareword(&mut self) -> Result<String> {
        if let Some(token) = self.next_token()? {
            match token.as_inner() {
                Token::Bareword(bareword) => Ok(bareword.clone()),
                _ => Err(self.err_expected_got("bareword", Some(&token))),
            }
        } else {
            Err(self.err_expected_got_eof("bareword"))
        }
    }

    fn next_op(&mut self) -> Result<Op> {
        let matches = if let Some(&Token::Op(_)) = self.curr.as_ref().map(|r| r.token()) {
            true
        } else {
            false
        };
        if matches {
            Ok(Token::from(self.next_token()?.unwrap()).into_op())
        } else {
            Err(self.err_expected_got("operator", self.curr.as_ref()))
        }
    }

    fn next_assign_op(&mut self) -> Result<AssignOp> {
        let matches = matches!(
            self.curr.as_ref().map(|r| r.token()),
            Some(&Token::AssignOp(_))
        );
        if matches {
            Ok(Token::from(self.next_token()?.unwrap()).into_assign_op())
        } else {
            Err(self.err_expected_got("operator", self.curr.as_ref()))
        }
    }

    fn is_curr_op(&self) -> bool {
        self.curr
            .as_ref()
            .map(|t| matches!(t.token(), &Token::Op(_)))
            .unwrap_or(false)
    }

    fn is_curr_assign_op(&self) -> bool {
        self.curr
            .as_ref()
            .map(|t| matches!(t.token(), &Token::AssignOp(_)))
            .unwrap_or(false)
    }

    fn is_any_token_match(&self, tokens: &[Token]) -> bool {
        tokens.iter().any(|t| self.is_token_match(t)) || (tokens.is_empty() && self.curr.is_none())
    }

    fn is_token_match(&self, token: &Token) -> bool {
        if let Some(ref curr) = self.curr {
            curr.token() == token
        } else {
            false
        }
    }

    fn is_lookahead<A: Ast>(&self) -> bool {
        if let Some(ref curr) = self.curr {
            curr.is_lookahead::<A>()
        } else {
            false
        }
    }

    fn match_token_preserve_newline(&mut self, token: Token) -> Result<RangedToken> {
        if self
            .curr
            .as_ref()
            .map(|r| r.token() == &token)
            .unwrap_or(false)
        {
            self.next_token_or_newline()?
                .ok_or_else(|| self.err_expected_got_eof(token.to_string()))
        } else {
            let expected = token.to_string();
            Err(self.err_expected_got(expected, self.curr.as_ref()))
        }
    }

    fn match_token(&mut self, token: Token) -> Result<RangedToken> {
        if self
            .curr
            .as_ref()
            .map(|r| r.token() == &token)
            .unwrap_or(false)
        {
            self.next_token()?
                .ok_or_else(|| self.err_expected_got_eof(token.to_string()))
        } else {
            let expected = token.to_string();
            Err(self.err_expected_got(expected, self.curr.as_ref()))
        }
    }

    /// Advances the lexer by one token, skipping over any comments as necessary.
    ///
    /// This method will not skip over newlines, and will instead return them as part of the normal
    /// token stream.
    fn next_token_or_newline(&mut self) -> Result<Option<RangedToken>> {
        let next = if let Some(result) = self.lexer.next() {
            Some(result?)
        } else {
            None
        };
        Ok(mem::replace(
            &mut self.curr,
            mem::replace(&mut self.next, next),
        ))
    }

    /// Advances the lexer by one token, skipping over any comments and newlines as necessary.
    ///
    /// This skips over newlines, since, *for the most part*, the language is newline-agnostic.
    /// Only statements are required to be ended with either newlines *or* line-end characters.
    fn next_token(&mut self) -> Result<Option<RangedToken>> {
        let mut token = self.next_token_or_newline()?;
        while self.is_token_match(&Token::Comment) {
            token = self.next_token_or_newline()?;
        }
        Ok(token)
    }

    /// Creates a new `Error` using the supplied expected item.
    /// # Args
    /// `expected` - the expected item.
    ///
    /// # Returns
    /// A `Error` with a synthesized error message based on the `expected` param, with the
    /// `got` message being an EOF.
    fn err_expected_got_eof<D: Display>(&self, expected: D) -> Error {
        self.err_expected_got(expected, Option::None::<D>)
    }

    /// Creates a new `Error` using the supplied expected item and the actual item
    /// encountered.
    ///
    /// # Args
    /// `expected` - the expected item.
    /// `got` - the item that was encountered in the parser.
    ///
    /// # Returns
    /// A `Error` with a synthesized error message based on the `expected` and `got` params.
    fn err_expected_got(&self, expected: impl Display, got: Option<impl Display>) -> Error {
        let got = got
            .map(|d| d.to_string())
            .unwrap_or_else(|| "EOF".to_string());
        self.err(ErrorKind::ExpectedGot(
            expected.to_string(),
            got,
            self.lexer.pos(),
        ))
    }

    /*
     * these two were only used for the self keyword which has been removed. I want to keep these
     * around just in case things change.
    fn err_unexpected(&self, what: impl ToString) -> Error {
        self.err(ErrorKind::Unexpected(what.to_string()))
    }

    /// Creates a new `Error` using the supplied message.
    ///
    /// # Args
    /// `message` - detailed error info.
    ///
    /// # Returns
    /// A `Error` with this parser's current position, as well the specified message.
    fn err_message(&self, message: impl ToString) -> Error {
        self.err(ErrorKind::Message(message.to_string()))
    }
    */

    fn err(&self, kind: ErrorKind) -> Error {
        Error::new(self.lexer.pos(), kind)
    }
}

#[cfg(test)]
mod test {
    use crate::common::lang::*;
    use crate::common::pos::*;
    use crate::syntax::token::*;
    use crate::syntax::tree::*;
    use crate::syntax::*;

    macro_rules! test_parser {
        ($input:expr) => {{
            let mut parser = Parser::new("test", $input);
            parser.init().unwrap();
            parser
        }};
    }

    macro_rules! token {
        ($($token:tt)+) => {
            RangedToken::new(
                Range::Src(SrcRange::new(Pos::default(),Pos::default())),
                $($token)+)
            }
    }

    #[test]
    fn test_parser_expr() {
        let mut parser = test_parser!("(1 + 2)");
        let expr = parser.next_expr().unwrap();
        assert_eq!(
            expr,
            Expr::Binary(
                Box::new(Expr::Atom(token!(Token::IntLit("1".to_string(), 10)))),
                Op::Plus,
                Box::new(Expr::Atom(token!(Token::IntLit("2".to_string(), 10))))
            )
        );
    }

    #[test]
    fn test_parser_match_token() {
        let mut parser = test_parser!("(1 + 2)");
        parser.match_token(Token::LParen).unwrap();
        parser
            .match_token(Token::IntLit("1".to_string(), 10))
            .unwrap();
        parser.match_token(Token::Op(Op::Plus)).unwrap();
        parser
            .match_token(Token::IntLit("2".to_string(), 10))
            .unwrap();
        parser.match_token(Token::RParen).unwrap();
    }
}
