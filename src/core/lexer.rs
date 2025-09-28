#![allow(dead_code, unused_variables, unused_mut)]

// ...existing code...
use crate::core::token::{Token, TokenKind};
use std::fmt;
use std::sync::{Arc, Mutex};
use unicode_ident::{is_xid_continue, is_xid_start};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroize;

/// Configurable source markers/delimiters for Aeonmi source code.
#[derive(Debug, Clone, PartialEq)]
pub struct Markers {
    pub ai_start: char,
    pub ai_end: char,
    pub line_comment: char,
    pub block_comment_start: char,
    pub block_comment_end: char,
    pub extra: Vec<char>,
}
impl Default for Markers {
    fn default() -> Self {
        Self {
            ai_start: '⚡',
            ai_end: '⛓',
            line_comment: '⍝',
            block_comment_start: '⦅',
            block_comment_end: '⦆',
            extra: Vec::new(),
        }
    }
}

/// Lexer options configuring behavior and security restrictions.
#[derive(Clone)]
pub struct LexerOptions {
    pub allow_mixed_numerals: bool,
    pub max_ai_block_size: usize,
    pub markers: Markers,
    pub ai_access_authorized: bool,
    pub language_mode: Option<String>,
    pub dynamic_config: Option<Arc<Mutex<LexerDynamicConfig>>>,
    pub dlp_plugins: Vec<Arc<dyn DlpPlugin + Send + Sync>>,
    pub cli_mode: bool,
}
impl Default for LexerOptions {
    fn default() -> Self {
        Self {
            allow_mixed_numerals: false,
            max_ai_block_size: 1024 * 1024,
            markers: Markers::default(),
            ai_access_authorized: false,
            language_mode: None,
            dynamic_config: None,
            dlp_plugins: Vec::new(),
            cli_mode: false,
        }
    }
}

/// Hot-reloadable lexing dynamic configuration.
#[derive(Debug, Clone)]
pub struct LexerDynamicConfig {
    pub enabled_plugins: Vec<String>,
}

// Use canonical Token/TokenKind from crate::core::token

/// Read-only snapshot for plugins to avoid borrow conflicts.
#[derive(Debug, Clone, Copy)]
pub struct LexerView {
    pub line: usize,
    pub col: usize,
    pub in_ai_block: bool,
}

/// Lexer error types with detailed location.
#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char, usize, usize),
    UnterminatedString(usize, usize),
    InvalidNumber(String, usize, usize),
    InvalidGlyph(String, usize, usize),
    UnterminatedComment(usize, usize),
    UnauthorizedAIAccess(usize, usize),
    AIContentTooLarge(usize, usize),
    PluginError(String, usize, usize),
    Diagnostic(String, usize, usize, Option<String>),
    InvalidQubitLiteral(String, usize, usize),
}
impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LexerError::*;
        match self {
            UnexpectedCharacter(ch, line, col) => {
                write!(f, "Unexpected character '{}' at {}:{}", ch, line, col)
            }
            UnterminatedString(line, col) => {
                write!(f, "Unterminated string starting at {}:{}", line, col)
            }
            InvalidNumber(num, line, col) => {
                write!(f, "Invalid number literal '{}' at {}:{}", num, line, col)
            }
            InvalidGlyph(g, line, col) => write!(
                f,
                "Invalid or unrecognized glyph \"{}\" at {}:{}",
                g, line, col
            ),
            UnterminatedComment(line, col) => write!(
                f,
                "Unterminated comment or block starting at {}:{}",
                line, col
            ),
            UnauthorizedAIAccess(line, col) => write!(
                f,
                "Unauthorized access to AI-only block at {}:{}",
                line, col
            ),
            AIContentTooLarge(line, col) => write!(
                f,
                "AI-only block exceeds configured size limit at {}:{}",
                line, col
            ),
            PluginError(msg, line, col) => write!(f, "Plugin error '{}' at {}:{}", msg, line, col),
            Diagnostic(msg, line, col, hint) => write!(
                f,
                "Diagnostic at {}:{}: {}{}",
                line,
                col,
                msg,
                hint.as_ref()
                    .map(|h| format!("\nHint: {}", h))
                    .unwrap_or_default()
            ),
            InvalidQubitLiteral(s, line, col) => {
                write!(f, "Invalid qubit literal '{}' at {}:{}", s, line, col)
            }
        }
    }
}
impl std::error::Error for LexerError {}

