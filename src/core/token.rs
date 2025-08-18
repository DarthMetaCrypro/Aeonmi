// src/core/token.rs
//! Core lexical items for Aeonmi/QUBE/Titan
//! Unified token definitions for standard, quantum, and hieroglyphic syntax.
//! Updated: 2025-08-10

#![allow(dead_code)]

/* ───────────────────────────────────────────────────────────
TOKEN ENUM
───────────────────────────────────────────────────────── */
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /* Keywords */
    Let,
    Function,
    Return,
    Log,
    Qubit,
    Superpose,
    Entangle,
    Measure,
    Dod,
    If,
    Else,
    For,
    While,
    In,

    /* Operators */
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

    /* Delimiters */
    Semicolon,  // ;
    Comma,      // ,
    OpenParen,  // (
    CloseParen, // )
    OpenBrace,  // {
    CloseBrace, // }

    /* Literals & identifiers */
    Identifier(String),
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    QubitLiteral(String),   // e.g. |0>, |ψ>, |+>, |->
    HieroglyphicOp(String), // e.g. 𓀀, 𓁀, 𓂀

    /* End of file */
    EOF,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        let s = match self {
            // keywords
            Let => "let",
            Function => "function",
            Return => "return",
            Log => "log",
            Qubit => "qubit",
            Superpose => "superpose",
            Entangle => "entangle",
            Measure => "measure",
            Dod => "dod",
            If => "if",
            Else => "else",
            For => "for",
            While => "while",
            In => "in",

            // operators
            Plus => "+",
            Minus => "-",
            Star => "*",
            Slash => "/",
            Equals => "=",
            DoubleEquals => "==",
            NotEquals => "!=",
            LessThan => "<",
            LessEqual => "<=",
            GreaterThan => ">",
            GreaterEqual => ">=",
            ColonEquals => ":=",
            Pipe => "|",

            // delimiters
            Semicolon => ";",
            Comma => ",",
            OpenParen => "(",
            CloseParen => ")",
            OpenBrace => "{",
            CloseBrace => "}",

            // literals & special
            Identifier(s) => return write!(f, "{s}"),
            NumberLiteral(_) => "number",
            StringLiteral(_) => "string",
            BooleanLiteral(true) => "true",
            BooleanLiteral(false) => "false",
            QubitLiteral(q) => return write!(f, "{q}"),
            HieroglyphicOp(sym) => return write!(f, "{sym}"),

            EOF => "EOF",
        };
        write!(f, "{s}")
    }
}

/* ───────────────────────────────────────────────────────────
TOKEN STRUCT
───────────────────────────────────────────────────────── */
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    #[inline]
    pub const fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }

    /* Classification helpers */
    pub fn is_keyword(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Let
                | TokenKind::Function
                | TokenKind::Return
                | TokenKind::Log
                | TokenKind::Qubit
                | TokenKind::Superpose
                | TokenKind::Entangle
                | TokenKind::Measure
                | TokenKind::Dod
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::For
                | TokenKind::While
                | TokenKind::In
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Equals
                | TokenKind::DoubleEquals
                | TokenKind::NotEquals
                | TokenKind::LessThan
                | TokenKind::LessEqual
                | TokenKind::GreaterThan
                | TokenKind::GreaterEqual
                | TokenKind::ColonEquals
                | TokenKind::Pipe
        )
    }

    pub fn is_delimiter(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Semicolon
                | TokenKind::Comma
                | TokenKind::OpenParen
                | TokenKind::CloseParen
                | TokenKind::OpenBrace
                | TokenKind::CloseBrace
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Identifier(_)
                | TokenKind::NumberLiteral(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::BooleanLiteral(_)
                | TokenKind::QubitLiteral(_)
        )
    }

    pub fn is_hieroglyphic(&self) -> bool {
        matches!(self.kind, TokenKind::HieroglyphicOp(_))
    }
}

/* Nice human-readable token printing */
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @{}:{}", self.kind, self.line, self.column)
    }
}

/* ───────────────────────────────────────────────────────────
TESTS
───────────────────────────────────────────────────────── */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_test() {
        let t = Token::new(TokenKind::Let, 1, 1);
        assert!(t.is_keyword());
        assert!(!t.is_operator());
        assert_eq!(format!("{t}"), "let @1:1");
    }

    #[test]
    fn literal_test() {
        let num = Token::new(TokenKind::NumberLiteral(42.0), 2, 5);
        assert!(num.is_literal());
        assert_eq!(format!("{num}"), "number @2:5");

        let s = Token::new(TokenKind::StringLiteral("hi".into()), 3, 2);
        assert!(s.is_literal());
        assert_eq!(format!("{s}"), "string @3:2");

        let b = Token::new(TokenKind::BooleanLiteral(true), 4, 4);
        assert!(b.is_literal());
        assert_eq!(format!("{b}"), "true @4:4");
    }

    #[test]
    fn qubit_test() {
        let q = Token::new(TokenKind::QubitLiteral("|ψ>".into()), 5, 1);
        assert!(q.is_literal());
        assert_eq!(format!("{q}"), "|ψ> @5:1");
    }

    #[test]
    fn hieroglyphic_test() {
        let h = Token::new(TokenKind::HieroglyphicOp("𓀀".into()), 6, 3);
        assert!(h.is_hieroglyphic());
        assert_eq!(format!("{h}"), "𓀀 @6:3");
    }

    #[test]
    fn comparison_ops_are_operators() {
        let le = Token::new(TokenKind::LessEqual, 1, 1);
        let ge = Token::new(TokenKind::GreaterEqual, 1, 1);
        assert!(le.is_operator());
        assert!(ge.is_operator());
        assert_eq!(format!("{}", le.kind), "<=");
        assert_eq!(format!("{}", ge.kind), ">=");
    }
}
