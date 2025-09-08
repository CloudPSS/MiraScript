import { Cp } from '../../../helpers.js';
import { $Call, $ToBoolean, $ToNumber } from '../../../operations.js';
import { isVmArray, type VmArray, type VmConst, type VmValue } from '../../../types/index.js';
import { VmLib, expectArray, expectArrayOrRecord, expectCallable, throwError } from '../../_helpers.js';
import { serialize } from '../../../../subtle.js';
import { entries } from './entries.js';

export * from './with.js';
export * from './entries.js';
export * from './len.js';
export * from './map-filter.js';

export const flatten = VmLib(
    (data, depth = 1) => {
        expectArray('data', data, data);
        return data.flat($ToNumber(depth) as 1);
    },
    {
        summary: '将数组扁平化',
        params: { data: '要扁平化的数组', depth: '扁平化的深度，默认为 1' },
        paramsType: { data: 'array', depth: 'number' },
        returnsType: 'array',
    },
);

export const reverse = VmLib(
    (arr) => {
        expectArray('arr', arr, null);
        const dup = [...arr];
        dup.reverse();
        return dup;
    },
    {
        summary: '返回数组的反转副本',
        params: { arr: '要反转的数组' },
        paramsType: { arr: 'array' },
        returnsType: 'array',
    },
);

export const zip = VmLib(
    (data) => {
        const ets = entries(data);
        let len = 0;
        for (const { 0: key, 1: arr } of ets) {
            if (!isVmArray(arr)) {
                throwError(`data[${serialize(key)}] is not an array`, null);
            }
            len = Math.max(len, arr.length);
        }
        if (len === 0) return [];
        const result: Array<Record<string | number, VmConst>> = [];
        const isArr = isVmArray(data);
        for (let i = 0; i < len; i++) {
            Cp();
            const obj: Record<number | string, VmConst> = isArr ? ([] as Record<number, VmConst>) : {};
            for (const { 0: key, 1: arr } of ets) {
                obj[key] = (arr as VmArray)[i] ?? null;
            }
            result.push(obj);
        }
        return result;
    },
    {
        summary: '将数组的数组/记录转换为数组/记录的数组',
        params: { data: '要转换的数组/记录' },
        paramsType: { data: 'array | record' },
        returnsType: '[array | record]',
    },
);

export const find = VmLib(
    (data, predicate) => {
        expectArrayOrRecord('data', data, null);
        expectCallable('predicate', predicate, data);
        const p = (value: VmValue, key: string | number, data: VmValue) => {
            const ret = $Call(predicate, [value, key, data]);
            return $ToBoolean(ret);
        };
        if (isVmArray(data)) {
            const { length } = data;
            for (let i = 0; i < length; i++) {
                Cp();
                const value = data[i] ?? null;
                const ret = p(value, i, data);
                if (!ret) continue;
                return { 0: i, 1: value };
            }
            return null;
        } else {
            for (const [key, v] of Object.entries(data)) {
                Cp();
                const value = v ?? null;
                const ret = p(value, key, data);
                if (!ret) continue;
                return { 0: key, 1: value };
            }
            return null;
        }
    },
    {
        summary: '查找数组或记录中的键值对，返回第一个满足条件的键值对',
        params: {
            data: '查找滤的数组或记录',
            predicate: '用于测试每个键值对的函数，返回 true 或 false',
        },
        paramsType: {
            data: 'array | record',
            predicate: 'fn(value: any, key: number | string | nil, input: type(data)) -> boolean',
        },
        returnsType: '(string | number, any) | nil',
    },
);
