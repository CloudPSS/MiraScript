import type { VmAny, VmRecord } from '../../vm/index.js';
import { rethrowControl } from '../../vm/effects/state.js';
import { isVmArray, isVmWrapper } from '../types/index.js';
import { VmError } from '../error.js';
import { keys } from '../utils.js';
import { display } from '../serialize/index.js';
import { numberToString } from '../serialize/simple.js';
import { displayFunction } from '../serialize/display.js';

/** 转换为 string */
export function innerToString(value: VmAny, useBraces: boolean): string {
    if (value == null) return 'nil';
    if (typeof value == 'number') return numberToString(value, false);
    if (typeof value == 'string' || typeof value == 'boolean') return String(value);
    if (typeof value == 'function') return displayFunction(value);
    if (isVmWrapper(value)) return value.toString(useBraces);
    if (isVmArray(value)) {
        const strings: string[] = [];
        for (const item of value) {
            strings.push(innerToString(item, true));
        }
        // 在 join 过程中会自动把 null/undefined 和 empty slot 转为 ''
        // 与 innerToString 行为不一致
        const results = strings.join(', ');
        if (!useBraces) return results;
        return `[${results}]`;
    } else {
        const entries = keys(value satisfies VmRecord)
            .map((key) => `${key}: ${innerToString(value[key], true)}`)
            .join(', ');
        if (!useBraces) return entries;
        return `(${entries})`;
    }
}

/** 转换为 string */
export function toString<F = undefined>(value: VmAny, fallback?: F): string | Exclude<F, undefined> {
    if (typeof value == 'string') return value;
    if (value == null) return '';
    try {
        return innerToString(value, false);
    } catch (ex) {
        rethrowControl(ex);
        if (fallback === undefined) {
            const e = new VmError(`Failed to convert value to string: ${display(value)}`, '');
            e.cause = ex;
            throw e;
        }
        return fallback as Exclude<F, undefined>;
    }
}
