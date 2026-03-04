/**
 * Shadow Cargo project manager.
 *
 * For each virtual-rust file (a `.rs` file with `//!` manifest comments),
 * this module creates a shadow Cargo project containing:
 *
 *   .virtual-rust/<project-name>/
 *   ├── Cargo.toml          ← generated from //! comments
 *   └── src/
 *       └── main.rs         ← symlink to the original .rs file
 *
 * The shadow project is then registered in `rust-analyzer.linkedProjects`
 * so that rust-analyzer can resolve the declared dependencies, providing
 * correct diagnostics, completions, and go-to-definition.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as crypto from 'crypto';
import { parseManifest, generateCargoToml } from './detector';

export class ShadowProjectManager {
    private shadowBaseDir: string;
    private managedProjects = new Map<string, string>();
    private updateTimer: ReturnType<typeof setTimeout> | null = null;
    private outputChannel: vscode.OutputChannel;

    constructor (private context: vscode.ExtensionContext) {
        this.outputChannel = vscode.window.createOutputChannel('Virtual Rust');

        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        const config = vscode.workspace.getConfiguration('virtual-rust');
        const dirName = config.get<string>('shadowProjectDir', '.virtual-rust');

        this.shadowBaseDir = workspaceRoot
            ? path.join(workspaceRoot, dirName)
            : path.join(context.globalStorageUri.fsPath, 'shadow-projects');
    }

    // ── Public API ─────────────────────────────────────────────────

    /**
     * Create or update the shadow Cargo project for a virtual-rust file
     * and schedule a `rust-analyzer.linkedProjects` update.
     *
     * Returns the shadow project directory on success, `null` if the
     * document does not contain a virtual-rust manifest.
     */
    async syncProject(document: vscode.TextDocument): Promise<string | null> {
        const text = document.getText();
        const manifest = parseManifest(text);

        if (!manifest) {
            return null;
        }

        const filePath = document.uri.fsPath;
        const projectName = this.makeProjectName(filePath);
        const projectDir = path.join(this.shadowBaseDir, projectName);
        const srcDir = path.join(projectDir, 'src');

        try {
            // Ensure directory structure exists
            fs.mkdirSync(srcDir, { recursive: true });

            // Generate Cargo.toml from the //! manifest
            const cargoToml = generateCargoToml(manifest, projectName);
            const cargoTomlPath = path.join(projectDir, 'Cargo.toml');
            this.writeIfChanged(cargoTomlPath, cargoToml);

            // Symlink src/main.rs → original file
            const mainRs = path.join(srcDir, 'main.rs');
            this.ensureSymlink(filePath, mainRs);

            this.managedProjects.set(filePath, projectDir);
            this.scheduleLinkedProjectsUpdate();

            this.outputChannel.appendLine(
                `[sync] ${path.basename(filePath)} → ${projectDir}`
            );
            return projectDir;
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            this.outputChannel.appendLine(
                `[error] Failed to sync shadow project for ${filePath}: ${msg}`
            );
            vscode.window.showErrorMessage(
                `Virtual Rust: failed to sync shadow project — ${msg}`
            );
            return null;
        }
    }

    /** Check whether a file is currently managed by a shadow project. */
    isManaged(filePath: string): boolean {
        return this.managedProjects.has(filePath);
    }

    /** Get the shadow project directory for a managed file. */
    getProjectDir(filePath: string): string | undefined {
        return this.managedProjects.get(filePath);
    }

    /** Remove all shadow projects and clean `linkedProjects`. */
    async cleanAll(): Promise<void> {
        if (fs.existsSync(this.shadowBaseDir)) {
            fs.rmSync(this.shadowBaseDir, { recursive: true, force: true });
            this.managedProjects.clear();
            this.outputChannel.appendLine('[clean] Removed all shadow projects');

            // Remove our entries from linkedProjects
            const config = vscode.workspace.getConfiguration('rust-analyzer');
            const current = config.get<(string | object)[]>('linkedProjects', []);
            const filtered = current.filter(
                (p) => typeof p !== 'string' || !p.startsWith(this.shadowBaseDir)
            );
            if (filtered.length !== current.length) {
                await config.update(
                    'linkedProjects',
                    filtered.length > 0 ? filtered : undefined,
                    vscode.ConfigurationTarget.Workspace
                );
            }
        }
        vscode.window.showInformationMessage('Virtual Rust: shadow projects cleaned');
    }

    /** Show the output channel for debugging. */
    showOutput(): void {
        this.outputChannel.show();
    }

    // ── Private helpers ────────────────────────────────────────────

    /**
     * Derive a Cargo-safe package name from a file path.
     * Includes a short hash to avoid collisions.
     */
    private makeProjectName(filePath: string): string {
        const base = path
            .basename(filePath, '.rs')
            .replace(/[^a-zA-Z0-9_-]/g, '-');
        const hash = crypto
            .createHash('md5')
            .update(filePath)
            .digest('hex')
            .slice(0, 8);
        return `vr-${base}-${hash}`;
    }

    /** Write a file only when its content has actually changed. */
    private writeIfChanged(filePath: string, content: string): void {
        try {
            const existing = fs.readFileSync(filePath, 'utf-8');
            if (existing === content) {
                return;
            }
        } catch {
            // File does not exist yet — fall through to write
        }
        fs.writeFileSync(filePath, content, 'utf-8');
    }

    /**
     * Ensure a symlink points at the expected target.
     * Falls back to a file copy on systems where symlinks fail
     * (e.g. Windows without developer mode).
     */
    private ensureSymlink(target: string, linkPath: string): void {
        try {
            const existing = fs.readlinkSync(linkPath);
            if (existing === target) {
                return; // Already correct
            }
            fs.unlinkSync(linkPath);
        } catch {
            // File doesn't exist or isn't a symlink — remove & recreate
            try {
                fs.unlinkSync(linkPath);
            } catch {
                /* ok */
            }
        }

        try {
            fs.symlinkSync(target, linkPath);
        } catch {
            // Symlink failed (e.g. Windows) — fall back to copy
            fs.copyFileSync(target, linkPath);
        }
    }

    /** Debounced update of `rust-analyzer.linkedProjects`. */
    private scheduleLinkedProjectsUpdate(): void {
        if (this.updateTimer) {
            clearTimeout(this.updateTimer);
        }
        this.updateTimer = setTimeout(() => {
            this.doUpdateLinkedProjects();
        }, 500);
    }

    private async doUpdateLinkedProjects(): Promise<void> {
        const config = vscode.workspace.getConfiguration('rust-analyzer');
        const current = config.get<(string | object)[]>('linkedProjects', []);
        const currentPaths = new Set(
            current.map((p) => (typeof p === 'string' ? p : ''))
        );

        const toAdd: string[] = [];
        for (const [, projectDir] of this.managedProjects) {
            const tomlPath = path.join(projectDir, 'Cargo.toml');
            if (!currentPaths.has(tomlPath)) {
                toAdd.push(tomlPath);
            }
        }

        if (toAdd.length > 0) {
            const updated = [...current, ...toAdd];
            await config.update(
                'linkedProjects',
                updated,
                vscode.ConfigurationTarget.Workspace
            );
            this.outputChannel.appendLine(
                `[rust-analyzer] Added ${toAdd.length} shadow project(s) to linkedProjects`
            );
        }
    }

    dispose(): void {
        if (this.updateTimer) {
            clearTimeout(this.updateTimer);
        }
        this.outputChannel.dispose();
    }
}
