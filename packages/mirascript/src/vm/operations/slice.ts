import { VmError } from '../../helpers/error.js';
import { display } from '../../helpers/serialize.js';
import { toNumber } from '../../helpers/convert/number.js';
import { isNaN, isSafeInteger } from '../../helpers/utils.js';
import { isVmArray } from '../../helpers/types/index.js';
import type { VmAny, VmArray } from '../types/index.js';
import { $AssertInit } from './common.js';
const { ceil } = Math;
const { slice } = Array.prototype;

const sliceCore = (value: VmAny, start: VmAny, end: VmAny, exclusive: boolean): VmArray => {
    $AssertInit(value);
    $AssertInit(start);
    $AssertInit(end);

    if (!isVmArray(value)) {
        throw new VmError(`Expected array, got ${display(value)}`, []);
    }
    const { length } = value;
    let s = start != null ? toNumber(start) : 0;
    let e = end != null ? toNumber(end) : length - (exclusive ? 0 : 1);

    if (isNaN(s)) s = 0;
    else if (s < 0) s = length + s;

    if (isNaN(e)) e = exclusive ? length : length - 1;
    else if (e < 0) e = length + e;

    s = ceil(s);
    if (exclusive || !isSafeInteger(e)) {
        e = ceil(e);
    } else {
        e = e + 1;
    }
    return slice.call(value, s, e) satisfies unknown[] as VmArray;
};

/** 获取数组切片 */
export const $Slice = (value: VmAny, start: VmAny, end: VmAny): VmArray => {
    return sliceCore(value, start, end, false);
};
/** 获取数组切片（不包含结束位置） */
export const $SliceExclusive = (value: VmAny, start: VmAny, end: VmAny): VmArray => {
    return sliceCore(value, start, end, true);
};
