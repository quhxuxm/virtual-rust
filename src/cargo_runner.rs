//! Compiles and runs Rust source files that declare external dependencies,
//! runs existing Cargo projects, or assembles loose `.rs` directories into
//! temporary Cargo projects.
//!
//! # Single-file dependencies
//!
//! When a `.rs` file contains embedded Cargo manifest sections in `//!`
//! doc comments (e.g. `[dependencies]`), this module creates a temporary
//! Cargo project, writes a proper `Cargo.toml`, and delegates to `cargo run`.
//!
//! ```rust,ignore
//! //! [dependencies]
//! //! rand = "0.8"
//! //! serde = { version = "1.0", features = ["derive"] }
//!
//! use rand::Rng;
//! use serde::Serialize;
//!
//! fn main() {
//!     let n: i32 = rand::thread_rng().gen_range(1..=100);
//!     println!("Random number: {n}");
//! }
//! ```
//!
//! # Cargo projects
//!
//! When pointed at a directory containing a `Cargo.toml`, virtual-rust will
//! compile and run the project with `cargo run` directly:
//!
//! ```bash
//! virtual-rust ./my-project
//! ```
//!
//! # Loose `.rs` directories
//!
//! When pointed at a directory containing `.rs` files but **no** `Cargo.toml`,
//! virtual-rust will automatically assemble them into a temporary Cargo project.
//! The file containing `fn main()` becomes `src/main.rs`, and all other files
//! are copied as sibling modules under `src/`.
//!
//! ```bash
//! virtual-rust ./my-scripts/    # directory with main.rs, utils.rs, etc.
//! ```

use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Manifest parsing ─────────────────────────────────────────────────

/// Embedded Cargo manifest extracted from `//!` doc comments.
pub struct EmbeddedManifest {
    /// Raw TOML content (may contain `[dependencies]`, `[package]`, etc.).
    pub toml_content: String,
}

/// Scans `//!` doc comments at the top of a Rust file for Cargo manifest sections.
///
/// Returns `Some(manifest)` if a TOML section header (e.g. `[dependencies]`) is found
/// within the leading `//!` block, `None` otherwise.
pub fn parse_embedded_manifest(source: &str) -> Option<EmbeddedManifest> {
    let mut toml_lines = Vec::new();
    let mut found_section = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("//!") {
            // Strip a single leading space after `//!` if present
            let content = rest.strip_prefix(' ').unwrap_or(rest);
            if content.starts_with('[') {
                found_section = true;
            }
            toml_lines.push(content.to_string());
        } else if trimmed.is_empty() {
            // Allow blank lines within the leading doc-comment block
            if found_section {
                toml_lines.push(String::new());
            }
        } else {
            // First non-doc-comment, non-empty line ends the manifest
            break;
        }
    }

    if !found_section {
        return None;
    }

    Some(EmbeddedManifest {
        toml_content: toml_lines.join("\n"),
    })
}

/// Returns `true` if the source contains an embedded dependency manifest.
pub fn has_dependencies(source: &str) -> bool {
    parse_embedded_manifest(source).is_some()
}

// ── Source cleaning ──────────────────────────────────────────────────

/// Strips leading `//!` manifest lines from the source, returning clean Rust code.
///
/// Only removes consecutive `//!` lines (and interleaved blank lines) at the
/// very start of the file. All other content is preserved.
fn strip_manifest_comments(source: &str) -> String {
    let mut result_lines: Vec<&str> = Vec::new();
    let mut in_header = true;

    for line in source.lines() {
        if in_header {
            let trimmed = line.trim();
            if trimmed.starts_with("//!") || trimmed.is_empty() {
                continue; // skip manifest & surrounding blank lines
            }
            in_header = false;
        }
        result_lines.push(line);
    }

    result_lines.join("\n")
}

// ── Cargo project generation ─────────────────────────────────────────

/// Creates a deterministic cache directory for the given source file.
///
/// Uses a hash of the canonical path so repeated runs reuse the same
/// directory and benefit from incremental compilation.
fn project_cache_dir(source_path: Option<&Path>) -> Result<PathBuf, String> {
    let base = std::env::temp_dir().join("virtual-rust-cache");

    let project_name = match source_path {
        Some(path) => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .hash(&mut hasher);
            format!("project_{:x}", hasher.finish())
        }
        None => "project_anonymous".to_string(),
    };

    let dir = base.join(project_name);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create cache directory: {e}"))?;
    Ok(dir)
}

