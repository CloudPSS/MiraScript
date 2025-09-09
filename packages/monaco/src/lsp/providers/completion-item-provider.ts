import { getVmFunctionInfo, type VmValue, isVmExtern, isVmModule, type VmFunctionInfo } from '@mirascript/mirascript';
import { lib, operations } from '@mirascript/mirascript/subtle';
import {
    type editor,
    languages,
    type CancellationToken,
    type IPosition,
    type IRange,
    type Position,
    Range,
} from '../../monaco-api.js';
import { Provider } from './base.js';
import { codeblock, getDeep, valueDoc, paramsList } from '../utils.js';
import { keywords, reservedKeywords } from '../../constants.js';
import type { LocalDefinition } from '../compile-result.js';

const DESC_GLOBAL = '(global)';
const DESC_LOCAL = '(local)';
const DESC_FIELD = '(field)';

const SUGGEST_KEYWORDS: string[] = [];

const loadSuggestKeywords = () => {
    if (SUGGEST_KEYWORDS.length > 0) return SUGGEST_KEYWORDS; // 已加载过
    const reserved = reservedKeywords();
    for (const kw of keywords()) {
        if (reserved.includes(kw)) continue; // 跳过保留关键字
        SUGGEST_KEYWORDS.push(kw);
    }
    return SUGGEST_KEYWORDS;
};

const COMMON_GLOBAL_SUGGESTIONS = (
    range: languages.CompletionItemRanges,
    extension: boolean,
): languages.CompletionItem[] => {
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
    ];
    if (!extension) {
        suggestions.push(
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
        );

        for (const kw of loadSuggestKeywords()) {
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
    }
    return suggestions;
};

/** 扩展完成项 */
interface CustomCompletionItem extends languages.CompletionItem {
    /** 是否为字段 */
    isField: boolean;
    /** 对应的变量值 */
    vmValue?: VmValue;
}

/** 构造 filterText */
function filterText(key: string, char: string | undefined): string {
    if (char == null || key.startsWith(char)) return key;
    return key.startsWith('@') || key.startsWith('$') ? key.slice(1) : key;
}

/** 构造选项 */
function completion(
    model: editor.ITextModel,
    description: string,
    key: string,
    value: VmValue | undefined,
    fn: VmFunctionInfo | LocalDefinition['fn'] | undefined,
    field: boolean,
): Pick<CustomCompletionItem, 'label' | 'kind' | 'commitCharacters' | 'vmValue' | 'isField'> {
    let detail = '';
    let kind: languages.CompletionItemKind;
    if (fn == null && typeof value == 'function') {
        fn = getVmFunctionInfo(value);
    }
    const callable = fn != null || (isVmExtern(value) && value.callable);
    if (callable) {
        detail = paramsList(model, fn);
        kind = field ? languages.CompletionItemKind.Function : languages.CompletionItemKind.Method;
    } else {
        if (isVmModule(value)) {
            kind = languages.CompletionItemKind.Module;
        } else if (!field && key.startsWith('@')) {
            kind = languages.CompletionItemKind.Constant;
        } else {
            kind = field ? languages.CompletionItemKind.Field : languages.CompletionItemKind.Variable;
        }
    }
    return {
        label: { label: key, description, detail },
        kind,
        commitCharacters: fn ? ['!', '('] : ['!', '.', '[', '('],
        vmValue: value,
        isField: field,
    };
}

/**
 * 自动完成
 */
