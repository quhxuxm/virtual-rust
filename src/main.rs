use std::fs;
use std::io::{self, BufRead, Write};

use clap::{Parser, Subcommand};
use virtual_rust::{eval_source, run_source};

// ── CLI definition ───────────────────────────────────────────────

/// A virtual machine that interprets and runs Rust source code directly.
#[derive(Parser)]
#[command(name = "virtual-rust", version, about, long_about = LONG_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to a Rust source file (.rs) or Cargo project directory to execute
    #[arg(value_name = "FILE|DIR")]
    input: Option<String>,

    /// Extra arguments passed through to cargo (e.g. --bin <name>, --release)
    #[arg(last = true, value_name = "CARGO_ARGS")]
    passthrough_args: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Start an interactive REPL session
    Repl,
    /// Evaluate a Rust expression or statement
    Eval {
        /// The Rust code to evaluate
        code: String,
    },
}

const LONG_ABOUT: &str = include_str!("long_about.txt");

// ── REPL ─────────────────────────────────────────────────────────

fn run_repl() {
    let version = env!("CARGO_PKG_VERSION");
    println!("Virtual Rust v{version} — Interactive REPL");
    println!("Type Rust expressions or statements. Type :quit to exit.\n");

    let stdin = io::stdin();
    let mut interpreter = virtual_rust::interpreter::Interpreter::new();
    let mut buffer = String::new();
    let mut brace_depth: i32 = 0;

    loop {
        if brace_depth > 0 {
            print!("...   ");
        } else {
            print!("rust> ");
        }
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let trimmed = line.trim();

        // REPL commands
        if brace_depth == 0 {
            match trimmed {
                ":quit" | ":exit" | ":q" => {
                    println!("Goodbye!");
                    break;
                }
                ":help" | ":h" => {
                    println!("REPL Commands:");
                    println!("  :quit, :exit, :q   Exit the REPL");
                    println!("  :help, :h          Show this help");
                    println!("  :clear, :c         Clear the environment");
                    println!("  :version, :v       Show version");
                    println!();
                    continue;
                }
                ":clear" | ":c" => {
                    interpreter = virtual_rust::interpreter::Interpreter::new();
                    println!("Environment cleared.");
                    continue;
                }
                ":version" | ":v" => {
                    let v = env!("CARGO_PKG_VERSION");
                    println!("Virtual Rust v{v}");
                    continue;
                }
                "" => continue,
                _ => {}
            }
        }

        // Track braces for multi-line input
        for ch in trimmed.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }

        buffer.push_str(&line);

        if brace_depth > 0 {
            continue; // Wait for closing braces
        }

        // Try to evaluate the buffer
        let source = buffer.trim().to_string();
        buffer.clear();
        brace_depth = 0;

        if source.is_empty() {
            continue;
        }

        // Tokenize
        let mut lexer = virtual_rust::lexer::Lexer::new(&source);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = virtual_rust::parser::Parser::new(tokens);
                match parser.parse_program() {
                    Ok(program) => {
                        let mut last_result = virtual_rust::interpreter::Value::Unit;
                        let mut had_error = false;
                        for expr in &program {
                            match interpreter.eval(expr) {
                                Ok(val) => {
                                    last_result = val;
                                }
                                Err(e) => {
                                    eprintln!("\x1b[31merror\x1b[0m: {}", e);
                                    had_error = true;
                                    break;
                                }
                            }
                        }
                        if !had_error {
                            match &last_result {
                                virtual_rust::interpreter::Value::Unit => {}
                                val => {
                                    println!("\x1b[32m=> {}\x1b[0m", val);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\x1b[31merror\x1b[0m: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("\x1b[31merror\x1b[0m: {}", e);
            }
        }
    }
}

// ── Entry point ──────────────────────────────────────────────────

fn run_file(file_path: &str, passthrough_args: &[String]) {
    let path = std::path::Path::new(file_path);

    // Check if argument is a Cargo project directory
    if virtual_rust::cargo_runner::is_cargo_project(path) {
        if let Err(e) = virtual_rust::cargo_runner::run_cargo_project(path, passthrough_args) {
            eprintln!("\x1b[31merror\x1b[0m: {e}");
            std::process::exit(1);
        }
        return;
    }

    // Check if argument is a directory of loose .rs files (no Cargo.toml)
    if virtual_rust::cargo_runner::is_rust_source_dir(path) {
        if let Err(e) = virtual_rust::cargo_runner::run_rust_dir(path, passthrough_args) {
            eprintln!("\x1b[31merror\x1b[0m: {e}");
            std::process::exit(1);
        }
        return;
    }

    match fs::read_to_string(file_path) {
        Ok(source) => {
            if virtual_rust::cargo_runner::has_dependencies(&source) {
                // Dependencies detected — compile & run with cargo
                if let Err(e) = virtual_rust::cargo_runner::run_with_cargo(&source, Some(path), passthrough_args) {
                    eprintln!("\x1b[31merror\x1b[0m: {e}");
                    std::process::exit(1);
                }
            } else {
                // No dependencies — use the interpreter
                if let Err(e) = run_source(&source) {
                    eprintln!("\x1b[31merror\x1b[0m: {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{file_path}': {e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Repl) => run_repl(),
        Some(Command::Eval { code }) => match eval_source(&code) {
            Ok(result) => {
                if result != "()" {
                    println!("{result}");
                }
            }
            Err(e) => {
                eprintln!("\x1b[31merror\x1b[0m: {e}");
                std::process::exit(1);
            }
        },
        None => match cli.input {
            Some(input) => run_file(&input, &cli.passthrough_args),
            None => run_repl(),
        },
    }
}
