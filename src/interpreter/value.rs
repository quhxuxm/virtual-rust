//! Runtime value representation for the Virtual Rust interpreter.
//!
//! This module defines [`Value`], the core enum that represents all possible
//! values that can exist at runtime during interpretation.

use std::collections::HashMap;
use std::fmt;

use crate::ast::{Expr, Type};
use crate::interpreter::environment::Environment;

/// A runtime value produced by evaluating an expression.
///
/// Every result of evaluation is represented as a `Value`. This includes
/// primitive types (int, float, bool, char, string), compound types
/// (array, tuple, struct), callable types (function, closure), and
/// control-flow sentinels (break, continue, return).
#[derive(Debug, Clone)]
pub enum Value {
    // ── Primitive types ──────────────────────────────────────────────
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),

    // ── Compound types ───────────────────────────────────────────────
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },

    // ── Callable types ───────────────────────────────────────────────
    Function {
        name: String,
        params: Vec<(String, Type)>,
        body: Box<Expr>,
        closure_env: Option<Environment>,
    },
    Closure {
        params: Vec<(String, Option<Type>)>,
        body: Box<Expr>,
        env: Environment,
    },

    // ── Special types ────────────────────────────────────────────────
    Option(Option<Box<Value>>),
    Unit,

    // ── Control-flow sentinels ───────────────────────────────────────
    /// Signals a `break` with an optional value.
    Break(Option<Box<Value>>),
    /// Signals a `continue`.
    Continue,
    /// Signals a `return` with an optional value.
    Return(Option<Box<Value>>),
}

// ── Display ──────────────────────────────────────────────────────────────

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => {
                if *n == n.floor() && !n.is_infinite() && !n.is_nan() {
                    write!(f, "{n:.1}")
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Bool(b) => write!(f, "{b}"),
            Value::Char(c) => write!(f, "{c}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Array(arr) => write_array(f, arr),
            Value::Tuple(elems) => write_tuple(f, elems),
            Value::Struct { name, fields } => write_struct(f, name, fields),
            Value::Function { name, .. } => write!(f, "<fn {name}>"),
            Value::Closure { .. } => write!(f, "<closure>"),
            Value::Option(Some(v)) => write!(f, "Some({v})"),
            Value::Option(None) => write!(f, "None"),
            Value::Unit => write!(f, "()"),
            Value::Break(_) => write!(f, "<break>"),
            Value::Continue => write!(f, "<continue>"),
            Value::Return(_) => write!(f, "<return>"),
        }
    }
}

fn write_array(f: &mut fmt::Formatter<'_>, arr: &[Value]) -> fmt::Result {
    write!(f, "[")?;
    for (i, v) in arr.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        match v {
            Value::String(s) => write!(f, "\"{s}\"")?,
            Value::Char(c) => write!(f, "'{c}'")?,
            _ => write!(f, "{v}")?,
        }
    }
    write!(f, "]")
}

fn write_tuple(f: &mut fmt::Formatter<'_>, elements: &[Value]) -> fmt::Result {
    write!(f, "(")?;
    for (i, v) in elements.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{v}")?;
    }
    if elements.len() == 1 {
        write!(f, ",")?;
    }
    write!(f, ")")
}

fn write_struct(
    f: &mut fmt::Formatter<'_>,
    name: &str,
    fields: &HashMap<String, Value>,
) -> fmt::Result {
    write!(f, "{name} {{ ")?;
    for (i, (k, v)) in fields.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{k}: {v}")?;
    }
    write!(f, " }}")
}

// ── Value helpers ────────────────────────────────────────────────────────

impl Value {
    /// Returns `true` if the value is considered "truthy" in boolean context.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Option(v) => v.is_some(),
            Value::Unit => false,
            _ => true,
        }
    }

    /// Returns the human-readable name of this value's type.
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "i64",
            Value::Float(_) => "f64",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
            Value::String(_) => "String",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Struct { name, .. } => name,
            Value::Function { .. } => "function",
            Value::Closure { .. } => "closure",
            Value::Option(_) => "Option",
            Value::Unit => "()",
            Value::Break(_) => "break",
            Value::Continue => "continue",
            Value::Return(_) => "return",
        }
    }

    /// Debug-style format that shows quotes around strings and chars.
    pub fn debug_fmt(&self) -> String {
        match self {
            Value::String(s) => format!("\"{s}\""),
            Value::Char(c) => format!("'{c}'"),
            other => format!("{other}"),
        }
    }
}