/// Generates a `Cargo.toml` string from the embedded manifest.
fn generate_cargo_toml(manifest: &EmbeddedManifest, source_path: Option<&Path>) -> String {
    let name = source_path
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("virtual-rust-script")
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-");

    let has_package = manifest.toml_content.contains("[package]");

    let mut toml = String::new();

    if !has_package {
        toml.push_str(&format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n"
        ));
    }

    toml.push_str(&manifest.toml_content);
    toml.push('\n');
    toml
}

// ── Execution ────────────────────────────────────────────────────────

/// Compiles and runs a Rust source file that has embedded dependencies.
///
/// 1. Parses the `//!` manifest from the source
/// 2. Creates a cached Cargo project directory
/// 3. Writes `Cargo.toml` and `src/main.rs`
/// 4. Invokes `cargo run --quiet`
///
/// Stdin/stdout/stderr are inherited, so the program interacts with the
/// terminal normally.
pub fn run_with_cargo(source: &str, source_path: Option<&Path>, extra_args: &[String]) -> Result<(), String> {
    let manifest =
        parse_embedded_manifest(source).ok_or("No embedded dependency manifest found")?;

    // Resolve cache directory
    let project_dir = project_cache_dir(source_path)?;
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src directory: {e}"))?;

    // Write Cargo.toml
    let cargo_toml = generate_cargo_toml(&manifest, source_path);
    fs::write(project_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {e}"))?;

    // Write cleaned source as src/main.rs
    let clean_source = strip_manifest_comments(source);
    fs::write(src_dir.join("main.rs"), &clean_source)
        .map_err(|e| format!("Failed to write main.rs: {e}"))?;

    // Print status
    eprintln!(
        "\x1b[1;32m   Compiling\x1b[0m {} with cargo (dependencies detected)",
        source_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("script")
    );

    // Run cargo build first for clearer error separation
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet"]).current_dir(&project_dir);

    // Pass extra arguments directly to cargo (e.g. --release, -- <program args>)
    if !extra_args.is_empty() {
        cmd.args(extra_args);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to invoke cargo: {e}. Is cargo installed?"))?;

    if !status.success() {
        return Err(format!(
            "Compilation failed (exit code: {})",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

// ── Cargo project support ────────────────────────────────────────────

/// Returns `true` if the given path is a Cargo project directory
/// (i.e. it is a directory containing a `Cargo.toml`).
pub fn is_cargo_project(path: &Path) -> bool {
    path.is_dir() && path.join("Cargo.toml").exists()
}

/// Returns `true` if the given path is a directory containing `.rs` files
/// but no `Cargo.toml` (i.e. a loose collection of Rust source files).
pub fn is_rust_source_dir(path: &Path) -> bool {
    if !path.is_dir() || path.join("Cargo.toml").exists() {
        return false;
    }
    // Check for at least one .rs file in the directory
    match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .any(|e| {
                e.path().extension().and_then(|ext| ext.to_str()) == Some("rs")
            }),
        Err(_) => false,
    }
}

/// Scans a directory of `.rs` files, finds the entry point (file containing
/// `fn main()`), and returns `(entry_file, all_rs_files)`.
fn find_entry_file(dir: &Path) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory '{}': {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .collect();

    if entries.is_empty() {
        return Err(format!("No .rs files found in '{}'", dir.display()));
    }

    let mut main_files = Vec::new();
    for path in &entries {
        if let Ok(content) = fs::read_to_string(path) {
            // Look for `fn main()` — a simple heuristic
            if content.contains("fn main()") {
                main_files.push(path.clone());
            }
        }
    }

    match main_files.len() {
        0 => Err(format!(
            "No entry point found: none of the .rs files in '{}' contain `fn main()`",
            dir.display()
        )),
        1 => Ok((main_files.into_iter().next().unwrap(), entries)),
        _ => {
            let names: Vec<String> = main_files
                .iter()
                .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
                .collect();
            Err(format!(
                "Multiple entry points found in '{}': {}. \
                 Please pass the specific .rs file to run instead.",
                dir.display(),
                names.join(", ")
            ))
        }
    }
}

/// Collects any embedded manifest (`//! [dependencies]`) from any `.rs` file
/// in the directory. Merges all `[dependencies]` into a single manifest.
fn collect_embedded_manifests(rs_files: &[PathBuf]) -> Option<EmbeddedManifest> {
    let mut all_deps_lines = Vec::new();
    let mut found_any = false;

    for path in rs_files {
        if let Ok(content) = fs::read_to_string(path) {
            if let Some(manifest) = parse_embedded_manifest(&content) {
                found_any = true;
                // Collect lines that are within [dependencies] sections
                let mut in_deps = false;
                for line in manifest.toml_content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('[') {
                        in_deps = trimmed == "[dependencies]";
                        if !in_deps {
                            // Preserve other sections as-is
                            all_deps_lines.push(line.to_string());
                        }
                        continue;
                    }
                    if in_deps && !trimmed.is_empty() {
                        // Avoid duplicate dependency lines
                        if !all_deps_lines.contains(&line.to_string()) {
                            all_deps_lines.push(line.to_string());
                        }
                    } else if !in_deps {
                        all_deps_lines.push(line.to_string());
                    }
                }
            }
        }
    }

    if !found_any {
        return None;
    }

    let mut toml_content = String::from("[dependencies]\n");
    toml_content.push_str(&all_deps_lines.join("\n"));

    Some(EmbeddedManifest { toml_content })
}

/// Runs a directory of loose `.rs` files by generating a temporary Cargo project.
///
/// 1. Finds the entry file (containing `fn main()`)
/// 2. Creates a cached Cargo project directory
/// 3. Copies the entry file as `src/main.rs` and all other files as `src/<name>.rs`
/// 4. Collects any embedded dependency manifests from all files
/// 5. Invokes `cargo run --quiet`
pub fn run_rust_dir(dir: &Path, extra_args: &[String]) -> Result<(), String> {
    let (entry_file, all_files) = find_entry_file(dir)?;

    let dir_name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("rust-project");

    // Use the directory path for deterministic caching
    let project_dir = project_cache_dir(Some(dir))?;
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src directory: {e}"))?;

    // Collect any embedded manifests for Cargo.toml generation
    let manifest = collect_embedded_manifests(&all_files);
    let cargo_toml = if let Some(ref m) = manifest {
        generate_cargo_toml(m, Some(dir))
    } else {
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            dir_name
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        )
    };
    fs::write(project_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {e}"))?;

    // Copy entry file as src/main.rs (strip manifest comments if any)
    let entry_source = fs::read_to_string(&entry_file)
        .map_err(|e| format!("Failed to read '{}': {e}", entry_file.display()))?;
    let clean_entry = strip_manifest_comments(&entry_source);
    fs::write(src_dir.join("main.rs"), &clean_entry)
        .map_err(|e| format!("Failed to write main.rs: {e}"))?;

    // Copy all other .rs files as modules in src/
    for file in &all_files {
        if file == &entry_file {
            continue;
        }
        let file_name = file
            .file_name()
            .ok_or_else(|| format!("Invalid file path: {}", file.display()))?;
        let source = fs::read_to_string(file)
            .map_err(|e| format!("Failed to read '{}': {e}", file.display()))?;
        let clean = strip_manifest_comments(&source);
        fs::write(src_dir.join(file_name), &clean)
            .map_err(|e| format!("Failed to write '{}': {e}", file_name.to_string_lossy()))?;
    }

    eprintln!(
        "\x1b[1;32m   Compiling\x1b[0m {} (rust source directory, {} files)",
        dir_name,
        all_files.len()
    );

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet"]).current_dir(&project_dir);

    // Pass extra arguments directly to cargo
    if !extra_args.is_empty() {
        cmd.args(extra_args);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to invoke cargo: {e}. Is cargo installed?"))?;

    if !status.success() {
        return Err(format!(
            "Compilation failed (exit code: {})",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

/// Runs an existing Cargo project directory with `cargo run`.
///
/// If the project has no dependencies (empty `[dependencies]` or none at all),
/// the interpreter *could* be used, but we always delegate to cargo for
/// full compatibility with the project's build configuration, build scripts,
/// proc macros, multiple source files, modules, etc.
pub fn run_cargo_project(project_dir: &Path, extra_args: &[String]) -> Result<(), String> {
    let cargo_toml = project_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(format!(
            "No Cargo.toml found in '{}'",
            project_dir.display()
        ));
    }

    // Read project name from Cargo.toml for display
    let display_name = read_project_name(&cargo_toml)
        .unwrap_or_else(|| project_dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .to_string());

    eprintln!(
        "\x1b[1;32m   Compiling\x1b[0m {} (cargo project)",
        display_name
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--quiet").current_dir(project_dir);

    // Pass extra arguments directly to cargo (e.g. --bin <name>, --release, -- <program args>)
    if !extra_args.is_empty() {
        cmd.args(extra_args);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to invoke cargo: {e}. Is cargo installed?"))?;

    if !status.success() {
        return Err(format!(
            "cargo run failed (exit code: {})",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

/// Reads the `name` field from a Cargo.toml (best-effort, no TOML parser).
fn read_project_name(cargo_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(cargo_toml).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim().trim_matches('"');
                return Some(rest.to_string());
            }
        }
    }
    None
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest_with_dependencies() {
        let source = r#"//! [dependencies]
//! rand = "0.8"
//! serde = { version = "1.0", features = ["derive"] }

use rand::Rng;

fn main() {
    println!("hello");
}
"#;
        let manifest = parse_embedded_manifest(source).unwrap();
        assert!(manifest.toml_content.contains("[dependencies]"));
        assert!(manifest.toml_content.contains("rand = \"0.8\""));
        assert!(manifest.toml_content.contains("serde"));
    }

    #[test]
    fn parse_manifest_none_without_deps() {
        let source = r#"fn main() {
    println!("hello");
}
"#;
        assert!(parse_embedded_manifest(source).is_none());
    }

    #[test]
    fn strip_manifest_preserves_code() {
        let source = r#"//! [dependencies]
//! rand = "0.8"

use rand::Rng;

fn main() {}
"#;
        let cleaned = strip_manifest_comments(source);
        assert!(!cleaned.contains("//!"));
        assert!(cleaned.contains("use rand::Rng;"));
        assert!(cleaned.contains("fn main()"));
    }

    #[test]
    fn has_dependencies_detection() {
        assert!(has_dependencies(
            "//! [dependencies]\n//! x = \"1\"\nfn main() {}"
        ));
        assert!(!has_dependencies("fn main() { println!(\"hi\"); }"));
    }

    #[test]
    fn generate_toml_includes_package() {
        let manifest = EmbeddedManifest {
            toml_content: "[dependencies]\nrand = \"0.8\"".to_string(),
        };
        let toml = generate_cargo_toml(&manifest, None);
        assert!(toml.contains("[package]"));
        assert!(toml.contains("edition = \"2021\""));
        assert!(toml.contains("[dependencies]"));
        assert!(toml.contains("rand = \"0.8\""));
    }

    #[test]
    fn generate_toml_respects_existing_package() {
        let manifest = EmbeddedManifest {
            toml_content: "[package]\nname = \"my-script\"\nedition = \"2021\"\n\n[dependencies]\nrand = \"0.8\"".to_string(),
        };
        let toml = generate_cargo_toml(&manifest, None);
        // Should NOT duplicate [package]
        assert_eq!(toml.matches("[package]").count(), 1);
        assert!(toml.contains("my-script"));
    }

    #[test]
    fn is_cargo_project_detection() {
        // The virtual-rust project itself is a cargo project
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(is_cargo_project(project_root));

        // A non-existent path is not
        assert!(!is_cargo_project(Path::new("/nonexistent/fake/path")));

        // A file is not a directory
        let cargo_toml = project_root.join("Cargo.toml");
        assert!(!is_cargo_project(&cargo_toml));
    }

    #[test]
    fn read_project_name_works() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let name = read_project_name(&project_root.join("Cargo.toml"));
        assert_eq!(name, Some("virtual-rust".to_string()));
    }

    #[test]
    fn is_rust_source_dir_detection() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));

        // A directory with .rs files and no Cargo.toml
        let loose_dir = project_root.join("examples").join("loose_modules");
        assert!(is_rust_source_dir(&loose_dir));

        // A Cargo project directory is NOT a rust source dir
        assert!(!is_rust_source_dir(project_root));

        // A non-existent path is not
        assert!(!is_rust_source_dir(Path::new("/nonexistent/fake/path")));

        // A file is not a directory
        assert!(!is_rust_source_dir(&project_root.join("Cargo.toml")));
    }

    #[test]
    fn find_entry_file_works() {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let loose_dir = project_root.join("examples").join("loose_modules");

        let (entry, all_files) = find_entry_file(&loose_dir).unwrap();
        assert_eq!(entry.file_name().unwrap(), "main.rs");
        assert_eq!(all_files.len(), 3); // main.rs, math.rs, greeting.rs
    }

    #[test]
    fn find_entry_file_no_main() {
        // Create a temp dir with a .rs file that has no main
        let tmp = std::env::temp_dir().join("virtual-rust-test-no-main");
        let _ = fs::create_dir_all(&tmp);
        fs::write(tmp.join("lib.rs"), "pub fn foo() {}").unwrap();
        let result = find_entry_file(&tmp);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No entry point found"));
        let _ = fs::remove_dir_all(&tmp);
    }
}
