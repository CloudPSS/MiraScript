import { isNaN, keys, getRecord } from '../../helpers/utils.js';
import { getVmType, isVmWrapper } from '../../helpers/types/index.js';
import type { VmAny, VmArray, VmRecord, VmValue } from '../types/index.js';

/**
 * 确定操作的重载
 * @returns 如果应按数字处理则返回 true；如果应按字符串处理则返回 false
 */
export const overloadNumberString = (a: VmAny, b: VmAny): boolean => {
    if (typeof a == 'number' || typeof b == 'number') return true;
    if (typeof a == 'string' || typeof b == 'string') return false;
    return true;
};

const isArraySame = (a: VmArray, b: VmArray): boolean => {
    const len = a.length;
    if (len !== b.length) return false;

    for (let i = 0; i < len; i++) {
        if (!isSame(a[i] ?? null, b[i] ?? null)) {
            return false;
        }
    }
    return true;
};

const isRecordSame = (a: VmRecord, b: VmRecord): boolean => {
    const aKeys = keys(a);
    const bKeys = keys(b);

    const len = aKeys.length;
    if (len !== bKeys.length) return false;
    if (len === 0) return true;

    for (let i = 0; i < len; i++) {
        const key = aKeys[i]!;
        const av = a[key] ?? null;
        const bv = getRecord(b, key);
        // Key not found in b
        if (bv === undefined) return false;
        // Key found in b, but values are not the same
        if (!isSame(av, bv)) return false;
    }
    return true;
};

/**
 * 检查两个 VmValue 是否相同
 */
export const isSame = (a: VmValue, b: VmValue): boolean => {
    // Check for NaN
    if (typeof a == 'number' && typeof b == 'number') {
        return a === b || (isNaN(a) && isNaN(b));
    }
    // Check all primitive types, and fast path for reference equality
    if (a === b) return true;
    // Any primitives and functions arrive here are not equal
    if (a == null || b == null || typeof a != 'object' || typeof b != 'object') return false;
    const aType = getVmType(a);
    const bType = getVmType(b);
    if (aType !== bType) return false;
    // Handle array values
    if (aType === 'array') {
        return isArraySame(a as VmArray, b as VmArray);
    }
    // Handle record values
    if (aType === 'record') {
        return isRecordSame(a as VmRecord, b as VmRecord);
    }
    // Handle wrapper values
    if (isVmWrapper(a)) return a.same(b);
    if (isVmWrapper(b)) return b.same(a);
    return false;
};
