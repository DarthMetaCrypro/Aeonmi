// lexer.rs

use std::fmt;
use std::sync::{Arc, Mutex};
use unicode_ident::{is_xid_start, is_xid_continue};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroize;

use crate::core::token::{Token, TokenKind};

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
            max_ai_block_size: 1 * 1024 * 1024,
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

/// Lexer error types with detailed location.
#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char, usize, usize),
    UnterminatedString(usize, usize),
    InvalidNumber(String, usize, usize),
    InvalidGlyph(String, usize, usize),
    InvalidQubitLiteral(String, usize, usize),
    UnterminatedComment(usize, usize),
    UnterminatedAIBlock(usize, usize),
    UnauthorizedAIAccess(usize, usize),
    AIContentTooLarge(usize, usize),
    PluginError(String, usize, usize),
    Diagnostic(String, usize, usize, Option<String>),
}
impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LexerError::*;
        match self {
            UnexpectedCharacter(ch, line, col) => write!(f, "Unexpected character '{}' at {}:{}", ch, line, col),
            UnterminatedString(line, col) => write!(f, "Unterminated string starting at {}:{}", line, col),
            InvalidNumber(num, line, col) => write!(f, "Invalid number literal '{}' at {}:{}", num, line, col),
            InvalidGlyph(g, line, col) => write!(f, "Invalid or unrecognized glyph \"{}\" at {}:{}", g, line, col),
            InvalidQubitLiteral(lit, line, col) => write!(f, "Invalid qubit literal '{}' at {}:{}", lit, line, col),
            UnterminatedComment(line, col) => write!(f, "Unterminated comment or block starting at {}:{}", line, col),
            UnterminatedAIBlock(line, col) => write!(f, "Unterminated AI block starting at {}:{}", line, col),
            UnauthorizedAIAccess(line, col) => write!(f, "Unauthorized access to AI-only block at {}:{}", line, col),
            AIContentTooLarge(line, col) => write!(f, "AI-only block exceeds configured size limit at {}:{}", line, col),
            PluginError(msg, line, col) => write!(f, "Plugin error '{}' at {}:{}", msg, line, col),
            Diagnostic(msg, line, col, hint) => write!(f, "Diagnostic at {}:{}: {}{}", line, col, msg, hint.as_ref().map(|h| format!("\nHint: {}", h)).unwrap_or_default()),
        }
    }
}
impl std::error::Error for LexerError {}

/// Custom token kind trait allowing multi-char token lexing.
pub trait CustomTokenKind: Send + Sync {
    /// Attempts to match a token at current lexer state.
    /// Returns Some(token) and consumes characters if matched.
    fn try_match(&self, lexer: &mut Lexer) -> Option<TokenKind>;
    fn name(&self) -> &str;
}

/// Read-only snapshot for plugins to avoid borrow conflicts.
#[derive(Debug, Clone, Copy)]
pub struct LexerView {
    pub line: usize,
    pub col: usize,
    pub in_ai_block: bool,
}

/// Lexer plugin trait.
pub trait LexerPlugin: Send + Sync {
    fn before_token(&mut self, _view: LexerView) {}
    fn after_token(&mut self, _view: LexerView, _token: &Token) {}
    fn on_error(&mut self, _view: LexerView, _error: &LexerError) {}
}

/// DLP plugin trait.
pub trait DlpPlugin: Send + Sync {
    fn before_emit_token(&self, token: &Token);
    fn after_emit_token(&self, token: &Token);
    fn on_security_event(&self, event: &str, token: Option<&Token>);
}

/// Main lexer struct (holds normalized String for lifetime safety)
pub struct Lexer {
    _normalized: String,
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
    consumed_eof: bool, // Prevent repeated EOF tokens
}

impl Lexer {
    /// Backward-compatible constructor (unauthorized AI by default)
    pub fn new(input: &str) -> Self {
        Self::with_ai_auth(input, false)
    }

    /// Explicit constructor with AI authorization flag.
    pub fn with_ai_auth(input: &str, ai_access_authorized: bool) -> Self {
        let mut options = LexerOptions::default();
        options.ai_access_authorized = ai_access_authorized;
        Self::with_options(input, options)
    }

