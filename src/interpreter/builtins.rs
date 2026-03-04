//! Built-in function and macro dispatch.
//!
//! This module handles:
//! - **Built-in functions**: `print`, `println`, `String::new`, `Vec::new`, etc.
//! - **Macros**: `println!`, `format!`, `assert!`, `assert_eq!`, `vec!`, `panic!`, etc.

use crate::ast::Expr;
use crate::interpreter::error::RuntimeError;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;

// ── Built-in functions ───────────────────────────────────────────────────

impl Interpreter {
    /// Tries to call a built-in function by name.
    /// Returns `Ok(Some(value))` if handled, `Ok(None)` if not a built-in.
    pub(crate) fn call_builtin(
        &mut self,
        name: &str,
        args: &[Value],
    ) -> Result<Option<Value>, RuntimeError> {
        match name {
            "print" => {
                for arg in args {
                    print!("{arg}");
                }
                Ok(Some(Value::Unit))
            }
            "println" => {
                for arg in args {
                    print!("{arg}");
                }
                println!();
                Ok(Some(Value::Unit))
            }
            "eprintln" => {
                for arg in args {
                    eprint!("{arg}");
                }
                eprintln!();
                Ok(Some(Value::Unit))
            }
            "dbg" => {
                if let Some(arg) = args.first() {
                    eprintln!("[dbg] = {}", arg.debug_fmt());
                    Ok(Some(arg.clone()))
                } else {
                    Ok(Some(Value::Unit))
                }
            }
            "String::new" => Ok(Some(Value::String(String::new()))),
            "String::from" => {
                let s = match args.first() {
                    Some(Value::String(s)) => s.clone(),
                    Some(v) => format!("{v}"),
                    None => String::new(),
                };
                Ok(Some(Value::String(s)))
            }
            "Vec::new" | "Vec::with_capacity" => Ok(Some(Value::Array(Vec::new()))),
            "Some" => {
                let val = args
                    .first()
                    .ok_or_else(|| RuntimeError::new("Some() requires one argument"))?;
                Ok(Some(Value::Option(Some(Box::new(val.clone())))))
            }
            "std::io::stdin" | "io::stdin" => Ok(Some(Value::Unit)),
            _ => Ok(None),
        }
    }
}

// ── Macro dispatch ───────────────────────────────────────────────────────

impl Interpreter {
    /// Evaluates a macro invocation (e.g. `println!`, `assert_eq!`, `vec!`).
    pub(crate) fn call_macro(&mut self, name: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
        match name {
            "println" => self.macro_print(args, true, false),
            "print" => self.macro_print(args, false, false),
            "eprintln" => self.macro_print(args, true, true),
            "eprint" => self.macro_print(args, false, true),
            "format" => self.macro_format(args),
            "dbg" => self.macro_dbg(args),
            "assert" => self.macro_assert(args),
            "assert_eq" => self.macro_assert_eq(args),
            "assert_ne" => self.macro_assert_ne(args),
            "panic" => self.macro_panic(args),
            "todo" => Err(RuntimeError::new("not yet implemented")),
            "unimplemented" => Err(RuntimeError::new("not implemented")),
            "unreachable" => Err(RuntimeError::new("entered unreachable code")),
            "vec" => self.macro_vec(args),
            "include_str" => self.macro_include_str(args),
            _ => Err(RuntimeError::new(format!("Unknown macro: '{name}!'"))),
        }
    }

    // ── Print macros ─────────────────────────────────────────────────

    fn macro_print(
        &mut self,
        args: &[Expr],
        newline: bool,
        stderr: bool,
    ) -> Result<Value, RuntimeError> {
        let output = if args.is_empty() {
            String::new()
        } else if let Some(Expr::StringLiteral(fmt)) = args.first() {
            let evaluated = self.eval_slice(&args[1..])?;
            self.format_string(fmt, &evaluated)?
        } else {
            let val = self.eval(&args[0])?;
            format!("{val}")
        };

        if stderr {
            if newline {
                eprintln!("{output}");
            } else {
                eprint!("{output}");
            }
        } else if newline {
            println!("{output}");
        } else {
            print!("{output}");
        }
        Ok(Value::Unit)
    }

