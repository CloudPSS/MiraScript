import type { VmAny } from '../../vm/index.js';
import { VmError } from '../error.js';
import { display } from '../serialize.js';

/** 转换为 boolean */
export function toBoolean<F = undefined>(value: VmAny, fallback?: F): boolean | Exclude<F, undefined> {
    if (typeof value === 'boolean') return value;
    if (fallback === undefined) {
        throw new VmError(`Failed to convert value to boolean: ${display(value)}`, false);
    }
    return fallback as Exclude<F, undefined>;
}
