# Virtual Rust — Requirements

## 1. Core Interpreter

Virtual Rust is a virtual machine that interprets and runs Rust source code directly without compilation.

### Supported Language Features

- **Variables**: `let`, `let mut` with type inference and explicit type annotations
- **Functions**: `fn` with parameters, return types, and recursion
- **Control flow**: `if`/`else`, `while`, `loop`, `for..in`, `break`, `continue`, `return`
- **Operators**: arithmetic, comparison, logical, bitwise, compound assignment (`+=`, `-=`, etc.)
- **Data types**: integers (i8–i128, u8–u128), floats (f32, f64), bool, char, String, tuples, arrays, Vec, Option
- **Structs**: definition, instantiation, field access
- **Match expressions**: literal patterns, ranges, wildcards, or-patterns
- **Closures**: `|params| body`, with capture semantics
- **Higher-order functions**: `map`, `filter`, `fold`, `for_each`, `any`, `all`, `find`, `position`, `flat_map`, `enumerate`, `zip`
- **String methods**: `len`, `contains`, `split`, `trim`, `replace`, `to_uppercase`, `to_lowercase`, `starts_with`, `ends_with`, `chars`, etc.
- **Math operations**: `abs`, `sqrt`, `pow`, `sin`, `cos`, `min`, `max`, etc.
- **Macros**: `println!`, `print!`, `format!`, `assert!`, `assert_eq!`, `assert_ne!`, `vec!`, `dbg!`, `panic!`, `include_str!`
- **Type casting**: `as` expressions
- **References**: `&`, `&mut`, `*` (dereference)
- **Range expressions**: `..`, `..=`

## 2. Execution Modes

### 2.1 File Execution

```bash
virtual-rust hello.rs
```

Runs a single `.rs` file through the interpreter. If the file contains a `fn main()`, it is called automatically.

### 2.2 Expression Evaluation

```bash
virtual-rust -e 'println!("Hello, World!")'
```

Evaluates a Rust expression or statement directly from the command line.

### 2.3 Interactive REPL

```bash
virtual-rust --repl
```

Starts an interactive Read-Eval-Print Loop. Supports multi-line input (brace tracking), REPL commands (`:quit`, `:help`, `:clear`, `:version`), and persistent environment across evaluations.

### 2.4 Single-File with Dependencies

```bash
virtual-rust script.rs
```

When a `.rs` file contains embedded Cargo manifest sections in `//!` doc comments, virtual-rust creates a temporary Cargo project and compiles/runs it with `cargo run`.

**Dependency syntax:**

```rust
//! [dependencies]
//! rand = "0.8"
//! serde = { version = "1.0", features = ["derive"] }
//! tokio = { version = "1", features = ["full"] }

use rand::Rng;

fn main() {
    let n: u32 = rand::thread_rng().gen_range(1..=100);
    println!("Random: {n}");
}
```

- Supports any valid `Cargo.toml` syntax in `//!` comments (dependencies, features, custom `[package]`, etc.)
- Uses a deterministic cache directory (`$TMPDIR/virtual-rust-cache/`) based on the file path hash for incremental compilation
- Supports async runtimes (e.g., tokio with `#[tokio::main]`)

### 2.5 Cargo Project Execution

```bash
virtual-rust ./my-project
virtual-rust ./my-project -- --flag value
```

When pointed at a directory containing a `Cargo.toml`, virtual-rust compiles and runs the project with `cargo run`. This supports:

- Full Cargo project structure (multiple source files, modules)
- Any dependencies including async runtimes (tokio, async-std, etc.)
- Build scripts, proc macros, workspaces
- Passthrough arguments after `--` are forwarded to the compiled program

## 3. Execution Strategy

| Input | Detection | Behavior |
| --- | --- | --- |
| `file.rs` without `//!` deps | No `[dependencies]` in `//!` comments | Interpreted directly by the tree-walking interpreter |
| `file.rs` with `//!` deps | `[dependencies]` found in `//!` comments | Temp Cargo project created → `cargo run` |
| Directory with `Cargo.toml` | `is_dir()` && `Cargo.toml` exists | `cargo run` in that directory |

## 4. Architecture

```text
Source code → Lexer → Tokens → Parser → AST → Interpreter
                                                    ↓
                                               Tree-walking evaluation
```

- **Lexer** (`src/lexer.rs`): Tokenizes Rust source code (strings, chars, numbers with hex/binary/octal, operators, keywords)
- **Tokens** (`src/token.rs`): Token type definitions
- **Parser** (`src/parser.rs`): Recursive-descent parser producing AST nodes
- **AST** (`src/ast.rs`): Expression/statement node types
- **Interpreter** (`src/interpreter/`): Tree-walking interpreter split into focused modules:
  - `mod.rs` — Core evaluation loop
  - `value.rs` — Runtime value types
  - `environment.rs` — Scoped variable storage
  - `error.rs` — Runtime error type
  - `builtins.rs` — Built-in functions and macros
  - `methods.rs` — Method dispatch per type
  - `format.rs` — Format string processing
  - `pattern.rs` — Pattern matching logic
  - `cast.rs` — Type casting
- **Cargo Runner** (`src/cargo_runner.rs`): Dependency detection, temp project generation, and `cargo run` delegation

## 5. Code Quality

- Zero `cargo clippy` warnings
- Modular architecture with focused, well-documented modules
- Section-organized source files with doc comments
- Unit tests for the Cargo runner module
