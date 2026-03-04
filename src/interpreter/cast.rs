//! Type casting (`as` expressions).

use crate::ast::Type;
use crate::interpreter::error::RuntimeError;
use crate::interpreter::value::Value;

/// Casts a [`Value`] to the specified target [`Type`].
///
/// Follows Rust's `as` casting semantics for numeric types.
/// Unknown casts are treated as no-ops.
pub fn type_cast(value: Value, target: &Type) -> Result<Value, RuntimeError> {
    match (value, target) {
        // ── Int → Int (narrowing/widening) ───────────────────────────
        (Value::Int(n), Type::I8) => Ok(Value::Int(n as i8 as i64)),
        (Value::Int(n), Type::I16) => Ok(Value::Int(n as i16 as i64)),
        (Value::Int(n), Type::I32) => Ok(Value::Int(n as i32 as i64)),
        (Value::Int(n), Type::I64 | Type::I128) => Ok(Value::Int(n)),
        (Value::Int(n), Type::U8) => Ok(Value::Int(n as u8 as i64)),
        (Value::Int(n), Type::U16) => Ok(Value::Int(n as u16 as i64)),
        (Value::Int(n), Type::U32) => Ok(Value::Int(n as u32 as i64)),
        (Value::Int(n), Type::U64 | Type::U128) => Ok(Value::Int(n as u64 as i64)),
        (Value::Int(n), Type::Usize) => Ok(Value::Int(n as usize as i64)),
        (Value::Int(n), Type::Isize) => Ok(Value::Int(n as isize as i64)),

        // ── Int → Float ──────────────────────────────────────────────
        (Value::Int(n), Type::F32) => Ok(Value::Float(n as f32 as f64)),
        (Value::Int(n), Type::F64) => Ok(Value::Float(n as f64)),

        // ── Int → Char ───────────────────────────────────────────────
        (Value::Int(n), Type::Char) => {
            Ok(Value::Char(char::from_u32(n as u32).unwrap_or('\0')))
        }

        // ── Float → Int ──────────────────────────────────────────────
        (Value::Float(n), Type::I32) => Ok(Value::Int(n as i32 as i64)),
        (Value::Float(n), Type::I64) => Ok(Value::Int(n as i64)),
        (Value::Float(n), Type::U32) => Ok(Value::Int(n as u32 as i64)),
        (Value::Float(n), Type::U64) => Ok(Value::Int(n as u64 as i64)),
        (Value::Float(n), Type::Usize) => Ok(Value::Int(n as usize as i64)),

        // ── Float → Float ────────────────────────────────────────────
        (Value::Float(n), Type::F32) => Ok(Value::Float(n as f32 as f64)),
        (Value::Float(n), Type::F64) => Ok(Value::Float(n)),

        // ── Char → Int ───────────────────────────────────────────────
        (Value::Char(c), Type::U8) => Ok(Value::Int(c as u8 as i64)),
        (Value::Char(c), Type::U32) => Ok(Value::Int(c as u32 as i64)),
        (Value::Char(c), Type::I32) => Ok(Value::Int(c as i32 as i64)),

        // ── Bool → Int ───────────────────────────────────────────────
        (Value::Bool(b), Type::I32 | Type::U8) => Ok(Value::Int(b as i64)),

        // ── Fallback: no-op ──────────────────────────────────────────
        (v, _) => Ok(v),
    }
}
