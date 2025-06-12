import type { editor, languages, Position, CancellationToken, IPosition, IRange } from 'monaco-editor';
import { Provider } from './base.js';
import { VmSharedGlobal } from 'mirascript/subtle';
import { codeblock, getGlobal, paramsList } from '../utils';
import { keywords, reservedKeywords } from '../../constants';
import { getVmFunctionInfo, DiagnosticCode } from 'mirascript';
import type { Monaco } from '../../index.js';

const DESC_GLOBAL = '(global)';
const DESC_LOCAL = '(local)';

const SUGGEST_KEYWORDS: string[] = [];
{
    const reserved = reservedKeywords();
    for (const kw of keywords()) {
        if (reserved.includes(kw)) continue; // 跳过保留关键字
        SUGGEST_KEYWORDS.push(kw);
    }
}

const COMMON_GLOBAL_SUGGESTIONS = ({ languages }: Monaco, range: IRange): languages.CompletionItem[] => {
    const suggestions: languages.CompletionItem[] = [
        {
            label: 'type',
            kind: languages.CompletionItemKind.Keyword,
            insertText: 'type',
            commitCharacters: ['('],
            documentation: {
                value: `使用 \`type()\` 调用获取表达式的类型。${codeblock('type(expression);\nexpression::type();')}`,
            },
            range,
        },
        {
            label: { label: 'if', description: 'If 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'if ${1:condition} {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
        },
        {
            label: { label: 'ifelse', description: 'If-Else 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'if ${1:condition} {\n\t$0\n} else {\n\t\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
        },
        {
            label: { label: 'loop', description: 'Loop 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'loop {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
        },
        {
            label: { label: 'while', description: 'While 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'while ${1:condition} {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
        },
        {
            label: { label: 'whileelse', description: 'While-Else 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'while ${1:condition} {\n\t$0\n} else {\n\t\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
        },
        {
            label: { label: 'for', description: 'For 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'for ${1:item} in ${2:collection} {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
        },
        {
            label: { label: 'forelse', description: 'For-Else 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'for ${1:item} in ${2:collection} {\n\t$0\n} else {\n\t\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            range,
        },
    ];
    for (const kw of SUGGEST_KEYWORDS) {
        const exist = suggestions.find(
            (item) => item.label === kw && item.kind === languages.CompletionItemKind.Keyword,
        );
        if (exist) continue;
        suggestions.push({
            label: kw,
            kind: languages.CompletionItemKind.Keyword,
            insertText: kw,
            range,
        });
    }
    return suggestions;
};

/**
 * 自动完成
 */
export class CompletionItemProvider extends Provider implements languages.CompletionItemProvider {
    readonly triggerCharacters?: string[] | undefined;
    /** 查找全局变量 */
    private completeGlobal(
        model: editor.ITextModel,
        char: string | undefined,
        range: IRange,
    ): languages.CompletionItem[] {
        const {
            languages: { CompletionItemKind },
        } = this.monaco;
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
                detail = paramsList(model, info);
            }
            suggestions.push({
                label: { label: key, description: DESC_GLOBAL, detail },
                kind: info ? CompletionItemKind.Function : CompletionItemKind.Variable,
                insertText: key,
                range,
                commitCharacters: info ? ['('] : undefined,
            });
        }
        return suggestions;
    }
    /** 查找局部变量 */
    private async completeLocal(
        model: editor.ITextModel,
        position: IPosition,
        char: string | undefined,
        range: IRange,
    ): Promise<languages.CompletionItem[]> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return [];
        const {
            languages: { CompletionItemKind },
            Range,
        } = this.monaco;
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
            for (const { definition, fn } of scope.locals) {
                const name = model.getValueInRange(definition.range);
                if (locals.has(name)) continue; // 子作用域可能会覆盖父作用域的变量
                if (char && !name.toLowerCase().includes(char)) {
                    continue;
                }
                locals.add(name);
                const isFunction = definition.code === DiagnosticCode.LocalFunction;
                let detail = '';
                if (isFunction) {
                    detail = paramsList(model, fn);
                }
                suggestions.push({
                    label: { label: name, description: DESC_LOCAL, detail },
                    kind: fn || isFunction ? CompletionItemKind.Function : CompletionItemKind.Variable,
                    insertText: name,
                    range,
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
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;

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

        // suggest variables
        let char: string | undefined;
        let range: IRange;
        const def = compiled.definition(model, position);
        if (def) {
            if (def.ref == null) {
                // 输入位置是变量定义
                return { suggestions: [] };
            }
            const d = def.def;
            range = d.references[def.ref]!.range;
            char = model.getValueInRange({
                startLineNumber: range.startLineNumber,
                startColumn: range.startColumn,
                endLineNumber: range.startLineNumber,
                endColumn: range.startColumn + 1,
            });
        } else if (word) {
            range = {
                startLineNumber: position.lineNumber,
                startColumn: word.startColumn,
                endLineNumber: position.lineNumber,
                endColumn: word.endColumn,
            };
            char = word.word[0];
        } else {
            range = {
                startLineNumber: position.lineNumber,
                startColumn: position.column - 1,
                endLineNumber: position.lineNumber,
                endColumn: position.column,
            };
        }
        char = char?.toLowerCase();

        const suggestions = COMMON_GLOBAL_SUGGESTIONS(this.monaco, range);
        suggestions.push(
            ...this.completeGlobal(model, char, range),
            ...(await this.completeLocal(model, position, char, range)),
        );

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
