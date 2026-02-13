import { vscode } from '#loader';
import * as monaco from '@private/monaco-editor/baseapi';
import { CompletionItemInsertTextRule } from './utils.js';

/** 向对象添加语言相关枚举 */
function addLanguages(obj: Record<string, unknown>): void {
    obj['IndentAction'] = vscode.IndentAction;
    obj['SymbolKind'] = vscode.SymbolKind;
    obj['DocumentHighlightKind'] = vscode.DocumentHighlightKind;
    obj['InlayHintKind'] = vscode.InlayHintKind;
    obj['CompletionItemKind'] = vscode.CompletionItemKind;
    obj['CompletionTriggerKind'] = vscode.CompletionTriggerKind;
    obj['CompletionItemTag'] = vscode.CompletionItemTag;
    obj['CompletionItemInsertTextRule'] = CompletionItemInsertTextRule;
    obj['SignatureHelpTriggerKind'] = vscode.SignatureHelpTriggerKind;
}

/** 创建 Monaco API */
export function createMonacoApi(): typeof import('@private/monaco-editor/api') {
    const api = { ...monaco } as typeof import('@private/monaco-editor/api');
    api.editor ??= {} as typeof import('@private/monaco-editor/api').editor;
    api.languages ??= {} as typeof import('@private/monaco-editor/api').languages;
    addLanguages(api.languages);
    return api;
}