    // ── format! ──────────────────────────────────────────────────────

    fn macro_format(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        if let Some(Expr::StringLiteral(fmt)) = args.first() {
            let evaluated = self.eval_slice(&args[1..])?;
            let formatted = self.format_string(fmt, &evaluated)?;
            Ok(Value::String(formatted))
        } else if let Some(arg) = args.first() {
            let val = self.eval(arg)?;
            Ok(Value::String(format!("{val}")))
        } else {
            Ok(Value::String(String::new()))
        }
    }

    // ── dbg! ─────────────────────────────────────────────────────────

    fn macro_dbg(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        for arg in args {
            let val = self.eval(arg)?;
            eprintln!("[dbg] = {}", val.debug_fmt());
        }
        if args.len() == 1 {
            self.eval(&args[0])
        } else {
            Ok(Value::Unit)
        }
    }

    // ── Assertion macros ─────────────────────────────────────────────

    fn macro_assert(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        if let Some(arg) = args.first() {
            let val = self.eval(arg)?;
            if !val.is_truthy() {
                return Err(RuntimeError::new("Assertion failed"));
            }
        }
        Ok(Value::Unit)
    }

    fn macro_assert_eq(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        if args.len() >= 2 {
            let left = self.eval(&args[0])?;
            let right = self.eval(&args[1])?;
            let equal = self.values_equal(&left, &right);
            if !equal {
                let msg = self.optional_message(args, 2)?;
                return Err(RuntimeError::new(format!(
                    "Assertion failed: left = {}, right = {}{msg}",
                    left.debug_fmt(),
                    right.debug_fmt(),
                )));
            }
        }
        Ok(Value::Unit)
    }

    fn macro_assert_ne(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        if args.len() >= 2 {
            let left = self.eval(&args[0])?;
            let right = self.eval(&args[1])?;
            let equal = self.values_equal(&left, &right);
            if equal {
                return Err(RuntimeError::new(format!(
                    "Assertion failed (values should not be equal): left = {}, right = {}",
                    left.debug_fmt(),
                    right.debug_fmt(),
                )));
            }
        }
        Ok(Value::Unit)
    }

    /// Extracts an optional message argument at the given index.
    fn optional_message(&mut self, args: &[Expr], index: usize) -> Result<String, RuntimeError> {
        if args.len() > index {
            if let Ok(Value::String(m)) = self.eval(&args[index]) {
                return Ok(format!(": {m}"));
            }
        }
        Ok(String::new())
    }

    // ── panic! / todo! / unreachable! ────────────────────────────────

    fn macro_panic(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        if let Some(Expr::StringLiteral(msg)) = args.first() {
            let evaluated = self.eval_slice(&args[1..])?;
            let formatted = self.format_string(msg, &evaluated)?;
            Err(RuntimeError::new(format!("panic: {formatted}")))
        } else if let Some(arg) = args.first() {
            let val = self.eval(arg)?;
            Err(RuntimeError::new(format!("panic: {val}")))
        } else {
            Err(RuntimeError::new("explicit panic"))
        }
    }

    // ── vec! / include_str! ──────────────────────────────────────────

    fn macro_vec(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        let values = self.eval_slice(args)?;
        Ok(Value::Array(values))
    }

    fn macro_include_str(&mut self, args: &[Expr]) -> Result<Value, RuntimeError> {
        if let Some(Expr::StringLiteral(path)) = args.first() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| RuntimeError::new(format!("Cannot read file '{path}': {e}")))?;
            Ok(Value::String(content))
        } else {
            Err(RuntimeError::new("include_str! expects a string literal"))
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────

    /// Evaluates a slice of expressions into a Vec of values.
    pub(crate) fn eval_slice(&mut self, exprs: &[Expr]) -> Result<Vec<Value>, RuntimeError> {
        exprs.iter().map(|e| self.eval(e)).collect()
    }
}
