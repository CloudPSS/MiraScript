import { Cp } from '../../helpers.js';
import { $Call, $ToBoolean, $ToNumber } from '../../operations.js';
import {
    isVmArray,
    isVmConst,
    isVmPrimitive,
    isVmRecord,
    type VmAny,
    type VmArray,
    type VmConst,
    type VmValue,
} from '../../types/index.js';
import { VmError } from '../../error.js';
import {
    VmLib,
    expectArray,
    expectArrayOrRecord,
    expectCallable,
    expectCompound,
    expectConst,
    throwError,
} from '../_helpers.js';
import { serialize } from '../../../subtle.js';

/** map */
export function mapImpl(
    data: VmConst,
    mapper: (value: VmConst, index: number | string | null, data: VmConst) => VmConst | undefined,
): VmConst {
    if (isVmPrimitive(data)) {
        return mapper(data, null, data) ?? null;
    }
    if (isVmArray(data)) {
        const result: VmConst[] = [];
        const { length } = data;
        for (let i = 0; i < length; i++) {
            Cp();
            const ret = mapper(data[i] ?? null, i, data);
            if (ret === undefined) continue;
            if (isVmConst(ret)) {
                result.push(ret);
            } else {
                result.push(null);
            }
        }
        return result;
    } else {
        const entries: Array<[string, VmConst]> = [];
        for (const [key, value] of Object.entries(data)) {
            Cp();
            const ret = mapper(value ?? null, key, data);
            if (ret === undefined) continue;
            if (isVmConst(ret)) {
                entries.push([key, ret]);
            } else {
                entries.push([key, null]);
            }
        }
        return Object.fromEntries(entries);
    }
}

/** map 和 filter 的实现 */
function mapImplWrapped(
    data: VmAny,
    fnName: string,
    fn: VmAny,
    mapper: (fn: VmValue, value: VmValue, index: number | string | null, data: VmValue) => VmValue | undefined,
): VmValue {
    expectConst('data', data, null);
    expectCallable(fnName, fn, data);
    return mapImpl(data, (value, index, data) => {
        const ret = mapper(fn, value, index, data);
        if (ret === undefined || isVmConst(ret)) return ret;
        return null;
    });
}

export const map = VmLib(
    (data, f) => mapImplWrapped(data, 'f', f, (fn, value, key, data) => $Call(fn, [value, key, data])),
    {
        summary: '对数组或记录中的每个元素应用函数，并返回结果',
        params: {
            data: '要映射的数组或记录',
            f: '应用于每个元素的函数',
        },
        paramsType: {
            data: 'array | record',
            f: 'fn(value: any, key: number | string | nil, input: type(data)) -> any',
        },
        returnsType: 'type(data)',
    },
);

export const filter = VmLib(
    (data, predicate) =>
        mapImplWrapped(data, 'predicate', predicate, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return $ToBoolean(ret) ? value : undefined;
        }),
    {
        summary: '过滤数组或记录中的元素，返回满足条件的元素',
        params: {
            data: '要过滤的数组或记录',
            predicate: '用于测试每个元素的函数，返回 true 或 false',
        },
        paramsType: {
            data: 'array | record',
            predicate: 'fn(value: any, key: number | string | nil, input: type(data)) -> boolean',
        },
        returnsType: 'type(data)',
    },
);

export const filter_map = VmLib(
    (data, f) =>
        mapImplWrapped(data, 'f', f, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return ret ?? undefined;
        }),
    {
        summary: '对数组或记录中的每个元素应用函数，并返回非 nil 的结果',
        params: {
            data: '要映射的数组或记录',
            f: '应用于每个元素的函数，返回 nil 或非 nil 的值',
        },
        paramsType: {
            data: 'array | record',
            f: 'fn(value: any, key: number | string | nil, input: type(data)) -> any | nil',
        },
        returnsType: 'type(data)',
    },
);

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
                const index = i % (arr as VmArray).length;
                obj[key] = (arr as VmArray)[index] ?? null;
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
        }
        if (isVmRecord(data)) {
            for (const [key, v] of Object.entries(data)) {
                Cp();
                const value = v ?? null;
                const ret = p(value, key, data);
                if (ret === undefined) continue;
                if (!ret) continue;
                return { 0: key, 1: value };
            }
            return null;
        }
        throw new VmError('First argument must be primitive, array, or record', null);
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

export const len = VmLib(
    (arr) => {
        expectArray('arr', arr, Number.NaN);
        return arr.length;
    },
    {
        summary: '返回数组的长度',
        params: { arr: '要求长度的数组' },
        paramsType: { arr: 'array' },
        returnsType: 'number',
    },
);

const { keys: _keys, values: _values, entries: _entries } = Object;
export const keys = VmLib(
    (data) => {
        expectCompound('data', data, []);
        if (isVmArray(data)) {
            return Array.from({ length: data.length }, (_, i) => i);
        }
        if (isVmRecord(data)) {
            return _keys(data);
        }
        return data.keys();
    },
    {
        summary: '返回数组、记录、外部对象或模块的键列表',
        params: { data: '要获取键的数组、记录、外部对象或模块' },
        paramsType: { data: 'array | record | extern | module' },
        returnsType: '[string | number]',
    },
);

export const values = VmLib(
    (data) => {
        expectArrayOrRecord('data', data, []);
        if (isVmArray(data)) {
            return data;
        }
        return _values(data);
    },
    {
        summary: '返回数组或记录的值列表',
        params: { data: '要获取值的数组或记录' },
        paramsType: { data: 'array | record' },
        returnsType: 'array',
    },
);

export const entries = VmLib(
    (data) => {
        expectArrayOrRecord('data', data, []);
        if (isVmArray(data)) {
            return Array.from({ length: data.length }, (_, i) => ({ 0: i, 1: data[i] ?? null }));
        }
        return _entries(data).map(([key, value]) => ({ 0: key, 1: value }));
    },
    {
        summary: '返回数组或记录的键值对列表',
        params: { data: '要获取键值对的数组或记录' },
        paramsType: { data: 'array | record' },
        returnsType: '[(string | number, any)]',
    },
);
