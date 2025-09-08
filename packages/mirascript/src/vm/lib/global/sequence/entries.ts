import { isVmArray, isVmRecord } from '../../../types/index.js';
import { VmLib, expectArrayOrRecord, expectCompound } from '../../_helpers.js';

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
