import { isVmWrapper, type VmAny } from '@mirascript/mirascript';
import { lib } from '@mirascript/mirascript/subtle';
import { getDeep, getField } from '../../utils.js';
import type { editor, languages, Position } from '../../../monaco-api.js';
import { REG_IDENTIFIER_FULL, REG_ORDINAL_FULL } from '../../../constants.js';
import { DESC_FIELD, type CustomCompletionItem } from './interface.js';
import { Provider } from '../base.js';
import { completion } from './utils.js';

/** 列出属性 */
function listFields(obj: VmAny, includeNonEnumerable: boolean): Array<string | number> {
    if (obj == null || typeof obj != 'object') return [];
    if (isVmWrapper(obj)) {
        try {
            return obj.keys(includeNonEnumerable);
        } catch {
            return [];
        }
    }
    return lib.keys(obj);
}

/** 查找变量字段 */
export async function completeFields(
    model: editor.ITextModel,
    position: Position,
    char: string | undefined,
    range: languages.CompletionItemRanges,
): Promise<CustomCompletionItem[]> {
    const compiled = await Provider.getCompileResult(model);
    if (!compiled) return [];
    const access = compiled.fieldAccessAt(model, position);
    if (!access || access.fields.length === 0) return [];
    const { def, fields } = access;
    if ('definition' in def.def) {
        // TODO: suggests local item fields
        return [];
    }
    const vmGlobal = await Provider.getContext(model);
    fields.pop(); // 移除最后一个部分，因为它是当前输入位置的字段名
    const [, value] = getDeep(vmGlobal, def.def.name, fields);
    if (value == null || typeof value != 'object') {
        return [];
    }
    const keys = listFields(value, true);
    const result: CustomCompletionItem[] = [];
    for (const k of keys) {
        const key = String(k);
        if (char && !String(key).toLowerCase().includes(char)) {
            continue;
        }
        if (!REG_IDENTIFIER_FULL.test(key) && !REG_ORDINAL_FULL.test(key)) {
            continue;
        }
        const field = getField(value, key);
        result.push({
            insertText: key,
            range,
            vmParent: isVmWrapper(value) ? value : undefined,
            ...completion(model, DESC_FIELD, key, field, undefined, true),
        });
    }
    return result;
}
