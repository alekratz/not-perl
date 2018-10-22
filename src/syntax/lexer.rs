use std::{
    mem,
    str::Chars,
};
use syntax::{
    Pos,
    Range,
    Result,
    SyntaxError,
    token::*,
};

/// A named character class with a predicate to check against.
///
/// The generic function type that comes with this type is cumbersome to type, so this type should
/// not be used directly. Use the `CharClass` type alias instead.
struct CharClassBase<F>(&'static str, F) where F: Fn(char) -> bool;

impl<F> CharClassBase<F>
    where F: Fn(char) -> bool
{
    /// Determines whether the supplied character matches this character class.
    ///
    /// # Arguments
    /// `c` - the character to test.
    fn is_match(&self, c: char) -> bool {
        (self.1)(c)
    }

    /// The human-readable name of this character class.
    fn name(&self) -> &'static str {
        self.0
    }
}

type CharClass = CharClassBase<fn(char) -> bool>;

macro_rules! char_class {
    ($class:ident, $name:expr, $func:expr) => {
        const $class: CharClass = CharClassBase($name, $func);
    };
}

char_class!(VARIABLE_NAME_CHARS, "variable name", |c| { c.is_alphanumeric() || "_-".contains(c) });
char_class!(OP_CHARS, "operator", |c| { "|&=+-*/~!@%^&?<>".contains(c) });
char_class!(BAREWORD_START_CHARS, "bareword", |c| { c.is_alphabetic() });
char_class!(BAREWORD_CHARS, "bareword", |c| { c.is_alphanumeric() || "_-".contains(c) });
char_class!(STR_LIT_ESCAPE_CHARS, "string escape", |c| { "trn\"\\".contains(c) });

/// A lexer, which converts a stream of characters into a stream of tokens.
pub struct Lexer<'n> {
    input: Chars<'n>,

    curr: Option<char>,
    next: Option<char>,
    pos: Pos<'n>,
}

impl<'n> Lexer<'n> {
    /// Creates a new lexer with the specified input and source name.
    pub fn new(source_name: &'n str, source_text: &'n str) -> Self {
        let mut input = source_text.chars();
        let next = input.next();
        Lexer {
            input,
            curr: None,
            next,
            pos: Pos::new(Some(source_name), source_text),
        }
    }

