//! Variable scoping and environment management.
//!
//! The [`Environment`] maintains a stack of scopes, where each scope is a
//! `HashMap` mapping variable names to their [`Variable`] (value + mutability).
//! New scopes are pushed on function calls and block entries, and popped on exit.

use std::collections::HashMap;

use crate::interpreter::error::RuntimeError;
use crate::interpreter::value::Value;

/// A stored variable with its current value and mutability flag.
#[derive(Debug, Clone)]
struct Variable {
    value: Value,
    mutable: bool,
}

/// Scoped variable environment using a stack of hash maps.
///
/// Variables are looked up from the innermost scope outward. Assignments
/// respect mutability — attempting to assign to an immutable binding
/// produces a [`RuntimeError`].
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Variable>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    /// Creates a new environment with a single (global) scope.
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    /// Pushes a new empty scope onto the stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pops the innermost scope from the stack.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Defines a new variable in the current (innermost) scope.
    pub fn define(&mut self, name: String, value: Value, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, Variable { value, mutable });
        }
    }

    /// Looks up a variable by name, searching from innermost to outermost scope.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|var| &var.value))
    }

    /// Sets an existing variable's value. Returns an error if the variable
    /// is not found or is immutable.
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                if !var.mutable {
                    return Err(RuntimeError::new(format!(
                        "Cannot assign to immutable variable '{name}'"
                    )));
                }
                var.value = value;
                return Ok(());
            }
        }
        Err(RuntimeError::new(format!(
            "Undefined variable: '{name}'"
        )))
    }
}
