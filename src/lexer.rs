//! Lexical analyzer that transforms Rust source code into a stream of tokens.
//!
//! Handles all Rust literal forms: strings (with escape sequences), characters,
//! integers (decimal, hex, binary, octal with `_` separators), floats, and
//! keyword/identifier discrimination.

use crate::token::Token;

/// Lexer that tokenizes Rust source code character by character.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

/// A lexical error with source location.
#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl LexError {
    fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
        LexError {
            message: message.into(),
            line,
            col,
        }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lex error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

// ── Character navigation ─────────────────────────────────────────────

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Returns the current character without consuming it.
    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Returns the next character (lookahead) without consuming.
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    /// Consumes and returns the current character, tracking line/col.
    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    /// Creates a `LexError` at the lexer's current position.
    fn error(&self, message: impl Into<String>) -> LexError {
        LexError::new(message, self.line, self.col)
    }

    // ── Whitespace & comments ─────────────────────────────────────────

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.current() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        // skip /*
        self.advance();
        self.advance();
        let mut depth = 1;
        while depth > 0 {
            match self.current() {
                Some('/') if self.peek() == Some('*') => {
                    self.advance();
                    self.advance();
                    depth += 1;
                }
                Some('*') if self.peek() == Some('/') => {
                    self.advance();
                    self.advance();
                    depth -= 1;
                }
                Some(_) => {
                    self.advance();
                }
                None => break,
            }
        }
    }

    // ── Literal readers ──────────────────────────────────────────────

    /// Reads a `"..."`-delimited string literal with escape processing.
    fn read_string(&mut self) -> Result<Token, LexError> {
        self.advance(); // skip opening "
        let mut s = String::new();
        loop {
            match self.current() {
                Some('"') => {
                    self.advance();
                    return Ok(Token::StringLiteral(s));
                }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => {
                            s.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            s.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            s.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            s.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            s.push('"');
                            self.advance();
                        }
                        Some('0') => {
                            s.push('\0');
                            self.advance();
                        }
                        Some(c) => {
                            return Err(self.error(format!("Unknown escape sequence: \\{c}")));
                        }
                        None => {
                            return Err(self.error("Unterminated string"));
                        }
                    }
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
                None => {
                    return Err(self.error("Unterminated string"));
                }
            }
        }
    }

    /// Reads a `'c'`-delimited character literal with escape processing.
    fn read_char(&mut self) -> Result<Token, LexError> {
        self.advance(); // skip opening '
        let ch = match self.current() {
            Some('\\') => {
                self.advance();
                match self.current() {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('0') => '\0',
                    _ => return Err(self.error("Invalid char escape")),
                }
            }
            Some(c) => c,
            None => return Err(self.error("Unterminated char literal")),
        };
        self.advance();
        if self.current() != Some('\'') {
            return Err(self.error("Unterminated char literal"));
        }
        self.advance(); // skip closing '
        Ok(Token::CharLiteral(ch))
    }

    /// Reads a numeric literal (decimal, hex `0x`, binary `0b`, octal `0o`).
    /// Handles `_` separators and optional type suffixes (e.g. `42i32`).
    fn read_number(&mut self) -> Result<Token, LexError> {
        let mut num = String::new();
        let mut is_float = false;

        // Check for hex, octal, binary
        if self.current() == Some('0') {
            match self.peek() {
                Some('x') | Some('X') => {
                    num.push('0');
                    self.advance();
                    num.push('x');
                    self.advance();
                    while let Some(c) = self.current() {
                        if c.is_ascii_hexdigit() || c == '_' {
                            if c != '_' {
                                num.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val = i64::from_str_radix(&num[2..], 16).map_err(|_| {
                        self.error(format!("Invalid hex literal: {num}"))
                    })?;
                    return Ok(Token::IntLiteral(val));
                }
                Some('b') | Some('B') => {
                    self.advance();
                    self.advance();
                    while let Some(c) = self.current() {
                        if c == '0' || c == '1' || c == '_' {
                            if c != '_' {
                                num.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val = i64::from_str_radix(&num, 2).map_err(|_| {
                        self.error(format!("Invalid binary literal: 0b{num}"))
                    })?;
                    return Ok(Token::IntLiteral(val));
                }
                Some('o') | Some('O') => {
                    self.advance();
                    self.advance();
                    while let Some(c) = self.current() {
                        if ('0'..='7').contains(&c) || c == '_' {
                            if c != '_' {
                                num.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val = i64::from_str_radix(&num, 8).map_err(|_| {
                        self.error(format!("Invalid octal literal: 0o{num}"))
                    })?;
                    return Ok(Token::IntLiteral(val));
                }
                _ => {}
            }
        }

        while let Some(c) = self.current() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' {
                    num.push(c);
                }
                self.advance();
            } else if c == '.' && !is_float {
                // Check if next char is a digit (to differentiate from method calls)
                if let Some(next) = self.peek() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        num.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip type suffix like i32, u64, f64 etc.
        if let Some(c) = self.current() {
            if c == 'i' || c == 'u' || c == 'f' {
                let start = self.pos;
                let mut suffix = String::new();
                while let Some(sc) = self.current() {
                    if sc.is_alphanumeric() {
                        suffix.push(sc);
                        self.advance();
                    } else {
                        break;
                    }
                }
                // Check if it's a valid type suffix
                match suffix.as_str() {
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                    | "u64" | "u128" | "usize" | "f32" | "f64" => {
                        if suffix.starts_with('f') {
                            is_float = true;
                        }
                    }
                    _ => {
                        // Not a valid suffix, rewind
                        self.pos = start;
                    }
                }
            }
        }

        if is_float {
            let val: f64 = num.parse().map_err(|_| {
                self.error(format!("Invalid float literal: {num}"))
            })?;
            Ok(Token::FloatLiteral(val))
        } else {
            let val: i64 = num.parse().map_err(|_| {
                self.error(format!("Invalid integer literal: {num}"))
            })?;
            Ok(Token::IntLiteral(val))
        }
    }

    /// Reads an identifier or keyword.
    fn read_ident(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Token::keyword_from_str(&ident).unwrap_or(Token::Ident(ident))
    }

    // ── Main tokenizer loop ───────────────────────────────────────────

    /// Tokenizes the entire input into a `Vec<Token>`, ending with `Token::Eof`.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            match self.current() {
                None => {
                    tokens.push(Token::Eof);
                    return Ok(tokens);
                }
                Some('/') => match self.peek() {
                    Some('/') => {
                        self.skip_line_comment();
                        continue;
                    }
                    Some('*') => {
                        self.skip_block_comment();
                        continue;
                    }
                    Some('=') => {
                        self.advance();
                        self.advance();
                        tokens.push(Token::SlashEq);
                    }
                    _ => {
                        self.advance();
                        tokens.push(Token::Slash);
                    }
                },
                Some('"') => {
                    tokens.push(self.read_string()?);
                }
                Some('\'') => {
                    // Could be char literal or lifetime - try char literal first
                    tokens.push(self.read_char()?);
                }
                Some(c) if c.is_ascii_digit() => {
                    tokens.push(self.read_number()?);
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_ident());
                }
                Some('+') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::PlusEq);
                    } else {
                        tokens.push(Token::Plus);
                    }
                }
                Some('-') => {
                    self.advance();
                    if self.current() == Some('>') {
                        self.advance();
                        tokens.push(Token::Arrow);
                    } else if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::MinusEq);
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                Some('*') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::StarEq);
                    } else {
                        tokens.push(Token::Star);
                    }
                }
                Some('%') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::PercentEq);
                    } else {
                        tokens.push(Token::Percent);
                    }
                }
                Some('=') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::EqEq);
                    } else if self.current() == Some('>') {
                        self.advance();
                        tokens.push(Token::FatArrow);
                    } else {
                        tokens.push(Token::Eq);
                    }
                }
                Some('!') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::NotEq);
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                Some('<') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::LtEq);
                    } else if self.current() == Some('<') {
                        self.advance();
                        tokens.push(Token::Shl);
                    } else {
                        tokens.push(Token::Lt);
                    }
                }
                Some('>') => {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        tokens.push(Token::GtEq);
                    } else if self.current() == Some('>') {
                        self.advance();
                        tokens.push(Token::Shr);
                    } else {
                        tokens.push(Token::Gt);
                    }
                }
                Some('&') => {
                    self.advance();
                    if self.current() == Some('&') {
                        self.advance();
                        tokens.push(Token::And);
                    } else {
                        tokens.push(Token::Ampersand);
                    }
                }
                Some('|') => {
                    self.advance();
                    if self.current() == Some('|') {
                        self.advance();
                        tokens.push(Token::Or);
                    } else {
                        tokens.push(Token::Pipe);
                    }
                }
                Some('^') => {
                    self.advance();
                    tokens.push(Token::Caret);
                }
                Some('~') => {
                    self.advance();
                    tokens.push(Token::Tilde);
                }
                Some('(') => {
                    self.advance();
                    tokens.push(Token::LParen);
                }
                Some(')') => {
                    self.advance();
                    tokens.push(Token::RParen);
                }
                Some('{') => {
                    self.advance();
                    tokens.push(Token::LBrace);
                }
                Some('}') => {
                    self.advance();
                    tokens.push(Token::RBrace);
                }
                Some('[') => {
                    self.advance();
                    tokens.push(Token::LBracket);
                }
                Some(']') => {
                    self.advance();
                    tokens.push(Token::RBracket);
                }
                Some(',') => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                Some(';') => {
                    self.advance();
                    tokens.push(Token::Semicolon);
                }
                Some(':') => {
                    self.advance();
                    if self.current() == Some(':') {
                        self.advance();
                        tokens.push(Token::ColonColon);
                    } else {
                        tokens.push(Token::Colon);
                    }
                }
                Some('.') => {
                    self.advance();
                    if self.current() == Some('.') {
                        self.advance();
                        if self.current() == Some('=') {
                            self.advance();
                            tokens.push(Token::DotDotEq);
                        } else {
                            tokens.push(Token::DotDot);
                        }
                    } else {
                        tokens.push(Token::Dot);
                    }
                }
                Some('#') => {
                    self.advance();
                    // Skip attributes like #[...]
                    if self.current() == Some('[') {
                        let mut depth = 1;
                        self.advance();
                        while depth > 0 {
                            match self.current() {
                                Some('[') => {
                                    depth += 1;
                                    self.advance();
                                }
                                Some(']') => {
                                    depth -= 1;
                                    self.advance();
                                }
                                Some(_) => {
                                    self.advance();
                                }
                                None => break,
                            }
                        }
                        continue;
                    }
                    tokens.push(Token::Hash);
                }
                Some(c) => {
                    return Err(self.error(format!("Unexpected character: '{c}'")));
                }
            }
        }
    }
}