    /// The position that the lexer is currently looking at.
    pub fn pos(&self) -> Pos<'n> {
        self.pos
    }

    /// Gets the next token in this stream, resulting in an error if an unexpected character is
    /// encountered.
    fn next_token(&mut self) -> Option<Result<'n, Token>> {
        match self.next_char()? {
            '#' => Some(self.next_comment()),
            '$' => Some(self.next_variable_token()),
            '"' => Some(self.next_str_lit()),
            '(' => Some(Ok(Token::LParen)),
            ')' => Some(Ok(Token::RParen)),
            '{' => Some(Ok(Token::LBrace)),
            '}' => Some(Ok(Token::RBrace)),
            '[' => Some(Ok(Token::LBracket)),
            ']' => Some(Ok(Token::RBracket)),
            ';' => Some(Ok(Token::LineEnd)),
            '\n' => Some(Ok(Token::NewLine)),
            ',' => Some(Ok(Token::Comma)),
            ':' => Some(Ok(Token::Colon)),
            '0' ... '9' => Some(self.next_numeric_token()),
            e if OP_CHARS.is_match(e) => Some(self.next_op_token()),
            e if BAREWORD_START_CHARS.is_match(e) => Some(self.next_bareword()),
            e if e.is_whitespace() => {
                while let Some(c) = self.next {
                    if !c.is_whitespace() {
                        break;
                    }
                    self.next_char();
                }
                // recursion is guaranteed to be only one layer deep
                self.next_token()
            }
            e => Some(Err(SyntaxError::new(format!("unexpected character: {:?}", e), self.pos))),
        }
    }

    /// Gets the next comment token.
    ///
    /// # Preconditions
    /// `self.curr` must be the line-comment start character `#`.
    fn next_comment(&mut self) -> Result<'n, Token> {
        assert_eq!(self.curr, Some('#'), "precondition failed");

        while let Some(c) = self.next_char() {
            if c == '\n' {
                break;
            }
        }
        Ok(Token::Comment)
    }

    /// Gets the next variable token.
    ///
    /// # Preconditions
    /// `self.curr` must be the variable sigil character `$`.
    fn next_variable_token(&mut self) -> Result<'n, Token> {
        assert_eq!(self.curr, Some('$'), "precondition failed");
        let mut var_name = String::new();
        var_name.push(self.next_char_expect(&VARIABLE_NAME_CHARS)?);
        while let Some(c) = self.next {
            if VARIABLE_NAME_CHARS.is_match(c) {
                var_name.push(c);
                self.next_char();
            } else {
                break;
            }
        }
        Ok(Token::Variable(var_name))
    }

    /// Gets the next operator token.
    ///
    /// # Preconditions
    /// `self.curr` must match the OP_CHARS character class.
    fn next_op_token(&mut self) -> Result<'n, Token> {
        assert!(OP_CHARS.is_match(self.curr.expect("precondition failed")));
        let mut op = String::new();
        op.push(self.curr.unwrap());
        while let Some(c) = self.next {
            if OP_CHARS.is_match(c) {
                op.push(c);
                self.next_char();
            } else {
                break;
            }
        }
        if let Some(assign_op) = AssignOp::from_str(&op) {
            Ok(Token::AssignOp(assign_op))
        } else {
            Ok(Token::Op(op.into()))
        }
    }

    /// Gets the next string literal token.
    ///
    /// # Preconditions
    /// `self.curr` must be the double quote character `"`.
    fn next_str_lit(&mut self) -> Result<'n, Token> {
        assert_eq!(self.curr, Some('"'), "precondition failed");
        let mut str_lit = String::new();
        loop {
            match self.next_char() {
                Some('\\') => match self.next_char_expect(&STR_LIT_ESCAPE_CHARS)? {
                    't' => str_lit.push('\t'),
                    'n' => str_lit.push('\n'),
                    'r' => str_lit.push('\r'),
                    '"' => str_lit.push('\"'),
                    '\\' => str_lit.push('\\'),
                    _ => unreachable!(),
                }
                Some('"') => break Ok(Token::StrLit(str_lit)),
                Some('\n') | Some('\r') =>
                    break Err(SyntaxError::new("reached newline while inside of string literal".to_string(), self.pos)),
                None => break Err(SyntaxError::new("reached EOF while inside of string literal".to_string(), self.pos)),
                Some(c) => str_lit.push(c),
            }
        }
    }

    /// Gets the next bareword token.
    ///
    /// # Preconditions
    /// `self.curr` must match the BAREWORD_START_CHARS character class.
    fn next_bareword(&mut self) -> Result<'n, Token> {
        assert!(BAREWORD_START_CHARS.is_match(self.curr.expect("precondition failed")), "precondition failed");
        let mut bareword = String::new();
        bareword.push(self.curr.unwrap());
        while let Some(c) = self.next {
            if BAREWORD_CHARS.is_match(c) {
                bareword.push(c);
                self.next_char();
            } else {
                break;
            }
        }

        // allow barewords to end with a question mark
        if let Some('?') = self.next {
            bareword.push('?');
            self.next_char();
        }

        match bareword.as_str() {
            "if" => Ok(Token::IfKw),
            "else" => Ok(Token::ElseKw),
            "while" => Ok(Token::WhileKw),
            "loop" => Ok(Token::LoopKw),
            "continue" => Ok(Token::ContinueKw),
            "break" => Ok(Token::BreakKw),
            "true" => Ok(Token::TrueKw),
            "false" => Ok(Token::FalseKw),
            "fun" => Ok(Token::FunKw),
            "return" => Ok(Token::ReturnKw),
            "type" => Ok(Token::TypeKw),
            "self" => Ok(Token::SelfKw),
            _ => Ok(Token::Bareword(bareword))
        }
    }

    /// Gets the next numeric token.
    ///
    /// # Preconditions
    /// `self.curr` must be a character from `'0'` to `'9'`.
    fn next_numeric_token(&mut self) -> Result<'n, Token> {
        assert!({ let c = self.curr.unwrap(); c >= '0' && c <= '9'}, "precondition failed");
        let mut number = String::new();

        let mut is_float = false;

        // select radix
        let radix: usize = if self.curr == Some('0') {
            match self.next {
                Some('x') => 16,
                Some('o') => 8,
                Some('b') => 2,
                _ => {
                    number.push(self.curr.unwrap());
                    10
                },
            }
        } else {
            number.push(self.curr.unwrap());
            10
        };

        // skip past the radix character, if necessary
        if radix != 10 {
            assert!("xob".contains(self.next.unwrap()));
            self.next_char();
        }

        while let Some(c) = self.next {
            if c == '.' {
                if radix != 10 {
                    return Err(SyntaxError::new("non-base-ten floating point literals are not supported".to_string(), self.pos));
                } else if is_float {
                    return Err(SyntaxError::new("second decimal encountered in floating point literal".to_string(), self.pos));
                } else {
                    number.push('.');
                    is_float = true;
                }
            } else if c.is_digit(radix as u32) {
                number.push(c);
            } else if c.is_alphanumeric() {
                return Err(SyntaxError::new(format!("unrecognized digit {:?}", c), self.pos));
            } else {
                break;
            }
            self.next_char();
        }

        if is_float {
            return Ok(Token::FloatLit(number));
        } else {
            return Ok(Token::IntLit(number, radix));
        }
    }

    /// Updates the next character in the lexer's stream, with the expectation that it will be a
    /// match in the given character class.
    ///
    /// # Arguments
    /// `char_class` - the character class that the next character in the stream should be.
    fn next_char_expect(&mut self, char_class: &CharClass) -> Result<'n, char> {
        match self.next_char() {
            Some(c) => {
                if char_class.is_match(c) {
                    Ok(c)
                } else {
                    Err(SyntaxError::new(format!("expected {} char, but got {:?} instead", char_class.name(), c), self.pos))
                }
            },
            None => Err(SyntaxError::new(format!("expected {} char, but got EOF instead", char_class.name()), self.pos)),
        }
    }

    /// Updates the next character in the lexer's stream, updating the current position in the
    /// source.
    ///
    /// # Returns
    /// The previous "current character" that has just been replaced.
    fn next_char(&mut self) -> Option<char> {
        let old = mem::replace(&mut self.curr, mem::replace(&mut self.next, self.input.next()));
        if let Some(c) = old {
            self.pos.adv();
            if c == '\n' {
                self.pos.line();
            }
        }
        self.curr.clone()
    }
}

