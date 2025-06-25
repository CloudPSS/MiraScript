import { getVmFunctionInfo, DiagnosticCode, type VmValue, isVmExtern } from 'mirascript';
import {
    type editor,
    languages,
    type CancellationToken,
    type IPosition,
    type IRange,
    Position,
    Range,
} from '../../monaco-api.js';
import { Provider } from './base.js';
import { codeblock, globalDoc, paramsList } from '../utils.js';
import { keywords, REG_IDENTIFIER, REG_ORDINAL, reservedKeywords } from '../../constants.js';
import { lib, operations } from 'mirascript/subtle';

const DESC_GLOBAL = '(global)';
const DESC_LOCAL = '(local)';
const DESC_FIELD = '(field)';

const SUGGEST_KEYWORDS: string[] = [];
{
    const reserved = reservedKeywords();
    for (const kw of keywords()) {
        if (reserved.includes(kw)) continue; // 跳过保留关键字
        SUGGEST_KEYWORDS.push(kw);
    }
}

const COMMON_GLOBAL_SUGGESTIONS = (range: languages.CompletionItemRanges): languages.CompletionItem[] => {
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

/** 扩展完成项 */
interface CustomCompletionItem extends languages.CompletionItem {
    /** 对应的全局变量 */
    global?: VmValue;
}

/**
 * 自动完成
 */
export class CompletionItemProvider extends Provider implements languages.CompletionItemProvider {
    readonly triggerCharacters: string[] = ['.'];
    /** 查找全局变量 */
    private async completeGlobal(
        model: editor.ITextModel,
        char: string | undefined,
        locals: readonly CustomCompletionItem[],
        range: languages.CompletionItemRanges,
    ): Promise<CustomCompletionItem[]> {
        const global = await this.getGlobals(model);
        const suggestions: CustomCompletionItem[] = [];
        const localKeys = new Set(locals.map((item) => item.insertText));
        for (const key in global) {
            if (char && !key.toLowerCase().includes(char)) {
                continue;
            }
            const element = global[key];
            if (element === undefined) continue;
            const info = getVmFunctionInfo(element);
            let detail = '';
            if (info) {
                detail = paramsList(model, info);
            }
            suggestions.push({
                label: { label: key, description: DESC_GLOBAL, detail },
                kind: info ? languages.CompletionItemKind.Function : languages.CompletionItemKind.Variable,
                insertText: localKeys.has(key) ? `global.${key}` : key, // 如果有同名局部变量，使用 global. 前缀
                range,
                commitCharacters: info ? ['('] : ['.', '[', '('],
                global: element,
            });
        }
        return suggestions;
    }
    /** 查找局部变量 */
    private async completeLocal(
        model: editor.ITextModel,
        position: IPosition,
        char: string | undefined,
        range: languages.CompletionItemRanges,
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

    /** 查找全局变量字段 */
    private async completeGlobalFields(
        model: editor.ITextModel,
        position: Position,
        char: string | undefined,
        range: languages.CompletionItemRanges,
    ): Promise<CustomCompletionItem[]> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return [];
        const { globals } = compiled.groupedTags(model);

        // 查找最靠近当前输入位置的上一个全局变量
        let prevGlobal: { range: IRange } | undefined;
        for (const global of globals) {
            for (const ref of global.references) {
                if (!Position.isBefore(Range.getEndPosition(ref.range), position)) continue;
                if (
                    prevGlobal &&
                    Position.isBefore(Range.getEndPosition(ref.range), Range.getEndPosition(prevGlobal.range))
                )
                    continue;
                prevGlobal = ref;
            }
        }
        if (!prevGlobal) return [];

        const chain = model.getValueInRange(Range.fromPositions(Range.getStartPosition(prevGlobal.range), position));
        const chainParts = chain.split(/\s*(?:!\.|\.)\s*/);
        if (
            // 至少包含全局变量名和当前输入位置的字段名
            chainParts.length < 2 ||
            !chainParts.every(
                (part, index) =>
                    // 如果是最后一个部分，则可以为空（表示输入位置的字段名），否则必须是合法的标识符
                    (index === chainParts.length - 1 ? !part : false) ||
                    REG_IDENTIFIER.test(part) ||
                    REG_ORDINAL.test(part),
            )
        ) {
            return [];
        }
        const vmGlobal = await this.getGlobals(model);
        chainParts.pop(); // 移除最后一个部分，因为它是当前输入位置的字段名
        let value: VmValue | undefined = vmGlobal[chainParts.shift()!];
        for (const part of chainParts) {
            value = operations.$Get(value, part);
            if (value == null || typeof value != 'object') {
                return [];
            }
        }
        if (value == null || typeof value != 'object') {
            return [];
        }
        const keys = lib.global.keys(value);
        return keys.map((key) => {
            const field = operations.$Get(value, key);
            const callable = typeof field == 'function' || (isVmExtern(field) && field.callable);
            return {
                label: { label: key, description: DESC_FIELD },
                kind: callable ? languages.CompletionItemKind.Method : languages.CompletionItemKind.Field,
                insertText: key,
                range,
                commitCharacters: callable ? ['!', '('] : ['!', '.'],
            } satisfies CustomCompletionItem;
        });
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
                startColumn: position.column,
                endLineNumber: position.lineNumber,
                endColumn: position.column,
            };
        }
        char = char?.toLowerCase();

        const completionRange: languages.CompletionItemRanges = {
            replace: range,
            insert: Range.fromPositions(Range.getStartPosition(range), position),
        };
        if (/[^.]\.$/u.test(prev)) {
            // TODO: suggests local item fields
            const suggestions = await this.completeGlobalFields(model, position, char, completionRange);
            return { suggestions };
        }

        const suggestions = COMMON_GLOBAL_SUGGESTIONS(completionRange);
        const locals = await this.completeLocal(model, position, char, completionRange);
        const globals = await this.completeGlobal(model, char, locals, completionRange);
        suggestions.push(...locals, ...globals);

        return { suggestions };
    }

    /** @inheritdoc */
    resolveCompletionItem(
        item: languages.CompletionItem,
        token: CancellationToken,
    ): languages.CompletionItem | undefined {
        if (typeof item.label == 'string') {
            // not a dynamic completion item
            return item;
        }
        const custom = item as CustomCompletionItem;
        const { label } = item.label;
        if (custom.global != null) {
            if (item.documentation) return item;
            const def = globalDoc(label, custom.global);
            item.documentation = {
                value: `${codeblock('\0' + def.script)}\n${def.doc}`,
            };
        }
        return item;
    }
}
