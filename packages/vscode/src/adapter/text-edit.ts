import type * as monaco from '@private/monaco-editor/baseapi';
import { vscode } from '#loader';
import { toRange } from './range.js';

/**
 * Converts a Monaco Editor TextEdit to a VS Code TextEdit.
 */
export function toTextEdit(edit: { range: monaco.IRange; text: string | null }): vscode.TextEdit {
    if (edit instanceof vscode.TextEdit) return edit;
    if (!edit.text) {
        return vscode.TextEdit.delete(toRange(edit.range));
    }
    return vscode.TextEdit.replace(toRange(edit.range), edit.text);
}
