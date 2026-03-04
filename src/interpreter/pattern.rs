//! Pattern matching logic for `match` expressions.

use crate::ast::{Expr, Pattern};
use crate::interpreter::error::RuntimeError;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;

impl Interpreter {
    /// Returns `true` if the given pattern matches the given value.
    pub(crate) fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<bool, RuntimeError> {
        match pattern {
            Pattern::Wildcard | Pattern::Ident(_) => Ok(true),
            Pattern::Literal(lit) => match_literal(lit, value),
            Pattern::Range {
                start,
                end,
                inclusive,
            } => match_range(start, end, *inclusive, value),
            Pattern::Or(patterns) => {
                for p in patterns {
                    if self.match_pattern(p, value)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Binds variables introduced by the pattern (e.g. `Pattern::Ident`).
    pub(crate) fn bind_pattern(
        &mut self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Ident(name) => {
                self.env.define(name.clone(), value.clone(), true);
            }
            Pattern::Or(patterns) => {
                for p in patterns {
                    if self.match_pattern(p, value)? {
                        self.bind_pattern(p, value)?;
                        break;
                    }
                }
            }
            _ => {} // Wildcards and literals don't bind
        }
        Ok(())
    }
}

/// Compares a literal expression against a runtime value.
fn match_literal(lit: &Expr, value: &Value) -> Result<bool, RuntimeError> {
    Ok(match (lit, value) {
        (Expr::IntLiteral(a), Value::Int(b)) => a == b,
        (Expr::FloatLiteral(a), Value::Float(b)) => a == b,
        (Expr::StringLiteral(a), Value::String(b)) => a == b,
        (Expr::CharLiteral(a), Value::Char(b)) => a == b,
        (Expr::BoolLiteral(a), Value::Bool(b)) => a == b,
        _ => false,
    })
}

/// Checks whether a value falls within a range pattern.
fn match_range(
    start: &Expr,
    end: &Expr,
    inclusive: bool,
    value: &Value,
) -> Result<bool, RuntimeError> {
    Ok(match (start, end, value) {
        (Expr::IntLiteral(s), Expr::IntLiteral(e), Value::Int(v)) => {
            if inclusive {
                v >= s && v <= e
            } else {
                v >= s && v < e
            }
        }
        _ => false,
    })
}