pub trait CustomTokenKind: Send + Sync {
    fn try_match(&self, lexer: &Lexer) -> Option<TokenKind>;
    fn name(&self) -> &str;
}

/// NOTE: Changed to accept a read-only `LexerView` to avoid E0502.
pub trait LexerPlugin: Send + Sync {
    fn before_token(&mut self, _view: LexerView) {}
    fn after_token(&mut self, _view: LexerView, _token: &Token) {}
    fn on_error(&mut self, _view: LexerView, _error: &LexerError) {}
}

pub trait DlpPlugin: Send + Sync {
    fn before_emit_token(&self, token: &Token);
    fn after_emit_token(&self, token: &Token);
    fn on_security_event(&self, event: &str, token: Option<&Token>);
}

/// Main lexer struct (holds normalized String for correct lifetime)
pub struct Lexer {
    normalized: String,
    src: *const str, // Only used for lifetime pinning
    chars: std::str::CharIndices<'static>,
    current: Option<(usize, char)>,
    line: usize,
    col: usize,
    options: LexerOptions,
    in_ai_block: bool,
    plugins: Vec<Box<dyn LexerPlugin>>,
    custom_token_kinds: Vec<Arc<dyn CustomTokenKind>>,
    pub token_cache: Vec<Token>,
    pub event_bus: Option<Arc<Mutex<Vec<String>>>>,
}

impl Lexer {
    /// New with explicit AI access flag.
    pub fn new(input: &str, ai_access_authorized: bool) -> Self {
    let options = LexerOptions { ai_access_authorized, ..Default::default() };
    Self::with_options(input, options)
    }

