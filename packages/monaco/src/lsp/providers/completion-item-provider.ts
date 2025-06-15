import {
    type editor,
    languages,
    type Position,
    type CancellationToken,
    type IPosition,
    type IRange,
} from '../../monaco-api.js';
import { Provider } from './base.js';
import { VmSharedGlobal } from 'mirascript/subtle';
import { codeblock, getGlobal, paramsList } from '../utils';
import { keywords, reservedKeywords } from '../../constants';
import { getVmFunctionInfo, DiagnosticCode } from 'mirascript';

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

const COMMON_GLOBAL_SUGGESTIONS = (range: IRange): languages.CompletionItem[] => {
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
            label: 'global',
            kind: languages.CompletionItemKind.Keyword,
            insertText: 'global',
            commitCharacters: ['.', '['],
            documentation: {
                value: `使用 \`global\` 获取全局变量。${codeblock('global.variableName;\nglobal["variableName"];\n"variableName" in global;')}`,
            },
            range,
        },
        {
            label: { label: 'if', description: 'If 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'if ${1:condition} {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`if\` 表达式进行条件判断。${codeblock('if condition {\n\t// code\n}')}`,
            },
            range,
        },
        {
            label: { label: 'ifelse', description: 'If-Else 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'if ${1:condition} {\n\t$0\n} else {\n\t\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`ifelse\` 表达式进行条件判断。${codeblock('if condition {\n\t// code\n} else {\n\t// code\n}')}`,
            },
            range,
        },
        {
            label: { label: 'match', description: 'Match 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText:
                'match ${1:value} {\n\tcase ${2:case1} {\n\t\t$0\n\t}\n\tcase ${3:case2} {\n\t\t\n\t}\n\tcase _ {\n\t\t\n\t}\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`match\` 表达式进行模式匹配。${codeblock('match value {\n\tcase case1 {\n\t\t// code\n\t}\n\tcase case2 {\n\t\t// code\n\t}\n\tcase _ {\n\t\t// code\n\t}\n}')}`,
            },
            range,
        },
        {
            label: { label: 'loop', description: 'Loop 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'loop {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`loop\` 表达式进行无限循环。${codeblock('loop {\n\t// code\n}')}`,
            },
            range,
        },
        {
            label: { label: 'while', description: 'While 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'while ${1:condition} {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`while\` 表达式进行条件循环。${codeblock('while condition {\n\t// code\n}')}`,
            },
            range,
        },
        {
            label: { label: 'whileelse', description: 'While-Else 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'while ${1:condition} {\n\t$0\n} else {\n\t\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`whileelse\` 表达式进行条件循环。${codeblock('while condition {\n\t// code\n} else {\n\t// code\n}')}`,
            },
            range,
        },
        {
            label: { label: 'for', description: 'For 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'for ${1:item} in ${2:collection} {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`for\` 表达式进行迭代循环。${codeblock('for item in collection {\n\t// code\n}')}`,
            },
            range,
        },
        {
            label: { label: 'forelse', description: 'For-Else 表达式' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'for ${1:item} in ${2:collection} {\n\t$0\n} else {\n\t\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`forelse\` 表达式进行迭代循环。${codeblock('for item in collection {\n\t// code\n} else {\n\t// code\n}')}`,
            },
            range,
        },
        {
            label: { label: 'fn', description: 'Fn 语句' },
            kind: languages.CompletionItemKind.Snippet,
            insertText: 'fn ${1:name}(${2:params}) {\n\t$0\n}',
            insertTextRules: languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
                value: `使用 \`fn\` 语句进行函数声明。${codeblock('fn name(params) {\n\t// code\n}')}`,
            },
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
                kind: info ? languages.CompletionItemKind.Function : languages.CompletionItemKind.Variable,
                insertText: key,
                range,
                commitCharacters: info ? ['('] : ['.', '[', '('],
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
        const suggestions: languages.CompletionItem[] = [];

        let scope = compiled.scopeAt(model, position);
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
                    kind:
                        fn || isFunction
                            ? languages.CompletionItemKind.Function
                            : languages.CompletionItemKind.Variable,
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

        const suggestions = COMMON_GLOBAL_SUGGESTIONS(range);
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
