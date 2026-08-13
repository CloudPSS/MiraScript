import { getVmFunctionInfo, type VmValue, isVmExtern, isVmModule, type VmFunctionInfo } from '@mirascript/mirascript';
import { type editor, type IPosition, type IRange, languages, Range } from '../../../monaco-api.js';
import { paramsList } from '../../utils.js';
import type { LocalDefinition } from '../../compile-result.js';
import type { CustomCompletionItem } from './interface.js';

/** 构造 filterText */
export function filterText(key: string, char: string | undefined): string {
    if (char == null || key.startsWith(char)) return key;
    if (key.startsWith('@')) return key.replace(/^@+/, '');
    if (key.startsWith('$')) return key.replace(/^\$+/, '');
    return key;
}

/** 构造选项 */
export function completion(
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
    if (fn != null) {
        detail = paramsList(model, fn);
        kind = field ? languages.CompletionItemKind.Function : languages.CompletionItemKind.Method;
    } else if (isVmModule(value)) {
        kind = languages.CompletionItemKind.Module;
    } else if (isVmExtern(value) && typeof value.value == 'function') {
        if (value.value.prototype != null && (key[0] ?? '').toUpperCase() === key[0]) {
            kind = languages.CompletionItemKind.Class;
        } else {
            detail = '(..)';
            kind = value.thisArg ? languages.CompletionItemKind.Method : languages.CompletionItemKind.Function;
        }
    } else if (!field && key.startsWith('@')) {
        kind = languages.CompletionItemKind.Constant;
    } else {
        kind = field ? languages.CompletionItemKind.Field : languages.CompletionItemKind.Variable;
    }
    return {
        label: { label: key, description, detail },
        kind,
        commitCharacters: fn ? ['!', '('] : ['!', '.', '[', '('],
        vmValue: value,
        isField: field,
    };
}

/** 获取完成范围 */
export function toCompletionItemRanges(position: IPosition, range: IRange): languages.CompletionItemRanges {
    return {
        replace: range,
        insert: Range.fromPositions(Range.getStartPosition(range), position),
    };
}
