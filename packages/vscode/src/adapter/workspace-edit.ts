import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { toRange } from './range.js';
import { toUri } from './uri.js';

/**
 * Converts a Monaco Editor WorkspaceEdit to a VS Code WorkspaceEdit.
 */
export function toWorkspaceEdit(edit: monacoLanguages.WorkspaceEdit): vscode.WorkspaceEdit {
    const we = new vscode.WorkspaceEdit();
    if (edit.edits) {
        for (const e of edit.edits) {
            if ('redo' in e) {
                throw new Error('Cannot convert ICustomEdit to VS Code WorkspaceEdit.');
            }
            if ('resource' in e) {
                const uri = toUri(e.resource);
                we.replace(uri, toRange(e.textEdit.range), e.textEdit.text, e.metadata);
            } else {
                throw new Error('Cannot convert IWorkspaceFileEdit to VS Code WorkspaceEdit.');
            }
        }
    }
    return we;
}
