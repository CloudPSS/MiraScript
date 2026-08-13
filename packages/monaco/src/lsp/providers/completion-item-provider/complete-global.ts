import { isVmModule, serialize } from '@mirascript/mirascript';
import type { editor, languages } from '../../../monaco-api.js';
import { Provider } from '../base.js';
import { isDeprecatedGlobal } from '../../utils.js';
import { REG_IDENTIFIER_FULL, isKeyword } from '../../../constants.js';
import { DESC_GLOBAL, type CustomCompletionItem } from './interface.js';
import { completion, filterText } from './utils.js';

/** 查找全局变量 */
export async function completeGlobal(
    model: editor.ITextModel,
    char: string | undefined,
    locals: readonly CustomCompletionItem[],
    range: languages.CompletionItemRanges,
    hasGlobalPrefix: boolean,
): Promise<CustomCompletionItem[]> {
    const global = await Provider.getContext(model);
    const suggestions: CustomCompletionItem[] = [];
    const localKeys = new Set(locals.map((item) => item.insertText));
    for (const key of new Set(global.keys())) {
        // skip deprecated globals in completion
        if (!global.has(key) || isDeprecatedGlobal(global, key)) continue;

        const element = global.get(key);

        let prefix = key;
        const edits: editor.ISingleEditOperation[] = [];
        if (!REG_IDENTIFIER_FULL.test(key)) {
            const kStr = serialize(key);
            if (hasGlobalPrefix) {
                prefix = `[${kStr}]`;
                // 删除多余的 . 前缀
                edits.push({
                    range: {
                        startLineNumber: range.replace.startLineNumber,
                        startColumn: range.replace.startColumn - 1,
                        endLineNumber: range.replace.endLineNumber,
                        endColumn: range.replace.endColumn,
                    },
                    text: '',
                });
            } else {
                prefix = `global[${serialize(key)}]`;
            }
        } else if (!hasGlobalPrefix && (localKeys.has(key) || isKeyword(key))) {
            prefix = `global.${key}`;
        }

        if (isVmModule(element)) {
            for (const f of element.keys()) {
                if (char && !f.toLowerCase().includes(char)) {
                    continue;
                }
                const field = element.get(f);
                if (field === undefined) continue;

                suggestions.push({
                    insertText: `${prefix}.${f}`,
                    filterText: filterText(f, char),
                    range,
                    additionalTextEdits: edits,
                    vmParent: element,
                    ...completion(model, DESC_GLOBAL, `${key}.${f}`, field, undefined, true),
                });
            }
        }

        if (char && !key.toLowerCase().includes(char)) {
            continue;
        }

        suggestions.push({
            insertText: prefix,
            filterText: filterText(key, char),
            range,
            additionalTextEdits: edits,
            vmParent: global,
            ...completion(model, DESC_GLOBAL, key, element, undefined, false),
        });
    }
    return suggestions;
}
