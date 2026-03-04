//! Runtime error type for the interpreter.

use std::fmt;

/// An error produced during interpretation (e.g. type mismatch, undefined variable).
#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    /// Creates a new `RuntimeError` with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        RuntimeError {
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}
