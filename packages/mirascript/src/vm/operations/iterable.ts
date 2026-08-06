import { VmError } from '../../helpers/error.js';
import { hasOwnEnumerable, keys } from '../../helpers/utils.js';
import { toString } from '../../helpers/convert/index.js';
import { display } from '../../helpers/serialize.js';
import { isVmPrimitive, isVmArray, isVmFunction, isVmWrapper } from '../../helpers/types/index.js';
import type { VmAny, VmRecord, VmValue } from '../types/index.js';
import { isSame } from './utils.js';
import { $AssertInit } from './common.js';

/** 检查值是否在可迭代对象中 */
export const $In = (value: VmAny, iterable: VmAny): boolean => {
    $AssertInit(value);
    $AssertInit(iterable);
    if (iterable == null) return false;
    if (typeof iterable != 'object') return false;
    if (isVmArray(iterable)) {
        if (value == null) {
            // array may have empty slots
            for (const item of iterable) if (item == null) return true;
            return false;
        }
        // JS %SameValueZero is same with `isSame` in this context
        if (isVmPrimitive(value)) return iterable.includes(value);
        // value is not null here, so it's ok to skip empty slots, since `isSame(null, something)` is always false
        return iterable.some((item = null) => isSame(item, value satisfies NonNullable<VmValue>));
    }
    // iterable is a record or an extern here, value should be a string
    const key = toString(value, undefined);
    if (isVmWrapper(iterable)) return iterable.has(key);
    return hasOwnEnumerable(iterable satisfies VmRecord, key);
};

/** 获取可迭代对象 */
export const $Iterable = (value: VmAny): Iterable<VmValue | undefined> => {
    $AssertInit(value);
    if (isVmWrapper(value)) return value.keys();
    if (isVmArray(value)) return value;
    if (value != null && typeof value == 'object') return keys(value);
    throw new VmError(`Value is not iterable: ${display(value)}`, isVmFunction(value) ? [] : [value]);
};
