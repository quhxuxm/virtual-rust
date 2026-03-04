use std::env;
use std::fs;
use std::io::{self, Write, BufRead};

use virtual_rust::{run_source, eval_source};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!("Virtual Rust v{}", VERSION);
    println!("A virtual machine that interprets Rust source code directly.\n");
    println!("USAGE:");
    println!("    virtual-rust [OPTIONS] [FILE|DIR] [-- ARGS...]");
    println!();
    println!("ARGS:");
    println!("    <FILE>    Path to a Rust source file (.rs) to execute");
    println!("    <DIR>     Path to a Cargo project directory to compile & run");
    println!("    [ARGS]    Arguments passed to the compiled program (after --)");
    println!();
    println!("OPTIONS:");
    println!("    -e, --eval <CODE>    Evaluate a Rust expression");
    println!("    -h, --help           Print help information");
    println!("    -V, --version        Print version information");
    println!("    --repl               Start interactive REPL mode\n");
    println!("EXAMPLES:");
    println!("    virtual-rust hello.rs");
    println!("    virtual-rust -e 'println!(\"Hello, World!\")'");
    println!("    virtual-rust --repl");    println!("    virtual-rust ./my-project");
    println!("    virtual-rust ./my-project -- --flag value");    println!("\nSUPPORTED FEATURES:");
    println!("    - Variables (let, let mut) with type inference");
    println!("    - Functions (fn) with parameters and return types");
    println!("    - Control flow: if/else, while, loop, for..in");
    println!("    - Arithmetic, comparison, logical, bitwise operators");
    println!("    - String, array/Vec, tuple operations");
    println!("    - Structs with field access");
    println!("    - Match expressions with patterns");
    println!("    - Closures and higher-order functions (map, filter, fold)");
    println!("    - Type casting (as)");
    println!("    - Macros: println!, print!, format!, assert!, assert_eq!, vec!, dbg!");
    println!("    - Iterator methods: map, filter, fold, enumerate, zip, etc.");
    println!("    - String methods: len, contains, split, trim, replace, etc.");
    println!("    - Math operations: abs, sqrt, pow, sin, cos, etc.");
    println!();
    println!("DEPENDENCIES:");
    println!("    Add //! comments at the top of your .rs file to declare dependencies:");
    println!("        //! [dependencies]");
    println!("        //! serde = \"1.0\"");
    println!("        //! rand = \"0.8\"");
    println!("    Files with dependencies are compiled with cargo automatically.");
    println!();
    println!("CARGO PROJECTS:");
    println!("    Point virtual-rust at a directory containing Cargo.toml:");
    println!("        virtual-rust ./my-project");
    println!("    The project will be compiled and run with cargo.");
}

fn run_repl() {
    println!("Virtual Rust v{} — Interactive REPL", VERSION);
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
                    println!("Virtual Rust v{}", VERSION);
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

/// Collects arguments after `--` to pass through to the target program.
fn collect_passthrough_args(args: &[String]) -> Vec<String> {
    if let Some(pos) = args.iter().position(|a| a == "--") {
        args[pos + 1..].to_vec()
    } else {
        Vec::new()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // No arguments - start REPL
        run_repl();
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_help();
        }
        "-V" | "--version" => {
            println!("virtual-rust {}", VERSION);
        }
        "--repl" => {
            run_repl();
        }
        "-e" | "--eval" => {
            if args.len() < 3 {
                eprintln!("Error: --eval requires a code argument");
                std::process::exit(1);
            }
            let code = &args[2];
            match eval_source(code) {
                Ok(result) => {
                    if result != "()" {
                        println!("{}", result);
                    }
                }
                Err(e) => {
                    eprintln!("\x1b[31merror\x1b[0m: {}", e);
                    std::process::exit(1);
                }
            }
        }
        file_path => {
            if file_path.starts_with('-') {
                eprintln!("Unknown option: {}", file_path);
                eprintln!("Run 'virtual-rust --help' for usage.");
                std::process::exit(1);
            }

            let path = std::path::Path::new(file_path);

            // Check if argument is a Cargo project directory
            if virtual_rust::cargo_runner::is_cargo_project(path) {
                let extra_args = collect_passthrough_args(&args);
                if let Err(e) =
                    virtual_rust::cargo_runner::run_cargo_project(path, &extra_args)
                {
                    eprintln!("\x1b[31merror\x1b[0m: {}", e);
                    std::process::exit(1);
                }
                return;
            }

            match fs::read_to_string(file_path) {
                Ok(source) => {
                    if virtual_rust::cargo_runner::has_dependencies(&source) {
                        // Dependencies detected — compile & run with cargo
                        let path = std::path::Path::new(file_path);
                        if let Err(e) =
                            virtual_rust::cargo_runner::run_with_cargo(&source, Some(path))
                        {
                            eprintln!("\x1b[31merror\x1b[0m: {}", e);
                            std::process::exit(1);
                        }
                    } else {
                        // No dependencies — use the interpreter
                        if let Err(e) = run_source(&source) {
                            eprintln!("\x1b[31merror\x1b[0m: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file_path, e);
                    std::process::exit(1);
                }
            }
        }
    }
}
