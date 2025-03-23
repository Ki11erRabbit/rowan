use std::collections::VecDeque;
use crate::ast::Text;

#[derive(Debug)]
pub struct SpannedToken<'a> {
    pub token: Token<'a>,
    pub start: usize,
    pub end: usize,
}

impl<'a> SpannedToken<'a> {
    pub fn new(token: Token<'a>, start: usize, end: usize) -> Self {
        Self { token, start, end }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    // Keywords
    // Control Flow
    If,
    Else,
    Match,
    Fun,
    Return,
    Break,
    Continue,
    While,
    For,
    In,
    Loop,
    With,
    As,
    // Data
    Record,
    Union,
    Into,
    AsA,
    // Classes
    Class,
    Signal,
    Static,
    At,
    Extends,
    Connect,
    Disconnect,
    Emit,
    New,
    To,
    Via,
    This,
    From,
    // Variables
    Let,
    Const,
    Mut,
    // Module
    Import,
    Module,
    // Visibility
    Pub,
    Prot,
    // Core Types
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Char,
    Bool,
    Void,
    Str,
    // Constants
    True,
    False,
    // Literals
    IntLiteral(Text<'a>),
    FloatLiteral(Text<'a>),
    CharLiteral(Text<'a>),
    StringLiteral(Text<'a>),
    // Identifiers
    Identifier(Text<'a>),
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Not,
    Or,
    And,
    Equals,
    NotEquals,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Assign,
    Concat,
    Scope,
    // Symbols
    BraceOpen,
    BraceClose,
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    AngleOpen,
    AngleClose,
    Comma,
    Colon,
    QMark,
    Bar,
    Dot,
    Arrow,
    /// This may be a semicolon or a newline
    LineBreak,
    Indent,
    /// Comment
    Comment,
    /// End of File
    Eof,
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Keywords
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Match => write!(f, "match"),
            Token::Fun => write!(f, "fun"),
            Token::Return => write!(f, "return"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Loop => write!(f, "loop"),
            Token::With => write!(f, "with"),
            Token::As => write!(f, "as"),
            Token::Record => write!(f, "record"),
            Token::Union => write!(f, "union"),
            Token::AsA => write!(f, "asa"),
            Token::Into => write!(f, "into"),
            Token::Class => write!(f, "class"),
            Token::Signal => write!(f, "signal"),
            Token::Static => write!(f, "static"),
            Token::At => write!(f, "@"),
            Token::Extends => write!(f, "extends"),
            Token::Connect => write!(f, "connect"),
            Token::Disconnect => write!(f, "disconnect"),
            Token::Emit => write!(f, "emit"),
            Token::New => write!(f, "new"),
            Token::To => write!(f, "to"),
            Token::Via => write!(f, "via"),
            Token::This => write!(f, "self"),
            Token::From => write!(f, "from"),
            Token::Let => write!(f, "let"),
            Token::Const => write!(f, "const"),
            Token::Mut => write!(f, "mut"),
            Token::Import => write!(f, "import"),
            Token::Module => write!(f, "module"),
            Token::Pub => write!(f, "pub"),
            Token::Prot => write!(f, "prot"),
            Token::U8 => write!(f, "u8"),
            Token::U16 => write!(f, "i16"),
            Token::U32 => write!(f, "i32"),
            Token::U64 => write!(f, "u64"),
            Token::I8 => write!(f, "i8"),
            Token::I16 => write!(f, "i16"),
            Token::I32 => write!(f, "i32"),
            Token::I64 => write!(f, "i64"),
            Token::F32 => write!(f, "f32"),
            Token::F64 => write!(f, "f64"),
            Token::Char => write!(f, "char"),
            Token::Bool => write!(f, "bool"),
            Token::Void => write!(f, "void"),
            Token::Str => write!(f, "str"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::IntLiteral(lit) => write!(f, "{}", lit),
            Token::FloatLiteral(lit) => write!(f, "{}", lit),
            Token::CharLiteral(lit) => write!(f, "{}", lit),
            Token::StringLiteral(lit) => write!(f, "{}", lit),
            Token::Identifier(ident) => write!(f, "{}", ident),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Modulo => write!(f, "%"),
            Token::Not => write!(f, "!"),
            Token::Or => write!(f, "||"),
            Token::And => write!(f, "&&"),
            Token::Equals => write!(f, "=="),
            Token::NotEquals => write!(f, "!="),
            Token::LessThanOrEqual => write!(f, "<="),
            Token::GreaterThanOrEqual => write!(f, ">="),
            Token::Assign => write!(f, "="),
            Token::Concat => write!(f, "++"),
            Token::Scope => write!(f, "::"),
            Token::BraceOpen => write!(f, "{{"),
            Token::BraceClose => write!(f, "}}"),
            Token::ParenOpen => write!(f, "("),
            Token::ParenClose => write!(f, ")"),
            Token::BracketOpen => write!(f, "["),
            Token::BracketClose => write!(f, "]"),
            Token::AngleOpen => write!(f, "<"),
            Token::AngleClose => write!(f, ">"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::QMark => write!(f, "?"),
            Token::Bar => write!(f, "|"),
            Token::Arrow => write!(f, "->"),
            Token::Dot => write!(f, "."),
            Token::LineBreak => write!(f, ";"),
            Token::Indent => write!(f, "    "),
            Token::Comment => write!(f, "/* ... */"),
            Token::Eof => write!(f, "end of file"),
        }
    }
}

#[derive(Debug)]
pub struct SpannedLexerError {
    pub error: LexerError,
    pub start: usize,
    pub end: usize,
}

impl SpannedLexerError {
    pub fn new(error: LexerError, start: usize, end: usize) -> Self {
        Self { error, start, end }
    }
}

impl std::fmt::Display for SpannedLexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}..{}] {}", self.start, self.end, self.error)
    }
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char),
    InvalidIdentifier(usize, usize),
    UnexpectedEndOfInput,
    UnclosedStringLiteral,
    UnclosedCharLiteral,
    UnclosedComment,
    UnknownError,
    ErrorCollection(Vec<SpannedLexerError>),
    InvalidOperator,
    Eof,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::UnexpectedCharacter(c) => write!(f, "Unexpected character '{}'", c),
            LexerError::InvalidIdentifier(start, end) => write!(f, "Invalid identifier from {} to {}", start, end),
            LexerError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            LexerError::UnclosedStringLiteral => write!(f, "Unclosed string literal"),
            LexerError::UnclosedCharLiteral => write!(f, "Unclosed char literal"),
            LexerError::UnclosedComment => write!(f, "Unclosed comment"),
            LexerError::UnknownError => write!(f, "Unknown error"),
            LexerError::ErrorCollection(errors) => {
                for error in errors {
                    writeln!(f, "{}", error)?;
                }
                Ok(())
            }
            LexerError::InvalidOperator => write!(f, "Invalid operator, expected either get[] or set[]"),
            LexerError::Eof => write!(f, "End of file"),
        }
    }
}

