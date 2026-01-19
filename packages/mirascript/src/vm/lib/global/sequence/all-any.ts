import { toBoolean } from '../../../../helpers/convert/to-boolean.js';
import { entries } from '../../../../helpers/utils.js';
import { Cp } from '../../../checkpoint.js';
import { $Call } from '../../../operations/index.js';
import { isVmArray } from '../../../types/index.js';
import { expectArrayOrRecord, expectCallable, VmLib } from '../../helpers.js';

export const all = VmLib(
    (data, predicate) => {
        expectArrayOrRecord('data', data, null);
        expectCallable('predicate', predicate, data);
        if (isVmArray(data)) {
            for (let i = 0; i < data.length; i++) {
                Cp();
                /* c8 ignore next */
                const value = data[i] ?? null;
                const ret = $Call(predicate, [value, i, data]);
                if (!toBoolean(ret, undefined)) return false;
            }
            return true;
        } else {
            for (const [key, v] of entries(data)) {
                Cp();
                /* c8 ignore next */
                const value = v ?? null;
                const ret = $Call(predicate, [value, key, data]);
                if (!toBoolean(ret, undefined)) return false;
            }
            return true;
        }
    },
    {
        summary: '检查数组或记录中的所有键值对是否都满足条件',
        params: { data: '要检查的数组或记录', predicate: '用于测试每个键值对的函数，返回 true 或 false' },
        paramsType: {
            data: 'array | record',
            predicate: 'fn(value: any, key: number | string, input: type(data)) -> boolean',
        },
        returnsType: 'boolean',
        examples: ['all([1, 2, 3], fn { it > 0 }) // true'],
    },
);

export const any = VmLib(
    (data, predicate) => {
        expectArrayOrRecord('data', data, null);
        expectCallable('predicate', predicate, data);
        if (isVmArray(data)) {
            for (let i = 0; i < data.length; i++) {
                Cp();
                /* c8 ignore next */
                const value = data[i] ?? null;
                const ret = $Call(predicate, [value, i, data]);
                if (toBoolean(ret, undefined)) return true;
            }
            return false;
        } else {
            for (const [key, v] of entries(data)) {
                Cp();
                /* c8 ignore next */
                const value = v ?? null;
                const ret = $Call(predicate, [value, key, data]);
                if (toBoolean(ret, undefined)) return true;
            }
            return false;
        }
    },
    {
        summary: '检查数组或记录中的是否存在满足条件的键值对',
        params: { data: '要检查的数组或记录', predicate: '用于测试每个键值对的函数，返回 true 或 false' },
        paramsType: {
            data: 'array | record',
            predicate: 'fn(value: any, key: number | string, input: type(data)) -> boolean',
        },
        returnsType: 'boolean',
        examples: ['any([0, 1, 2], fn { it > 1 }) // true'],
    },
);
