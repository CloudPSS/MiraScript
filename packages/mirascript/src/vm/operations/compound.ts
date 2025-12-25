import { VmError } from '../../helpers/error.js';
import { hasOwnEnumerable, isNaN, NotNumber, keys, create } from '../../helpers/utils.js';
import { toNumber, toString } from '../../helpers/convert/index.js';
import { display } from '../../helpers/serialize.js';
import { isVmPrimitive, isVmArray, isVmRecord, isVmFunction, isVmExtern, isVmWrapper } from '../../helpers/types.js';
import type { VmAny, VmRecord, VmValue, VmConst } from '../types/index.js';
import { isSame } from './utils.js';
import { $AssertInit } from './common.js';
import { $ToString } from './convert.js';

const { trunc } = Math;
const { at } = Array.prototype;

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
export const $Length = (value: VmAny): number => {
    $AssertInit(value);
    if (isVmArray(value)) return value.length;
    if (isVmRecord(value)) return keys(value).length;
    if (isVmWrapper(value)) return value.keys().length;

    throw new VmError(`Value has no length: ${display(value)}`, 0);
};
export const $Omit = (value: VmAny, omitted: ReadonlyArray<number | string>): VmRecord => {
    $AssertInit(value);
    if (value == null || !isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    const valueKeys = keys(value);
    const omittedSet = new Set(omitted.map($ToString));
    for (const key of valueKeys) {
        if (!omittedSet.has(key)) {
            /* c8 ignore next */
            result[key] = value[key] ?? null;
        }
    }
    return result;
};
export const $Pick = (value: VmAny, picked: ReadonlyArray<number | string>): VmRecord => {
    $AssertInit(value);
    if (value == null || !isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    for (const key of picked) {
        const k = $ToString(key);
        if (hasOwnEnumerable(value, k)) {
            result[k] = value[k] ?? null;
        }
    }
    return result;
};
export const $Has = (obj: VmAny, key: VmAny): boolean => {
    $AssertInit(obj);
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return false;
    if (isVmWrapper(obj)) return obj.has(pk);
    return hasOwnEnumerable(obj, pk);
};
export const $Get = (obj: VmAny, key: VmAny): VmValue => {
    $AssertInit(obj);
    if (isVmArray(obj)) {
        $AssertInit(key);
        const index = toNumber(key, NotNumber);
        if (isNaN(index)) return null;
        return (at.call(obj, trunc(index)) as VmConst | undefined) ?? null;
    }
    const pk = $ToString(key);
    if (obj == null || typeof obj != 'object') return null;
    if (isVmWrapper(obj)) return obj.get(pk) ?? null;
    if (!hasOwnEnumerable(obj, pk)) return null;
    return obj[pk] ?? null;
};
export const $Set = (obj: VmAny, key: VmAny, value: VmAny): void => {
    $AssertInit(obj);
    $AssertInit(value);
    const pk = $ToString(key);
    if (obj == null) return;
    if (!isVmExtern(obj)) throw new VmError(`Expected extern, got ${display(obj)}`, undefined);
    obj.set(pk, value);
};
export const $Iterable = (value: VmAny): Iterable<VmValue | undefined> => {
    $AssertInit(value);
    if (isVmWrapper(value)) return value.keys();
    if (isVmArray(value)) return value;
    if (value != null && typeof value == 'object') return keys(value);
    throw new VmError(`Value is not iterable: ${display(value)}`, isVmFunction(value) ? [] : [value]);
};

export const $RecordSpread = (record: VmAny): VmRecord | null => {
    $AssertInit(record);
    if (record == null || isVmRecord(record)) return record;
    if (isVmArray(record)) {
        const result: Record<string, VmConst> = {};
        const len = record.length;
        for (let i = 0; i < len; i++) {
            const item = record[i];
            result[i] = item ?? null;
        }
        return result;
    }
    if (isVmExtern(record)) {
        const result: Record<string, VmConst> = create(null);
        for (const key of record.keys()) {
            const value = record.get(key) ?? null;
            // 当前只有 Primitive 不会进行二次包装
            if (isVmPrimitive(value)) {
                result[key] = value;
            }
        }
        return result;
    }
    throw new VmError(`Expected record, array, extern or nil, got ${display(record)}`, null);
};

export const $ArraySpread = (array: VmAny): Iterable<VmConst | undefined> => {
    $AssertInit(array);
    if (array == null) return [];
    if (isVmArray(array)) return array;
    if (isVmExtern(array) && typeof (array.value as Iterable<unknown>)[Symbol.iterator] == 'function') {
        const result: VmConst[] = [];
        for (const item of array.value as Iterable<unknown>) {
            // 当前只有 Primitive 不会进行二次包装
            if (isVmPrimitive(item)) {
                result.push(item);
            } else {
                result.push(null);
            }
        }
        return result;
    }
    throw new VmError(`Expected array, iterable extern or nil, got ${display(array)}`, []);
};
