/**
 * Virtual Rust — VSCode extension entry point.
 *
 * Provides IDE support for the Virtual Rust single-file format where Cargo
 * dependencies are declared inline via `//!` doc comments:
 *
 * ```rust
 * //! [dependencies]
 * //! rand = "0.8"
 *
 * use rand::Rng;
 * fn main() { ... }
 * ```
 *
 * Features:
 * - Shadow Cargo project generation for rust-analyzer integration
 * - "▶ Run with Virtual Rust" CodeLens above `fn main()`
 * - Manifest block syntax decorations
 * - Status bar indicator
 */

import * as vscode from 'vscode';
import { isVirtualRustSource } from './detector';
import { ShadowProjectManager } from './shadow';
import { Runner } from './runner';
import { VirtualRustCodeLensProvider } from './codeLens';
import { ManifestDecorationProvider } from './decoration';

let shadowManager: ShadowProjectManager;
let runner: Runner;
let decorationProvider: ManifestDecorationProvider;
let codeLensProvider: VirtualRustCodeLensProvider;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext): void {
    // ── Initialise components ────────────────────────────────────

    shadowManager = new ShadowProjectManager(context);
    runner = new Runner(shadowManager);
    decorationProvider = new ManifestDecorationProvider();
    codeLensProvider = new VirtualRustCodeLensProvider();

    // ── Status bar ───────────────────────────────────────────────

    statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        100
    );
    statusBarItem.text = '$(beaker) Virtual Rust';
    statusBarItem.tooltip = 'This file uses Virtual Rust inline dependencies';
    statusBarItem.command = 'virtual-rust.run';
    context.subscriptions.push(statusBarItem);

    // ── Commands ─────────────────────────────────────────────────

    context.subscriptions.push(
        vscode.commands.registerCommand(
            'virtual-rust.run',
            (uri?: vscode.Uri) => runner.runFile(uri)
        )
    );

    context.subscriptions.push(
        vscode.commands.registerCommand(
            'virtual-rust.syncProject',
            async () => {
                const editor = vscode.window.activeTextEditor;
                if (!editor || editor.document.languageId !== 'rust') {
                    vscode.window.showWarningMessage(
                        'Virtual Rust: open a .rs file first'
                    );
                    return;
                }
                const dir = await shadowManager.syncProject(editor.document);
                if (dir) {
                    vscode.window.showInformationMessage(
                        `Virtual Rust: shadow project synced → ${dir}`
                    );
                } else {
                    vscode.window.showWarningMessage(
                        'Virtual Rust: no //! manifest found in this file'
                    );
                }
            }
        )
    );

    context.subscriptions.push(
        vscode.commands.registerCommand(
            'virtual-rust.cleanShadowProjects',
            () => shadowManager.cleanAll()
        )
    );

    // ── CodeLens ─────────────────────────────────────────────────

    context.subscriptions.push(
        vscode.languages.registerCodeLensProvider(
            { language: 'rust' },
            codeLensProvider
        )
    );

    // ── Event handlers ───────────────────────────────────────────

    const isAutoSync = (): boolean =>
        vscode.workspace
            .getConfiguration('virtual-rust')
            .get<boolean>('autoSync', true);

    // Auto-sync on file open
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument(async (doc) => {
            if (
                doc.languageId === 'rust' &&
                isAutoSync() &&
                isVirtualRustSource(doc.getText())
            ) {
                await shadowManager.syncProject(doc);
            }
        })
    );

    // Auto-sync on file save
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(async (doc) => {
            if (
                doc.languageId === 'rust' &&
                isAutoSync() &&
                isVirtualRustSource(doc.getText())
            ) {
                await shadowManager.syncProject(doc);
                codeLensProvider.refresh();
            }
        })
    );

    // Update decorations & status bar when the active editor changes
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor((editor) => {
            if (editor) {
                updateStatusBar(editor);
                decorationProvider.updateDecorations(editor);
            } else {
                statusBarItem.hide();
            }
        })
    );

    // Update decorations on text change (live feedback while typing)
    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument((event) => {
            const editor = vscode.window.activeTextEditor;
            if (editor && event.document === editor.document) {
                decorationProvider.updateDecorations(editor);
                updateStatusBar(editor);
            }
        })
    );

    // ── Process already-open documents ───────────────────────────

    if (vscode.window.activeTextEditor) {
        const editor = vscode.window.activeTextEditor;
        updateStatusBar(editor);
        decorationProvider.updateDecorations(editor);

        if (
            isAutoSync() &&
            editor.document.languageId === 'rust' &&
            isVirtualRustSource(editor.document.getText())
        ) {
            shadowManager.syncProject(editor.document);
        }
    }

    for (const doc of vscode.workspace.textDocuments) {
        if (
            doc.languageId === 'rust' &&
            isAutoSync() &&
            isVirtualRustSource(doc.getText())
        ) {
            shadowManager.syncProject(doc);
        }
    }

    // ── Ensure .gitignore covers shadow dir ──────────────────────

    ensureGitignore();
}

// ── Helpers ──────────────────────────────────────────────────────

function updateStatusBar(editor: vscode.TextEditor): void {
    if (
        editor.document.languageId === 'rust' &&
        isVirtualRustSource(editor.document.getText())
    ) {
        statusBarItem.show();
    } else {
        statusBarItem.hide();
    }
}

/**
 * Append the shadow project directory to `.gitignore` if it isn't
 * already listed.
 */
async function ensureGitignore(): Promise<void> {
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!workspaceRoot) {
        return;
    }

    const config = vscode.workspace.getConfiguration('virtual-rust');
    const shadowDirName = config.get<string>(
        'shadowProjectDir',
        '.virtual-rust'
    );
    const gitignoreUri = vscode.Uri.joinPath(
        vscode.Uri.file(workspaceRoot),
        '.gitignore'
    );

    try {
        const raw = await vscode.workspace.fs.readFile(gitignoreUri);
        const content = Buffer.from(raw).toString('utf-8');
        if (!content.includes(shadowDirName)) {
            const updated =
                content.trimEnd() +
                '\n\n# Virtual Rust shadow projects\n' +
                shadowDirName +
                '/\n';
            await vscode.workspace.fs.writeFile(
                gitignoreUri,
                Buffer.from(updated)
            );
        }
    } catch {
        // .gitignore doesn't exist — that's fine
    }
}

export function deactivate(): void {
    shadowManager?.dispose();
    runner?.dispose();
    decorationProvider?.dispose();
}
