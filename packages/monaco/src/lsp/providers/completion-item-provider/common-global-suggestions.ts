import { KEYWORDS as HELP_KEYWORDS } from '@mirascript/help';
import { KEYWORDS, RESERVED_KEYWORDS } from '../../../constants.js';
import { languages } from '../../../monaco-api.js';
import { codeblock } from '../../utils.js';

/** 构造关键字选项 */
export function kwSuggestion(kw: string, range: languages.CompletionItemRanges): languages.CompletionItem {
    const doc = (HELP_KEYWORDS as Record<string, string | undefined>)[kw];
    return {
        label: kw,
        kind: languages.CompletionItemKind.Keyword,
        insertText: kw,
        documentation: doc ? { value: doc } : undefined,
        range,
    };
}

const SUGGEST_KEYWORDS: string[] = [];

const loadSuggestKeywords = () => {
    if (SUGGEST_KEYWORDS.length > 0) return SUGGEST_KEYWORDS; // 已加载过
    for (const kw of KEYWORDS) {
        if (RESERVED_KEYWORDS.includes(kw as never)) continue; // 跳过保留关键字
        SUGGEST_KEYWORDS.push(kw);
    }
    return SUGGEST_KEYWORDS;
};

export const COMMON_GLOBAL_SUGGESTIONS = (
    range: languages.CompletionItemRanges,
    extension: boolean,
): languages.CompletionItem[] => {
    const suggestions: languages.CompletionItem[] = [
        {
            label: 'type',
            kind: languages.CompletionItemKind.Keyword,
            insertText: 'type',
            commitCharacters: ['('],
            documentation: { value: HELP_KEYWORDS.type },
            range,
        },
        {
            label: 'global',
            kind: languages.CompletionItemKind.Keyword,
            insertText: 'global',
            commitCharacters: ['.', '['],
            documentation: { value: HELP_KEYWORDS.global },
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
            const exist = suggestions.some(
                (item) => item.label === kw && item.kind === languages.CompletionItemKind.Keyword,
            );
            if (exist) continue;
            suggestions.push(kwSuggestion(kw, range));
        }
    }
    return suggestions;
};
