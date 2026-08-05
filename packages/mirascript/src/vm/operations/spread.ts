import { VmError } from '../../helpers/error.js';
import { create } from '../../helpers/utils.js';
import { display } from '../../helpers/serialize.js';
import { isVmArray, isVmRecord, isVmExtern, isVmConst } from '../../helpers/types.js';
import { wrapToVmConst } from '../types/boundary.js';
import type { VmAny, VmRecord, VmConst } from '../types/index.js';
import { $AssertInit } from './common.js';

/** 展开记录 */
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
            if (isVmConst(value)) {
                result[key] = value;
            }
        }
        return result;
    }
    throw new VmError(`Expected record, array, extern or nil, got ${display(record)}`, null);
};

/** 展开数组 */
export const $ArraySpread = (array: VmAny): Iterable<VmConst | undefined> => {
    $AssertInit(array);
    if (array == null) return [];
    if (isVmArray(array)) return array;
    if (isVmExtern(array)) {
        if (array.isArrayLike()) {
            const result: VmConst[] = [];
            for (let i = 0, len = array.value.length; i < len; i++) {
                const item = array.value[i];
                result.push(wrapToVmConst(item, (v) => array.assumeVmValue(v, i)));
            }
            return result;
        } else if (typeof (array.value as Iterable<unknown>)[Symbol.iterator] == 'function') {
            const result: VmConst[] = [];
            for (const item of array.value as Iterable<unknown>) {
                result.push(wrapToVmConst(item, (v) => array.assumeVmValue(v, Symbol.iterator as never)));
            }
            return result;
        }
    }
    throw new VmError(`Expected array, iterable extern or nil, got ${display(array)}`, []);
};
