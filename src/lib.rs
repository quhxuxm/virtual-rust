//! # Virtual Rust
//!
//! A tree-walking interpreter that runs Rust source code directly
//! without compilation. Supports variables, functions, closures,
//! control flow, structs, arrays/Vec, match expressions, and more.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! // Run a Rust source file
//! virtual_rust::run_source(r#"
//!     fn main() {
//!         println!("Hello from Virtual Rust!");
//!     }
//! "#).unwrap();
//! ```

pub mod ast;
pub mod cargo_runner;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

/// Compiles source code through the lexer → parser → AST pipeline.
fn compile(source: &str) -> Result<Vec<ast::Expr>, String> {
    let tokens = Lexer::new(source).tokenize().map_err(|e| format!("{e}"))?;

    Parser::new(tokens)
        .parse_program()
        .map_err(|e| format!("{e}"))
}

/// Returns `true` if the program contains a `fn main()` definition.
fn has_main(program: &[ast::Expr]) -> bool {
    program
        .iter()
        .any(|expr| matches!(expr, ast::Expr::FnDef { name, .. } if name == "main"))
}

/// Evaluates all top-level statements. If a `fn main()` exists, calls it.
fn execute(
    interpreter: &mut Interpreter,
    program: &[ast::Expr],
) -> Result<interpreter::Value, String> {
    let mut result = interpreter::Value::Unit;
    for expr in program {
        result = interpreter.eval(expr).map_err(|e| format!("{e}"))?;
    }

    if has_main(program) {
        result = interpreter
            .eval(&ast::Expr::FnCall {
                name: "main".to_string(),
                args: vec![],
            })
            .map_err(|e| format!("{e}"))?;
    }

    Ok(result)
}

/// Runs Rust source code directly without compilation.
///
/// If the source contains a `fn main()`, it will be called automatically.
pub fn run_source(source: &str) -> Result<(), String> {
    let program = compile(source)?;
    let mut interpreter = Interpreter::new();
    execute(&mut interpreter, &program)?;
    Ok(())
}

/// Runs Rust source code and returns the final result as a display string.
///
/// Useful for REPL-style evaluation where the result should be shown.
pub fn eval_source(source: &str) -> Result<String, String> {
    let program = compile(source)?;
    let mut interpreter = Interpreter::new();
    let result = execute(&mut interpreter, &program)?;
    Ok(format!("{result}"))
}
