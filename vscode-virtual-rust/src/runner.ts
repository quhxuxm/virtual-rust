/**
 * Run virtual-rust files via a VSCode terminal.
 *
 * Supports two execution modes:
 * 1. `cargo run` in the shadow project directory (default, ensures deps compile)
 * 2. `virtual-rust <file>` using the virtual-rust binary
 */

import * as vscode from 'vscode';
import { ShadowProjectManager } from './shadow';

export class Runner {
    private terminal: vscode.Terminal | undefined;

    constructor (private shadowManager: ShadowProjectManager) { }

    /**
     * Run the file at the given URI (or the active editor's file).
     */
    async runFile(uri?: vscode.Uri): Promise<void> {
        const filePath =
            uri?.fsPath ?? vscode.window.activeTextEditor?.document.uri.fsPath;

        if (!filePath) {
            vscode.window.showWarningMessage('Virtual Rust: no file to run');
            return;
        }

        const config = vscode.workspace.getConfiguration('virtual-rust');
        const useCargo = config.get<boolean>('runWithCargo', true);
        const binaryPath = config.get<string>('binaryPath', 'virtual-rust');

        const terminal = this.getOrCreateTerminal();

        if (useCargo) {
            // Prefer running via the shadow project so dependencies compile
            const projectDir = this.shadowManager.getProjectDir(filePath);
            if (projectDir) {
                terminal.sendText(`cd "${projectDir}" && cargo run`);
            } else {
                // No shadow project yet — try to sync first
                const editor = vscode.window.activeTextEditor;
                if (editor) {
                    const dir = await this.shadowManager.syncProject(editor.document);
                    if (dir) {
                        terminal.sendText(`cd "${dir}" && cargo run`);
                    } else {
                        // Not a virtual-rust file — fall back to binary
                        terminal.sendText(`"${binaryPath}" "${filePath}"`);
                    }
                } else {
                    terminal.sendText(`"${binaryPath}" "${filePath}"`);
                }
            }
        } else {
            terminal.sendText(`"${binaryPath}" "${filePath}"`);
        }

        terminal.show();
    }

    private getOrCreateTerminal(): vscode.Terminal {
        if (this.terminal && this.terminal.exitStatus === undefined) {
            return this.terminal;
        }
        this.terminal = vscode.window.createTerminal({
            name: 'Virtual Rust',
            iconPath: new vscode.ThemeIcon('beaker'),
        });
        return this.terminal;
    }

    dispose(): void {
        this.terminal?.dispose();
    }
}
