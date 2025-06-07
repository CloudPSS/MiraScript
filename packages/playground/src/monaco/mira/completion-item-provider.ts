import {
    type editor,
    languages,
    type Position,
    type CancellationToken,
    Range,
    type IPosition,
} from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { VmSharedGlobal } from '../../vm/types/global.js';
import { codeblock, getGlobal } from './utils';
import { DiagnosticCode, keywords } from 'mira-wasm';
import { getVmFunctionInfo } from '../../vm';

const DESC_GLOBAL = '(global)';
const DESC_LOCAL = '(local)';

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
    /** 查找全局变量 */
    private completeGlobal(model: editor.ITextModel, char: string): languages.CompletionItem[] {
        const suggestions: languages.CompletionItem[] = [];
        for (const key in VmSharedGlobal) {
            if (char && !key.toLowerCase().includes(char)) {
                continue;
            }
            const element = VmSharedGlobal[key];
            if (element === undefined) continue;
            const info = getVmFunctionInfo(element);
            let detail = '';
            if (info) {
                if (info.params) {
                    detail = `(${Object.keys(info.params).join(', ')})`;
                } else {
                    detail = '(..)';
                }
            }
            suggestions.push({
                label: { label: key, description: DESC_GLOBAL, detail },
                kind: info ? languages.CompletionItemKind.Function : languages.CompletionItemKind.Variable,
                insertText: key,
                range: undefined as never,
                commitCharacters: info ? ['('] : undefined,
            });
        }
        return suggestions;
    }
    /** 查找局部变量 */
    private async completeLocal(
        model: editor.ITextModel,
        position: IPosition,
        char: string,
    ): Promise<languages.CompletionItem[]> {
        const compiled = await Provider.getCompileResult(model, false);
        if (!compiled) return [];
        const suggestions: languages.CompletionItem[] = [];
        const scopes = compiled.scopes(model);
        let scope = scopes.findLast((s) => Range.containsPosition(s.range, position)) ?? scopes[0]!; // 从根作用域开始查找
        while (scope.children.length > 0) {
            const inner = scope.children.find((s) => Range.containsPosition(s.range, position));
            if (!inner) break;
            scope = inner;
        }
        const locals = new Set<string>();
        while (scope) {
            for (const { definition, range } of scope.locals) {
                const name = model.getValueInRange(range);
                if (locals.has(name)) continue; // 子作用域可能会覆盖父作用域的变量
                if (char && !name.toLowerCase().includes(char)) {
                    continue;
                }
                locals.add(name);
                const isFunction = definition.code === DiagnosticCode.LocalFunction;
                let detail = '';
                if (isFunction) {
                    const funcScope = scope.children.find((s) => Range.compareRangesUsingStarts(s.range, range) > 0);
                    if (funcScope) {
                        const args = funcScope.locals.filter(
                            (l) =>
                                l.definition.code === DiagnosticCode.ParameterIt ||
                                l.definition.code === DiagnosticCode.UnusedParameterIt ||
                                l.definition.code === DiagnosticCode.ParameterMutable ||
                                l.definition.code === DiagnosticCode.ParameterImmutable ||
                                l.definition.code === DiagnosticCode.ParameterMutableRest ||
                                l.definition.code === DiagnosticCode.ParameterImmutableRest,
                        );
                        if (args[0]?.definition.code === DiagnosticCode.UnusedParameterIt) {
                            detail = '()';
                        } else if (args[0]?.definition.code === DiagnosticCode.ParameterIt) {
                            detail = '(it)';
                        } else {
                            detail = `(${args.map((a) => model.getValueInRange(a.range)).join(', ')})`;
                        }
                    } else {
                        detail = '(..)';
                    }
                }
                suggestions.push({
                    label: { label: name, description: DESC_LOCAL, detail },
                    kind: isFunction ? languages.CompletionItemKind.Function : languages.CompletionItemKind.Variable,
                    insertText: name,
                    range: undefined as never,
                    commitCharacters: isFunction ? ['('] : undefined,
                } satisfies languages.CompletionItem);
            }
            if (!scope.parent) break;
            scope = scope.parent;
        }
        return suggestions;
    }

    /** @inheritdoc */
    async provideCompletionItems(
        model: editor.ITextModel,
        position: Position,
        context: languages.CompletionContext,
        token: CancellationToken,
    ): Promise<languages.CompletionList | undefined> {
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

        const suggestions: languages.CompletionItem[] = structuredClone(COMMON_GLOBAL_SUGGESTIONS);

        // suggest variables
        const char = (word?.word[0] ?? '').toLowerCase();
        suggestions.push(...this.completeGlobal(model, char), ...(await this.completeLocal(model, position, char)));

        return { suggestions };
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
            const def = getGlobal(label);
            item.documentation = {
                value: `${codeblock('\0' + def.script)}\n${def.doc}`,
            };
        }
        return item;
    }
}

languages.registerCompletionItemProvider('mirascript', new CompletionItemProvider());
