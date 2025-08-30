// src/core/token.rs
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Identifiers and literals
    Identifier(String),
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    QubitLiteral(String),
    
    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Equals,       // =
    DoubleEquals, // ==
    NotEquals,    // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    ColonEquals,  // :=
    Pipe,         // |
    AndAnd,       // &&
    OrOr,         // ||
    
    // Delimiters
    OpenParen,    // (
    CloseParen,   // )
    OpenBrace,    // {
    CloseBrace,   // }
    Comma,        // ,
    Semicolon,    // ;
    
    // Keywords
    Function,
    Let,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Log,
    Qubit,
    
    // Quantum operations
    Superpose,
    Entangle,
    Measure,
    Dod,
    
    // Hieroglyphic operations
    HieroglyphicOp(String),
    
    // Special
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }
}

// Implement Display for TokenKind for better error messages
impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            TokenKind::Identifier(_) => "identifier",
            TokenKind::NumberLiteral(_) => "number",
            TokenKind::StringLiteral(_) => "string",
            TokenKind::BooleanLiteral(_) => "boolean",
            TokenKind::QubitLiteral(_) => "qubit",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Equals => "=",
            TokenKind::DoubleEquals => "==",
            TokenKind::NotEquals => "!=",
            TokenKind::LessThan => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::GreaterThan => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::ColonEquals => ":=",
            TokenKind::Pipe => "|",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",
            TokenKind::OpenParen => "(",
            TokenKind::CloseParen => ")",
            TokenKind::OpenBrace => "{",
            TokenKind::CloseBrace => "}",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Function => "function",
            TokenKind::Let => "let",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::Return => "return",
            TokenKind::Log => "log",
            TokenKind::Qubit => "qubit",
            TokenKind::Superpose => "superpose",
            TokenKind::Entangle => "entangle",
            TokenKind::Measure => "measure",
            TokenKind::Dod => "dod",
            TokenKind::HieroglyphicOp(_) => "hieroglyphic",
            TokenKind::EOF => "end of file",
        };
        write!(f, "{}", name)
    }
}

// Implement Display for full Token (kind plus optional lexeme snippet)
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TokenKind::Identifier(name) => write!(f, "Identifier('{}') @{}:{}", name, self.line, self.column),
            TokenKind::NumberLiteral(v) => write!(f, "Number({}) @{}:{}", v, self.line, self.column),
            TokenKind::StringLiteral(s) => write!(f, "String(\"{}\") @{}:{}", s, self.line, self.column),
            TokenKind::BooleanLiteral(b) => write!(f, "Boolean({}) @{}:{}", b, self.line, self.column),
            TokenKind::QubitLiteral(q) => write!(f, "Qubit({}) @{}:{}", q, self.line, self.column),
            TokenKind::HieroglyphicOp(sym) => write!(f, "Hieroglyphic('{}') @{}:{}", sym, self.line, self.column),
            other => write!(f, "{} @{}:{}", other, self.line, self.column),
        }
    }
}