    /// Back-compat convenience: single-argument constructor (authorized=false)
    pub fn from_str(input: &str) -> Self {
        Self::new(input, false)
    }
    pub fn with_options(input: &str, options: LexerOptions) -> Self {
        let normalized: String = input.nfc().collect();
        // Pin the normalized string so char_indices is safe
        let boxed = Box::new(normalized);
        let static_ref: &'static str = Box::leak(boxed);
        let mut lexer = Self {
            normalized: static_ref.to_string(),
            src: static_ref as *const str,
            chars: static_ref.char_indices(),
            current: None,
            line: 1,
            col: 0,
            options,
            in_ai_block: false,
            plugins: Vec::new(),
            custom_token_kinds: Vec::new(),
            token_cache: Vec::new(),
            event_bus: None,
        };
        lexer.advance_char();
        lexer
    }
    pub fn add_plugin<P: LexerPlugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }
    pub fn register_custom_token_kind(&mut self, kind: Arc<dyn CustomTokenKind>) {
        self.custom_token_kinds.push(kind);
    }
    pub fn set_event_bus(&mut self, bus: Arc<Mutex<Vec<String>>>) {
        self.event_bus = Some(bus);
    }

    /// Lightweight, copyable snapshot for plugins.
    #[inline]
    fn view(&self) -> LexerView {
        LexerView {
            line: self.line,
            col: self.col,
            in_ai_block: self.in_ai_block,
        }
    }

    #[inline]
    fn pos(&self) -> (usize, usize) {
        (self.line, self.col)
    }
    #[inline]
    fn advance_char(&mut self) {
        self.current = self.chars.next();
        if let Some((_, ch)) = self.current {
            if ch == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
    }
    #[inline]
    fn peek_char(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, c)| c)
    }
    pub fn next_token(&mut self) -> Result<Option<Token>, LexerError> {
        loop {
            let ch = match self.current {
                Some((_, ch)) => ch,
                None => {
                    let (line, col) = self.pos();
                    return Ok(Some(Token::new(TokenKind::EOF, String::from(""), line, col)));
                }
            };

            // --- plugin: before token ---
            {
                let view = self.view();
                for plugin in self.plugins.iter_mut() {
                    plugin.before_token(view);
                }
            }

            // Custom token kinds
            for kind in self.custom_token_kinds.iter() {
                if let Some(tok) = kind.try_match(self) {
                    let (line, col) = self.pos();
                    self.advance_char();
                    let token = Token::new(tok, String::new(), line, col);

                    // plugin: after token
                    {
                        let view = self.view();
                        for plugin in self.plugins.iter_mut() {
                            plugin.after_token(view, &token);
                        }
                    }
                    return Ok(Some(token));
                }
            }

            // Map standalone Unicode operator glyphs directly to their token kinds.
            // This allows users to paste code containing ≤ ≥ ≠ ＝ ⇔ ∷ ≔ without needing chord expansion.
            if let Some(tok_kind) = match ch {
                '≤' => Some(TokenKind::LessEqual),
                '≥' => Some(TokenKind::GreaterEqual),
                '≠' => Some(TokenKind::NotEquals),
                '＝' => Some(TokenKind::DoubleEquals),
                '≔' => Some(TokenKind::ColonEquals), // map ≔ to :=
                _ => None,
            } {
                let (line, col) = self.pos();
                self.advance_char();
                let token = Token::new(tok_kind, ch.to_string(), line, col);
                {
                    let view = self.view();
                    for plugin in self.plugins.iter_mut() { plugin.after_token(view, &token); }
                }
                return Ok(Some(token));
            }

            let result = if self.in_ai_block {
                self.lex_in_ai_block(ch)
            } else if ch == '/' && self.peek_char() == Some('/') {
                // Support C-style '//' line comments (common in test sources).
                // Consume the first '/' then let lex_line_comment skip to EOL.
                self.advance_char();
                self.lex_line_comment();
                continue;
            } else if ch == self.options.markers.line_comment {
                self.lex_line_comment();
                continue;
            } else if ch == self.options.markers.block_comment_start {
                self.lex_block_comment()
            } else if is_safe_whitespace(ch) {
                self.advance_char();
                continue;
            } else if ch == self.options.markers.ai_start {
                self.enter_ai_block()
            } else if let Some(tok) = self.match_multi_char_operator(ch) {
                let (line, col) = self.pos();
                self.advance_char();
                self.advance_char();
                Ok(Some(Token::new(tok, String::new(), line, col)))
            } else if ch == '|' {
                // '|' may start a qubit literal (e.g. |0>, |ψ>, |+>, |->).
                // If the glyph sequence is followed by a closing '>' produce a
                // QubitLiteral; if it never closes, signal InvalidQubitLiteral.
                // Otherwise fall back to the plain Pipe token.
                let (line, col) = self.pos();
                match self.peek_char() {
                    Some(next) if is_identifier_start(next)
                        || is_numeric_glyph(next)
                        || next == '>'
                        || next == '+'
                        || next == '-' =>
                    {
                        // Attempt to lex as a qubit literal (may error)
                        match self.lex_qubit_literal() {
                            Ok(tok) => Ok(Some(tok)),
                            Err(e) => Err(e),
                        }
                    }
                    _ => {
                        // Plain pipe operator
                        self.advance_char();
                        Ok(Some(Token::new(TokenKind::Pipe, String::from("|"), line, col)))
                    }
                }
            } else if let Some(tok) = self.match_single_char_token(ch) {
                let (line, col) = self.pos();
                self.advance_char();
                Ok(Some(Token::new(tok, String::new(), line, col)))
            } else if ch.is_ascii_digit() || is_numeric_glyph(ch) {
                self.lex_number()
            } else if ch == '"' {
                self.lex_string().map(Some)
            } else if is_identifier_start(ch) {
                Ok(Some(self.lex_identifier()))
            } else {
                let (l, c) = self.pos();
                Err(LexerError::UnexpectedCharacter(ch, l, c))
            };

            match result {
                Ok(Some(token)) => {
                    // DLP (before)
                    for dlp in self.options.dlp_plugins.iter() {
                        dlp.before_emit_token(&token);
                    }
                    // plugin: after token
                    {
                        let view = self.view();
                        for plugin in self.plugins.iter_mut() {
                            plugin.after_token(view, &token);
                        }
                    }
                    // DLP (after)
                    for dlp in self.options.dlp_plugins.iter() {
                        dlp.after_emit_token(&token);
                    }

                    if self.options.cli_mode {
                        if let Some(bus) = &self.event_bus {
                            let msg = format!(
                                "Token: {:?} at {}:{}",
                                token.kind, token.line, token.column
                            );
                            bus.lock().unwrap().push(msg);
                        }
                    }
                    return Ok(Some(token));
                }
                Ok(None) => { /* No token produced, continue */ }
                Err(e) => {
                    // plugin: on error
                    {
                        let view = self.view();
                        for plugin in self.plugins.iter_mut() {
                            plugin.on_error(view, &e);
                        }
                    }
                    if self.options.cli_mode {
                        if let Some(bus) = &self.event_bus {
                            let msg = format!("LexerError: {}", e);
                            bus.lock().unwrap().push(msg);
                        }
                    }
                    return Err(e);
                }
            }
        }
    }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token.clone());
            if let TokenKind::EOF = token.kind {
                break;
            }
        }
        Ok(tokens)
    }
    fn lex_in_ai_block(&mut self, ch: char) -> Result<Option<Token>, LexerError> {
        if ch == self.options.markers.ai_end {
            self.in_ai_block = false;
            self.advance_char();
            return Ok(None);
        }
        if !self.options.ai_access_authorized {
            let (line, col) = self.pos();
            return Err(LexerError::UnauthorizedAIAccess(line, col));
        }
        self.lex_ai_block().map(Some)
    }
    fn enter_ai_block(&mut self) -> Result<Option<Token>, LexerError> {
        if !self.options.ai_access_authorized {
            let (line, col) = self.pos();
            return Err(LexerError::UnauthorizedAIAccess(line, col));
        }
        self.in_ai_block = true;
        self.advance_char();
        Ok(None)
    }
    fn lex_line_comment(&mut self) {
        while let Some((_, ch)) = self.current {
            if ch == '\n' {
                self.advance_char();
                break;
            }
            self.advance_char();
        }
    }
    fn lex_block_comment(&mut self) -> Result<Option<Token>, LexerError> {
        let (start_line, start_col) = self.pos();
        self.advance_char();
        let mut depth = 1usize;
        while let Some((_, ch)) = self.current {
            if ch == self.options.markers.block_comment_start {
                depth += 1;
            } else if ch == self.options.markers.block_comment_end {
                depth -= 1;
                self.advance_char();
                if depth == 0 {
                    return Ok(None);
                }
                continue;
            }
            self.advance_char();
        }
        Err(LexerError::UnterminatedComment(start_line, start_col))
    }
    fn lex_ai_block(&mut self) -> Result<Token, LexerError> {
        let (line, col) = self.pos();
        let mut content = String::new();
        let mut size = 0usize;
        while let Some((_, ch)) = self.current {
            if ch == self.options.markers.ai_end {
                break;
            }
            size += ch.len_utf8();
            if size > self.options.max_ai_block_size {
                content.zeroize();
                return Err(LexerError::AIContentTooLarge(line, col));
            }
            content.push(ch);
            self.advance_char();
        }
        if self.current.is_none() {
            content.zeroize();
            return Err(LexerError::UnterminatedComment(line, col));
        }
        Ok(Token::new(TokenKind::StringLiteral(content), String::new(), line, col))
    }
    fn lex_number(&mut self) -> Result<Option<Token>, LexerError> {
        if self.options.allow_mixed_numerals {
            self.lex_number_mixed().map(Some)
        } else if self
            .current
            .map(|(_, c)| c)
            .unwrap_or('\0')
            .is_ascii_digit()
        {
            self.lex_ascii_number().map(Some)
        } else {
            self.lex_glyph_number().map(Some)
        }
    }
    fn lex_ascii_number(&mut self) -> Result<Token, LexerError> {
        let (line, col) = self.pos();
        let mut num_str = String::new();
        let mut has_decimal = false;
        while let Some((_, ch)) = self.current {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance_char();
            } else if ch == '.' && !has_decimal {
                has_decimal = true;
                num_str.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }
        num_str
            .parse::<f64>()
            .map(|n| Token::new(TokenKind::NumberLiteral(n), num_str.clone(), line, col))
            .map_err(|_| LexerError::InvalidNumber(num_str, line, col))
    }
    fn lex_glyph_number(&mut self) -> Result<Token, LexerError> {
        let (line, col) = self.pos();
        let mut glyph_str = String::new();
        while let Some((_, ch)) = self.current {
            if is_numeric_glyph(ch) {
                glyph_str.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }
        let value = glyph_str
            .chars()
            .filter_map(glyph_to_digit)
            .fold(0.0, |acc, d| acc * 10.0 + d as f64);
        Ok(Token::new(TokenKind::NumberLiteral(value), glyph_str.clone(), line, col))
    }
    fn lex_number_mixed(&mut self) -> Result<Token, LexerError> {
        let (line, col) = self.pos();
        let mut num_str = String::new();
        let mut has_decimal = false;
        while let Some((_, ch)) = self.current {
            if ch.is_ascii_digit() || is_numeric_glyph(ch) {
                num_str.push(ch);
                self.advance_char();
            } else if ch == '.' && !has_decimal {
                has_decimal = true;
                num_str.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }
        let ascii_str: String = num_str
            .chars()
            .map(|c| {
                if is_numeric_glyph(c) {
                    glyph_to_digit(c).map(|d| (b'0' + d) as char).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect();
        ascii_str
            .parse::<f64>()
            .map(|n| Token::new(TokenKind::NumberLiteral(n), ascii_str.clone(), line, col))
            .map_err(|_| LexerError::InvalidNumber(ascii_str, line, col))
    }
    fn lex_string(&mut self) -> Result<Token, LexerError> {
        let (line, col) = self.pos();
        self.advance_char(); // consume opening quote
        let mut content = String::new();
        let mut escape = false;
        while let Some((_, ch)) = self.current {
            if !escape {
                match ch {
                    '"' => {
                        self.advance_char();
                        return Ok(Token::new(TokenKind::StringLiteral(content.clone()), content, line, col));
                    }
                    '\\' => {
                        escape = true;
                        self.advance_char();
                    }
                    _ => {
                        content.push(ch);
                        self.advance_char();
                    }
                }
            } else {
                let esc_ch = match ch {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    'u' => {
                        self.advance_char();
                        let unicode_char = self.parse_unicode_escape()?;
                        content.push(unicode_char);
                        escape = false;
                        continue;
                    }
                    other => other,
                };
                content.push(esc_ch);
                self.advance_char();
                escape = false;
            }
        }
        Err(LexerError::UnterminatedString(line, col))
    }
    fn parse_unicode_escape(&mut self) -> Result<char, LexerError> {
        if self.current.map(|(_, c)| c) != Some('{') {
            return Err(LexerError::UnexpectedCharacter(
                self.current.map(|(_, c)| c).unwrap_or(' '),
                self.line,
                self.col,
            ));
        }
        self.advance_char();
        let mut hex_str = String::new();
        while let Some((_, ch)) = self.current {
            if ch == '}' {
                self.advance_char();
                break;
            }
            if ch.is_ascii_hexdigit() {
                hex_str.push(ch);
                self.advance_char();
            } else {
                return Err(LexerError::UnexpectedCharacter(ch, self.line, self.col));
            }
        }
        let code_point = u32::from_str_radix(&hex_str, 16)
            .map_err(|_| LexerError::InvalidGlyph(hex_str.clone(), self.line, self.col))?;
        std::char::from_u32(code_point).ok_or_else(|| {
            LexerError::InvalidGlyph(format!("\\u{{{}}}", code_point), self.line, self.col)
        })
    }
    fn lex_identifier(&mut self) -> Token {
        let (line, col) = self.pos();
        let mut ident = String::new();
        while let Some((_, ch)) = self.current {
            if is_identifier_part(ch) {
                ident.push(ch);
                self.advance_char();
            } else {
                break;
            }
        }
        // Map reserved words / keywords to their TokenKind so parser can
        // distinguish declarations and control flow correctly.
        match ident.as_str() {
            "let" => Token::new(TokenKind::Let, String::from("let"), line, col),
            "function" => Token::new(TokenKind::Function, String::from("function"), line, col),
            "return" => Token::new(TokenKind::Return, String::from("return"), line, col),
            "log" => Token::new(TokenKind::Log, String::from("log"), line, col),
            "qubit" => Token::new(TokenKind::Qubit, String::from("qubit"), line, col),
            "superpose" => Token::new(TokenKind::Superpose, String::from("superpose"), line, col),
            "entangle" => Token::new(TokenKind::Entangle, String::from("entangle"), line, col),
            "measure" => Token::new(TokenKind::Measure, String::from("measure"), line, col),
            "dod" => Token::new(TokenKind::Dod, String::from("dod"), line, col),
            "if" => Token::new(TokenKind::If, String::from("if"), line, col),
            "else" => Token::new(TokenKind::Else, String::from("else"), line, col),
            "for" => Token::new(TokenKind::For, String::from("for"), line, col),
            "while" => Token::new(TokenKind::While, String::from("while"), line, col),
            "in" => Token::new(TokenKind::In, String::from("in"), line, col),
            "true" => Token::new(TokenKind::BooleanLiteral(true), String::from("true"), line, col),
            "false" => Token::new(TokenKind::BooleanLiteral(false), String::from("false"), line, col),
            _ => Token::new(TokenKind::Identifier(ident.clone()), ident, line, col),
        }
    }
    fn lex_qubit_literal(&mut self) -> Result<Token, LexerError> {
        // Current char is expected to be '|' (caller ensures this).
        let (line, col) = self.pos();
        // consume the '|'
        self.advance_char();
        let mut content = String::new();
        // read until closing '>' or EOF
        while let Some((_, ch)) = self.current {
            if ch == '>' {
                // consume '>' and finish
                self.advance_char();
                let literal = format!("|{}>", content);
                return Ok(Token::new(TokenKind::QubitLiteral(literal.clone()), literal, line, col));
            }
            // Accept identifier parts or numeric glyphs or common qubit symbols
            if is_identifier_part(ch) || is_numeric_glyph(ch) || ch == '+' || ch == '-' {
                content.push(ch);
                self.advance_char();
                continue;
            }
            // Unexpected character inside qubit literal -> error
            return Err(LexerError::InvalidQubitLiteral(content, line, col));
        }
        // EOF reached without closing '>'
        Err(LexerError::InvalidQubitLiteral(content, line, col))
    }
    fn match_multi_char_operator(&mut self, ch: char) -> Option<TokenKind> {
        match (ch, self.peek_char()) {
            ('=', Some('=')) => Some(TokenKind::DoubleEquals),
            ('!', Some('=')) => Some(TokenKind::NotEquals),
            ('<', Some('=')) => Some(TokenKind::LessEqual),
            ('>', Some('=')) => Some(TokenKind::GreaterEqual),
            (':', Some('=')) => Some(TokenKind::ColonEquals),
            ('&', Some('&')) => Some(TokenKind::AndAnd),
            ('|', Some('|')) => Some(TokenKind::OrOr),
            _ => None,
        }
    }
    fn match_single_char_token(&self, ch: char) -> Option<TokenKind> {
        match ch {
            '+' => Some(TokenKind::Plus),
            '-' => Some(TokenKind::Minus),
            '*' => Some(TokenKind::Star),
            '/' => Some(TokenKind::Slash),
            '=' => Some(TokenKind::Equals),
            ';' => Some(TokenKind::Semicolon),
            ',' => Some(TokenKind::Comma),
            '(' => Some(TokenKind::OpenParen),
            ')' => Some(TokenKind::CloseParen),
            '{' => Some(TokenKind::OpenBrace),
            '}' => Some(TokenKind::CloseBrace),
            '<' => Some(TokenKind::LessThan),
            '>' => Some(TokenKind::GreaterThan),
            '|' => Some(TokenKind::Pipe), // single '|' retained for qubit or pipe future, '||' handled above

            // Many specialized glyphs are represented as hieroglyphic operations
            glyph if is_hieroglyphic(glyph) => Some(TokenKind::HieroglyphicOp(glyph.to_string())),

            _ => None,
        }
    }
}

//-- UTILITIES --

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || is_xid_start(ch)
}
fn is_identifier_part(ch: char) -> bool {
    ch == '_' || is_xid_continue(ch)
}
fn is_safe_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n' | '\u{FEFF}')
}
fn is_numeric_glyph(ch: char) -> bool {
    (0x1D360..=0x1D369).contains(&(ch as u32))
}
fn glyph_to_digit(ch: char) -> Option<u8> {
    match ch as u32 {
        0x1D360..=0x1D369 => Some((ch as u32 - 0x1D360) as u8),
        _ => None,
    }
}

fn is_hieroglyphic(ch: char) -> bool {
    // Consider non-ascii symbol characters as hieroglyphic markers for now.
    (ch as u32) > 0x007F
}

#[cfg(test)]
mod tests {
    
    // Add your tests here!
}
