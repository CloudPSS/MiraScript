import { isFinite } from '../../helpers/utils.js';
import { VM_ARRAY_MAX_LENGTH } from '../../helpers/constants.js';
import type { VmAny } from '../types/index.js';
import { $ToNumber } from './convert.js';

const makeArray = (start: VmAny, end: VmAny, exclusive: boolean): number[] => {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (!isFinite(s) || !isFinite(e) || s > e) {
        return [];
    }
    const n = exclusive ? Math.ceil(e - s) : Math.floor(e - s + 1);
    if (n > VM_ARRAY_MAX_LENGTH) {
        throw new RangeError(`Array length exceeds maximum limit of ${VM_ARRAY_MAX_LENGTH}`);
    }
    const arr = [];
    for (let i = 0; i < n; i++) {
        arr[i] = s + i;
    }
    return arr;
};
/** 构造范围数组 */
export const $ArrayRange = (start: VmAny, end: VmAny): number[] => {
    return makeArray(start, end, false);
};
/** 构造范围数组（不包含结束值） */
export const $ArrayRangeExclusive = (start: VmAny, end: VmAny): number[] => {
    return makeArray(start, end, true);
};
