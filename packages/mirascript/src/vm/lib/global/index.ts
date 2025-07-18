import { Element } from '../../helpers.js';
import { $ToNumber, $ToString } from '../../operations.js';
import { isVmArray, isVmRecord, type VmConst } from '../../types/index.js';
import { VmError } from '../../error.js';
import { VmLib, expectArray, expectArrayOrRecord, expectCompound } from '../helpers.js';

export * from './math.js';
export * from './map-filter.js';
export * from './debug.js';
export * from './json.js';
export * from './to-primitive.js';
export * from './string.js';

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

const _with = VmLib(
    (data, ...entries) => {
        expectArrayOrRecord('data', data, data);
        if (entries.length % 2 !== 0) {
            throw new VmError('Invalid number of arguments, expected even number of arguments', data);
        }
        if (isVmArray(data)) {
            const result: VmConst[] = [...data];
            for (let i = 0; i < entries.length; i += 2) {
                const index = Math.trunc($ToNumber(entries[i]));
                if (!Number.isFinite(index) || index < 0 || index >= Number.MAX_SAFE_INTEGER) continue;
                const value = entries[i + 1];
                while (index > result.length) {
                    result.push(null);
                }
                result[index] = Element(value);
            }
            return result;
        } else {
            const result: Record<string, VmConst> = { ...data };
            for (let i = 0; i < entries.length; i += 2) {
                const key = $ToString(entries[i]);
                const value = entries[i + 1];
                result[key] = Element(value);
            }
            return result;
        }
    },
    {
        summary: '在数组或记录中设置多个键值对',
        params: { data: '要设置的数组或记录', '..entries': '要设置的键值对，成对出现' },
        paramsType: { data: 'array | record', '..entries': '[string | number | any]' },
        returnsType: 'type(data)',
    },
);
export { _with as 'with' };

const { keys: _keys, values: _values, entries: _entries } = Object;
export const keys = VmLib(
    (data) => {
        expectCompound('data', data, []);
        if (isVmArray(data)) {
            return Array.from({ length: data.length }, (_, i) => $ToString(i));
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
        returnsType: '[string]',
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
            return Array.from({ length: data.length }, (_, i) => ({ 0: $ToString(i), 1: data[i] ?? null }));
        }
        return _entries(data).map(([key, value]) => ({ 0: key, 1: value }));
    },
    {
        summary: '返回数组或记录的键值对列表',
        params: { data: '要获取键值对的数组或记录' },
        paramsType: { data: 'array | record' },
        returnsType: '[(string, any)]',
    },
);
