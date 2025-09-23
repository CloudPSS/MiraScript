import { entries } from '../../../../helpers/utils.js';
import { Cp } from '../../../helpers.js';
import { $Call, $ToBoolean } from '../../../operations.js';
import { isVmArray } from '../../../types/index.js';
import { expectArrayOrRecord, expectCallable, VmLib } from '../../_helpers.js';

export const all = VmLib(
    (data, predicate) => {
        expectArrayOrRecord('data', data, null);
        expectCallable('predicate', predicate, data);
        if (isVmArray(data)) {
            for (let i = 0; i < data.length; i++) {
                Cp();
                const value = data[i] ?? null;
                const ret = $Call(predicate, [value, i, data]);
                if (!$ToBoolean(ret)) return false;
            }
            return true;
        } else {
            for (const [key, v] of entries(data)) {
                Cp();
                const value = v ?? null;
                const ret = $Call(predicate, [value, key, data]);
                if (!$ToBoolean(ret)) return false;
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
    },
);

export const any = VmLib(
    (data, predicate) => {
        expectArrayOrRecord('data', data, null);
        expectCallable('predicate', predicate, data);
        if (isVmArray(data)) {
            for (let i = 0; i < data.length; i++) {
                Cp();
                const value = data[i] ?? null;
                const ret = $Call(predicate, [value, i, data]);
                if ($ToBoolean(ret)) return true;
            }
            return false;
        } else {
            for (const [key, v] of entries(data)) {
                Cp();
                const value = v ?? null;
                const ret = $Call(predicate, [value, key, data]);
                if ($ToBoolean(ret)) return true;
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
    },
);
