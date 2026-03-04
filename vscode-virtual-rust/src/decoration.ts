/**
 * Decorations for the `//!` manifest block in virtual-rust files.
 *
 * Applies three decoration layers:
 * - A subtle background tint on the whole manifest region
 * - Bold gold colour on TOML section headers (`[dependencies]`)
 * - Blue colour on dependency key names (`rand`, `serde`)
 */

import * as vscode from 'vscode';
import { parseManifest } from './detector';

export class ManifestDecorationProvider {
    private bgDecoration: vscode.TextEditorDecorationType;
    private sectionDecoration: vscode.TextEditorDecorationType;
    private keyDecoration: vscode.TextEditorDecorationType;
    private versionDecoration: vscode.TextEditorDecorationType;

    constructor () {
        this.bgDecoration = vscode.window.createTextEditorDecorationType({
            backgroundColor: new vscode.ThemeColor(
                'editor.linkedEditingBackground'
            ),
            isWholeLine: true,
        });

        this.sectionDecoration = vscode.window.createTextEditorDecorationType({
            color: '#e5c07b',
            fontWeight: 'bold',
        });

        this.keyDecoration = vscode.window.createTextEditorDecorationType({
            color: '#61afef',
        });

        this.versionDecoration = vscode.window.createTextEditorDecorationType({
            color: '#98c379',
        });
    }

    /**
     * Recompute and apply decorations to the active editor.
     */
    updateDecorations(editor: vscode.TextEditor): void {
        if (editor.document.languageId !== 'rust') {
            return;
        }

        const text = editor.document.getText();
        const manifest = parseManifest(text);

        if (!manifest) {
            this.clearDecorations(editor);
            return;
        }

        const bgRanges: vscode.DecorationOptions[] = [];
        const sectionRanges: vscode.DecorationOptions[] = [];
        const keyRanges: vscode.DecorationOptions[] = [];
        const versionRanges: vscode.DecorationOptions[] = [];

        for (let i = manifest.startLine; i <= manifest.endLine; i++) {
            const line = editor.document.lineAt(i);
            const trimmed = line.text.trim();

            if (!trimmed.startsWith('//!')) {
                continue;
            }

            // Subtle background for the whole manifest block
            bgRanges.push({ range: line.range });

            const content = trimmed.slice(3).trim();

            // Section headers: [dependencies], [package], etc.
            const sectionMatch = content.match(/^\[([^\]]+)\]/);
            if (sectionMatch) {
                const openIdx = line.text.indexOf('[');
                const closeIdx = line.text.indexOf(']', openIdx);
                if (openIdx >= 0 && closeIdx >= 0) {
                    sectionRanges.push({
                        range: new vscode.Range(i, openIdx, i, closeIdx + 1),
                    });
                }
            }

            // Key names: `rand = "0.8"` or `serde = { ... }`
            const kvMatch = content.match(/^([a-zA-Z_][\w-]*)\s*=\s*(.*)/);
            if (kvMatch) {
                const keyStart = line.text.indexOf(kvMatch[1]);
                if (keyStart >= 0) {
                    keyRanges.push({
                        range: new vscode.Range(
                            i,
                            keyStart,
                            i,
                            keyStart + kvMatch[1].length
                        ),
                    });
                }

                // Version strings: "0.8", "1.0", etc.
                const versionMatch = kvMatch[2].match(/"([^"]+)"/);
                if (versionMatch) {
                    const vStart = line.text.indexOf(
                        `"${versionMatch[1]}"`,
                        keyStart
                    );
                    if (vStart >= 0) {
                        versionRanges.push({
                            range: new vscode.Range(
                                i,
                                vStart,
                                i,
                                vStart + versionMatch[0].length
                            ),
                        });
                    }
                }
            }
        }

        editor.setDecorations(this.bgDecoration, bgRanges);
        editor.setDecorations(this.sectionDecoration, sectionRanges);
        editor.setDecorations(this.keyDecoration, keyRanges);
        editor.setDecorations(this.versionDecoration, versionRanges);
    }

    private clearDecorations(editor: vscode.TextEditor): void {
        editor.setDecorations(this.bgDecoration, []);
        editor.setDecorations(this.sectionDecoration, []);
        editor.setDecorations(this.keyDecoration, []);
        editor.setDecorations(this.versionDecoration, []);
    }

    dispose(): void {
        this.bgDecoration.dispose();
        this.sectionDecoration.dispose();
        this.keyDecoration.dispose();
        this.versionDecoration.dispose();
    }
}
