//! Parser for Aeonmi/QUBE/Titan with precedence parsing + spanned errors.

use crate::core::ast::{ASTNode, FunctionParam};
use crate::core::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}:{}", self.message, self.line, self.column)
    }
}

impl std::error::Error for ParserError {}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create new parser instance; ensure trailing EOF token present
    pub fn new(mut tokens: Vec<Token>) -> Self {
        let needs_eof = match tokens.last() {
            Some(t) => !matches!(t.kind, TokenKind::EOF),
            None => true,
        };
        if needs_eof {
            tokens.push(Token { kind: TokenKind::EOF, lexeme: String::new(), line: 0, column: 0 });
        }
        Parser { tokens, pos: 0 }
    }

    /// Main parse entrypoint: parses all tokens into program AST
    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        let mut nodes = Vec::new();
        while !self.is_at_end() {
            nodes.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program(nodes))
    }

    /// Parses a single statement based on current token peek
    fn parse_statement(&mut self) -> Result<ASTNode, ParserError> {
        match self.peek().kind.clone() {
            TokenKind::Let => self.parse_variable_decl(),
            TokenKind::Function => self.parse_function_decl(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Log => self.parse_log(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::OpenBrace => Ok(self.parse_block()?),
            TokenKind::Superpose | TokenKind::Entangle | TokenKind::Measure | TokenKind::Dod => {
                self.parse_quantum_op()
            }
            TokenKind::HieroglyphicOp(_) => self.parse_hieroglyphic_op(),
            _ => {
                let expr = self.parse_expression()?;
                let _ = self.match_token(&[TokenKind::Semicolon]); // optional semicolon
                Ok(expr)
            }
        }
    }

    fn parse_block(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::OpenBrace, "Expected '{' to start block")?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.consume(TokenKind::CloseBrace, "Expected '}' after block")?;
        Ok(ASTNode::Block(stmts))
    }

    fn parse_variable_decl(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Let, "Expected 'let'")?;
        let line = self.peek().line;
        let column = self.peek().column;
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(TokenKind::Equals, "Expected '=' in variable declaration")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after variable declaration")?;
        Ok(ASTNode::new_variable_decl_at(&name, value, line, column))
    }

    fn parse_function_decl(&mut self) -> Result<ASTNode, ParserError> {
        let func_tok = self.consume(TokenKind::Function, "Expected 'function'")?;
        let name_tok_line = self.peek().line;
        let name_tok_col = self.peek().column;
        let name = self.consume_identifier("Expected function name")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after function name")?;
        let mut params: Vec<FunctionParam> = Vec::new();
        if !self.check(&TokenKind::CloseParen) {
            loop {
                let p_line = self.peek().line; let p_col = self.peek().column;
                let pname = self.consume_identifier("Expected parameter name")?;
                params.push(FunctionParam { name: pname, line: p_line, column: p_col });
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenKind::CloseParen, "Expected ')' after parameters")?;
        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Function body must be a block")),
        };
        Ok(ASTNode::new_function_at(&name, func_tok.line, func_tok.column, params, body))
    }

    fn parse_return(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Return, "Expected 'return'")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after return value")?;
        Ok(ASTNode::new_return(value))
    }

    fn parse_log(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Log, "Expected 'log'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after log")?;
        let expr = self.parse_expression()?;
        self.consume(TokenKind::CloseParen, "Expected ')' after log arg")?;
        self.consume(TokenKind::Semicolon, "Expected ';' after log")?;
        Ok(ASTNode::new_log(expr))
    }

    fn parse_if(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::If, "Expected 'if'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after if")?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::CloseParen, "Expected ')' after condition")?;
        let then_branch = self.parse_statement()?;
        let else_branch = if self.match_token(&[TokenKind::Else]) {
            Some(self.parse_statement()?)
        } else {
            None
        };
        Ok(ASTNode::new_if(cond, then_branch, else_branch))
    }

    fn parse_while(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::While, "Expected 'while'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after while")?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::CloseParen, "Expected ')' after condition")?;
        let body = self.parse_statement()?;
        Ok(ASTNode::new_while(cond, body))
    }

    fn parse_for(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::For, "Expected 'for'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after for")?;
        let init = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_statement()?)
        } else {
            self.advance(); // consume ';'
            None
        };
        let condition = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expected ';' after loop condition")?;
        let increment = if !self.check(&TokenKind::CloseParen) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(TokenKind::CloseParen, "Expected ')' after for clauses")?;
        let body = self.parse_statement()?;
        Ok(ASTNode::new_for(init, condition, increment, body))
    }

    fn parse_quantum_op(&mut self) -> Result<ASTNode, ParserError> {
        let op = self.advance().kind.clone();
        let mut qubits = Vec::new();
        if self.match_token(&[TokenKind::OpenParen]) {
            while !self.check(&TokenKind::CloseParen) {
                qubits.push(self.parse_expression()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.consume(TokenKind::CloseParen, "Expected ')' after qubits")?;
        }
        self.consume(TokenKind::Semicolon, "Expected ';' after quantum op")?;
        Ok(ASTNode::new_quantum_op(op, qubits))
    }

    fn parse_hieroglyphic_op(&mut self) -> Result<ASTNode, ParserError> {
        let symbol = match self.advance().kind.clone() {
            TokenKind::HieroglyphicOp(sym) => sym,
            _ => return Err(self.err_here("Expected hieroglyphic symbol")),
        };
        let mut args = Vec::new();
        if self.match_token(&[TokenKind::OpenParen]) {
            while !self.check(&TokenKind::CloseParen) {
                args.push(self.parse_expression()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.consume(TokenKind::CloseParen, "Expected ')' after args")?;
        }
        self.consume(TokenKind::Semicolon, "Expected ';' after hieroglyphic op")?;
        Ok(ASTNode::new_hieroglyphic_op(&symbol, args))
    }

    /* ── Precedence ───────────────────────────────────────── */
    pub fn parse_expression(&mut self) -> Result<ASTNode, ParserError> {
        self.parse_assignment()
    }

    // assignment: Identifier '=' assignment | equality
    fn parse_assignment(&mut self) -> Result<ASTNode, ParserError> {
        let expr = self.parse_equality()?;
        if self.match_token(&[TokenKind::Equals]) {
            match expr {
                ASTNode::Identifier(name) => {
                    let line = self.previous().line; let column = self.previous().column; let value = self.parse_assignment()?; Ok(ASTNode::new_assignment_at(&name, value, line, column))
                }
                ASTNode::IdentifierSpanned { name, line: id_line, column: id_col, .. } => {
                    // use identifier's own position for better span
                    let value = self.parse_assignment()?; Ok(ASTNode::new_assignment_at(name, value, *id_line, *id_col))
                }
                _ => Err(self.err_here("Invalid assignment target")),
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_equality(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_comparison()?;
        while self.match_token(&[TokenKind::DoubleEquals, TokenKind::NotEquals]) {
            let op = self.previous().kind.clone();
            let right = self.parse_comparison()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_term()?;
        while self.match_token(&[
            TokenKind::LessThan,
            TokenKind::LessEqual,
            TokenKind::GreaterThan,
            TokenKind::GreaterEqual,
        ]) {
            let op = self.previous().kind.clone();
            let right = self.parse_term()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_factor()?;
        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = self.previous().kind.clone();
            let right = self.parse_factor()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_unary()?;
        while self.match_token(&[TokenKind::Star, TokenKind::Slash]) {
            let op = self.previous().kind.clone();
            let right = self.parse_unary()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ASTNode, ParserError> {
        if self.match_token(&[TokenKind::Minus, TokenKind::Plus]) {
            let op = self.previous().kind.clone();
            let right = self.parse_unary()?;
            return Ok(ASTNode::new_unary_expr(op, right));
        }
        self.parse_call()
    }

    // support simple calls: primary ('(' args? ')')*
    fn parse_call(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.match_token(&[TokenKind::OpenParen]) {
                let mut args = Vec::new();
                if !self.check(&TokenKind::CloseParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::CloseParen, "Expected ')' after arguments")?;
                expr = ASTNode::new_call(expr, args);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParserError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::NumberLiteral(v) => Ok(ASTNode::NumberLiteral(v)),
            TokenKind::StringLiteral(s) => Ok(ASTNode::StringLiteral(s)),
            TokenKind::BooleanLiteral(b) => Ok(ASTNode::BooleanLiteral(b)),
            TokenKind::Identifier(name) => Ok(ASTNode::new_identifier_spanned(&name, tok.line, tok.column, name.len())),
            TokenKind::OpenParen => {
                let expr = self.parse_expression()?;
                self.consume(TokenKind::CloseParen, "Expected ')'")?;
                Ok(expr)
            }
            _ => Err(ParserError {
                message: format!("Unexpected token {:?}", tok.kind),
                line: tok.line,
                column: tok.column,
            }),
        }
    }

    /* ── Token utils ─────────────────────────────────────── */
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        if self.pos == 0 {
            &self.tokens[0]
        } else {
            &self.tokens[self.pos - 1]
        }
    }

    fn peek(&self) -> &Token {
        // Safe: we ensure there's always an EOF at the end
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        !self.is_at_end() && &self.peek().kind == kind
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<&Token, ParserError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(self.err_at(msg, self.peek().line, self.peek().column))
        }
    }

    fn consume_identifier(&mut self, msg: &str) -> Result<String, ParserError> {
        if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            self.advance();
            Ok(name)
        } else {
            Err(self.err_at(msg, self.peek().line, self.peek().column))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EOF)
    }

    fn err_here(&self, msg: &str) -> ParserError {
        self.err_at(msg, self.peek().line, self.peek().column)
    }

    fn err_at(&self, msg: &str, line: usize, column: usize) -> ParserError {
        ParserError {
            message: msg.into(),
            line,
            column,
        }
    }
}
