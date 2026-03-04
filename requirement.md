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

## 5. VSCode Extension

When writing virtual-rust format files (`.rs` files with `//!` dependency declarations), rust-analyzer reports false compile errors because it cannot resolve the declared dependencies. A VSCode extension solves this by generating **shadow Cargo projects**.

### 5.1 Shadow Cargo Project Generation

- On file open or save, the extension detects `//!` manifest comments and creates a shadow Cargo project under `.virtual-rust/`:

```text
.virtual-rust/
└── vr-<name>-<hash>/
    ├── Cargo.toml          ← generated from //! comments
    └── src/
        └── main.rs         ← symlink to the original .rs file
```

- The shadow project's `Cargo.toml` path is automatically added to the `rust-analyzer.linkedProjects` workspace setting
- rust-analyzer discovers dependencies via the shadow project, providing full IDE support (completions, diagnostics, go-to-definition)
- Uses deterministic naming (`vr-<basename>-<md5hash>`) to avoid collisions
- Writes files only when content changes to minimize disk I/O
- Falls back to file copy when symlinks are unavailable (e.g., Windows without developer mode)
- The `.virtual-rust/` directory is automatically appended to `.gitignore`

### 5.2 Run Support

- **CodeLens**: A `▶ Run with Virtual Rust` button appears above `fn main()` (including `async fn main()` and `pub fn main()`)
- **Editor title bar**: Play button for quick runs
- **Context menu**: Right-click → **Virtual Rust: Run File**
- **Terminal execution**: Runs via `cargo run` in the shadow project directory, or via the `virtual-rust` binary if configured

### 5.3 Visual Enhancements

- **Manifest block decorations**:
  - Subtle background tint on the entire `//!` manifest region
  - Bold gold color on TOML section headers (`[dependencies]`, `[package]`)
  - Blue color on dependency key names (`rand`, `serde`, `tokio`)
  - Green color on version strings (`"0.8"`, `"1.0"`)
- **Status bar indicator**: Shows `$(beaker) Virtual Rust` when editing a virtual-rust file; clicking it runs the file

### 5.4 Commands

| Command | Description |
| --- | --- |
| `Virtual Rust: Run File` | Compile and run the current virtual-rust file |
| `Virtual Rust: Sync Shadow Project` | Force-regenerate the shadow Cargo project for the current file |
| `Virtual Rust: Clean All Shadow Projects` | Remove all generated shadow projects and reset `linkedProjects` |

### 5.5 Configuration

| Setting | Default | Description |
| --- | --- | --- |
| `virtual-rust.autoSync` | `true` | Automatically generate shadow projects on file open/save |
| `virtual-rust.binaryPath` | `"virtual-rust"` | Path to the virtual-rust binary |
| `virtual-rust.shadowProjectDir` | `".virtual-rust"` | Shadow project directory name (relative to workspace root) |
| `virtual-rust.runWithCargo` | `true` | Use `cargo run` in the shadow project instead of the binary |

### 5.6 Extension Architecture

```text
vscode-virtual-rust/
├── src/
│   ├── extension.ts      — Entry point: registers commands, events, CodeLens
│   ├── detector.ts       — Parses //! embedded manifest from source text
│   ├── shadow.ts         — Shadow Cargo project lifecycle management
│   ├── runner.ts         — Terminal-based file execution
│   ├── codeLens.ts       — "▶ Run with Virtual Rust" above fn main()
│   └── decoration.ts     — Manifest block syntax decorations
├── package.json          — Extension manifest with commands, settings, menus
└── tsconfig.json
```

### 5.7 Requirements

- VSCode ≥ 1.85
- rust-analyzer extension (for IDE integration)
- Rust toolchain (for compiling shadow projects)
- Activates on `onLanguage:rust`

## 6. Code Quality

- Zero `cargo clippy` warnings
- Modular architecture with focused, well-documented modules
- Section-organized source files with doc comments
- Unit tests for the Cargo runner module
