/**
 * CodeLens provider – shows a "▶ Run with Virtual Rust" button above
 * `fn main()` in files that contain a `//!` manifest or are part of a
 * loose `.rs` source directory.
 */

import * as vscode from 'vscode';
import { isVirtualRustSource, detectLooseModules } from './detector';

export class VirtualRustCodeLensProvider implements vscode.CodeLensProvider {
    private _onDidChangeCodeLenses = new vscode.EventEmitter<void>();
    readonly onDidChangeCodeLenses = this._onDidChangeCodeLenses.event;

    provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
        const text = document.getText();
        const filePath = document.uri.fsPath;
        const isManifest = isVirtualRustSource(text);
        const looseInfo = detectLooseModules(filePath);

        // Show CodeLens for virtual-rust manifest files or loose module entry files
        if (!isManifest && !(looseInfo && looseInfo.entryFile === filePath)) {
            return [];
        }

        const lenses: vscode.CodeLens[] = [];

        for (let i = 0; i < document.lineCount; i++) {
            const line = document.lineAt(i);
            // Match `fn main(` or `async fn main(` with optional `pub`
            if (/^\s*(?:pub\s+)?(?:async\s+)?fn\s+main\s*\(/.test(line.text)) {
                const range = new vscode.Range(i, 0, i, line.text.length);
                const tooltip = looseInfo
                    ? `Compile and run this directory (${looseInfo.allFiles.length} files)`
                    : 'Compile and run this file with its //! dependencies';
                lenses.push(
                    new vscode.CodeLens(range, {
                        title: '▶ Run with Virtual Rust',
                        tooltip,
                        command: 'virtual-rust.run',
                        arguments: [document.uri],
                    })
                );
                break; // Only one main function per file
            }
        }

        return lenses;
    }

    /** Force a CodeLens refresh (e.g. after a file save). */
    refresh(): void {
        this._onDidChangeCodeLenses.fire();
    }
}
