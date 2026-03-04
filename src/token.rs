//! Token definitions for the Rust subset supported by VirtualRust.
//!
//! Each token represents an atomic syntactic element produced by the lexer.

/// All token types recognized by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Literals ────────────────────────────────────────────────────
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // ── Identifier ──────────────────────────────────────────────────
    Ident(String),

    // ── Keywords ────────────────────────────────────────────────────
    Let,
    Mut,
    Fn,
    Return,
    If,
    Else,
    While,
    Loop,
    For,
    In,
    Break,
    Continue,
    Struct,
    Impl,
    Self_,
    Enum,
    Match,
    Pub,
    Use,
    Mod,
    As,
    Ref,

    // ── Type keywords ────────────────────────────────────────────────
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    Str,
    String_,
    Usize,
    Isize,

    // ── Operators ───────────────────────────────────────────────────
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Not,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    Shl,
    Shr,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,

    // ── Delimiters ──────────────────────────────────────────────────
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // ── Punctuation ─────────────────────────────────────────────────
    Comma,
    Semicolon,
    Colon,
    ColonColon,
    Arrow,    // ->
    FatArrow, // =>
    Dot,
    DotDot,     // ..
    DotDotEq,   // ..=
    Hash,       // #
    Underscore, // _

    // ── Special ─────────────────────────────────────────────────────
    Eof,
}

impl Token {
    /// Returns the keyword `Token` for a given string, or `None` if not a keyword.
    pub fn keyword_from_str(s: &str) -> Option<Token> {
        match s {
            "let" => Some(Token::Let),
            "mut" => Some(Token::Mut),
            "fn" => Some(Token::Fn),
            "return" => Some(Token::Return),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "while" => Some(Token::While),
            "loop" => Some(Token::Loop),
            "for" => Some(Token::For),
            "in" => Some(Token::In),
            "break" => Some(Token::Break),
            "continue" => Some(Token::Continue),
            "struct" => Some(Token::Struct),
            "impl" => Some(Token::Impl),
            "self" => Some(Token::Self_),
            "enum" => Some(Token::Enum),
            "match" => Some(Token::Match),
            "pub" => Some(Token::Pub),
            "use" => Some(Token::Use),
            "mod" => Some(Token::Mod),
            "as" => Some(Token::As),
            "ref" => Some(Token::Ref),
            "true" => Some(Token::BoolLiteral(true)),
            "false" => Some(Token::BoolLiteral(false)),
            "i8" => Some(Token::I8),
            "i16" => Some(Token::I16),
            "i32" => Some(Token::I32),
            "i64" => Some(Token::I64),
            "i128" => Some(Token::I128),
            "u8" => Some(Token::U8),
            "u16" => Some(Token::U16),
            "u32" => Some(Token::U32),
            "u64" => Some(Token::U64),
            "u128" => Some(Token::U128),
            "f32" => Some(Token::F32),
            "f64" => Some(Token::F64),
            "bool" => Some(Token::Bool),
            "char" => Some(Token::Char),
            "str" => Some(Token::Str),
            "String" => Some(Token::String_),
            "usize" => Some(Token::Usize),
            "isize" => Some(Token::Isize),
            "_" => Some(Token::Underscore),
            _ => None,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::IntLiteral(n) => write!(f, "{}", n),
            Token::FloatLiteral(n) => write!(f, "{}", n),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::CharLiteral(c) => write!(f, "'{}'", c),
            Token::BoolLiteral(b) => write!(f, "{}", b),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Let => write!(f, "let"),
            Token::Mut => write!(f, "mut"),
            Token::Fn => write!(f, "fn"),
            Token::Return => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::Loop => write!(f, "loop"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Struct => write!(f, "struct"),
            Token::Impl => write!(f, "impl"),
            Token::Self_ => write!(f, "self"),
            Token::Enum => write!(f, "enum"),
            Token::Match => write!(f, "match"),
            _ => write!(f, "{:?}", self),
        }
    }
}
