import { isVmArray, isVmRecord, type VmConst } from '../../../types/index.js';
import { VmLib, expectArrayOrRecord, expectCompound } from '../../_helpers.js';
import { keys as _keys, values as _values, entries as _entries } from '../../../../helpers/utils.js';

export const keys = VmLib(
    (data) => {
        expectCompound('data', data, []);
        if (isVmArray(data)) {
            const arr: number[] = [];
            const len = data.length;
            for (let i = 0; i < len; i++) {
                arr.push(i);
            }
            return arr;
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
        returnsType: '(string | number)[]',
        examples: ['keys([10, 20]) // [0, 1]', 'keys((10, 20)) // ["0", "1"]'],
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
        examples: ['values((a: 1, b: 2)) // [1, 2]'],
    },
);

export const entries = VmLib(
    (data) => {
        expectArrayOrRecord('data', data, []);
        if (isVmArray(data)) {
            const arr: Array<{ 0: number; 1: VmConst }> = [];
            const len = data.length;
            for (let i = 0; i < len; i++) {
                arr.push({ 0: i, 1: data[i] ?? null });
            }
            return arr;
        }
        return _entries(data).map(([key, value]) => ({ 0: key, 1: value ?? null }));
    },
    {
        summary: '返回数组或记录的键值对列表',
        params: { data: '要获取键值对的数组或记录' },
        paramsType: { data: 'array | record' },
        returnsType: '(string | number, any)[]',
        examples: ['entries([1]) // [(0, 1)]', 'entries((a: 1)) // [("a", 1)]'],
    },
);
