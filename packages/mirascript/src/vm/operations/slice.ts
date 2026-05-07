import { VmError } from '../../helpers/error.js';
import { display } from '../../helpers/serialize.js';
import { isNaN, isSafeInteger } from '../../helpers/utils.js';
import { isVmArray } from '../../helpers/types.js';
import type { VmAny, VmArray } from '../types/index.js';
import { $AssertInit } from './common.js';
import { $ToNumber } from './convert.js';
const { ceil } = Math;
const { slice } = Array.prototype;

const sliceCore = (value: VmArray, start: number, end: number, exclusive: boolean): VmArray => {
    const { length } = value;

    if (isNaN(start)) start = 0;
    else if (start < 0) start = length + start;

    if (isNaN(end)) end = exclusive ? length : length - 1;
    else if (end < 0) end = length + end;

    start = ceil(start);
    if (exclusive || !isSafeInteger(end)) {
        end = ceil(end);
    } else {
        end = end + 1;
    }
    return slice.call(value, start, end) satisfies unknown[] as VmArray;
};

/** 获取数组切片 */
export function $Slice(value: VmAny, start: VmAny, end: VmAny): VmArray {
    $AssertInit(value);
    if (!isVmArray(value)) throw new VmError(`Expected array, got ${display(value)}`, []);
    const s = start != null ? $ToNumber(start) : 0;
    const e = end != null ? $ToNumber(end) : value.length - 1;
    return sliceCore(value, s, e, false);
}
/** 获取数组切片（不包含结束位置） */
export function $SliceExclusive(value: VmAny, start: VmAny, end: VmAny): VmArray {
    $AssertInit(value);
    if (!isVmArray(value)) throw new VmError(`Expected array, got ${display(value)}`, []);
    const s = start != null ? $ToNumber(start) : 0;
    const e = end != null ? $ToNumber(end) : value.length;
    return sliceCore(value, s, e, true);
}