impl<'n> Iterator for Lexer<'n> {
    type Item = Result<'n, RangeToken<'n>>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos;
        let token = self.next_token();
        let end = self.pos;
        // next_token returns Option<Result<Token>>, we need O<R<RangeToken>>
        token.map(|r| r.map(|t| RangeToken::new(Range::new(start, end), t)))
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    /// Creates a new lexer with the given input string with a source name of "testing".
    macro_rules! test_lexer {
        ($input:expr) => {{ Lexer::new($input.chars(), "testing") }};
    }

    /// Gets the first token from the given string.
    ///
    /// Utility macro for testing.
    macro_rules! first_token {
        ($input:expr) => {{ 
            let mut _lexer = test_lexer!($input);
            _lexer.next_token().unwrap().unwrap()
        }};
    }

    #[test]
    fn test_lexer_variable() {
        let boop = first_token!("$boop");
        assert_eq!(boop, Token::Variable(String::from("boop")));

        let one = first_token!("$1");
        assert_eq!(one, Token::Variable(String::from("1")));

        let unicode = first_token!("$中文");
        assert_eq!(unicode, Token::Variable(String::from("中文")));
    }

    #[test]
    fn test_lexer_comment() {
        let comment = first_token!("# this is a single line comment");
        assert_eq!(comment, Token::Comment);
    }

    #[test]
    fn test_lexer_op() {

        let op = first_token!("+");
        assert_eq!(op, Token::Op(Op::Plus));
        let op = first_token!("-");
        assert_eq!(op, Token::Op(Op::Minus));

        let op = first_token!("*");
        assert_eq!(op, Token::Op(Op::Splat));

        let op = first_token!("/");
        assert_eq!(op, Token::Op(Op::FSlash));

        let op = first_token!("~");
        assert_eq!(op, Token::Op(Op::Tilde));

        let op = first_token!("==");
        assert_eq!(op, Token::Op(Op::DoubleEquals));

        let op = first_token!("%%");
        assert_eq!(op, Token::Op(Op::DoublePercent));


        let double_tilde = first_token!("~~");
        assert_eq!(double_tilde, Token::Op(Op::DoubleTilde));

        let very_long_op = first_token!("/<+~-~+>/");
        assert_eq!(very_long_op, Token::Op(Op::Custom("/<+~-~+>/".to_string())));
    }

    #[test]
    fn test_lexer_str_lit() {
        let africa = first_token!("\"hurry boy, she's waiting there for you\"");
        assert_eq!(africa, Token::StrLit(String::from("hurry boy, she's waiting there for you")));

        let boxer = first_token!(r#""\"I am leaving, I am leaving,\" but the fighter still remains""#);
        assert_eq!(boxer, Token::StrLit(String::from("\"I am leaving, I am leaving,\" but the fighter still remains")));
    }

    #[test]
    fn test_lexer_bareword() {
        let boop = first_token!("boop");
        assert_eq!(boop, Token::Bareword("boop".to_string()));

        let numeric = first_token!("b00p");
        assert_eq!(numeric, Token::Bareword("b00p".to_string()));

        let ifkw = first_token!("if");
        assert_eq!(ifkw, Token::IfKw);

        let elsekw = first_token!("else");
        assert_eq!(elsekw, Token::ElseKw);

        let whilekw = first_token!("while");
        assert_eq!(whilekw, Token::WhileKw);

        let loopkw = first_token!("loop");
        assert_eq!(loopkw, Token::LoopKw);

        let continuekw = first_token!("continue");
        assert_eq!(continuekw, Token::ContinueKw);
    }

    #[test]
    fn test_lexer_numerics() {
        let int10 = first_token!("199");
        assert_eq!(int10, Token::IntLit("199".to_string(), 10));

        let zero_int10 = first_token!("0000");
        assert_eq!(zero_int10, Token::IntLit("0000".to_string(), 10));

        let int8 = first_token!("0o777");
        assert_eq!(int8, Token::IntLit("777".to_string(), 8));

        let int16 = first_token!("0xdecafdad");
        assert_eq!(int16, Token::IntLit("decafdad".to_string(), 16));

        let float = first_token!("0.0");
        assert_eq!(float, Token::FloatLit("0.0".to_string()));
    }

    #[test]
    #[should_panic]
    fn test_lexer_str_lit_newline() {
        first_token!("\"\n\"");
    }

    #[test]
    #[should_panic]
    fn test_lexer_str_lit_unclosed() {
        first_token!("\"unclosed string");
    }
}
*/
