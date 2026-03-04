//! Abstract Syntax Tree (AST) types for the VirtualRust interpreter.
//!
//! The parser produces these AST nodes, and the interpreter walks them
//! to execute the program.

/// Type annotations supported in `let` bindings and function signatures.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive integer types
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
    // Floating-point types
    F32,
    F64,
    // Other primitives
    Bool,
    Char,
    String,
    Usize,
    Isize,
    // Compound types
    Array(Box<Type>, Option<usize>),
    Vec(Box<Type>),
    Tuple(Vec<Type>),
    Option(Box<Type>),
    Custom(String),
    Unit,
    Inferred,
    /// `(inner_type, is_mutable)`
    Reference(Box<Type>, bool),
}

/// An expression or statement in the AST.
///
/// In VirtualRust, everything is an expression — even `let` bindings,
/// `if`/`while`/`for` control flow, and function definitions.
#[derive(Debug, Clone)]
pub enum Expr {
    // ── Literals ─────────────────────────────────────────────────
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // ── Identifiers ──────────────────────────────────────────────
    Ident(String),

    // ── Operators ────────────────────────────────────────────────
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    // Unary operations
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    // ── Bindings & assignments ───────────────────────────────────
    /// Simple assignment (`x = expr`).
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    /// Compound assignment (`x += expr`, `x -= expr`, etc.).
    CompoundAssign {
        target: Box<Expr>,
        op: BinOp,
        value: Box<Expr>,
    },

    /// Variable declaration (`let [mut] name [: type] = expr`).
    Let {
        name: String,
        mutable: bool,
        type_ann: Option<Type>,
        value: Option<Box<Expr>>,
    },

    // ── Control flow ─────────────────────────────────────────────
    /// A `{ ... }` block of statements.
    Block(Vec<Expr>),

    /// `if condition { ... } [else { ... }]`
    If {
        condition: Box<Expr>,
        then_block: Box<Expr>,
        else_block: Option<Box<Expr>>,
    },

    // While loop
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
    },

    // Loop
    Loop {
        body: Box<Expr>,
    },

    // For loop
    For {
        var: String,
        iterator: Box<Expr>,
        body: Box<Expr>,
    },

    // Range expression
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },

    // Break/Continue
    Break(Option<Box<Expr>>),
    Continue,

    // Return
    Return(Option<Box<Expr>>),

    // ── Functions & closures ─────────────────────────────────────
    /// Named function definition.
    FnDef {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Option<Type>,
        body: Box<Expr>,
    },

    // Function call
    FnCall {
        name: String,
        args: Vec<Expr>,
    },

    // Method call
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    // Macro call (println!, format!, etc.)
    MacroCall {
        name: String,
        args: Vec<Expr>,
    },

    // ── Collections & access ─────────────────────────────────────
    /// `[a, b, c]`
    ArrayLiteral(Vec<Expr>),

    /// `[expr; count]`
    ArrayRepeat {
        value: Box<Expr>,
        count: Box<Expr>,
    },

    // Tuple literal
    TupleLiteral(Vec<Expr>),

    // Index access
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    // Field access
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    // ── Structs ───────────────────────────────────────────────────
    /// `struct Name { ... }`
    StructDef {
        name: String,
        fields: Vec<(String, Type)>,
    },

    // Struct instantiation
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },

    // ── Match ─────────────────────────────────────────────────────
    /// `match expr { arms... }`
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    // ── Type operations ─────────────────────────────────────────
    /// `expr as Type`
    TypeCast {
        expr: Box<Expr>,
        target_type: Type,
    },

    // ── Closures & references ────────────────────────────────────
    /// `|params| body`
    Closure {
        params: Vec<(String, Option<Type>)>,
        body: Box<Expr>,
    },

    /// `&expr` or `&mut expr`
    Ref {
        expr: Box<Expr>,
        mutable: bool,
    },

    /// `*expr`
    Deref(Box<Expr>),

    /// `vec![...]`
    VecMacro(Vec<Expr>),

    /// The unit value `()`.
    Unit,
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// A single arm in a `match` expression.
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

/// A pattern used in `match` arms.
#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Expr),
    Ident(String),
    Wildcard,
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    Or(Vec<Pattern>),
}
