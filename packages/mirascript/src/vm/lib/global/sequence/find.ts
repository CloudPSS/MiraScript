import { entries } from '../../../../helpers/utils.js';
import { Cp } from '../../../helpers.js';
import { $Call, $ToBoolean } from '../../../operations.js';
import { type VmValue, isVmArray } from '../../../types/index.js';
import { VmLib, expectArrayOrRecord, expectCallable } from '../../_helpers.js';

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
            for (const [key, v] of entries(data)) {
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
            data: '查的数组或记录',
            predicate: '用于测试每个键值对的函数，返回 true 或 false',
        },
        paramsType: {
            data: 'array | record',
            predicate: 'fn(value: any, key: number | string | nil, input: type(data)) -> boolean',
        },
        returnsType: '(string | number, any) | nil',
        examples: ['find([3, 5, 8], fn (v) { v % 2 == 0 }) // (2, 8)'],
    },
);
