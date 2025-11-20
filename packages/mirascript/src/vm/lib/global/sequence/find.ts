import { toBoolean } from '../../../../helpers/convert.js';
import { entries } from '../../../../helpers/utils.js';
import { Cp } from '../../../helpers.js';
import { $Call, $Same } from '../../../operations.js';
import { type VmValue, isVmArray, isVmCallable } from '../../../types/index.js';
import { VmLib, expectArrayOrRecord, required } from '../../helpers.js';

export const find = VmLib(
    (data, predicate) => {
        expectArrayOrRecord('data', data, null);
        required('predicate', predicate, null);
        const p = isVmCallable(predicate)
            ? (value: VmValue, key: number | string, data: VmValue) => {
                  const ret = $Call(predicate, [value, key, data]);
                  return toBoolean(ret, undefined);
              }
            : (value: VmValue) => $Same(predicate, value);
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
            data: '要查找的数组或记录',
            predicate: '用于测试每个键值对的函数，或要查找的值',
        },
        paramsType: {
            data: 'array | record',
            predicate: '(fn(value: any, key: number | string, input: type(data)) -> boolean) | any',
        },
        returnsType: '(number | string, any) | nil',
        examples: ['find([3, 5, 8], fn (v) { v % 2 == 0 }) // (2, 8)', `find((x: 1, y: 2, z: 3), 2) // ('y', 2)`],
    },
);