export class CompletionItemProvider extends Provider implements languages.CompletionItemProvider {
    readonly triggerCharacters: string[] = ['.', ':'];
    /** 查找全局变量 */
    private async completeGlobal(
        model: editor.ITextModel,
        char: string | undefined,
        locals: readonly CustomCompletionItem[],
        range: languages.CompletionItemRanges,
    ): Promise<CustomCompletionItem[]> {
        const global = await this.getContext(model);
        const suggestions: CustomCompletionItem[] = [];
        const localKeys = new Set(locals.map((item) => item.insertText));
        for (const key of new Set(global.keys())) {
            const element = global.get(key);
            if (element === undefined) continue;

            if (isVmModule(element)) {
                for (const f of element.keys()) {
                    if (char && !f.toLowerCase().includes(char)) {
                        continue;
                    }
                    const field = element.get(f);
                    if (field === undefined) continue;

                    suggestions.push({
                        insertText: localKeys.has(key) ? `global.${key}.${f}` : `${key}.${f}`,
                        filterText: filterText(f, char),
                        range,
                        ...completion(model, DESC_GLOBAL, `${key}.${f}`, field, undefined, true),
                    });
                }
            }

            if (char && !key.toLowerCase().includes(char)) {
                continue;
            }

            suggestions.push({
                insertText: localKeys.has(key) ? `global.${key}` : key, // 如果有同名局部变量，使用 global. 前缀
                filterText: filterText(key, char),
                range,
                ...completion(model, DESC_GLOBAL, key, element, undefined, false),
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
    ): Promise<CustomCompletionItem[]> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return [];
        const suggestions: CustomCompletionItem[] = [];

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
                suggestions.push({
                    insertText: name,
                    filterText: filterText(name, char),
                    range,
                    ...completion(model, DESC_LOCAL, name, undefined, fn, false),
                });
            }
            if (!scope.parent) break;
            scope = scope.parent;
        }
        return suggestions;
    }

    /** 查找变量字段 */
    private async completeFields(
        model: editor.ITextModel,
        position: Position,
        char: string | undefined,
        range: languages.CompletionItemRanges,
    ): Promise<CustomCompletionItem[]> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return [];
        const access = compiled.fieldAccessAt(model, position);
        if (!access || access.fields.length === 0) return [];
        const { def, fields } = access;
        if ('definition' in def.def) {
            // TODO: suggests local item fields
            return [];
        }
        const vmGlobal = await this.getContext(model);
        fields.pop(); // 移除最后一个部分，因为它是当前输入位置的字段名
        const value = getDeep(vmGlobal.get(def.def.name), fields);
        if (value == null || typeof value != 'object') {
            return [];
        }
        const keys = lib.keys(value);
        const result: CustomCompletionItem[] = [];
        for (const k of keys) {
            const key = String(k);
            if (char && !String(key).toLowerCase().includes(char)) {
                continue;
            }
            const field = operations.$Get(value, key);
            result.push({
                insertText: key,
                range,
                ...completion(model, DESC_FIELD, key, field, undefined, true),
            });
        }
        return result;
    }

    /** 获取完成范围 */
    private toCompletionItemRanges(position: IPosition, range: IRange): languages.CompletionItemRanges {
        return {
            replace: range,
            insert: Range.fromPositions(Range.getStartPosition(range), position),
        };
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

        if (context.triggerCharacter === '.') {
            const prevWord = model.getWordAtPosition({
                lineNumber: position.lineNumber,
                column: position.column - 1,
            });
            if (prevWord?.word === 'global') {
                const globals = await this.completeGlobal(
                    model,
                    undefined,
                    [],
                    undefined as unknown as languages.CompletionItemRanges,
                );
                return { suggestions: globals };
            }
        }

        const word = model.getWordAtPosition(position);
        const prev = model.getValueInRange({
            startLineNumber: position.lineNumber,
            startColumn: (word?.startColumn ?? position.column) - 2,
            endLineNumber: position.lineNumber,
            endColumn: word?.startColumn ?? position.column,
        });

        if (context.triggerCharacter === ':' && prev !== '::') {
            return undefined; // 不是 :: 触发的
        }

        // suggest variables
        let char: string | undefined;
        let range: IRange;
        const def = compiled.variableAccessAt(model, position);
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

        const completionRange = this.toCompletionItemRanges(position, range);

        if (/[^.]\.$/u.test(prev)) {
            const suggestions = await this.completeFields(model, position, char, completionRange);
            return { suggestions };
        }

        const suggestions = COMMON_GLOBAL_SUGGESTIONS(completionRange, prev === '::');
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
        const { vmValue, isField } = item as CustomCompletionItem;
        const { label } = item.label;
        if (vmValue != null) {
            if (item.documentation) return item;
            const last = label.split('.').pop()!;
            const def = valueDoc(last, vmValue, isField);
            item.documentation = {
                value: `${codeblock('\0' + def.script)}\n${def.doc}`,
            };
        }
        return item;
    }
}