    pub fn with_options(input: &str, options: LexerOptions) -> Self {
        let normalized: String = input.nfc().collect();
        // Store normalized with leaked lifetime only once (no double allocation)
        let leaked: &'static str = Box::leak(normalized.into_boxed_str());
        let chars = leaked.char_indices();

        Self {
            _normalized: leaked.to_string(), 
            chars,
            current: None,
            line: 1,
            col: 0,
            options,
            in_ai_block: false,
            plugins: Vec::new(),
            custom_token_kinds: Vec::new(),
            token_cache: Vec::new(),
            event_bus: None,
            consumed_eof: false,
        }.init_first_char()
    }

    #[inline]
    fn init_first_char(mut self) -> Self {
        // Prime the first character
        self.advance_char();
        self
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
        if self.consumed_eof {
            // Already reached EOF previously, no more tokens
            return Ok(None);
        }

        loop {
            let ch = match self.current {
                Some((_, ch)) => ch,
                None => {
                    self.consumed_eof = true;
                    let (line, col) = self.pos();
                    let eof = Token {
                        kind: TokenKind::EOF,
                        line,
                        col,
                        end_line: line,
                        end_col: col,
                        hash: None,
                    };
                    return Ok(Some(eof));
                }
            };

            // --- plugin: before token ---
            {
                let view = self.view();
                for plugin in self.plugins.iter_mut() {
                    plugin.before_token(view);
                }
            }

            // Try custom token kinds (consume multi-char). Clone Arcs to avoid borrow conflict.
            let custom_kinds = self.custom_token_kinds.clone();
            for kind in custom_kinds.iter() {
                if let Some(tok_kind) = kind.try_match(self) {
                    let (line, col) = self.pos();
                    let token = Token {
                        kind: tok_kind,
                        line,
                        col,
                        end_line: self.line,
                        end_col: self.col,
                        hash: None,
                    };
                    let view = self.view();
                    for plugin in self.plugins.iter_mut() {
                        plugin.after_token(view, &token);
                    }
                    return Ok(Some(token));
                }
            }

            // ASCII comment support (// and /* */) handled early so the main branch stays type-consistent
            if ch == '/' {
                if let Some(next) = self.peek_char() {
                    if next == '/' {
                        self.lex_line_comment_ascii();
                        continue;
                    } else if next == '*' {
                        self.lex_block_comment_ascii()?;
                        continue;
                    }
                }
            }

            let result = if self.in_ai_block {
                self.lex_in_ai_block(ch)
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
            } else if ch == '|' && self.peek_char().map(is_qubit_name_start).unwrap_or(false) {
                // Qubit literal attempt
                self.lex_qubit_literal().map(Some)
            } else if let Some(tok) = self.match_multi_char_operator(ch) {
                let (line, col) = self.pos();
                self.advance_char();
                self.advance_char();
                Ok(Some(Token { kind: tok, line, col, end_line: self.line, end_col: self.col, hash: None }))
            } else if let Some(tok) = self.match_single_char_token(ch) {
                let (line, col) = self.pos();
                self.advance_char();
                Ok(Some(Token { kind: tok, line, col, end_line: self.line, end_col: self.col, hash: None }))
            } else if ch.is_ascii_digit() || is_numeric_glyph(ch) {
                self.lex_number()
            } else if ch == '"' {
                self.lex_string().map(Some)
            } else if is_identifier_start(ch) {
                Ok(Some(self.lex_identifier()))
            } else if is_hieroglyphic_glyph(ch) {
                // Treat standalone Egyptian hieroglyphs as hieroglyphic ops
                let (line, col) = self.pos();
                let glyph = ch;
                self.advance_char();
                Ok(Some(Token { kind: TokenKind::HieroglyphicOp(glyph.to_string()), line, col, end_line: self.line, end_col: self.col, hash: None }))
            } else {
                let (l, c) = self.pos();
                Err(LexerError::UnexpectedCharacter(ch, l, c))
            };

            match result {
                Ok(Some(mut token)) => {
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
                    token.hash = None; // TODO: Implement hash chaining
                    if self.options.cli_mode {
                        if let Some(bus) = &self.event_bus {
                            let msg = format!("Token: {:?} at {}:{}", token.kind, token.line, token.col);
                            bus.lock().unwrap().push(msg);
                        }
                    }
                    return Ok(Some(token));
                }
                Ok(None) => { /* No token produced; continue looping */ }
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

    /// Tokenizes entire source input; avoids unnecessary token cloning.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            let is_eof = matches!(token.kind, TokenKind::EOF);
            tokens.push(token);
            if is_eof {
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

    // ASCII line comment: // ... \n
    fn lex_line_comment_ascii(&mut self) {
        // consume first '/'
        self.advance_char();
        // consume second '/'
        if matches!(self.current, Some((_, '/'))) { self.advance_char(); }
        while let Some((_, ch)) = self.current {
            if ch == '\n' { self.advance_char(); break; }
            self.advance_char();
        }
    }

    // ASCII block comment: /* ... */ (non-nested for now)
    fn lex_block_comment_ascii(&mut self) -> Result<(), LexerError> {
        let (start_line, start_col) = self.pos();
        // consume '/'
        self.advance_char();
        // consume '*'
        if matches!(self.current, Some((_, '*'))) { self.advance_char(); } else { return Ok(()); }
        while let Some((_, ch)) = self.current {
            if ch == '*' {
                self.advance_char();
                if let Some((_, '/')) = self.current { self.advance_char(); return Ok(()); }
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
            return Err(LexerError::UnterminatedAIBlock(line, col));
        }
        Ok(Token {
            kind: TokenKind::AIOnlyBlock(content),
            line,
            col,
            end_line: self.line,
            end_col: self.col,
            hash: None,
        })
    }

    fn lex_number(&mut self) -> Result<Option<Token>, LexerError> {
        if self.options.allow_mixed_numerals {
            self.lex_number_mixed().map(Some)
        } else if self.current.map(|(_, c)| c).unwrap_or('\0').is_ascii_digit() {
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
        num_str.parse::<f64>()
            .map(|n| Token {
                kind: TokenKind::NumberLiteral(n),
                line,
                col,
                end_line: self.line,
                end_col: self.col,
                hash: None,
            })
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
        let value = glyph_str.chars()
            .filter_map(glyph_to_digit)
            .fold(0.0, |acc, d| acc * 10.0 + d as f64);
        Ok(Token {
            kind: TokenKind::NumberLiteral(value),
            line,
            col,
            end_line: self.line,
            end_col: self.col,
            hash: None,
        })
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
        // Validation to reject partial glyph sequences when mixed numerals disallowed
        if !self.options.allow_mixed_numerals {
            let has_ascii = num_str.chars().any(|c| c.is_ascii_digit());
            let has_glyph = num_str.chars().any(is_numeric_glyph);
            if has_ascii && has_glyph {
                return Err(LexerError::InvalidNumber(num_str.clone(), line, col));
            }
        }
        let ascii_str: String = num_str.chars()
            .map(|c| if is_numeric_glyph(c) {
                glyph_to_digit(c).map(|d| (b'0' + d) as char).unwrap_or(c)
            } else {
                c
            })
            .collect();
        ascii_str.parse::<f64>()
            .map(|n| Token {
                kind: TokenKind::NumberLiteral(n),
                line,
                col,
                end_line: self.line,
                end_col: self.col,
                hash: None,
            })
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
                        return Ok(Token {
                            kind: TokenKind::StringLiteral(content),
                            line,
                            col,
                            end_line: self.line,
                            end_col: self.col,
                            hash: None,
                        });
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
                        self.parse_unicode_escape()?
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

    fn lex_qubit_literal(&mut self) -> Result<Token, LexerError> {
        let (line, col) = self.pos();
        self.advance_char(); // consume '|'
        let mut name = String::new();
        while let Some((_, ch)) = self.current {
            if ch == '>' {
                self.advance_char();
                return Ok(Token { kind: TokenKind::QubitLiteral(name), line, col, end_line: self.line, end_col: self.col, hash: None });
            } else if is_qubit_name_part(ch) {
                name.push(ch);
                self.advance_char();
            } else if ch.is_whitespace() || ch == '\n' {
                return Err(LexerError::InvalidQubitLiteral(name, line, col));
            } else {
                return Err(LexerError::InvalidQubitLiteral(name, line, col));
            }
        }
        Err(LexerError::InvalidQubitLiteral(name, line, col))
    }

    fn parse_unicode_escape(&mut self) -> Result<char, LexerError> {
        if self.current.map(|(_, c)| c) != Some('{') {
            return Err(LexerError::UnexpectedCharacter(self.current.map(|(_, c)| c).unwrap_or(' '), self.line, self.col));
        }
        self.advance_char();
        let mut hex_str = String::new();
        while let Some((_, ch)) = self.current {
            if ch == '}' {
                if hex_str.is_empty() {
                    return Err(LexerError::InvalidGlyph("Empty unicode escape".into(), self.line, self.col));
                }
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
        std::char::from_u32(code_point)
            .ok_or_else(|| LexerError::InvalidGlyph(format!("\\u{{{}}}", code_point), self.line, self.col))
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
        // Keyword matching - case-sensitive, reserved
        let kind = match ident.as_str() {
            "let" => TokenKind::Let,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "log" => TokenKind::Log,
            "qubit" => TokenKind::QubitLiteral(String::new()), // placeholder for real lexing
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
        Token { kind, line, col, end_line: self.line, end_col: self.col, hash: None }
    }

    fn match_multi_char_operator(&mut self, ch: char) -> Option<TokenKind> {
        match (ch, self.peek_char()) {
            ('=', Some('=')) => Some(TokenKind::DoubleEquals),
            ('!', Some('=')) => Some(TokenKind::NotEquals),
            ('<', Some('=')) => Some(TokenKind::LessEqual),
            ('>', Some('=')) => Some(TokenKind::GreaterEqual),
            (':', Some('=')) => Some(TokenKind::ColonEquals),
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
            ':' => Some(TokenKind::Colon),
            '<' => Some(TokenKind::LessThan),
            '>' => Some(TokenKind::GreaterThan),
            '|' => Some(TokenKind::Pipe),
            ';' => Some(TokenKind::Semicolon),
            ',' => Some(TokenKind::Comma),
            '(' => Some(TokenKind::OpenParen),
            ')' => Some(TokenKind::CloseParen),
            '{' => Some(TokenKind::OpenBrace),
            '}' => Some(TokenKind::CloseBrace),

            // Special Unicode tokens
            'Ⓐ' => Some(TokenKind::PrimitiveTypeMarker),
            'Ⓑ' => Some(TokenKind::CompositeTypeMarker),
            'Ⓒ' => Some(TokenKind::NumericType),
            'Ⓓ' => Some(TokenKind::BooleanType),
            'Ⓔ' => Some(TokenKind::StringType),
            'Ⓕ' => Some(TokenKind::AIModelType),
            'Ⓖ' => Some(TokenKind::TensorType),
            '⨁' => Some(TokenKind::AdditionOp),
            '⨂' => Some(TokenKind::MultiplicationOp),
            '⨃' => Some(TokenKind::SubtractionOp),
            '⨄' => Some(TokenKind::DivisionOp),
            '⨅' => Some(TokenKind::ExponentiationOp),
            '⨆' => Some(TokenKind::ModuloOp),
            '⟐' => Some(TokenKind::ConditionalStart),
            '⟡' => Some(TokenKind::ConditionalElse),
            '⟫' => Some(TokenKind::LoopStart),
            '⟬' => Some(TokenKind::LoopEnd),
            '⎔' => Some(TokenKind::FunctionDef),
            '⎖' => Some(TokenKind::FuncParamStart),
            '⎗' => Some(TokenKind::FuncParamEnd),
            '⎘' => Some(TokenKind::FuncBodyStart),
            '⎙' => Some(TokenKind::FuncBodyEnd),
            '☰' => Some(TokenKind::AIModelDef),
            '☱' => Some(TokenKind::TrainingProc),
            '☲' => Some(TokenKind::InferenceProc),
            '☳' => Some(TokenKind::SelfModifyingCode),
            '☴' => Some(TokenKind::AIHyperparameters),
            '☵' => Some(TokenKind::AIDataInput),
            '⧈' => Some(TokenKind::ParallelExec),
            '⧉' => Some(TokenKind::AsyncExec),
            '⧊' => Some(TokenKind::SecureOp),
            '⧋' => Some(TokenKind::Encryption),
            '⧌' => Some(TokenKind::Decryption),

            _ => None,
        }
    }
}

// Utility functions for identifiers, whitespace, glyph numbers

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

// Egyptian Hieroglyph block basic range handling
fn is_hieroglyphic_glyph(ch: char) -> bool {
    let cp = ch as u32;
    (0x13000..=0x1342F).contains(&cp)
}

fn is_qubit_name_start(ch: char) -> bool {
    is_identifier_start(ch) || matches!(ch, 'ψ' | 'φ' | 'α' | 'β' | 'q')
}
fn is_qubit_name_part(ch: char) -> bool {
    is_identifier_part(ch) || matches!(ch, 'ψ' | 'φ' | 'α' | 'β') || ch.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    // Comprehensive tests live externally. Add focused lexer unit tests here if needed.
}

// End of file
