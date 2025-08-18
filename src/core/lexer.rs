// src/core/lexer.rs
//! Unified Lexer for Aeonmi + Q.U.B.E.
//! Integrates standard syntax, quantum syntax, and hieroglyphics.
//! Updated: 2025-08-10 (DoubleEquals, <=, >=, multi-line comments, BOM handling)

use crate::core::token::{Token, TokenKind};
use std::iter::Peekable;
use std::str::Chars;

/// Errors that can occur during lexing
#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char, usize, usize),
    UnterminatedString(usize, usize),
    InvalidNumber(String, usize, usize),
    InvalidQubitLiteral(String, usize, usize),
    UnterminatedComment(usize, usize),
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::UnexpectedCharacter(ch, line, col) => {
                write!(f, "Unexpected character '{}' at {}:{}", ch, line, col)
            }
            LexerError::UnterminatedString(line, col) => {
                write!(f, "Unterminated string at {}:{}", line, col)
            }
            LexerError::InvalidNumber(num, line, col) => {
                write!(f, "Invalid number '{}' at {}:{}", num, line, col)
            }
            LexerError::InvalidQubitLiteral(q, line, col) => {
                write!(f, "Invalid qubit literal '{}' at {}:{}", q, line, col)
            }
            LexerError::UnterminatedComment(line, col) => {
                write!(f, "Unterminated multi-line comment at {}:{}", line, col)
            }
        }
    }
}
impl std::error::Error for LexerError {}

