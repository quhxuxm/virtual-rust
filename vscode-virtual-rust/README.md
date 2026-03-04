# Virtual Rust — VSCode Extension

IDE support for the **Virtual Rust** single-file format, where Cargo dependencies
are declared inline via `//!` doc comments.

## The Problem

When you write a Rust file with embedded `//!` dependency declarations like this:

```rust
//! [dependencies]
//! rand = "0.8"
//! serde = { version = "1.0", features = ["derive"] }

use rand::Rng;
use serde::Serialize;

fn main() {
    let n: i32 = rand::thread_rng().gen_range(1..=100);
    println!("{n}");
}
```

**rust-analyzer reports false errors** because it doesn't know about the `rand`
and `serde` dependencies — they aren't in any `Cargo.toml` it can see.

## The Solution

This extension creates **shadow Cargo projects** for each virtual-rust file:

```text
.virtual-rust/
└── vr-with_deps-a1b2c3d4/
    ├── Cargo.toml          ← generated from //! comments
    └── src/
        └── main.rs         ← symlink to your original .rs file
```

The shadow project is automatically added to `rust-analyzer.linkedProjects`,
giving you **full IDE support**: completions, diagnostics, go-to-definition,
and type information for all declared dependencies.

## Features

### Shadow Project Generation

- **Auto-sync**: When you open or save a virtual-rust file, a shadow Cargo
  project is automatically generated (configurable).
- **Manual sync**: Use the command **Virtual Rust: Sync Shadow Project**.
- **Clean up**: Use **Virtual Rust: Clean All Shadow Projects** to remove
  generated projects and reset `linkedProjects`.

### Run Support

- **CodeLens**: A `▶ Run with Virtual Rust` button appears above `fn main()`
  in virtual-rust files.
- **Context menu**: Right-click → **Run File** in any `.rs` file.
- **Editor title bar**: Play button in the editor title for quick runs.
- **Terminal**: Runs via `cargo run` in the shadow project (or the
  `virtual-rust` binary if configured).

### Visual Enhancements

- **Manifest decorations**: The `//!` dependency block is highlighted with:
  - Subtle background tint for the entire block
  - **Bold gold** for section headers (`[dependencies]`)
  - **Blue** for dependency names (`rand`, `serde`)
  - **Green** for version strings (`"0.8"`, `"1.0"`)
- **Status bar**: Shows `$(beaker) Virtual Rust` when editing a virtual-rust file.

## Configuration

| Setting | Default | Description |
|---|---|---|
| `virtual-rust.autoSync` | `true` | Auto-generate shadow projects on open/save |
| `virtual-rust.binaryPath` | `"virtual-rust"` | Path to the virtual-rust binary |
| `virtual-rust.shadowProjectDir` | `".virtual-rust"` | Shadow project directory name |
| `virtual-rust.runWithCargo` | `true` | Use `cargo run` instead of the binary |

## Commands

| Command | Description |
|---|---|
| **Virtual Rust: Run File** | Compile and run the current file |
| **Virtual Rust: Sync Shadow Project** | Force-regenerate the shadow project |
| **Virtual Rust: Clean All Shadow Projects** | Remove all generated projects |

## Requirements

- **VSCode** ≥ 1.85
- **rust-analyzer** extension (for IDE integration)
- **Rust toolchain** (for compiling shadow projects)
- **virtual-rust** binary (optional, for the Run command without cargo)

## How It Works

1. The extension monitors `.rs` files for `//!` comment blocks containing
   TOML section headers like `[dependencies]`.

2. When detected, it creates a shadow Cargo project under `.virtual-rust/`
   with a `Cargo.toml` derived from the `//!` comments and a symlink
   from `src/main.rs` to your original file.

3. The shadow project's `Cargo.toml` path is added to the
   `rust-analyzer.linkedProjects` workspace setting.

4. rust-analyzer picks up the new project, runs `cargo metadata` to discover
   dependencies, and provides full IDE support for the file.

5. The `.virtual-rust/` directory is automatically added to `.gitignore`.

## Virtual Rust File Format

```rust
//! [dependencies]
//! rand = "0.8"
//! serde = { version = "1.0", features = ["derive"] }
//! tokio = { version = "1", features = ["full"] }
//!
//! [package]
//! edition = "2021"

use rand::Rng;
use serde::Serialize;

#[tokio::main]
async fn main() {
    // Your code here — full IDE support!
}
```

The `//!` comments must appear at the very top of the file and must contain
at least one TOML section header (e.g. `[dependencies]`).

## Development

```bash
cd vscode-virtual-rust
npm install
npm run compile
```

To test: press **F5** in VSCode to launch the Extension Development Host.

To package:

```bash
npm run package
```

This produces a `.vsix` file that can be installed via
**Extensions → ⋯ → Install from VSIX…**
