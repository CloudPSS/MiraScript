import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { createAdapterFactory } from './base.js';
import { toDiagnostic } from './diagnostic.js';
import { toWorkspaceEdit } from './workspace-edit.js';

export const [toCodeAction, fromCodeAction] = createAdapterFactory<monacoLanguages.CodeAction, vscode.CodeAction>(
    (action) => {
        return new vscode.CodeAction(action.title);
    },
    (action, ca) => {
        switch (action.kind) {
            case 'quickfix':
                ca.kind = vscode.CodeActionKind.QuickFix;
                break;
            case 'refactor':
                ca.kind = vscode.CodeActionKind.Refactor;
                break;
            case 'refactor.extract':
                ca.kind = vscode.CodeActionKind.RefactorExtract;
                break;
            case 'refactor.inline':
                ca.kind = vscode.CodeActionKind.RefactorInline;
                break;
            case 'refactor.move':
                ca.kind = vscode.CodeActionKind.RefactorMove;
                break;
            case 'refactor.rewrite':
                ca.kind = vscode.CodeActionKind.RefactorRewrite;
                break;
            case 'source':
                ca.kind = vscode.CodeActionKind.Source;
                break;
            case 'source.organizeImports':
                ca.kind = vscode.CodeActionKind.SourceOrganizeImports;
                break;
            case 'source.fixAll':
                ca.kind = vscode.CodeActionKind.SourceFixAll;
                break;
            case undefined:
            default:
                break;
        }
        if (action.edit) ca.edit = toWorkspaceEdit(action.edit);
        if (action.diagnostics) ca.diagnostics = action.diagnostics.map(toDiagnostic);
        if (action.command) throw new Error('Cannot convert CodeAction with command to VS Code CodeAction.');
        ca.isPreferred = action.isPreferred;
        ca.disabled = action.disabled ? { reason: action.disabled } : undefined;
    },
);
