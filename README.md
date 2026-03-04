# Virtual Rust 🦀

A virtual machine that **interprets and runs Rust source code directly** — no compilation needed.

Virtual Rust is a Rust interpreter written in Rust. It reads `.rs` source files (or inline expressions) and executes them immediately via a **Lexer → Parser → AST → Tree-walking Interpreter** pipeline.

## Quick Start

```bash
# Build
cargo build --release

# Run a Rust source file directly
cargo run -- examples/hello.rs

# Run a cargo project with multiple binaries
cargo run -- examples/multi_bin -- --bin server

# Evaluate an expression
cargo run -- -e 'println!("Hello from Virtual Rust!");'

# Start interactive REPL
cargo run
```

## Features

### Language Support

| Feature | Status |
|---------|--------|
| Variables (`let`, `let mut`) | ✅ |
| Type annotations & inference | ✅ |
| Functions (`fn`) with params & return types | ✅ |
| Closures (`\|x\| x * 2`) | ✅ |
| `if` / `else if` / `else` (statement & expression) | ✅ |
| `while` loops | ✅ |
| `loop` with `break` value | ✅ |
| `for..in` with ranges | ✅ |
| `match` with patterns, ranges, wildcards | ✅ |
| Arithmetic / comparison / logical / bitwise ops | ✅ |
| Compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`) | ✅ |
| Type casting (`as`) | ✅ |
| Arrays, `Vec`, tuples | ✅ |
| Structs with field access | ✅ |
| String, char, integer, float, bool types | ✅ |
| Nested functions and recursion | ✅ |
| Higher-order functions (`map`, `filter`, `fold`) | ✅ |
| Iterator chains (`.iter().map().filter().collect()`) | ✅ |
| Comments (`//` and `/* */`) | ✅ |
| Attributes (`#[...]`) skip | ✅ |

### Built-in Macros

- `println!`, `print!`, `eprintln!`, `eprint!` — with format string support (`{}`, `{:?}`, `{:.2}`)
- `format!` — string formatting
- `vec!` — vector creation
- `assert!`, `assert_eq!`, `assert_ne!` — assertions
- `dbg!` — debug printing
- `panic!`, `todo!`, `unimplemented!`, `unreachable!`
- `include_str!` — file reading

### Methods

**String**: `len`, `is_empty`, `contains`, `starts_with`, `ends_with`, `trim`, `to_uppercase`, `to_lowercase`, `replace`, `split`, `chars`, `bytes`, `push_str`, `repeat`, `lines`, `parse`

**Array/Vec**: `len`, `is_empty`, `push`, `pop`, `first`, `last`, `contains`, `reverse`, `iter`, `map`, `filter`, `fold`, `for_each`, `enumerate`, `zip`, `sum`, `product`, `min`, `max`, `join`, `any`, `all`, `find`, `position`, `skip`, `take`, `count`, `flat_map`, `collect`

**Integer**: `abs`, `pow`, `min`, `max`, `clamp`, `to_string`

**Float**: `abs`, `sqrt`, `floor`, `ceil`, `round`, `sin`, `cos`, `tan`, `ln`, `log2`, `log10`, `powi`, `powf`, `is_nan`, `is_infinite`, `is_finite`

**Option**: `unwrap`, `unwrap_or`, `is_some`, `is_none`, `map`

**Char**: `is_alphabetic`, `is_numeric`, `is_alphanumeric`, `is_whitespace`, `is_uppercase`, `is_lowercase`

## Usage

```bash
# Run a file
virtual-rust program.rs

# Inline evaluation
virtual-rust -e 'let x = (1..=10).sum(); println!("Sum = {}", x);'

# Interactive REPL
virtual-rust --repl

# Help
virtual-rust --help
```

### REPL Commands

| Command | Description |
|---------|-------------|
| `:quit`, `:q` | Exit the REPL |
| `:help`, `:h` | Show help |
| `:clear`, `:c` | Clear environment |
| `:version`, `:v` | Show version |

### Running Cargo Projects

Virtual Rust can run entire Cargo project directories. Extra arguments after `--` are passed directly to `cargo run`, so you can use any cargo flags like `--bin`, `--release`, etc.

```bash
# Run a Cargo project directory
virtual-rust ./my-project

# Run a specific binary in a multi-bin project
virtual-rust ./examples/multi_bin -- --bin server

# Pass both cargo flags and program arguments
virtual-rust ./examples/multi_bin -- --bin server -- 9090

# Build in release mode
virtual-rust ./my-project -- --release
```

The `examples/multi_bin` project demonstrates this — it defines three binaries (`server`, `client`, `health`) and requires `--bin <name>` to select which one to run.

### Running Loose `.rs` Directories

Virtual Rust can also run a directory of plain `.rs` files that have **no `Cargo.toml`**. It automatically:

1. Finds the entry point (the file containing `fn main()`)
2. Generates a temporary Cargo project
3. Copies the entry file as `src/main.rs` and all other `.rs` files as modules
4. Compiles and runs the project

```bash
# Given a folder with main.rs, math.rs, greeting.rs (no Cargo.toml)
virtual-rust ./examples/loose_modules
```

The entry file can use `mod` to reference sibling files:

```rust
// main.rs
mod math;
mod greeting;

fn main() {
    println!("{}", greeting::hello("World"));
    println!("Sum = {}", math::sum(&[1, 2, 3]));
}
```

The `examples/loose_modules` directory demonstrates this pattern.

## Examples

```rust
// examples/fibonacci.rs
fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    let mut a = 0;
    let mut b = 1;
    let mut i = 2;
    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i += 1;
    }
    b
}

fn main() {
    for i in 0..10 {
        println!("fib({}) = {}", i, fibonacci(i));
    }
}
```

```bash
$ cargo run -- examples/fibonacci.rs
fib(0) = 0
fib(1) = 1
fib(2) = 1
fib(3) = 2
fib(4) = 3
fib(5) = 5
fib(6) = 8
fib(7) = 13
fib(8) = 21
fib(9) = 34
```

## Architecture

```
Source Code (.rs)
       │
       ▼
   ┌────────┐
   │ Lexer  │  Tokenizes source into tokens
   └───┬────┘
       │ Vec<Token>
       ▼
   ┌────────┐
   │ Parser │  Builds Abstract Syntax Tree
   └───┬────┘
       │ Vec<Expr> (AST)
       ▼
   ┌──────────────┐
   │ Interpreter  │  Tree-walking execution
   └──────────────┘
       │
       ▼
     Output
```

## License

MIT