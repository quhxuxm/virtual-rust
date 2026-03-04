//! Recursive-descent parser that transforms a token stream into an AST.
//!
//! # Operator Precedence (lowest → highest)
//!
//! 1. Assignment (`=`, `+=`, `-=`, …)
//! 2. Logical OR (`||`)
//! 3. Logical AND (`&&`)
//! 4. Comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`)
//! 5. Bitwise OR (`|`)
//! 6. Bitwise XOR (`^`)
//! 7. Bitwise AND (`&`)
//! 8. Shift (`<<`, `>>`)
//! 9. Additive (`+`, `-`)
//! 10. Multiplicative (`*`, `/`, `%`)
//! 11. Type cast (`as`)
//! 12. Unary (`-`, `!`, `&`, `*`)
//! 13. Postfix (`.method()`, `[index]`, `(args)`)
//! 14. Primary (literals, identifiers, blocks, closures)

use crate::ast::*;
use crate::token::Token;

/// Recursive-descent parser for a subset of Rust syntax.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

/// A parse-time error with a human-readable message.
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        ParseError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

// ── Token navigation ─────────────────────────────────────────────────────

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Returns the current token without advancing.
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    /// Returns the next token (one ahead) without advancing.
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(&Token::Eof)
    }

    /// Consumes and returns the current token.
    fn advance(&mut self) -> Token {
        let tok = self.current().clone();
        self.pos += 1;
        tok
    }

    /// Consumes the current token if it matches `expected`, otherwise errors.
    fn expect(&mut self, expected: &Token) -> Result<Token, ParseError> {
        if std::mem::discriminant(self.current()) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(format!(
                "Expected {expected:?}, found {:?}",
                self.current()
            )))
        }
    }

    /// Consumes the current token if it matches `expected`. Returns whether it matched.
    fn eat(&mut self, expected: &Token) -> bool {
        if std::mem::discriminant(self.current()) == std::mem::discriminant(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ── Program & statements ─────────────────────────────────────────

    /// Parses the entire program as a sequence of top-level statements.
    pub fn parse_program(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut stmts = Vec::new();
        while *self.current() != Token::Eof {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    /// Parses a single statement (let, fn, if, while, loop, for, etc.).
    fn parse_statement(&mut self) -> Result<Expr, ParseError> {
        match self.current() {
            Token::Let => self.parse_let(),
            Token::Fn => self.parse_fn_def(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Loop => self.parse_loop(),
            Token::For => self.parse_for(),
            Token::Return => self.parse_return(),
            Token::Break => self.parse_break(),
            Token::Continue => {
                self.advance();
                self.eat(&Token::Semicolon);
                Ok(Expr::Continue)
            }
            Token::Struct => self.parse_struct_def(),
            Token::LBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expr()?;
                self.eat(&Token::Semicolon);
                Ok(expr)
            }
        }
    }

    // ── Let binding ──────────────────────────────────────────────────

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Let)?;
        let mutable = self.eat(&Token::Mut);

        let name = match self.advance() {
            Token::Ident(name) => name,
            Token::Underscore => "_".to_string(),
            other => {
                return Err(ParseError::new(format!(
                    "Expected identifier after let, found {other:?}"
                )));
            }
        };

        let type_ann = if self.eat(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if self.eat(&Token::Eq) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.eat(&Token::Semicolon);

        Ok(Expr::Let {
            name,
            mutable,
            type_ann,
            value,
        })
    }

    // ── Type annotations ─────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        // Handle reference types: &T, &mut T
        if self.eat(&Token::Ampersand) {
            let mutable = self.eat(&Token::Mut);
            let inner = self.parse_type()?;
            return Ok(Type::Reference(Box::new(inner), mutable));
        }

        // Map simple type-keyword tokens to their AST type
        let simple = match self.current() {
            Token::I8 => Some(Type::I8),
            Token::I16 => Some(Type::I16),
            Token::I32 => Some(Type::I32),
            Token::I64 => Some(Type::I64),
            Token::I128 => Some(Type::I128),
            Token::U8 => Some(Type::U8),
            Token::U16 => Some(Type::U16),
            Token::U32 => Some(Type::U32),
            Token::U64 => Some(Type::U64),
            Token::U128 => Some(Type::U128),
            Token::F32 => Some(Type::F32),
            Token::F64 => Some(Type::F64),
            Token::Bool => Some(Type::Bool),
            Token::Char => Some(Type::Char),
            Token::Str | Token::String_ => Some(Type::String),
            Token::Usize => Some(Type::Usize),
            Token::Isize => Some(Type::Isize),
            _ => None,
        };
        if let Some(ty) = simple {
            self.advance();
            return Ok(ty);
        }

        let ty = match self.current() {
            Token::LParen => {
                self.advance();
                if self.eat(&Token::RParen) {
                    return Ok(Type::Unit);
                }
                let mut types = vec![self.parse_type()?];
                while self.eat(&Token::Comma) {
                    types.push(self.parse_type()?);
                }
                self.expect(&Token::RParen)?;
                Type::Tuple(types)
            }
            Token::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                let size = if self.eat(&Token::Semicolon) {
                    let size_expr = self.parse_expr()?;
                    match size_expr {
                        Expr::IntLiteral(n) => Some(n as usize),
                        _ => None,
                    }
                } else {
                    None
                };
                self.expect(&Token::RBracket)?;
                Type::Array(Box::new(inner), size)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                match name.as_str() {
                    "Vec" => {
                        if self.eat(&Token::Lt) {
                            let inner = self.parse_type()?;
                            self.expect(&Token::Gt)?;
                            Type::Vec(Box::new(inner))
                        } else {
                            Type::Custom(name)
                        }
                    }
                    "Option" => {
                        if self.eat(&Token::Lt) {
                            let inner = self.parse_type()?;
                            self.expect(&Token::Gt)?;
                            Type::Option(Box::new(inner))
                        } else {
                            Type::Custom(name)
                        }
                    }
                    _ => Type::Custom(name),
                }
            }
            _ => {
                return Err(ParseError::new(format!(
                    "Expected type, found {:?}",
                    self.current()
                )));
            }
        };

        Ok(ty)
    }

    // ── Function definitions ──────────────────────────────────────────

    fn parse_fn_def(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Fn)?;
        let name = self.expect_ident("function name")?;

        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        while *self.current() != Token::RParen {
            let param_name = match self.advance() {
                Token::Ident(name) => name,
                Token::Underscore => "_".to_string(),
                other => {
                    return Err(ParseError::new(format!(
                        "Expected parameter name, found {other:?}"
                    )));
                }
            };
            self.expect(&Token::Colon)?;
            let param_type = self.parse_type()?;
            params.push((param_name, param_type));
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RParen)?;

        let return_type = if self.eat(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = Box::new(self.parse_block()?);

        Ok(Expr::FnDef {
            name,
            params,
            return_type,
            body,
        })
    }

    // ── Block & control flow ────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while *self.current() != Token::RBrace && *self.current() != Token::Eof {
            stmts.push(self.parse_statement()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::Block(stmts))
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::If)?;
        let condition = Box::new(self.parse_expr()?);
        let then_block = Box::new(self.parse_block()?);

        let else_block = if self.eat(&Token::Else) {
            if *self.current() == Token::If {
                Some(Box::new(self.parse_if()?))
            } else {
                Some(Box::new(self.parse_block()?))
            }
        } else {
            None
        };

        Ok(Expr::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::While)?;
        let condition = Box::new(self.parse_expr()?);
        let body = Box::new(self.parse_block()?);
        Ok(Expr::While { condition, body })
    }

    fn parse_loop(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Loop)?;
        let body = Box::new(self.parse_block()?);
        Ok(Expr::Loop { body })
    }

    fn parse_for(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::For)?;
        let var = match self.advance() {
            Token::Ident(name) => name,
            Token::Underscore => "_".to_string(),
            other => {
                return Err(ParseError::new(format!(
                    "Expected variable name in for loop, found {other:?}"
                )));
            }
        };
        self.expect(&Token::In)?;
        let iterator = Box::new(self.parse_expr()?);
        let body = Box::new(self.parse_block()?);
        Ok(Expr::For {
            var,
            iterator,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Return)?;
        if *self.current() == Token::Semicolon || *self.current() == Token::RBrace {
            self.eat(&Token::Semicolon);
            Ok(Expr::Return(None))
        } else {
            let value = self.parse_expr()?;
            self.eat(&Token::Semicolon);
            Ok(Expr::Return(Some(Box::new(value))))
        }
    }

    fn parse_break(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Break)?;
        if *self.current() == Token::Semicolon || *self.current() == Token::RBrace {
            self.eat(&Token::Semicolon);
            Ok(Expr::Break(None))
        } else {
            let value = self.parse_expr()?;
            self.eat(&Token::Semicolon);
            Ok(Expr::Break(Some(Box::new(value))))
        }
    }

    // ── Struct definitions ─────────────────────────────────────────────

    fn parse_struct_def(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Struct)?;
        let name = self.expect_ident("struct name")?;

        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while *self.current() != Token::RBrace {
            self.eat(&Token::Pub); // skip optional `pub`
            let field_name = self.expect_ident("field name")?;
            self.expect(&Token::Colon)?;
            let field_type = self.parse_type()?;
            fields.push((field_name, field_type));
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RBrace)?;

        Ok(Expr::StructDef { name, fields })
    }

    // ── Match expressions ────────────────────────────────────────────

    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::Match)?;
        let expr = Box::new(self.parse_expr()?);
        self.expect(&Token::LBrace)?;

        let mut arms = Vec::new();
        while *self.current() != Token::RBrace {
            let pattern = self.parse_pattern()?;
            self.expect(&Token::FatArrow)?;

            let body = if *self.current() == Token::LBrace {
                self.parse_block()?
            } else {
                self.parse_expr()?
            };

            self.eat(&Token::Comma);
            arms.push(MatchArm { pattern, body });
        }
        self.expect(&Token::RBrace)?;

        Ok(Expr::Match { expr, arms })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let first = self.parse_single_pattern()?;

        if self.eat(&Token::Pipe) {
            let mut patterns = vec![first];
            patterns.push(self.parse_single_pattern()?);
            while self.eat(&Token::Pipe) {
                patterns.push(self.parse_single_pattern()?);
            }
            Ok(Pattern::Or(patterns))
        } else {
            Ok(first)
        }
    }

    fn parse_single_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.current() {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::IntLiteral(_)
            | Token::FloatLiteral(_)
            | Token::StringLiteral(_)
            | Token::CharLiteral(_)
            | Token::BoolLiteral(_) => {
                let lit = self.parse_primary()?;
                // Check for range pattern
                if *self.current() == Token::DotDot || *self.current() == Token::DotDotEq {
                    let inclusive = *self.current() == Token::DotDotEq;
                    self.advance();
                    let end = self.parse_primary()?;
                    Ok(Pattern::Range {
                        start: Box::new(lit),
                        end: Box::new(end),
                        inclusive,
                    })
                } else {
                    Ok(Pattern::Literal(lit))
                }
            }
            Token::Minus => {
                self.advance();
                if let Token::IntLiteral(n) = self.current().clone() {
                    self.advance();
                    Ok(Pattern::Literal(Expr::IntLiteral(-n)))
                } else {
                    Err(ParseError::new(format!(
                        "Expected number after minus in pattern, found {:?}",
                        self.current()
                    )))
                }
            }
            Token::Ident(_) => {
                let Token::Ident(name) = self.advance() else {
                    unreachable!()
                };
                Ok(Pattern::Ident(name))
            }
            _ => Err(ParseError::new(format!(
                "Expected pattern, found {:?}",
                self.current()
            ))),
        }
    }

    // ── Expression parsing (by precedence) ───────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    /// Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `%=`
    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_or()?;

        // Check for compound assignment operators
        let compound_op = match self.current() {
            Token::PlusEq => Some(BinOp::Add),
            Token::MinusEq => Some(BinOp::Sub),
            Token::StarEq => Some(BinOp::Mul),
            Token::SlashEq => Some(BinOp::Div),
            Token::PercentEq => Some(BinOp::Mod),
            _ => None,
        };

        if let Some(op) = compound_op {
            self.advance();
            let value = self.parse_expr()?;
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op,
                value: Box::new(value),
            });
        }

        if self.eat(&Token::Eq) {
            let value = self.parse_expr()?;
            return Ok(Expr::Assign {
                target: Box::new(expr),
                value: Box::new(value),
            });
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while *self.current() == Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        while *self.current() == Token::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_or()?;
        loop {
            let op = match self.current() {
                Token::EqEq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                Token::Lt => BinOp::Lt,
                Token::LtEq => BinOp::LtEq,
                Token::Gt => BinOp::Gt,
                Token::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_bitwise_or()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_xor()?;
        while *self.current() == Token::Pipe {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::BitOr,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_and()?;
        while *self.current() == Token::Caret {
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::BitXor,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_shift()?;
        while *self.current() == Token::Ampersand {
            self.advance();
            let right = self.parse_shift()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::BitAnd,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.current() {
                Token::Shl => BinOp::Shl,
                Token::Shr => BinOp::Shr,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.current() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_type_cast()?;
        loop {
            let op = match self.current() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_type_cast()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_type_cast(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        while self.eat(&Token::As) {
            let target_type = self.parse_type()?;
            expr = Expr::TypeCast {
                expr: Box::new(expr),
                target_type,
            };
        }
        Ok(expr)
    }

    /// Unary: `-`, `!`, `&`, `*`
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.current() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Token::Ampersand => {
                self.advance();
                let mutable = self.eat(&Token::Mut);
                let expr = self.parse_unary()?;
                Ok(Expr::Ref {
                    expr: Box::new(expr),
                    mutable,
                })
            }
            Token::Star => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Deref(Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    /// Postfix: `.method()`, `.field`, `[index]`, `(args)`, `..`, `..=`
    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current() {
                Token::Dot => {
                    self.advance();
                    let method_name = self.expect_ident("method/field name after '.'")?;

                    if *self.current() == Token::LParen {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: method_name,
                            args,
                        };
                    } else {
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field: method_name,
                        };
                    }
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::LParen => {
                    // Function call on an expression
                    if let Expr::Ident(ref name) = expr {
                        let name = name.clone();
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;
                        expr = Expr::FnCall { name, args };
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        // Check for range expressions
        if *self.current() == Token::DotDot || *self.current() == Token::DotDotEq {
            let inclusive = *self.current() == Token::DotDotEq;
            self.advance();
            // Check if there's an end expression
            let end = if *self.current() != Token::RBrace
                && *self.current() != Token::RParen
                && *self.current() != Token::RBracket
                && *self.current() != Token::Semicolon
                && *self.current() != Token::Comma
                && *self.current() != Token::Eof
            {
                Some(Box::new(self.parse_additive()?))
            } else {
                None
            };
            expr = Expr::Range {
                start: Some(Box::new(expr)),
                end,
                inclusive,
            };
        }

        Ok(expr)
    }

    /// Primary: literals, identifiers, macros, struct init, blocks, closures.
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current().clone() {
            Token::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            Token::FloatLiteral(n) => {
                self.advance();
                Ok(Expr::FloatLiteral(n))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            Token::CharLiteral(c) => {
                self.advance();
                Ok(Expr::CharLiteral(c))
            }
            Token::BoolLiteral(b) => {
                self.advance();
                Ok(Expr::BoolLiteral(b))
            }
            Token::Ident(name) => {
                self.advance();

                // Check for macro invocation: name!(...)
                if *self.current() == Token::Not
                    && (*self.peek() == Token::LParen || *self.peek() == Token::LBracket)
                {
                    self.advance(); // eat !

                    if name == "vec" {
                        self.expect(&Token::LBracket)?;
                        let args = self.parse_args()?;
                        self.expect(&Token::RBracket)?;
                        return Ok(Expr::VecMacro(args));
                    }

                    let (open, close) = if *self.current() == Token::LBracket {
                        (Token::LBracket, Token::RBracket)
                    } else {
                        (Token::LParen, Token::RParen)
                    };
                    self.expect(&open)?;
                    let args = self.parse_macro_args()?;
                    self.expect(&close)?;
                    return Ok(Expr::MacroCall { name, args });
                }

                // Check for struct initialization: Name { ... }
                if *self.current() == Token::LBrace && self.is_struct_init_context(&name) {
                    return self.parse_struct_init(name);
                }

                // Check for function call: name(...)
                if *self.current() == Token::LParen {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::FnCall { name, args });
                }

                // Check for path call: name::method(...)
                if *self.current() == Token::ColonColon {
                    self.advance();
                    let method = self.expect_ident("method name after '::'")?;

                    let full_name = format!("{}::{}", name, method);

                    if *self.current() == Token::LParen {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;
                        return Ok(Expr::FnCall {
                            name: full_name,
                            args,
                        });
                    }

                    return Ok(Expr::Ident(full_name));
                }

                Ok(Expr::Ident(name))
            }
            Token::LParen => {
                self.advance();
                if *self.current() == Token::RParen {
                    self.advance();
                    return Ok(Expr::Unit);
                }

                let expr = self.parse_expr()?;

                // Check if it's a tuple
                if *self.current() == Token::Comma {
                    let mut elements = vec![expr];
                    while self.eat(&Token::Comma) {
                        if *self.current() == Token::RParen {
                            break;
                        }
                        elements.push(self.parse_expr()?);
                    }
                    self.expect(&Token::RParen)?;
                    return Ok(Expr::TupleLiteral(elements));
                }

                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance();
                if *self.current() == Token::RBracket {
                    self.advance();
                    return Ok(Expr::ArrayLiteral(Vec::new()));
                }

                let first = self.parse_expr()?;

                // Check for array repeat syntax: [expr; count]
                if self.eat(&Token::Semicolon) {
                    let count = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    return Ok(Expr::ArrayRepeat {
                        value: Box::new(first),
                        count: Box::new(count),
                    });
                }

                let mut elements = vec![first];
                while self.eat(&Token::Comma) {
                    if *self.current() == Token::RBracket {
                        break;
                    }
                    elements.push(self.parse_expr()?);
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::ArrayLiteral(elements))
            }
            Token::If => self.parse_if(),
            Token::Match => self.parse_match(),
            Token::Loop => self.parse_loop(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::LBrace => self.parse_block(),
            Token::Pipe => {
                // Closure: |params| body
                self.advance();
                let mut params = Vec::new();
                while *self.current() != Token::Pipe {
                    let name = self.expect_ident("closure parameter name")?;
                    let type_ann = if self.eat(&Token::Colon) {
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    params.push((name, type_ann));
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::Pipe)?;
                let body = if *self.current() == Token::LBrace {
                    self.parse_block()?
                } else {
                    self.parse_expr()?
                };
                Ok(Expr::Closure {
                    params,
                    body: Box::new(body),
                })
            }
            Token::Or => {
                // Empty closure: || body
                self.advance();
                let body = if *self.current() == Token::LBrace {
                    self.parse_block()?
                } else {
                    self.parse_expr()?
                };
                Ok(Expr::Closure {
                    params: Vec::new(),
                    body: Box::new(body),
                })
            }
            _ => Err(ParseError::new(format!(
                "Unexpected token: {:?}",
                self.current()
            ))),
        }
    }

    // ── Argument lists ───────────────────────────────────────────────

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if *self.current() == Token::RParen || *self.current() == Token::RBracket {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while self.eat(&Token::Comma) {
            if *self.current() == Token::RParen || *self.current() == Token::RBracket {
                break;
            }
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn parse_macro_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if *self.current() == Token::RParen || *self.current() == Token::RBracket {
            return Ok(args);
        }
        // First argument (for println! this is typically a format string)
        args.push(self.parse_expr()?);
        while self.eat(&Token::Comma) {
            if *self.current() == Token::RParen || *self.current() == Token::RBracket {
                break;
            }
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    // ── Struct initialization ────────────────────────────────────────

    fn parse_struct_init(&mut self, name: String) -> Result<Expr, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while *self.current() != Token::RBrace {
            let field_name = self.expect_ident("field name")?;

            let value = if self.eat(&Token::Colon) {
                self.parse_expr()?
            } else {
                // Shorthand: { x } is equivalent to { x: x }
                Expr::Ident(field_name.clone())
            };

            fields.push((field_name, value));
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::StructInit { name, fields })
    }

    fn is_struct_init_context(&self, name: &str) -> bool {
        // Heuristic: if the name starts with an uppercase letter, it's likely a struct
        name.chars().next().is_some_and(|c| c.is_uppercase())
    }

    // ── Helpers ──────────────────────────────────────────────────────

    /// Consumes the current token if it's an identifier, returning the name.
    fn expect_ident(&mut self, context: &str) -> Result<String, ParseError> {
        match self.advance() {
            Token::Ident(name) => Ok(name),
            other => Err(ParseError::new(format!(
                "Expected {context}, found {other:?}"
            ))),
        }
    }
}
