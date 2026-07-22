import { isFinite } from '../../helpers/utils.js';
import { VM_ARRAY_MAX_LENGTH } from '../../helpers/constants.js';
import type { VmAny, VmArray } from '../types/index.js';
import { $ToNumber } from './convert.js';

const assertArrayLength = (len: number) => {
    if (len > VM_ARRAY_MAX_LENGTH) {
        throw new RangeError(`Array length exceeds maximum limit of ${VM_ARRAY_MAX_LENGTH}`);
    }
};
const isEmptyRange = (start: number, end: number) => {
    return !isFinite(start) || !isFinite(end) || start > end;
};
/** 构造范围数组 */
export const $ArrayRange = (start: VmAny, end: VmAny): VmArray => {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (isEmptyRange(s, e)) return [];
    const n = Math.floor(e - s + 1);
    assertArrayLength(n);
    const arr = [];
    arr.length = n;
    for (let i = 0; i < n; i++) {
        arr[i] = s + i;
    }
    return arr;
};
/** 构造范围数组（不包含结束值） */
export const $ArrayRangeExclusive = (start: VmAny, end: VmAny): VmArray => {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (isEmptyRange(s, e)) return [];
    const n = Math.ceil(e - s);
    assertArrayLength(n);
    const arr = [];
    arr.length = n;
    for (let i = 0; i < n; i++) {
        arr[i] = s + i;
    }
    return arr;
};
