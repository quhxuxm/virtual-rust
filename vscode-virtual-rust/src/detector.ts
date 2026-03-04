/**
 * Detect and parse Virtual Rust embedded manifests from `//!` doc comments.
 *
 * Virtual Rust files declare Cargo dependencies inline:
 * ```rust
 * //! [dependencies]
 * //! rand = "0.8"
 * //! serde = { version = "1.0", features = ["derive"] }
 * ```
 */

/** Parsed manifest information extracted from `//!` comments. */
export interface ManifestInfo {
    /** Raw TOML content (e.g. `[dependencies]\nrand = "0.8"`) */
    tomlContent: string;
    /** First manifest line (0-indexed) */
    startLine: number;
    /** Last manifest line (0-indexed, inclusive) */
    endLine: number;
}

/**
 * Quick test: does this source text contain a Virtual Rust manifest?
 */
export function isVirtualRustSource(text: string): boolean {
    return parseManifest(text) !== null;
}

/**
 * Parse `//!` doc comments at the top of a Rust source file for embedded
 * Cargo manifest sections (`[dependencies]`, `[package]`, etc.).
 *
 * Returns `null` when no TOML section header is found in the leading
 * `//!` block.
 */
export function parseManifest(text: string): ManifestInfo | null {
    const lines = text.split('\n');
    const tomlLines: string[] = [];
    let foundSection = false;
    let startLine = -1;
    let endLine = -1;

    for (let i = 0; i < lines.length; i++) {
        const trimmed = lines[i].trim();

        if (trimmed.startsWith('//!')) {
            let content = trimmed.slice(3);
            // Strip a single leading space after `//!`
            if (content.startsWith(' ')) {
                content = content.slice(1);
            }
            if (content.startsWith('[')) {
                foundSection = true;
            }
            if (startLine === -1) {
                startLine = i;
            }
            endLine = i;
            tomlLines.push(content);
        } else if (trimmed === '') {
            // Allow blank lines within the leading doc-comment block
            if (foundSection) {
                tomlLines.push('');
            }
        } else {
            // First non-doc-comment, non-empty line — end of manifest
            break;
        }
    }

    if (!foundSection) {
        return null;
    }

    return {
        tomlContent: tomlLines.join('\n'),
        startLine,
        endLine,
    };
}

/**
 * Generate a complete `Cargo.toml` from a parsed manifest.
 *
 * If the manifest already contains a `[package]` section it is used as-is;
 * otherwise a default package header is prepended.
 */
export function generateCargoToml(manifest: ManifestInfo, name: string): string {
    if (manifest.tomlContent.includes('[package]')) {
        return manifest.tomlContent + '\n';
    }

    return [
        '[package]',
        `name = "${name}"`,
        'version = "0.1.0"',
        'edition = "2021"',
        '',
        manifest.tomlContent,
        '',
    ].join('\n');
}
