import { DiagnosticCode } from '@mirascript/mirascript/subtle';
import { type editor, languages, type IPosition } from '../../../monaco-api.js';
import { DESC_LOCAL, type CustomCompletionItem } from './interface.js';
import { completion, filterText } from './utils.js';
import { Provider } from '../base.js';
import type { LocalDefinition } from '../../compile-result.js';

/** 创建局部变量 */
function createLocalCompletionItem(
    model: editor.ITextModel,
    char: string | undefined,
    range: languages.CompletionItemRanges,
    locals: Set<string>,
    { definition, fn }: LocalDefinition,
): CustomCompletionItem | null {
    const name = definition.code === DiagnosticCode.ParameterIt ? 'it' : model.getValueInRange(definition.range);
    if (char && !name.toLowerCase().includes(char)) return null;
    if (locals.has(name)) return null; // 子作用域可能会覆盖父作用域的变量

    locals.add(name);
    const suggestion = {
        insertText: name,
        filterText: filterText(name, char),
        range,
        ...completion(model, DESC_LOCAL, name, undefined, fn, false),
    };
    if (definition.code === DiagnosticCode.LocalModule) {
        suggestion.kind = languages.CompletionItemKind.Module;
    } else if (definition.code === DiagnosticCode.LocalFunction) {
        suggestion.kind = languages.CompletionItemKind.Function;
    }
    return suggestion;
}

/** 查找局部变量 */
export async function completeLocal(
    model: editor.ITextModel,
    position: IPosition,
    char: string | undefined,
    range: languages.CompletionItemRanges,
): Promise<CustomCompletionItem[]> {
    const compiled = await Provider.getCompileResult(model);
    if (!compiled) return [];
    const suggestions: CustomCompletionItem[] = [];

    let scope = compiled.scopeAt(model, position);
    const locals = new Set<string>();
    while (scope) {
        for (const def of scope.locals) {
            const suggestion = createLocalCompletionItem(model, char, range, locals, def);
            if (suggestion) suggestions.push(suggestion);
        }
        if (!scope.parent) break;
        scope = scope.parent;
    }
    return suggestions;
}
