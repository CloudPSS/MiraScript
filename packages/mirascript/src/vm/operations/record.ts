import { hasOwnEnumerable, keys, setRecord } from '../../helpers/utils.js';
import { isVmRecord } from '../../helpers/types/index.js';
import type { VmAny, VmRecord, VmConst } from '../types/index.js';
import { $AssertInit } from './common.js';

/** 删除记录中的指定字段 */
export const $Omit = (value: VmAny, omitted: ReadonlyArray<number | string>): VmRecord => {
    $AssertInit(value);
    if (!isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    const valueKeys = keys(value);
    const omittedSet = new Set(omitted.map(String));
    for (const key of valueKeys) {
        if (!omittedSet.has(key)) {
            setRecord(result, key, value[key]);
        }
    }
    return result;
};

/** 选择记录中的指定字段 */
export const $Pick = (value: VmAny, picked: ReadonlyArray<number | string>): VmRecord => {
    $AssertInit(value);
    if (!isVmRecord(value)) return {};
    const result: Record<string, VmConst> = {};
    for (const key of picked) {
        const k = String(key);
        if (hasOwnEnumerable(value, k)) {
            setRecord(result, k, value[k]);
        }
    }
    return result;
};