pub type LexerResult<'a> = Result<SpannedToken<'a>, SpannedLexerError>;

pub trait Lexer<'a> {
    fn next_token(&mut self) -> LexerResult<'a>;
    fn peek_token(&mut self) -> Result<&SpannedToken<'a>, SpannedLexerError>;
    fn collect_errors(&mut self, error: SpannedLexerError) -> SpannedLexerError;
}


pub struct TokenLexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    peak: Option<SpannedToken<'a>>,
}

impl<'a> TokenLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            peak: None,
        }
    }

    fn next_token_inner(&mut self) -> LexerResult<'a> {
        let (start, c) = match self.chars.next() {
            Some((start, c)) => (start, c),
            None => return Ok(SpannedToken::new(Token::Eof, 0, 0)),
        };

        match c {
            '@' => Ok(SpannedToken::new(Token::At, start, start + 1)),
            '+' => {
                if let Some((_, '+')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::Concat, start, end))
                } else {
                    Ok(SpannedToken::new(Token::Plus, start, start + 1))
                }
            }
            '-' => {
                if let Some((_, '>')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::Arrow, start, end))
                } else {
                    Ok(SpannedToken::new(Token::Minus, start, start + 1))
                }
            }
            '*' => Ok(SpannedToken::new(Token::Multiply, start, start + 1)),
            '/' => {
                if let Some((_, '/')) = self.chars.peek() {
                    let mut end = start + 2;
                    while let Some((_, c)) = self.chars.next() {
                        end += 1;
                        if c == '\n' {
                            break;
                        }
                    }
                    self.next_token_inner()
                } else if let Some((_, '*')) = self.chars.peek() {
                    let mut end = start + 2;
                    let mut found_star = false;
                    while let Some((_, c)) = self.chars.next() {
                        end += 1;
                        if c == '*' {
                            found_star = true;
                        } else if c == '/' && found_star {
                            break;
                        } else {
                            found_star = false;
                        }
                    }
                    if found_star {
                        self.next_token_inner()
                    } else {
                        Err(SpannedLexerError::new(LexerError::UnclosedComment, start, end))
                    }
                } else {
                    Ok(SpannedToken::new(Token::Divide, start, start + 1))
                }
            }
            '%' => Ok(SpannedToken::new(Token::Modulo, start, start + 1)),
            '!' => {
                if let Some((_, '=')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::NotEquals, start, end))
                } else {
                    Ok(SpannedToken::new(Token::Not, start, start + 1))
                }
            }
            '|' => {
                if let Some((_, '|')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::Or, start, end))
                } else {
                    Ok(SpannedToken::new(Token::Bar, start, start + 1))
                }
            }
            '&' => {
                if let Some((_, '&')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::And, start, end))
                } else {
                    Err(SpannedLexerError::new(LexerError::UnexpectedCharacter('&'), start, start + 1))
                }
            }
            '=' => {
                if let Some((_, '=')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::Equals, start, end))
                } else {
                    Ok(SpannedToken::new(Token::Assign, start, start + 1))
                }
            }
            '<' => {
                if let Some((_, '=')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::LessThanOrEqual, start, end))
                } else {
                    Ok(SpannedToken::new(Token::AngleOpen, start, start + 1))
                }
            }
            '>' => {
                if let Some((_, '=')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::GreaterThanOrEqual, start, end))
                } else {
                    Ok(SpannedToken::new(Token::AngleClose, start, start + 1))
                }
            }
            '(' => Ok(SpannedToken::new(Token::ParenOpen, start, start + 1)),
            ')' => Ok(SpannedToken::new(Token::ParenClose, start, start + 1)),
            '[' => Ok(SpannedToken::new(Token::BracketOpen, start, start + 1)),
            ']' => Ok(SpannedToken::new(Token::BracketClose, start, start + 1)),
            '{' => Ok(SpannedToken::new(Token::BraceOpen, start, start + 1)),
            '}' => Ok(SpannedToken::new(Token::BraceClose, start, start + 1)),
            ',' => Ok(SpannedToken::new(Token::Comma, start, start + 1)),
            ':' => {
                if let Some((_, ':')) = self.chars.peek() {
                    let Some((end, _)) = self.chars.next() else {
                        panic!("Unexpected end of input");
                    };
                    Ok(SpannedToken::new(Token::Scope, start, end))
                } else {
                    Ok(SpannedToken::new(Token::Colon, start, start + 1))
                }
            }
            '?' => Ok(SpannedToken::new(Token::QMark, start, start + 1)),
            ';' => {
                let mut end = start + 1;
                while let Some((_, c)) = self.chars.peek() {
                    if (c.is_whitespace() && (*c == '\n' || *c == '\r')) || *c == ';' {
                        end += 1;
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                Ok(SpannedToken::new(Token::LineBreak, start, end))
            }
            '.' => Ok(SpannedToken::new(Token::Dot, start, start + 1)),
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut end = start + 1;
                while let Some((_, c)) = self.chars.peek() {
                    if c.is_ascii_alphanumeric() || *c == '_' || *c == '-' {
                        end += 1;
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                while let Some((_, '\'')) = self.chars.peek() {
                    end += 1;
                    self.chars.next();
                }
                let ident = &self.input[start..end];

                match ident {
                    "if" => Ok(SpannedToken::new(Token::If, start, end)),
                    "else" => Ok(SpannedToken::new(Token::Else, start, end)),
                    "match" => Ok(SpannedToken::new(Token::Match, start, end)),
                    "fn" => Ok(SpannedToken::new(Token::Fun, start, end)),
                    "return" => Ok(SpannedToken::new(Token::Return, start, end)),
                    "break" => Ok(SpannedToken::new(Token::Break, start, end)),
                    "continue" => Ok(SpannedToken::new(Token::Continue, start, end)),
                    "while" => Ok(SpannedToken::new(Token::While, start, end)),
                    "for" => Ok(SpannedToken::new(Token::For, start, end)),
                    "in" => Ok(SpannedToken::new(Token::In, start, end)),
                    "with" => Ok(SpannedToken::new(Token::With, start, end)),
                    "as" => Ok(SpannedToken::new(Token::As, start, end)),
                    "record" => Ok(SpannedToken::new(Token::Record, start, end)),
                    "union" => Ok(SpannedToken::new(Token::Union, start, end)),
                    "asa" => Ok(SpannedToken::new(Token::AsA, start, end)),
                    "into" => Ok(SpannedToken::new(Token::Into, start, end)),
                    "class" => Ok(SpannedToken::new(Token::Class, start, end)),
                    "signal" => Ok(SpannedToken::new(Token::Signal, start, end)),
                    "static" => Ok(SpannedToken::new(Token::Static, start, end)),
                    "extends" => Ok(SpannedToken::new(Token::Extends, start, end)),
                    "connect" => Ok(SpannedToken::new(Token::Connect, start, end)),
                    "disconnect" => Ok(SpannedToken::new(Token::Disconnect, start, end)),
                    "emit" => Ok(SpannedToken::new(Token::Emit, start, end)),
                    "new" => Ok(SpannedToken::new(Token::New, start, end)),
                    "to" => Ok(SpannedToken::new(Token::To, start, end)),
                    "via" => Ok(SpannedToken::new(Token::Via, start, end)),
                    "self" => Ok(SpannedToken::new(Token::This, start, end)),
                    "from" => Ok(SpannedToken::new(Token::From, start, end)),
                    "let" => Ok(SpannedToken::new(Token::Let, start, end)),
                    "const" => Ok(SpannedToken::new(Token::Const, start, end)),
                    "mut" => Ok(SpannedToken::new(Token::Mut, start, end)),
                    "import" => Ok(SpannedToken::new(Token::Import, start, end)),
                    "module" => Ok(SpannedToken::new(Token::Module, start, end)),
                    "pub" => Ok(SpannedToken::new(Token::Pub, start, end)),
                    "prot" => Ok(SpannedToken::new(Token::Prot, start, end)),
                    "u8" => Ok(SpannedToken::new(Token::U8, start, end)),
                    "u16" => Ok(SpannedToken::new(Token::U16, start, end)),
                    "u32" => Ok(SpannedToken::new(Token::U32, start, end)),
                    "u64" => Ok(SpannedToken::new(Token::U64, start, end)),
                    "i8" => Ok(SpannedToken::new(Token::I8, start, end)),
                    "i16" => Ok(SpannedToken::new(Token::I16, start, end)),
                    "i32" => Ok(SpannedToken::new(Token::I32, start, end)),
                    "i64" => Ok(SpannedToken::new(Token::I64, start, end)),
                    "f32" => Ok(SpannedToken::new(Token::F32, start, end)),
                    "f64" => Ok(SpannedToken::new(Token::F64, start, end)),
                    "char" => Ok(SpannedToken::new(Token::Char, start, end)),
                    "bool" => Ok(SpannedToken::new(Token::Bool, start, end)),
                    "void" => Ok(SpannedToken::new(Token::Void, start, end)),
                    "str" => Ok(SpannedToken::new(Token::Str, start, end)),
                    "True" => Ok(SpannedToken::new(Token::True, start, end)),
                    "False" => Ok(SpannedToken::new(Token::False, start, end)),
                    _ => {
                        // Check that the identifier doesn't end with a dash (-)
                        // Check that the identifier doesn't end with a dash then a number
                        let mut chars = ident.chars().rev();
                        let mut queue = VecDeque::new();
                        while let Some(c) = chars.next() {
                            if c == '-' {
                                if queue.is_empty() {
                                    return Err(SpannedLexerError::new(LexerError::InvalidIdentifier(end - 1, end), start, end));
                                }
                                let string = queue.iter().collect::<String>();
                                if string.parse::<u64>().is_ok() && string.parse::<f64>().is_ok() && string.chars().all(|c| c.is_numeric()) {
                                    return Err(SpannedLexerError::new(LexerError::InvalidIdentifier(end - queue.len(), end), start, end));
                                }
                                break;
                            }
                            if c.is_numeric() {
                                queue.push_front(c);
                            } else {
                                break;
                            }
                        }
                        Ok(SpannedToken::new(Token::Identifier(Text::Borrowed(ident)), start, end))
                    }
                }
            }
            '0'..='9' => {
                let mut end = start + 1;
                while let Some((_, c)) = self.chars.peek() {
                    if c.is_ascii_digit() {
                        end += 1;
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                if let Some((_, '.')) = self.chars.peek() {
                    end += 1;
                    self.chars.next();
                    while let Some((_, c)) = self.chars.peek() {
                        if c.is_ascii_digit() {
                            end += 1;
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    let lit = &self.input[start..end];
                    Ok(SpannedToken::new(Token::FloatLiteral(Text::Borrowed(lit)), start, end))
                } else {
                    let lit = &self.input[start..end];
                    Ok(SpannedToken::new(Token::IntLiteral(Text::Borrowed(lit)), start, end))
                }
            }
            '\'' => {
                let mut end = start + 1;
                let mut found_backslash = false;
                let mut found_end = false;
                while let Some((_, c)) = self.chars.peek() {
                    let c = *c;
                    end += 1;
                    self.chars.next();
                    if c == '\'' && !found_backslash {
                        found_end = true;
                        break;
                    } else if c == '\\' {
                        found_backslash = true;
                    } else if found_backslash {
                        found_backslash = false;
                    }
                }
                if !found_end {
                    return Err(SpannedLexerError::new(LexerError::UnclosedCharLiteral, start, end));
                }
                let lit = &self.input[start + 1..end - 1];
                Ok(SpannedToken::new(Token::CharLiteral(Text::Borrowed(lit)), start, end))
            }
            '"' => {
                let mut end = start + 1;
                let mut found_backslash = false;
                let mut found_end = false;
                while let Some((_, c)) = self.chars.peek() {
                    let c = *c;
                    end += 1;
                    self.chars.next();
                    if c == '"' && !found_backslash {
                        found_end = true;
                        break;
                    } else if c == '\\' {
                        found_backslash = true;
                    } else if found_backslash {
                        found_backslash = false;
                    }
                }
                if !found_end {
                    return Err(SpannedLexerError::new(LexerError::UnclosedStringLiteral, start, end));
                }
                let lit = &self.input[start + 1..end - 1];
                Ok(SpannedToken::new(Token::StringLiteral(Text::Borrowed(lit)), start, end))
            }
            '\n' => {
                let mut end = start + 1;
                while let Some((_, c)) = self.chars.peek() {
                    if (c.is_whitespace() && (*c == '\n' || *c == '\r')) || *c == ';' {
                        end += 1;
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                Ok(SpannedToken::new(Token::LineBreak, start, end))
            }
            c if c.is_whitespace() => {
                let mut end = start + 1;
                while let Some((_, c)) = self.chars.peek() {
                    if c.is_whitespace() {
                        self.chars.next();
                        end += 1;
                    } else {
                        break;
                    }
                }
                //Ok(SpannedToken::new(Token::Indent, start, end))
                self.next_token_inner()
            }
            _ => Err(SpannedLexerError::new(LexerError::UnknownError, start, start + 1)),
        }
    }
}

impl<'a> Lexer<'a> for TokenLexer<'a> {
    fn next_token(&mut self) -> LexerResult<'a> {
        self.next_token_inner()
    }

    fn peek_token(&mut self) -> Result<&SpannedToken<'a>, SpannedLexerError> {
        if self.peak.is_some() {
            Ok(self.peak.as_ref().unwrap())
        } else {
            let token = self.next_token_inner()?;
            self.peak = Some(token);
            Ok(self.peak.as_ref().unwrap())
        }
    }

    fn collect_errors(&mut self, error: SpannedLexerError) -> SpannedLexerError {
        let mut errors = Vec::new();
        errors.push(error);
        loop {
            match self.next_token_inner() {
                Ok(token) => {
                    if token.token == Token::Eof {
                        break;
                    }
                }
                Err(err) => errors.push(err),
            }
        }
        SpannedLexerError::new(LexerError::ErrorCollection(errors), 0, 0)
    }
}


impl<'a> Iterator for TokenLexer<'a> {
    type Item = Result<(usize, Token<'a>, usize), SpannedLexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(token) => {
                if token.token == Token::Eof {
                    None
                } else {
                    Some(Ok((token.start, token.token, token.end)))
                }
            }
            Err(err) => {
                match err.error {
                    LexerError::Eof => None,
                    _ => {
                        Some(Err(self.collect_errors(err)))
                    }
                }

            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let input = "if else match fn return break continue while for in record union class signal static extends connect disconnect let const import module pub u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char bool True False";

        let mut lexer = TokenLexer::new(input);
        let expected = vec![
            Token::If,
            Token::Else,
            Token::Match,
            Token::Fun,
            Token::Return,
            Token::Break,
            Token::Continue,
            Token::While,
            Token::For,
            Token::In,
            Token::Record,
            Token::Union,
            Token::Class,
            Token::Signal,
            Token::Static,
            Token::Extends,
            Token::Connect,
            Token::Disconnect,
            Token::Let,
            Token::Const,
            Token::Import,
            Token::Module,
            Token::Pub,
            Token::U8,
            Token::U16,
            Token::U32,
            Token::U64,
            Token::I8,
            Token::I16,
            Token::I32,
            Token::I64,
            Token::F32,
            Token::F64,
            Token::Char,
            Token::Bool,
            Token::True,
            Token::False
        ];

        for token in expected {
            let result = lexer.next_token().unwrap();
            assert_eq!(result.token, token);
        }
    }

    #[test]
    fn test_operators() {
        let input = "+ - * / % ! || && == != <= >= = ++ ::";
        let mut lexer = TokenLexer::new(input);
        let expected = vec![
            Token::Plus,
            Token::Minus,
            Token::Multiply,
            Token::Divide,
            Token::Modulo,
            Token::Not,
            Token::Or,
            Token::And,
            Token::Equals,
            Token::NotEquals,
            Token::LessThanOrEqual,
            Token::GreaterThanOrEqual,
            Token::Assign,
            Token::Concat,
            Token::Scope,
        ];
        for token in expected {
            let result = lexer.next_token().unwrap();
            assert_eq!(result.token, token);
        }
    }

    #[test]
    fn test_symbols() {
        let input = "{ } ( ) [ ] < > , : ? | . ;";
        let mut lexer = TokenLexer::new(input);
        let expected = vec![
            Token::BraceOpen,
            Token::BraceClose,
            Token::ParenOpen,
            Token::ParenClose,
            Token::BracketOpen,
            Token::BracketClose,
            Token::AngleOpen,
            Token::AngleClose,
            Token::Comma,
            Token::Colon,
            Token::QMark,
            Token::Bar,
            Token::Dot,
            Token::LineBreak,
        ];
        for token in expected {
            let result = lexer.next_token().unwrap();
            assert_eq!(result.token, token);
        }
    }

    #[test]
    fn test_literals() {
        let input = "123 123.456 'a' \"hello\"";
        let mut lexer = TokenLexer::new(input);
        let expected = vec![
            Token::IntLiteral(Text::Borrowed("123")),
            Token::FloatLiteral(Text::Borrowed("123.456")),
            Token::CharLiteral(Text::Borrowed("a")),
            Token::StringLiteral(Text::Borrowed("hello")),
        ];
        for token in expected {
            let result = lexer.next_token().unwrap();
            assert_eq!(result.token, token);
        }
    }

    #[test]
    fn test_identifiers() {
        let input = "abc _abc abc123 _abc123 abc' abc-abc abc-a2' abc''";
        let mut lexer = TokenLexer::new(input);
        let expected = vec![
            Token::Identifier(Text::Borrowed("abc")),
            Token::Identifier(Text::Borrowed("_abc")),
            Token::Identifier(Text::Borrowed("abc123")),
            Token::Identifier(Text::Borrowed("_abc123")),
            Token::Identifier(Text::Borrowed("abc'")),
            Token::Identifier(Text::Borrowed("abc-abc")),
            Token::Identifier(Text::Borrowed("abc-a2'")),
            Token::Identifier(Text::Borrowed("abc''")),
        ];
        for token in expected {
            let result = lexer.next_token().unwrap();
            assert_eq!(result.token, token);
        }
    }

    #[test]
    fn test_line_breaks() {
        let input = ";\n; \n; \r; \r\n; \n\r; \n\n; \r\r; \r\n\r\n; \n\n\n; \r\r\r; \r\n\r\n\r\n; \n\n\n\n; \r\r\r\r; \r\n\r\n\r\n\r\n;";
        let mut lexer = TokenLexer::new(input);
        let expected = vec![
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            Token::LineBreak,
            ];

        for token in expected {
            let result = lexer.next_token().unwrap();
            assert_eq!(result.token, token);
        }
    }

    #[test]
    fn test_errors() {
        let input = "abc-123 abc-123-";
        let mut lexer = TokenLexer::new(input);
        for _ in 0..2 {
            let result = lexer.next_token();
            println!("{:?}", result);
            assert!(result.is_err());
        }
    }
}