/// Lexer struct
pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    current: Option<char>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut lx = Lexer {
            source: src.chars().peekable(),
            current: None,
            line: 1,
            col: 0,
        };
        lx.advance();
        lx
    }

    /// Main tokenize loop
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.current {
            match ch {
                // whitespace & BOMs/ZWSP
                ' ' | '\t' | '\r' | '\u{FEFF}' | '\u{200B}' => self.advance(),
                '\n' => {
                    self.line += 1;
                    self.col = 0;
                    self.advance();
                }

                // hieroglyphics
                '𓀀' => {
                    tokens.push(self.mk(TokenKind::HieroglyphicOp("𓀀".into())));
                    self.advance();
                }
                '𓁀' => {
                    tokens.push(self.mk(TokenKind::HieroglyphicOp("𓁀".into())));
                    self.advance();
                }
                '𓂀' => {
                    tokens.push(self.mk(TokenKind::HieroglyphicOp("𓂀".into())));
                    self.advance();
                }

                // operators & delimiters
                '+' => {
                    tokens.push(self.mk(TokenKind::Plus));
                    self.advance();
                }
                '-' => {
                    tokens.push(self.mk(TokenKind::Minus));
                    self.advance();
                }
                '*' => {
                    tokens.push(self.mk(TokenKind::Star));
                    self.advance();
                }
                '/' => match self.peek() {
                    Some('/') => self.skip_single_comment(),
                    Some('*') => self.skip_multi_comment()?,
                    _ => {
                        tokens.push(self.mk(TokenKind::Slash));
                        self.advance();
                    }
                },
                '=' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(self.mk(TokenKind::DoubleEquals));
                    } else {
                        tokens.push(self.mk(TokenKind::Equals));
                    }
                }
                ':' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(self.mk(TokenKind::ColonEquals));
                    } else {
                        return Err(LexerError::UnexpectedCharacter(':', self.line, self.col));
                    }
                }
                '!' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(self.mk(TokenKind::NotEquals));
                    } else {
                        return Err(LexerError::UnexpectedCharacter('!', self.line, self.col));
                    }
                }
                '<' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(self.mk(TokenKind::LessEqual));
                    } else {
                        tokens.push(self.mk(TokenKind::LessThan));
                    }
                }
                '>' => {
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(self.mk(TokenKind::GreaterEqual));
                    } else {
                        tokens.push(self.mk(TokenKind::GreaterThan));
                    }
                }
                ';' => {
                    tokens.push(self.mk(TokenKind::Semicolon));
                    self.advance();
                }
                ',' => {
                    tokens.push(self.mk(TokenKind::Comma));
                    self.advance();
                }
                '(' => {
                    tokens.push(self.mk(TokenKind::OpenParen));
                    self.advance();
                }
                ')' => {
                    tokens.push(self.mk(TokenKind::CloseParen));
                    self.advance();
                }
                '{' => {
                    tokens.push(self.mk(TokenKind::OpenBrace));
                    self.advance();
                }
                '}' => {
                    tokens.push(self.mk(TokenKind::CloseBrace));
                    self.advance();
                }

                // qubit literal
                '|' => {
                    tokens.push(self.lex_qubit()?);
                }

                // string literal
                '"' => {
                    tokens.push(self.lex_string()?);
                }

                // numbers
                ch if ch.is_ascii_digit() => {
                    tokens.push(self.lex_number()?);
                }

                // identifiers & keywords
                ch if is_identifier_start(ch) => {
                    let ident = self.lex_identifier();
                    let kind = match ident.as_str() {
                        "let" => TokenKind::Let,
                        "function" => TokenKind::Function,
                        "return" => TokenKind::Return,
                        "log" => TokenKind::Log,
                        "qubit" => TokenKind::Qubit,
                        "superpose" => TokenKind::Superpose,
                        "entangle" => TokenKind::Entangle,
                        "measure" => TokenKind::Measure,
                        "dod" => TokenKind::Dod,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        "for" => TokenKind::For,
                        "while" => TokenKind::While,
                        "in" => TokenKind::In,
                        "true" => TokenKind::BooleanLiteral(true),
                        "false" => TokenKind::BooleanLiteral(false),
                        _ => TokenKind::Identifier(ident),
                    };
                    tokens.push(self.mk(kind));
                }

                // unknown
                _ => return Err(LexerError::UnexpectedCharacter(ch, self.line, self.col)),
            }
        }

        tokens.push(self.mk(TokenKind::EOF));
        Ok(tokens)
    }

    /* ─── helpers ───────────────────────────────────────── */

    fn advance(&mut self) {
        self.current = self.source.next();
        self.col += 1;
    }

    fn peek(&mut self) -> Option<char> {
        self.source.peek().copied()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.current == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn mk(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.line, self.col)
    }

    fn skip_single_comment(&mut self) {
        while let Some(ch) = self.current {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_multi_comment(&mut self) -> Result<(), LexerError> {
        self.advance(); // consume '/'
        self.advance(); // consume '*'
        while let Some(ch) = self.current {
            if ch == '*' && self.peek() == Some('/') {
                self.advance();
                self.advance();
                return Ok(());
            }
            if ch == '\n' {
                self.line += 1;
                self.col = 0;
            }
            self.advance();
        }
        Err(LexerError::UnterminatedComment(self.line, self.col))
    }

    fn lex_string(&mut self) -> Result<Token, LexerError> {
        self.advance(); // skip "
        let mut content = String::new();

        while let Some(ch) = self.current {
            if ch == '"' {
                self.advance();
                return Ok(self.mk(TokenKind::StringLiteral(content)));
            } else if ch == '\\' {
                self.advance();
                if let Some(esc) = self.current {
                    content.push(match esc {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        other => other,
                    });
                    self.advance();
                } else {
                    return Err(LexerError::UnterminatedString(self.line, self.col));
                }
            } else {
                content.push(ch);
                self.advance();
            }
        }

        Err(LexerError::UnterminatedString(self.line, self.col))
    }

    fn lex_number(&mut self) -> Result<Token, LexerError> {
        let mut num = String::new();
        let mut has_dot = false;

        while let Some(ch) = self.current {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                num.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        num.parse::<f64>()
            .map(|v| self.mk(TokenKind::NumberLiteral(v)))
            .map_err(|_| LexerError::InvalidNumber(num, self.line, self.col))
    }

    fn lex_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.current {
            if is_identifier_part(ch) {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn lex_qubit(&mut self) -> Result<Token, LexerError> {
        let mut q = String::new();
        q.push('|');
        self.advance();

        while let Some(ch) = self.current {
            q.push(ch);
            self.advance();
            if ch == '>' {
                return Ok(self.mk(TokenKind::QubitLiteral(q)));
            }
        }

        Err(LexerError::InvalidQubitLiteral(q, self.line, self.col))
    }
}

/* Identifier helpers */
fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || unicode_ident::is_xid_start(ch)
}
fn is_identifier_part(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || unicode_ident::is_xid_continue(ch)
}

/* ─── tests ────────────────────────────────────────────── */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qubit_literals_work() {
        let mut lx = Lexer::new("|0> |ψ>");
        let toks = lx.tokenize().unwrap();
        assert!(matches!(toks[0].kind, TokenKind::QubitLiteral(_)));
        assert!(matches!(toks[1].kind, TokenKind::QubitLiteral(_)));
    }

    #[test]
    fn hieroglyphics_work() {
        let mut lx = Lexer::new("𓀀𓁀𓂀");
        let toks = lx.tokenize().unwrap();
        assert!(matches!(toks[0].kind, TokenKind::HieroglyphicOp(_)));
    }

    #[test]
    fn multi_line_comment_skips() {
        let mut lx = Lexer::new("/* comment */ let x = 1;");
        let toks = lx.tokenize().unwrap();
        assert!(matches!(toks[0].kind, TokenKind::Let));
    }

    #[test]
    fn double_and_relational_detected() {
        let mut lx = Lexer::new("a == b != c < d <= e > f >= g");
        let toks = lx.tokenize().unwrap();
        assert!(toks
            .iter()
            .any(|t| matches!(t.kind, TokenKind::DoubleEquals)));
        assert!(toks.iter().any(|t| matches!(t.kind, TokenKind::NotEquals)));
        assert!(toks.iter().any(|t| matches!(t.kind, TokenKind::LessThan)));
        assert!(toks.iter().any(|t| matches!(t.kind, TokenKind::LessEqual)));
        assert!(toks
            .iter()
            .any(|t| matches!(t.kind, TokenKind::GreaterThan)));
        assert!(toks
            .iter()
            .any(|t| matches!(t.kind, TokenKind::GreaterEqual)));
    }
}
