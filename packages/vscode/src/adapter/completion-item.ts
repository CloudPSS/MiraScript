import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { CompletionItemInsertTextRule } from './api.js';
import { createAdapterFactory } from './base.js';
import { toCommand } from './command.js';
import { toMarkdownString } from './markdown-string.js';
import { toRange } from './range.js';
import { toTextEdit } from './text-edit.js';

export const [toCompletionItem, fromCompletionItem] = createAdapterFactory<
    monacoLanguages.CompletionItem,
    vscode.CompletionItem
>(
    (item) => {
        return new vscode.CompletionItem(item.label);
    },
    (item, ci) => {
        ci.label = item.label;
        ci.kind = item.kind as unknown as vscode.CompletionItemKind;
        ci.tags = item.tags;
        ci.detail = item.detail;
        ci.documentation = toMarkdownString(item.documentation);
        ci.sortText = item.sortText;
        ci.filterText = item.filterText;
        ci.preselect = item.preselect;
        ci.insertText =
            item.insertTextRules === CompletionItemInsertTextRule.InsertAsSnippet
                ? new vscode.SnippetString(item.insertText)
                : item.insertText;
        const range =
            'insert' in item.range
                ? {
                      inserting: toRange(item.range.insert),
                      replacing: toRange(item.range.replace),
                  }
                : toRange(item.range);
        ci.range = range;
        ci.commitCharacters = item.commitCharacters;
        ci.additionalTextEdits = item.additionalTextEdits?.map(toTextEdit);
        ci.command = toCommand(item.command);
    },
);
