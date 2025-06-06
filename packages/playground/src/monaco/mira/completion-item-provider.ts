import { type editor, languages, type Position, type CancellationToken } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { VmSharedGlobal } from '../../vm/types/global.js';
import { codeblock, getGlobalDocument } from './utils';
import { keywords } from 'mira-wasm';

const DESC_GLOBAL = '(global)';

const COMMON_GLOBAL_SUGGESTIONS: languages.CompletionItem[] = [
    {
        label: 'type',
        kind: languages.CompletionItemKind.Keyword,
        insertText: 'type',
        commitCharacters: ['('],
        documentation: {
            value: `使用 \`type()\` 调用获取表达式的类型。${codeblock('type(expression);\nexpression::type();')}`,
        },
        range: undefined as never,
    },
    {
        label: { label: 'if', description: 'If 表达式' },
        kind: languages.CompletionItemKind.Snippet,
        insertText: 'if ${1:condition} {\n\t$0\n}',
        insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range: undefined as never,
    },
    {
        label: { label: 'ifelse', description: 'If-Else 表达式' },
        kind: languages.CompletionItemKind.Snippet,
        insertText: 'if ${1:condition} {\n\t$0\n} else {\n\t\n}',
        insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range: undefined as never,
    },
    {
        label: { label: 'loop', description: 'Loop 表达式' },
        kind: languages.CompletionItemKind.Snippet,
        insertText: 'loop {\n\t$0\n}',
        insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range: undefined as never,
    },
    {
        label: { label: 'while', description: 'While 表达式' },
        kind: languages.CompletionItemKind.Snippet,
        insertText: 'while ${1:condition} {\n\t$0\n}',
        insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range: undefined as never,
    },
    {
        label: { label: 'whileelse', description: 'While-Else 表达式' },
        kind: languages.CompletionItemKind.Snippet,
        insertText: 'while ${1:condition} {\n\t$0\n} else {\n\t\n}',
        insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range: undefined as never,
    },
    {
        label: { label: 'for', description: 'For 表达式' },
        kind: languages.CompletionItemKind.Snippet,
        insertText: 'for ${1:item} in ${2:collection} {\n\t$0\n}',
        insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range: undefined as never,
    },
    {
        label: { label: 'forelse', description: 'For-Else 表达式' },
        kind: languages.CompletionItemKind.Snippet,
        insertText: 'for ${1:item} in ${2:collection} {\n\t$0\n} else {\n\t\n}',
        insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range: undefined as never,
    },
];

for (const kw of keywords()) {
    const exist = COMMON_GLOBAL_SUGGESTIONS.find(
        (item) => item.label === kw && item.kind === languages.CompletionItemKind.Keyword,
    );
    if (exist) continue;
    COMMON_GLOBAL_SUGGESTIONS.push({
        label: kw,
        kind: languages.CompletionItemKind.Keyword,
        insertText: kw,
        range: undefined as never,
    });
}

/**
 * 自动完成
 */
class CompletionItemProvider extends Provider implements languages.CompletionItemProvider {
    readonly triggerCharacters?: string[] | undefined;
    /** @inheritdoc */
    provideCompletionItems(
        model: editor.ITextModel,
        position: Position,
        context: languages.CompletionContext,
        token: CancellationToken,
    ): languages.ProviderResult<languages.CompletionList> {
        const word = model.getWordAtPosition(position);
        const prev = model.getValueInRange({
            startLineNumber: position.lineNumber,
            startColumn: (word?.startColumn ?? position.column) - 2,
            endLineNumber: position.lineNumber,
            endColumn: word?.startColumn ?? position.column,
        });
        if (/\d$|\d\.$/gu.test(prev)) {
            // inputting number, do not suggest
            return {
                suggestions: [],
            };
        }
        if (/[^.]\.$/u.test(prev)) {
            // TODO: suggests item fields
            return {
                suggestions: [],
            };
        }
        // suggest global & local variables
        // TODO: suggests local variables
        const suggestions: languages.CompletionItem[] = structuredClone(COMMON_GLOBAL_SUGGESTIONS);

        for (const key in VmSharedGlobal) {
            const element = VmSharedGlobal[key];
            if (element === undefined) continue;
            const isFunction = typeof element === 'function';
            suggestions.push({
                label: { label: key, description: DESC_GLOBAL },
                kind: isFunction ? languages.CompletionItemKind.Function : languages.CompletionItemKind.Variable,
                insertText: key,
                range: undefined as never,
                commitCharacters: isFunction ? ['('] : undefined,
            });
        }
        return {
            suggestions,
        };
    }
    /** @inheritdoc */
    resolveCompletionItem(
        item: languages.CompletionItem,
        token: CancellationToken,
    ): languages.ProviderResult<languages.CompletionItem> {
        if (typeof item.label == 'string') {
            // not a dynamic completion item
            return item;
        }
        const { label, description } = item.label;
        if (description === DESC_GLOBAL) {
            if (item.documentation) return item;
            item.documentation = {
                value: getGlobalDocument(label),
            };
        }
        return item;
    }
}

languages.registerCompletionItemProvider('mirascript', new CompletionItemProvider());
