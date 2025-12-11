import { hasOwnEnumerable, isNaN, keys } from '../../helpers/utils.js';
import { isVmArray, isVmWrapper } from '../../helpers/types.js';
import type { VmAny, VmValue } from '../types/index.js';

/**
 * 确定操作的重载
 * @returns 如果应按数字处理则返回 true；如果应按字符串处理则返回 false
 */
export function overloadNumberString(a: VmAny, b: VmAny): boolean {
    if (typeof a == 'number' || typeof b == 'number') return true;
    if (typeof a == 'string' || typeof b == 'string') return false;
    return true;
}

/**
 * 检查两个 VmValue 是否相同
 */
export function isSame(a: VmValue, b: VmValue): boolean {
    // Check for NaN
    if (typeof a == 'number' && typeof b == 'number') {
        return a === b || (isNaN(a) && isNaN(b));
    }
    // Check all primitive types, and fast path for reference equality
    if (a === b) return true;
    // Any primitives and functions arrive here are not equal
    if (a == null || typeof a != 'object' || b == null || typeof b != 'object') return false;
    // Handle wrapper values
    if (isVmWrapper(a)) return a.same(b);
    if (isVmWrapper(b)) return b.same(a);
    // Handle array values
    if (isVmArray(a) && isVmArray(b)) {
        const len = a.length;
        if (len !== b.length) {
            return false;
        }
        // Compare array items
        for (let i = 0; i < len; i++) {
            if (!isSame(a[i] ?? null, b[i] ?? null)) {
                return false;
            }
        }
        return true;
    }
    // Handle record values
    if (!isVmArray(a) && !isVmArray(b)) {
        // Compare record fields
        const aKeys = keys(a);
        const bKeys = keys(b);
        if (aKeys.length !== bKeys.length) {
            return false;
        }
        for (const key of aKeys) {
            if (!hasOwnEnumerable(b, key)) {
                return false;
            }
            /* c8 ignore next 2 */
            const av = a[key] ?? null;
            const bv = b[key] ?? null;
            if (!isSame(av, bv)) {
                return false;
            }
        }
        return true;
    }
    return false;
}
