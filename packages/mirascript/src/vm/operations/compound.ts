import { VmError } from '../../helpers/error.js';
import { hasOwnEnumerable, NotNumber, isFinite, getRecord, keysLength } from '../../helpers/utils.js';
import { toNumber } from '../../helpers/convert/index.js';
import { display } from '../../helpers/serialize.js';
import { isVmArray, isVmRecord, isVmExtern, isVmWrapper } from '../../helpers/types.js';
import type { VmAny, VmValue } from '../types/index.js';
import { $AssertInit } from './common.js';
import { $ToString } from './convert.js';
import { $El } from './helpers.js';

const { trunc } = Math;
const { at } = Array.prototype;

/** 获取值的长度 */
export const $Length = (value: VmAny): number => {
    $AssertInit(value);
    if (isVmArray(value)) return value.length;
    if (isVmRecord(value)) return keysLength(value);
    if (isVmWrapper(value)) return value.keys().length;

    throw new VmError(`Value has no length: ${display(value)}`, 0);
};

/** 检查是否拥有字段 */
export const $Has = (obj: VmAny, key: VmAny): boolean => {
    $AssertInit(obj);
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return false;
    if (isVmWrapper(obj)) return obj.has(pk);
    return hasOwnEnumerable(obj, pk);
};

/** 获取字段 */
export const $Get = (obj: VmAny, key: VmAny): VmValue => {
    $AssertInit(obj);
    if (isVmArray(obj)) {
        $AssertInit(key);
        const index = toNumber(key, NotNumber);
        if (!isFinite(index)) return null;
        return $El((at.call(obj, trunc(index)) ?? null) as VmAny);
    }
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return null;
    if (isVmWrapper(obj)) {
        if (isFinite(key) && isVmExtern(obj) && obj.isArrayLike()) {
            let index = trunc(key as number);
            const { length } = obj.value;
            if (index < 0) index += length;
            if (index >= 0 && index < length) {
                return obj.get(String(index)) ?? null;
            }
        }
        return obj.get(pk) ?? null;
    }
    return $El(getRecord(obj, pk) ?? null);
};
/** 设置字段 */
export const $Set = (obj: VmAny, key: VmAny, value: VmAny): void => {
    $AssertInit(obj);
    $AssertInit(value);
    const pk = $ToString(key);
    if (!isVmExtern(obj)) throw new VmError(`Expected extern, got ${display(obj)}`, undefined);
    obj.set(pk, value);
};
